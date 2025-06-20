use futures::channel::mpsc::{unbounded, UnboundedSender, UnboundedReceiver};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::{MessageEvent, WebSocket};


use futures::future::LocalBoxFuture;
use futures::FutureExt;

type RouteCallback = Rc<dyn Fn(i16, String) -> LocalBoxFuture<'static, ()>>;
type RouteBigPayloadCallback = Rc<dyn Fn(i16, String, String) -> LocalBoxFuture<'static, ()>>;


#[derive(Clone)]
pub struct WsClient {
    inner: Rc<RefCell<WsClientInner>>,
}

struct WsClientInner {
    uid: String,
    url: String,
    routes: HashMap<String, RouteCallback>,
    routes_big_payload: HashMap<String, RouteBigPayloadCallback>,
    ws: Option<WebSocket>,
    tx: Option<UnboundedSender<String>>,
}

#[derive(Serialize)]
struct WsRequest {
    t: String,
    r: String,
    p: String,
}

#[derive(Deserialize)]
struct WsResponse {
    c: i16,
    p: String,
    r: String,
}

impl WsClient {
    pub fn new(uid: String, url: String) -> Self {
        Self {
            inner: Rc::new(RefCell::new(WsClientInner {
                uid,
                url,
                routes: HashMap::new(),
                routes_big_payload: HashMap::new(),
                ws: None,
                tx: None,
            })),
        }
    }

    pub fn route_ws<F, Fut>(&self, api: &str, callback: F)
    where
        F: Fn(i16, String) -> Fut + 'static,
        Fut: std::future::Future<Output = ()> + 'static,
    {
        let cb: RouteCallback = Rc::new(move |code, payload| {
            let fut = callback(code, payload);
            fut.boxed_local()
        });
        self.inner.borrow_mut().routes.insert(api.to_string(), cb);
    }

    pub fn route_ws_big_payload<F, Fut>(&self, api: &str, callback: F)
    where
        F: Fn(i16, String, String) -> Fut + 'static,
        Fut: std::future::Future<Output = ()> + 'static,
    {
        let cb: RouteBigPayloadCallback = Rc::new(move |code, payload, big_payload| {
            let fut = callback(code, payload, big_payload);
            fut.boxed_local()
        });
        self.inner.borrow_mut().routes_big_payload.insert(api.to_string(), cb);
    }

    pub fn start_ws(&self) {
        let inner = self.inner.clone();

        spawn_local(async move {
            let (tx, mut rx): (UnboundedSender<String>, UnboundedReceiver<String>) = unbounded();

            let ws = {
                let inner_ref = inner.borrow();
                WebSocket::new(&inner_ref.url).unwrap()
            };

            ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

            // onmessage
            {
                let inner = inner.clone();
                let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
                    if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                        let text = txt.as_string().unwrap_or_default();

                        if text.len() < 4 {
                            return;
                        }

                        let hex_len_str = &text[..4];
                        if let Ok(json_len) = u32::from_str_radix(hex_len_str, 16) {
                            let json_end = 4 + json_len as usize;
                            if text.len() < json_end {
                                return;
                            }
                            let json_str = &text[4..json_end];

                            if let Ok(parsed) = serde_json::from_str::<WsResponse>(json_str) {
                                let big_payload = &text[json_end..];

                                if big_payload.len() > 0{
                                    let cb_opt = {
                                        let inner_ref = inner.borrow();
                                        inner_ref.routes_big_payload.get(&parsed.r).cloned()
                                    };
                                    if let Some(cb) = cb_opt {
                                        spawn_local(cb(parsed.c, parsed.p, big_payload.to_string()));
                                    }
                                }else{
                                    let cb_opt = {
                                        let inner_ref = inner.borrow();
                                        inner_ref.routes.get(&parsed.r).cloned()
                                    };
                                    if let Some(cb) = cb_opt {
                                        spawn_local(cb(parsed.c, parsed.p));
                                    }
                                }
                            }
                            
                        }
                    }
                }) as Box<dyn FnMut(_)>);
                ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
                onmessage_callback.forget();
            }

            // onsend
            {
                let ws_clone = ws.clone();
                spawn_local(async move {
                    while let Some(msg) = rx.next().await {
                        let _ = ws_clone.send_with_str(&msg);
                    }
                });
            }

            // onclose
            {
                let inner = inner.clone();
                let onclose_callback = Closure::wrap(Box::new(move |_e: web_sys::CloseEvent| {
                    let inner = inner.clone();
                    spawn_local(async move {
                        gloo_timers::future::TimeoutFuture::new(3000).await;
                        let client = WsClient { inner };
                        client.start_ws();
                    });
                }) as Box<dyn FnMut(_)>);
                ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
                onclose_callback.forget();
            }

            // onerror
            {
                let onerror_callback = Closure::wrap(Box::new(move |e: web_sys::ErrorEvent| {
                    web_sys::console::error_1(&format!("WebSocket error: {:?}", e.message()).into());
                }) as Box<dyn FnMut(_)>);
                ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
                onerror_callback.forget();
            }

            {
                let mut inner_mut = inner.borrow_mut();
                inner_mut.tx = Some(tx);
                inner_mut.ws = Some(ws);
            }
        });
    }

    pub async fn send(&self, route: String, payload: String) {
        self.send_big_payload(route, payload, "".to_string()).await;
    }

    pub async fn send_big_payload(&self, route: String, payload: String, big_payload: String) {
        let (msg, tx_opt) = {
            let inner = self.inner.borrow();
            let req = WsRequest {
                t: inner.uid.clone(),
                r: route,
                p: payload,
            };
            let json_str = serde_json::to_string(&req).unwrap();

            let hex_len = {
                let mut s = format!("{:x}", json_str.len());
                if s.len() < 4 {
                    s = format!("{:0>4}", s);
                }
                s
            };

            let final_msg = hex_len + &json_str + &big_payload;

            (final_msg, inner.tx.clone())
        };

        if let Some(tx) = tx_opt {
            let _ = tx.unbounded_send(msg);
        }
    }
}
