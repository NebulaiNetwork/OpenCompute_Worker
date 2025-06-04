

use crate::protocol::{worker_hello};
use crate::sleep_ms;


pub async fn heat_beat(){
	sleep_ms(2000).await;

	// web_sys::console::log_1(&format!("wait hello").into());
	worker_hello();

	// web_sys::console::log_1(&format!("hello").into());
	loop{
		sleep_ms(60000).await;
		worker_hello();
		web_sys::console::log_1(&format!("heat beat").into());
	}
}