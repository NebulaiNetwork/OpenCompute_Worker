use crate::sleep_ms;
use crate::gpu_init_rune_func::init_gpu;
// use crate::gpu_init_rune_func::get_gpu;

// use crate::gpu_init::{GpuManager};
// use dynamic_code::{DynamicCode};


// pub fn now_time_ms() -> u64{
//     js_sys::Date::now() as u64
// }

pub async fn test_gpu(){

    init_gpu().await;

	// let gpu = get_gpu();

    // let res = gpu.run_add_u32(3_u32, 3_u32).await.unwrap();
    // web_sys::console::log_1(&format!("add Result: {:?}", res).into());

    // let a: [f32; 2] = [3.0, 2.0];
    // let b: [f32; 2] = [4.0, 3.0];
    // let res = gpu.add(&a, &b).await.unwrap();
    // web_sys::console::log_1(&format!("add: {:?}", res).into());

    // let a = vec![
    //     vec![1.0f32, 2.0, 3.0],
    //     vec![4.0, 5.0, 6.0],
    // ];
    // let b = vec![
    //     vec![7.0f32, 8.0],
    //     vec![9.0, 10.0],
    //     vec![11.0, 12.0],
    // ];
    // let res = gpu.matrix_multiply(a, b).await.unwrap();
    // web_sys::console::log_1(&format!("Matrix Multiply Result: {:?}", res).into());

    // let script = r#"
    //     pub async fn multi_matrix(a, b){
    //         let result = gpu_matrix_multiply(a, b).await;
    //         result
    //     }

    //     pub fn matmul_row(row, b) {
    //         let b_cols = b[0].len();
    //         let b_rows = b.len();

    //         let result_row = [];

    //         for j in 0..b_cols {
    //             let sum = 0.0;
    //             for k in 0..b_rows {
    //                 sum += row[k] * b[k][j];
    //             }
    //             result_row.push(sum);
    //         }

    //         result_row
    //     }

    //     pub async fn gpu_matmul_row(row, b) {
    //         let result = gpu_vec_matrix_multiply(row, b).await;
    //         result
    //     }
    // "#;


    // match DynamicCode::new(script){
    //     Ok(mut dynamic_code) =>{
    //         // match dynamic_code.use_func::<i64, i64>("add", (100244, 2048)){
    //         //     Ok(result) => web_sys::console::log_1(&format!("result is {:?}", result).into()),
    //         //     Err(e) => web_sys::console::log_1(&format!("error {:?}", e).into()),
    //         // }

    //         // match dynamic_code.use_func_dyn::<Vec<Vec<f32>>>("multi_matrix", "[[[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]], [[7.0, 8.0], [9.0, 10.0], [11.0, 12.0]]]").await{
    //         //     Ok(result) => web_sys::console::log_1(&format!("result is {:?}", result).into()),
    //         //     Err(e) => web_sys::console::log_1(&format!("error {:?}", e).into()),
    //         // }

    //         let input_str = "[[1.0, 2.0], [[3.0, 4.0, 5.0], [3.0, 4.0, 5.0]]]";

    //         let start_time_1 = now_time_ms();

    //         match dynamic_code.use_func_dyn::<Vec<f32>>("matmul_row", input_str).await{
    //             // Ok(result) => web_sys::console::log_1(&format!("result is {:?}", result).into()),
    //             Ok(result) => todo!(),
    //             Err(e) => web_sys::console::log_1(&format!("error {:?}", e).into()),
    //         }

    //         let end_time_1 = now_time_ms();
            
    //         let start_time_2 = now_time_ms();

    //         match dynamic_code.use_func_dyn::<Vec<f32>>("gpu_matmul_row", input_str).await{
    //             // Ok(result) => web_sys::console::log_1(&format!("result is {:?}", result).into()),
    //             Ok(result) => todo!(),
    //             Err(e) => web_sys::console::log_1(&format!("error {:?}", e).into()),
    //         }

    //         let end_time_2 = now_time_ms();
            
    //         let duration_1 = end_time_1 - start_time_1;
    //         let duration_2 = end_time_2 - start_time_2;


    //         web_sys::console::log_1(&format!("local run use[{:?}], gpu run use[{:?}]", duration_1, duration_2).into());
    //     },
    //     Err(e) => web_sys::console::log_1(&format!("error {:?}", e).into()),
    // };

	sleep_ms(2000).await;
}