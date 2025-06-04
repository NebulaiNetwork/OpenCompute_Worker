use once_cell::unsync::OnceCell;
use std::cell::RefCell;

use crate::gpu_init::GpuManager;
use dynamic_code::{rune::Module, MODULE_REGISTRY};

thread_local! {
    static GPU: RefCell<OnceCell<GpuManager>> = RefCell::new(OnceCell::new());
}

pub async fn init_gpu() {
    let gpu = GpuManager::new().await;
    GPU.with(|cell| {
        let _ = cell.borrow_mut().set(gpu);
    });


    let register = Box::new(move |module: &mut Module| {
        module.async_function(["gpu_matrix_multiply"], matrix_multiply)?;
        Ok(())
    });
    MODULE_REGISTRY.lock().unwrap().push(register);

    let register = Box::new(move |module: &mut Module| {
        module.async_function(["gpu_vec_matrix_multiply"], vec_matrix_multiply)?;
        Ok(())
    });
    MODULE_REGISTRY.lock().unwrap().push(register);

}


pub fn get_gpu() -> GpuManager {
    GPU.with(|cell| {
		let binding = cell.borrow();
		let gpu_ref: &GpuManager = binding.get().expect("GpuManager not initialized");
        (*gpu_ref).clone()
    })
}


pub async fn matrix_multiply(a: Vec<Vec<f32>>, b: Vec<Vec<f32>>) -> Vec<Vec<f32>> {
    let gpu = get_gpu();
    let result = gpu.matrix_multiply(a, b).await;
        
    result.expect("gpu run error")
}

pub async fn vec_matrix_multiply(a: Vec<f32>, b: Vec<Vec<f32>>) -> Vec<f32> {
    let gpu = get_gpu();
    let result = gpu.vec_matrix_multiply(a, b).await;
        
    result.expect("gpu run error")
}