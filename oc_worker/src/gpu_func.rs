use crate::gpu_init::GpuManager;
use bytemuck::{Pod, Zeroable};
use futures_intrusive::channel::shared::oneshot_channel;
use wgpu::util::DeviceExt;

macro_rules! define_input_struct {
    ($self_:expr, $data:expr) => {
        $self_
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::bytes_of(&$data),
                usage: wgpu::BufferUsages::STORAGE,
            })
    };
}

macro_rules! define_input_array {
    ($self_:expr, $data:expr) => {
        $self_
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&$data),
                usage: wgpu::BufferUsages::STORAGE,
            })
    };
}

macro_rules! define_output_struct {
    ($self_:expr, $ty:ty) => {
        $self_.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<$ty>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        })
    };
}

macro_rules! define_output_array {
    ($self_:expr, $len: expr) => {
        $self_.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: ($len as u32 * std::mem::size_of::<f32>() as u32) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        })
    };
}

macro_rules! run_gpu_result_struct {
    ( $self_:expr, $call_func:expr, $group: expr, $result_type: ty, $run_x: expr, $run_y: expr, $run_z: expr, $buffer_result: expr, $( $binding:expr => $buffer:expr ),* $(,)? ) => {
        {
            let bind_group = {
                let entries = &[
                    $(
                        wgpu::BindGroupEntry {
                            binding: $binding,
                            resource: $buffer.as_entire_binding(),
                        }
                    ),*
                ];
                $self_.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &$self_.pipelines[$call_func].get_bind_group_layout($group),
                    entries,
                    label: None,
                })
            };

            let mut encoder = $self_
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: None,
                });

            {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: None,
                    timestamp_writes: None,
                });
                compute_pass.set_pipeline(&$self_.pipelines[$call_func]);
                compute_pass.set_bind_group($group, &bind_group, &[]);
                compute_pass.dispatch_workgroups($run_x, $run_y, $run_z);
            }

            let result_readback = $self_.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: std::mem::size_of::<$result_type>() as u64,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });

            encoder.copy_buffer_to_buffer(
                &$buffer_result,
                0,
                &result_readback,
                0,
                std::mem::size_of::<$result_type>() as u64,
            );

            $self_.queue.submit(Some(encoder.finish()));

            let slice = result_readback.slice(..);
            let (sender, receiver) = oneshot_channel();
            slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
            receiver
                .receive()
                .await
                .unwrap()
                .map_err(|_| "map failed")?;

            let data = slice.get_mapped_range();
            let result = bytemuck::from_bytes::<$result_type>(&data).value;
            drop(data);
            result_readback.unmap();

            result
        }
    };
}

macro_rules! run_gpu_result_array {
    ( $self_:expr, $call_func:expr, $group: expr, $len: expr, $run_x: expr, $run_y: expr, $run_z: expr, $buffer_result: expr, $( $binding:expr => $buffer:expr ),* $(,)? ) => {
        {
            let bind_group = {
                let entries = &[
                    $(
                        wgpu::BindGroupEntry {
                            binding: $binding,
                            resource: $buffer.as_entire_binding(),
                        }
                    ),*
                ];
                $self_.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &$self_.pipelines[$call_func].get_bind_group_layout($group),
                    entries,
                    label: None,
                })
            };

            let mut encoder = $self_
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: None,
                });

            {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: None,
                    timestamp_writes: None,
                });
                compute_pass.set_pipeline(&$self_.pipelines[$call_func]);
                compute_pass.set_bind_group($group, &bind_group, &[]);
                compute_pass.dispatch_workgroups($run_x, $run_y, $run_z);
            }

            let result_readback = $self_.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: ($len as u32 * std::mem::size_of::<f32>() as u32) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });

            encoder.copy_buffer_to_buffer(
                &$buffer_result,
                0,
                &result_readback,
                0,
                ($len as u32 * std::mem::size_of::<f32>() as u32) as wgpu::BufferAddress,
            );

            //for debug log
            $self_.device.push_error_scope(wgpu::ErrorFilter::Validation);

            $self_.queue.submit(Some(encoder.finish()));

            //for debug log
            let err = $self_.device.pop_error_scope().await;
            if let Some(e) = err {
                log::error!("GPU Validation Error: {:?}", e);
            }

            let slice = result_readback.slice(..);
            let (sender, receiver) = oneshot_channel();
            slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
            receiver
                .receive()
                .await
                .unwrap()
                .map_err(|_| "map failed")?;

            let data = slice.get_mapped_range();
            let result: Vec<f32> = bytemuck::cast_slice(&data).to_vec();

            drop(data);
            result_readback.unmap();

            result
        }
    };
}


