use wasm_bindgen_futures::spawn_local;
use std::future::Future;

use once_cell::sync::Lazy;
use futures::channel::mpsc::{unbounded, UnboundedSender, UnboundedReceiver};
use std::sync::{Arc, Mutex};
use futures::stream::StreamExt;

const WASM_THREAD_COUNT: usize = 10;

#[derive(Debug, Clone)]
struct Channel<T> {
    tx: UnboundedSender<T>,
    rx: Arc<Mutex<UnboundedReceiver<T>>>,
}

pub struct WasmChannelPool<T> {
    channels: Vec<Channel<T>>,
}

impl<T: 'static> WasmChannelPool<T> {
    // init channel
    pub fn new() -> Self {
        let mut channels = Vec::with_capacity(WASM_THREAD_COUNT);

        for _ in 0..WASM_THREAD_COUNT {
            let (tx, rx) = unbounded();
            channels.push(Channel {
                tx,
                rx: Arc::new(Mutex::new(rx)),
            });
        }

        Self { channels }
    }

    // send msg to some thread
    pub fn send(&self, thread_id: usize, msg: T) {
        let _ = self.channels[thread_id].tx.unbounded_send(msg);
    }

    // async recv msg
    pub async fn recv(&self, thread_id: usize) -> Option<T> {
        let rx = self.channels[thread_id].rx.clone();
        let mut rx = rx.lock().unwrap();
        rx.next().await
    }

    // try to recv msg
    pub fn try_recv(&self, thread_id: usize) -> Option<T> {
        let rx = self.channels[thread_id].rx.clone();
        let mut rx = rx.lock().unwrap();
        rx.try_next().ok().flatten()
    }
}

// use lazy confirm create once
static WASM_CHANNEL_POOL: Lazy<WasmChannelPool<Box<dyn std::any::Any + Send>>> = Lazy::new(WasmChannelPool::new);

pub fn send_msg<T: 'static + Send>(thread_id: usize, msg: T) {
    WASM_CHANNEL_POOL.send(thread_id, Box::new(msg));
}

pub async fn recv_msg<T: 'static + Send>(thread_id: usize) -> Option<T> {
    if let Some(msg) = WASM_CHANNEL_POOL.recv(thread_id).await {
        msg.downcast::<T>().ok().map(|boxed| *boxed)
    } else {
        None
    }
}

pub fn try_recv_msg<T: 'static + Send>(thread_id: usize) -> Option<T> {
    if let Some(msg) = WASM_CHANNEL_POOL.try_recv(thread_id) {
        msg.downcast::<T>().ok().map(|boxed| *boxed)
    } else {
        None
    }
}



pub struct WasmThreadManager;

impl WasmThreadManager {
    pub fn spawn_task<F>(task: F)
    where
        F: Future<Output = ()> + 'static,
    {
        spawn_local(task);
    }

    pub fn spawn_all(tasks: Vec<Box<dyn FnOnce() + Send>>) {
        for task in tasks {
            spawn_local(async move {
                task();
            });
        }
    }
}