pub mod http;
mod websocket;

extern crate alloc;
extern crate core;

use tokio::runtime::Runtime;

/// tokio context
pub struct TokioContext {
    runtime: Runtime,
}

#[no_mangle]
pub extern "C" fn rust_net_tokio_new(thread_count: u32) -> *mut TokioContext {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(thread_count as usize)
        .enable_all()
        .build()
        .expect("tokio runtime fail");

    Box::into_raw(Box::new(TokioContext { runtime }))
}

#[no_mangle]
pub unsafe extern "C" fn rust_net_tokio_free(handler: *mut TokioContext) {
    let handler = Box::from_raw(handler);
    drop(handler)
}
