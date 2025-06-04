use std::collections::HashMap;

use crate::{gpu_shade::SHADER_SOURCE};

#[derive(Clone)]
pub struct GpuManager {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub pipelines: HashMap<String, wgpu::ComputePipeline>,
}

macro_rules! define_pipeline {
    (
        $name:expr,
        $device_:expr,
        $shader_module_: expr,
        bindings = [$($binding:expr),+ $(,)?]
    ) => {{

        let mut entries = Vec::new();
        let bindings = vec![$($binding),+];
        let last = bindings.len() - 1;

        for (i, binding) in bindings.iter().enumerate() {
            let read_only = i != last;
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: *binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
        }

        let bind_group_layout = $device_.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("{}_bgl", $name)),
            entries: &entries,
        });

        let pipeline_layout = $device_.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{}_layout", $name)),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        $device_.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some($name),
            layout: Some(&pipeline_layout),
            module: $shader_module_,
            entry_point: Some($name),
            cache: None,
            compilation_options: Default::default(),
        })
    }};
}


impl GpuManager {
    /// async init, get adapter,device,queue and load shader, crate mult pipeline
    pub async fn new() -> Self {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("Failed to create device");

        let shader_source_ = SHADER_SOURCE;

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("MultiComputeShader"),
            source: wgpu::ShaderSource::Wgsl(shader_source_.into()),
        });

        let add_t_pipeline = define_pipeline!("add_u32",&device, &shader_module,bindings = [0, 1, 2]);
        let add = define_pipeline!("add",&device, &shader_module,bindings = [3, 4, 5]);
        let matrix_mult = define_pipeline!("matrix_multiply",&device, &shader_module,bindings = [6, 7, 8, 9, 10]);
        let vector_matrix_multiply = define_pipeline!("vector_matrix_multiply",&device, &shader_module, bindings = [11, 12, 13, 14]);

        let mut pipelines = HashMap::new();

        pipelines.insert("add_u32".to_string(), add_t_pipeline);
        pipelines.insert("matrix_multiply".to_string(), matrix_mult);
        pipelines.insert("add".to_string(), add);
        pipelines.insert("vector_matrix_multiply".to_string(), vector_matrix_multiply);
        

        GpuManager {
            device,
            queue,
            pipelines,
        }
    }   
}