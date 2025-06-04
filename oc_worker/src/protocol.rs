

use crate::thread_ws_send::send_msg_to_ws_server;
use serde::{Deserialize, Serialize};
use public::{encode, decode, build_json, parse_json, rand_u64};
use crate::{G_AUTH_CODE};

#[derive(Deserialize, Serialize)]
pub struct BaseMsg {
    pub event_id: u64,

    pub payload: String,

    #[serde(skip)]
    msg_info: MsgInfo,
    #[serde(skip)]
    already_init: bool,
}

impl BaseMsg {
    pub fn new(event_id: u64, msg: MsgInfo) -> Self {
        let payload = encode(&msg, event_id);

        Self {
            event_id: event_id,
            payload: payload,
            already_init: false,
            msg_info: msg,
        }
    }

    pub fn get_msg(&mut self) -> MsgInfo {
        if self.already_init {
            self.msg_info.clone()
        } else {
            let decode_result = decode(&self.payload.clone(), self.event_id);

            self.msg_info = parse_json(&decode_result).unwrap();
            self.already_init = true;
            self.msg_info.clone()
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MsgInfo {
    pub operator_id: u64,
    pub payload: String,
    auth_code: u64,
}

impl MsgInfo {
    pub fn new(operator_id_: u64, payload: String) -> Self {
        Self {
            operator_id: operator_id_,
            payload: payload,
            auth_code: *G_AUTH_CODE,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RunCodePayload {
    func: String,
    input: String,
    output: i16,
}

#[derive(Deserialize, Serialize)]
struct RunCodeResult{
	source_uid	: String,
	error	: String,
	result	: String,
}


#[derive(Debug, Deserialize, Serialize)]
pub struct InitCodePayload {
    pub source_uid: String,
    pub code: String,
}

#[derive(Serialize)]
struct InitCodeResult{
    source_uid: String,
    succ    : bool,
    payload : String,
}

pub fn direct_send_error_msg(other_msg: String) {
    let error_msg_s = BaseMsg::new(0, MsgInfo::new(0, other_msg));

    let json_str = build_json(&error_msg_s).unwrap();
    
    send_msg_to_ws_server("error".to_string(), json_str);
}

pub fn send_msg_to_verifier(route: String, other_msg: String){
    let msg_s = BaseMsg::new(rand_u64(), MsgInfo::new(0, other_msg));

    let json_str = build_json(&msg_s).unwrap();

    send_msg_to_ws_server(route, json_str);
}

pub fn send_msg_to_verifier_by_event_id(event_id: u64, route: String, other_msg: String){
    let msg_s = BaseMsg::new(event_id, MsgInfo::new(0, other_msg));

    let json_str = build_json(&msg_s).unwrap();

    send_msg_to_ws_server(route, json_str);
}

pub fn send_msg_to_verifier_by_event_id_op_id(route: String, event_id: u64, op_id: u64, other_msg: String){
    let msg_s = BaseMsg::new(event_id, MsgInfo::new(op_id, other_msg));

    let json_str = build_json(&msg_s).unwrap();

    send_msg_to_ws_server(route, json_str);
}



pub fn worker_hello(){
    send_msg_to_verifier("worker/hello".to_string(), "".to_string());
}



pub fn worker_init(event_id: u64, source_uid: String, succ: bool, more_info: String){

    let init_result = InitCodeResult{source_uid: source_uid, succ: succ, payload: more_info};

    let result_payload = build_json(&init_result).unwrap();

    send_msg_to_verifier_by_event_id_op_id("worker/init".to_string(), event_id, 0, result_payload);
}

pub fn worker_run(event_id: u64, op_id: u64, source_uid: String, result: String, error: String){
    let tmp_payload = RunCodeResult {source_uid, error, result};

    let run_code_payload = build_json(&tmp_payload).unwrap();
    send_msg_to_verifier_by_event_id_op_id(
        "worker/run".to_string(),
        event_id,
        op_id,
        run_code_payload,
    );
}

// pub fn worker_close(event_id: u64){
//     send_msg_to_verifier_by_event_id_op_id("worker/close".to_string(), event_id, 0, "".to_string());
// }