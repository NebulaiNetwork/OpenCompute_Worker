
use crate::{protocol::BaseMsg};
use public::{parse_json};

use crate::{protocol};
use dynamic_code::{DynamicCode};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use std::cell::RefCell;



thread_local! {
    pub static DYNC_CODE: Rc<RefCell<Option<DynamicCode>>> = Rc::new(RefCell::new(None));
}

pub async fn worker_hello(code: i16, _payload: String){
	if code == 0{
		web_sys::console::log_1(&format!("worker hello succ").into());
	}else{
		web_sys::console::log_1(&format!("worker hello failed").into());
	}
}

pub async fn worker_init(_code: i16, payload: String){

	match parse_json::<BaseMsg>(&payload){
		Ok(mut base_msg) => {
			let msg_info = base_msg.get_msg();

			match parse_json::<protocol::InitCodePayload>(&msg_info.payload){
				Ok(init_code_payload) => {
					match DynamicCode::new(&init_code_payload.code){
						Ok(dcm) => {
							DYNC_CODE.with(|code| {
							    code.borrow_mut().replace(dcm);
							});

							protocol::worker_init(base_msg.event_id, init_code_payload.source_uid, true, "".to_string());
						},
						Err(e) => protocol::worker_init(base_msg.event_id, init_code_payload.source_uid, false, e.to_string()),
					}
				},
				Err(e) => web_sys::console::log_1(&format!("parse InitCodePayload msg error:{}", e.to_string()).into())
			};
		},
		Err(e) => web_sys::console::log_1(&format!("parse base msg error:{}", e.to_string()).into()),
	};
}


#[derive(Debug, Deserialize, Serialize)]
struct DynamicRunCodeInfo{
	source_uid	: String,
	func	: String,
	input	: String,
	output	: i16,
}

macro_rules! call_and_send {
    ($manager:expr, $ty:ty, $payload:expr, $payload_2:expr) => {
        match $manager.use_func_dyn::<$ty>($payload, $payload_2).await {
            Ok(result) => (format!("{:?}", result), "".to_string()),
            Err(e) => ("".to_string(), e.to_string()),
        }
    };
}

pub async fn worker_run(_code: i16, payload: String) {
    match parse_json::<BaseMsg>(&payload) {
        Ok(mut base_msg) => {
            let msg_info = base_msg.get_msg();

            let maybe_code_ptr = DYNC_CODE.with(|code| {
                let mut maybe_code = code.borrow_mut();
                maybe_code.as_mut().map(|c| c as *mut _)
            });

            if let Some(c_ptr) = maybe_code_ptr {
                let c:&mut DynamicCode = unsafe { &mut *c_ptr };

                let call_func: DynamicRunCodeInfo = parse_json(&msg_info.payload).unwrap();

                let (result, error) = match call_func.output {
                    1 => call_and_send!(c, i32, &call_func.func, &call_func.input),
                    2 => call_and_send!(c, f32, &call_func.func, &call_func.input),
                    3 => call_and_send!(c, String, &call_func.func, &call_func.input),
                    4 => call_and_send!(c, Vec<i32>, &call_func.func, &call_func.input),
                    5 => call_and_send!(c, Vec<f32>, &call_func.func, &call_func.input),
                    6 => call_and_send!(c, Vec<Vec<i32>>, &call_func.func, &call_func.input),
                    7 => call_and_send!(c, Vec<Vec<f32>>, &call_func.func, &call_func.input),
                    _ => ("".to_string(), "no support this type".to_string()),
                };

                protocol::worker_run(base_msg.event_id, msg_info.operator_id, call_func.source_uid, result, error);
            } else {
                protocol::worker_run(base_msg.event_id, msg_info.operator_id, "".to_string(), "".to_string(), "dync manager no exsis".to_string());
            }
        }
        Err(e) => protocol::send_msg_to_verifier("worker/error".to_string(), e.to_string()),
    }
}


pub async fn worker_close(_code: i16, _payload: String){
}

