use crate::http::Pair;
use crate::TokioContext;
use anyhow::Result;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use http::header::COOKIE;
use http::HeaderValue;
use std::collections::VecDeque;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::{Mutex, OnceCell};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};

enum WsMessage {
    // 连接成功
    ConnectSuccess,
    // 连接失败
    ConnectFailed(String),
    // 断开连接
    Disconnect(String),
    // 收到Ping
    RecvPing,
    // 收到Pong
    RecvPong,
    // 收到文本消息
    RecvText(String),
    // 收到二进制消息
    RecvBinary(Vec<u8>),
}

enum WsWriterMessage {
    Send(Vec<u8>),
    Close,
}

pub struct WsContext {
    msg_queue: Arc<Mutex<VecDeque<WsMessage>>>,
    tx: Arc<OnceCell<UnboundedSender<WsWriterMessage>>>,
}

#[no_mangle]
pub unsafe extern "C" fn rust_net_ws_connect(
    context: &mut TokioContext,
    url: *const c_char,
    cookies: *const c_char,
) -> *mut WsContext {
    let url = CStr::from_ptr(url).to_str().unwrap().to_string();
    let cookies = CStr::from_ptr(cookies).to_str().unwrap().to_string();

    let ws_context = WsContext {
        tx: Arc::new(OnceCell::new()),
        msg_queue: Arc::new(Mutex::new(VecDeque::new())),
    };

    let tx_cloned = ws_context.tx.clone();
    let msg_queue = ws_context.msg_queue.clone();

    context.runtime.spawn(async move {
        let result = ws_connect(url, cookies).await;

        // 调用了 rust_net_ws_connect 之后 立即调用 rust_net_ws_free 销毁了WsContext
        if Arc::strong_count(&tx_cloned) == 1 {
            return;
        }

        match result {
            Ok(ws_stream) => {
                let (tx, rx) = unbounded_channel::<WsWriterMessage>();
                if let Ok(_) = tx_cloned.set(tx) {
                    msg_queue.lock().await.push_back(WsMessage::ConnectSuccess);
                } else {
                    msg_queue
                        .lock()
                        .await
                        .push_back(WsMessage::ConnectFailed("init failed".to_string()));
                }

                let (writer, reader) = ws_stream.split();
                select! {
                    _ = poll_read(reader, msg_queue.clone()) => {}
                    _ = poll_write(writer, rx, msg_queue) => {}
                }
            }
            Err(err) => {
                msg_queue
                    .lock()
                    .await
                    .push_back(WsMessage::ConnectFailed(err.to_string()));
            }
        }
    });

    Box::into_raw(Box::new(ws_context))
}

#[no_mangle]
pub unsafe extern "C" fn rust_net_ws_send(
    ws_context: &mut WsContext,
    data: *const u8,
    length: usize,
) {
    if data.is_null() {
        return;
    }

    let data_slice = std::slice::from_raw_parts(data, length);

    if let Some(tx) = ws_context.tx.get() {
        let _ = tx.send(WsWriterMessage::Send(data_slice.to_vec()));
    }
}

#[no_mangle]
pub unsafe extern "C" fn rust_net_ws_get_message(ws_context: &mut WsContext) -> WsMessageData {
    if let Ok(mut queue) = ws_context.msg_queue.try_lock() {
        if let Some(message) = queue.pop_front() {
            return WsMessageData::from(message);
        }
    }

    WsMessageData::default()
}

#[no_mangle]
pub unsafe extern "C" fn rust_net_ws_free(ws_context: *mut WsContext) {
    let ws_context = Box::from_raw(ws_context);

    if let Some(tx) = ws_context.tx.get() {
        let _ = tx.send(WsWriterMessage::Close);
    }

    drop(ws_context)
}

#[no_mangle]
pub extern "C" fn rust_net_ws_free_message(resp: WsMessageData) {
    if resp.data.is_null() || resp.cap <= 0 {
        return;
    }
    unsafe {
        let buffer = Vec::from_raw_parts(resp.data as *mut u8, resp.len, resp.cap);
        // Rust 会在这里清理内存
        drop(buffer);
    }
}