#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
struct SingleU32 {
    value: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct MatrixHeader {
    long: u32,
    width: u32,
}


impl GpuManager {
    pub async fn run_add_u32(&self, a: u32, b: u32) -> Result<u32, String> {
        let a_data = SingleU32 { value: a };
        let b_data = SingleU32 { value: b };

        let buffer_a = define_input_struct!(self, a_data);

        let buffer_b = define_input_struct!(self, b_data);

        let buffer_result = define_output_struct!(self, SingleU32);

        let result: u32 = run_gpu_result_struct!(self, "add_u32", 0, SingleU32, 1, 1, 1, buffer_result,
            0 => buffer_a,
            1 => buffer_b,
            2 => buffer_result,
        );

        Ok(result)
    }

    pub async fn add(
        &self,
        input_a: &[f32],
        input_b: &[f32],
    ) -> Result<Vec<f32>, String> {

        let len = input_a.len();
        if input_b.len() != len {
            return Err("Input length mismatch".into());
        }
        
        let buffer_a = define_input_array!(self, input_a);
        let buffer_b = define_input_array!(self, input_b);
        let buffer_result = define_output_array!(self, len);
        
        let result = run_gpu_result_array!(self, "add", 0, len, len as u32, 1, 1, buffer_result,
            3 => buffer_a,
            4 => buffer_b,
            5 => buffer_result,
        );
        Ok(result)
    }

    pub async fn matrix_multiply(
        &self,
        a: Vec<Vec<f32>>,
        b: Vec<Vec<f32>>,
    ) -> Result<Vec<Vec<f32>>, String> {
        let a_long = a.len() as u32;
        let a_width = a.first().map_or(0, |r| r.len() as u32);
        let b_long = b.len() as u32;
        let b_width = b.first().map_or(0, |r| r.len() as u32);

        assert_eq!(
            a_width, b_long,
            "Matrix A's width must equal Matrix B's height."
        );

        let out_long = a_long;
        let out_width = b_width;
        let out_size = out_long * out_width;
        let flatten = |m: Vec<Vec<f32>>| m.into_iter().flatten().collect::<Vec<_>>();
        let a_data = flatten(a);
        let b_data = flatten(b);

        let a_matrix_info = define_input_struct!(self, MatrixHeader { long: a_long, width:a_width});
        let b_matrix_info = define_input_struct!(self, MatrixHeader { long: b_long, width:b_width});
        let a_matrix = define_input_array!(self, a_data);
        let b_matrix = define_input_array!(self, b_data);
        
        let buffer_result = define_output_array!(self, out_size);
        
        let result = run_gpu_result_array!(self, "matrix_multiply", 0, out_size, (out_width + 15) / 16, (out_long + 15) / 16, 1, buffer_result,
            6 => a_matrix_info,
            7 => b_matrix_info,
            8 => a_matrix,
            9 => b_matrix,
            10 =>buffer_result
        );
        
        let matrix = result
            .chunks(out_width as usize)
            .map(|row| row.to_vec())
            .collect::<Vec<_>>();

        Ok(matrix)
    }

    pub async fn vec_matrix_multiply(
        &self,
        a: Vec<f32>,
        b: Vec<Vec<f32>>,
    ) -> Result<Vec<f32>, String> {

        let b_long = b.len() as u32;
        let b_width = b.first().map_or(0, |r| r.len() as u32);

        assert_eq!(a.len() as u32, b_long, "Vector A's width must equal Matrix B's height.");

        let flatten = |m: Vec<Vec<f32>>| m.into_iter().flatten().collect::<Vec<_>>();
        let b_data = flatten(b);

        let vector_buffer = define_input_array!(self, a);
        let matrix_header = define_input_struct!(self, MatrixHeader { long: b_long, width: b_width });
        let matrix_buffer = define_input_array!(self, b_data);
        let result_buffer = define_output_array!(self, b_width);

        let workgroup_count_x = (b_width + 63) / 64;

        let result = run_gpu_result_array!(self, "vector_matrix_multiply", 0, b_width, workgroup_count_x, 1, 1,
            result_buffer,
            11 => vector_buffer,
            12 => matrix_header,
            13 => matrix_buffer,
            14 => result_buffer
        );

        Ok(result)
    }
    
}
