
use wasm_bindgen::prelude::*;
use gloo_timers::future::TimeoutFuture;
use wasm_thread_manager::{send_msg, recv_msg, WasmThreadManager};
use public::{define_global, define_static};

mod client_process;
mod config;
mod thread_ws_send;
mod thread_keep_alive;
mod thread_test;
mod protocol;
mod gpu_init;
mod gpu_shade;
mod gpu_func;
mod gpu_init_rune_func;

define_global!(USER_TOKEN, String, String::new());
define_static!(G_AUTH_CODE, u64, 0x24420251131_u64);

pub async fn sleep_ms(ms: u32) {
    TimeoutFuture::new(ms).await;
}

#[wasm_bindgen]
pub fn worker_start(token : &str) -> Result<bool, JsValue>{

    let mut token_ = USER_TOKEN.lock().unwrap();
    *token_ = token.to_string().clone();

    WasmThreadManager::spawn_task(thread_ws_send::thread_ws_send(token.to_string(), config::WS_SERVER_URL.to_string()));
    WasmThreadManager::spawn_task(thread_keep_alive::heat_beat());
    WasmThreadManager::spawn_task(thread_test::test_gpu());

    //for debug log
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("could not initialize logger");

    Ok(true)
}