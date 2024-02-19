use crate::TokioContext;
use reqwest::{Response, Version};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Arc;
use tokio::sync::OnceCell;

/// client context
pub struct ClientContext {
    client: reqwest::Client,
    items: slab::Slab<Arc<OnceCell<RespResult>>>,
}

pub struct ResponseData {
    status: u16,
    data: Vec<u8>,
    version: Version,
}

enum RespResult {
    Data(ResponseData),
    Error(String),
}

#[repr(C)]
pub struct RequestResponse {
    data: *const u8,
    len: usize,
    status: u32,
    version: i32,
}

impl RequestResponse {
    fn from(data: &ResponseData) -> Self {
        let buffer = data.data.clone();

        let this = Self {
            data: buffer.as_ptr(),
            len: buffer.len(),
            status: data.status as u32,
            version: {
                if data.version == Version::HTTP_09 {
                    9
                } else if data.version == Version::HTTP_10 {
                    10
                } else if data.version == Version::HTTP_11 {
                    11
                } else if data.version == Version::HTTP_2 {
                    20
                } else if data.version == Version::HTTP_3 {
                    30
                } else {
                    1
                }
            },
        };
        // 防止 Rust 在离开这个函数时自动清理 buffer
        std::mem::forget(buffer);
        this
    }
}

#[no_mangle]
pub extern "C" fn rust_net_client_new(brotli: bool, cookie_store: bool) -> *mut ClientContext {
    match reqwest::Client::builder()
        .use_rustls_tls()
        .brotli(brotli)
        .cookie_store(cookie_store)
        .build()
    {
        Ok(client) => Box::into_raw(Box::new(ClientContext {
            client,
            items: Default::default(),
        })),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn rust_net_client_free(handler: *mut ClientContext) {
    let handler = Box::from_raw(handler);
    drop(handler)
}

async fn handle_response(
    response_result: Result<Response, reqwest::Error>,
    item: Arc<OnceCell<RespResult>>,
) {
    // 请求被取消
    if Arc::strong_count(&item) == 1 {
        return;
    }
    match response_result {
        Ok(response) => {
            if response.status().is_success() {
                let status = response.status().as_u16();
                let version = response.version();
                match response.bytes().await {
                    Ok(bytes) => {
                        let _ = item.set(RespResult::Data(ResponseData {
                            status,
                            data: bytes.to_vec(),
                            version,
                        }));
                    }
                    Err(error) => {
                        let _ = item.set(RespResult::Error(error.to_string()));
                    }
                }
            } else {
                let _ = item.set(RespResult::Data(ResponseData {
                    status: response.status().as_u16(),
                    data: Vec::new(),
                    version: response.version(),
                }));
            }
        }
        Err(error) => {
            let _ = item.set(RespResult::Error(error.to_string()));
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn rust_net_post(
    tokio_context: &mut TokioContext,
    client_context: &mut ClientContext,
    url: *const c_char,
    data: *const u8,
    length: usize,
) -> u64 {
    if data.is_null() {
        return 0;
    }

    let client_cloned = client_context.client.clone();

    let url = CStr::from_ptr(url).to_str().unwrap().to_string();
    let data_slice = std::slice::from_raw_parts(data, length);

    let item = Arc::new(OnceCell::new());
    let key = client_context.items.insert(item.clone());

    tokio_context.runtime.spawn(async move {
        let response_result = client_cloned.post(url).body(data_slice).send().await;
        handle_response(response_result, item).await;
    });

    key as u64
}

#[no_mangle]
pub unsafe extern "C" fn rust_net_get(
    tokio_context: &mut TokioContext,
    client_context: &mut ClientContext,
    url: *const c_char,
) -> u64 {
    let client_cloned = client_context.client.clone();

    let url = CStr::from_ptr(url).to_str().unwrap().to_string();

    let item = Arc::new(OnceCell::new());
    let key = client_context.items.insert(item.clone());

    tokio_context.runtime.spawn(async move {
        let response_result = client_cloned.get(url).send().await;
        handle_response(response_result, item).await;
    });

    key as u64
}

/// 获取reqwest请求状态
/// 0正在请求
/// -1请求失败
/// 1请求成功
/// -2请求不存在
#[no_mangle]
pub extern "C" fn rust_net_get_request_state(client_context: &mut ClientContext, key: u64) -> i32 {
    if let Some(item) = client_context.items.get(key as usize) {
        if let Some(resp) = item.get() {
            match resp {
                RespResult::Data(_) => 1,
                RespResult::Error(_) => -1,
            }
        } else {
            // 正在请求中
            0
        }
    } else {
        -2
    }
}

/// 获取reqwest请求结果中的错误信息
/// 使用完成之后 调用 rust_net_free_string 释放内存
#[no_mangle]
pub extern "C" fn rust_net_get_request_error(
    client_context: &mut ClientContext,
    key: u64,
) -> *mut c_char {
    if let Some(item) = client_context.items.get(key as usize) {
        if let Some(resp) = item.get() {
            if let RespResult::Error(error) = resp {
                // 将 Rust 字符串转换为 C 风格的 `CString`
                return match CString::new(error.as_str()) {
                    Ok(cstr) => {
                        // 释放 CString 的所有权，这样它就不会在这个函数结束时被销毁
                        // 这是必要的，因为我们将把内存的控制权转移给 C
                        cstr.into_raw()
                    }
                    Err(_) => std::ptr::null_mut(), // 如果转换失败，返回空指针
                };
            }
        }
    }
    std::ptr::null_mut()
}

/// 获取reqwest请求结果
/// 使用完成之后 调用 rust_net_free_request_response 释放内存
#[no_mangle]
pub extern "C" fn rust_net_get_request_response(
    client_context: &mut ClientContext,
    key: u64,
) -> RequestResponse {
    if let Some(item) = client_context.items.get(key as usize) {
        if let Some(resp) = item.get() {
            if let RespResult::Data(data) = resp {
                return RequestResponse::from(data);
            }
        }
    }

    RequestResponse {
        data: std::ptr::null(),
        len: 0,
        status: 0,
        version: 0,
    }
}

#[no_mangle]
pub unsafe extern "C" fn rust_net_free_string(s: *mut c_char) {
    if !s.is_null() {
        // 重新获得 CString 的所有权并将其丢弃，这将释放字符串的内存
        let cs = CString::from_raw(s);
        drop(cs);
    }
}

#[no_mangle]
pub extern "C" fn rust_net_free_request_response(resp: RequestResponse) {
    if resp.data.is_null() {
        return;
    }
    unsafe {
        let buffer = Vec::from_raw_parts(resp.data as *mut u8, resp.len, resp.len);
        // Rust 会在这里清理内存
        drop(buffer);
    }
}