async fn ws_connect(
    url: String,
    cookies: String,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
    let cookies: Vec<Pair> = serde_json::from_str(&cookies)?;
    let mut req = url.into_client_request()?;
    let headers = req.headers_mut();
    for pair in cookies.iter() {
        headers.append(
            COOKIE,
            HeaderValue::from_str(&format!("{}={}", pair.key, pair.value))?,
        );
    }

    let (ws_stream, _) = connect_async(req).await?;
    Ok(ws_stream)
}

async fn poll_write(
    mut writer: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    mut rx: UnboundedReceiver<WsWriterMessage>,
    msg_queue: Arc<Mutex<VecDeque<WsMessage>>>,
) {
    while let Some(message) = rx.recv().await {
        match message {
            WsWriterMessage::Send(data) => {
                if let Err(err) = writer.send(Message::from(data)).await {
                    msg_queue
                        .lock()
                        .await
                        .push_back(WsMessage::Disconnect(err.to_string()));
                    break;
                }
            }
            WsWriterMessage::Close => {
                msg_queue
                    .lock()
                    .await
                    .push_back(WsMessage::Disconnect("proactively disconnect".to_string()));
                break;
            }
        }
    }
    rx.close();
}

async fn poll_read(
    mut reader: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    msg_queue: Arc<Mutex<VecDeque<WsMessage>>>,
) {
    while let Some(result) = reader.next().await {
        let msg = {
            match result {
                Ok(msg) => msg,
                Err(err) => {
                    msg_queue
                        .lock()
                        .await
                        .push_back(WsMessage::Disconnect(err.to_string()));
                    break;
                }
            }
        };
        match msg {
            Message::Text(data) => {
                msg_queue.lock().await.push_back(WsMessage::RecvText(data));
            }
            Message::Binary(data) => {
                msg_queue
                    .lock()
                    .await
                    .push_back(WsMessage::RecvBinary(data));
            }
            Message::Ping(_) => {
                msg_queue.lock().await.push_back(WsMessage::RecvPing);
            }
            Message::Pong(_) => {
                msg_queue.lock().await.push_back(WsMessage::RecvPong);
            }
            Message::Close(_) => {
                msg_queue
                    .lock()
                    .await
                    .push_back(WsMessage::Disconnect("close".into()));
            }
            Message::Frame(_) => {}
        }
    }
}

#[repr(C)]
pub struct WsMessageData {
    message_type: i32,
    data: *const u8,
    len: usize,
    cap: usize,
}

impl WsMessageData {
    fn default() -> Self {
        Self {
            message_type: 0,
            data: std::ptr::null(),
            len: 0,
            cap: 0,
        }
    }

    fn from(msg: WsMessage) -> Self {
        let message_type;
        let mut buffer: Option<Vec<u8>> = None;
        match msg {
            WsMessage::ConnectSuccess => message_type = 1,
            WsMessage::ConnectFailed(data) => {
                message_type = 2;
                buffer = Some(data.into());
            }
            WsMessage::Disconnect(data) => {
                message_type = 3;
                buffer = Some(data.into());
            }
            WsMessage::RecvPing => {
                message_type = 4;
            }
            WsMessage::RecvPong => {
                message_type = 5;
            }
            WsMessage::RecvText(data) => {
                message_type = 6;
                buffer = Some(data.into());
            }
            WsMessage::RecvBinary(data) => {
                message_type = 7;
                buffer = Some(data);
            }
        };

        if let Some(buffer) = buffer {
            let ret = Self {
                message_type,
                data: buffer.as_ptr(),
                len: buffer.len(),
                cap: buffer.capacity(),
            };
            // 防止 Rust 在离开这个函数时自动清理 buffer
            std::mem::forget(buffer);
            ret
        } else {
            Self {
                message_type,
                data: std::ptr::null(),
                len: 0,
                cap: 0,
            }
        }
    }
}
