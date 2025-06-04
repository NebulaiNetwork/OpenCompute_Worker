
use route_websocket_client::{WsClient};
use crate::{send_msg, recv_msg, config, client_process};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct WsClientMsg{
    route:  String,
    payload:  String,
}

pub fn send_msg_to_ws_server(route: String, payload: String){
    let tmp = WsClientMsg{
        route: route,
        payload: payload,
    };

    send_msg::<WsClientMsg>(config::THREAD_WS_SEND, tmp);
}

pub async fn thread_ws_send(token: String, ip: String){

    let ws = WsClient::new(token, ip);

    ws.route_ws("worker/hello", client_process::worker_hello);
    ws.route_ws("worker/init", client_process::worker_init);
    ws.route_ws("worker/run", client_process::worker_run);
    ws.route_ws("worker/close", client_process::worker_close);

    ws.start_ws();

    loop {
        if let Some(msg) = recv_msg::<WsClientMsg>(config::THREAD_WS_SEND).await {
            ws.send(msg.route, msg.payload).await;
        } else {
            break;
        }
    }
}