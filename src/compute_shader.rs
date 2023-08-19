use std::borrow::Cow;
use std::time::Instant;

use wgpu::{BindGroup, Buffer, BufferAddress, CommandEncoder, ComputePipeline, Device, Queue};
use wgpu::util::DeviceExt;
use crate::N;

// Indicates a u32 overflow in an intermediate Collatz value
const OVERFLOW: i32 = i32::MAX;

pub struct GpuExecutor {
    pub device: Device,
    pub queue: Queue,
    pub compute_pipeline: ComputePipeline,
    pub size: BufferAddress,
    pub staging_buffer: Buffer,
    pub storage_buffer: Buffer,
    pub bind_group: BindGroup,
}

/// **1500ms**
pub async fn setup_gpu() -> Option<(Device, Queue)> {
    /// **1300ms**
    // Instantiates instance of WebGPU
    let instance = wgpu::Instance::default();

    /// **150ms**
    // `request_adapter` instantiates the general connection to the GPU
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await?;

    /// **13ms**
    // `request_device` instantiates the feature specific connection to the GPU, defining some parameters,
    //  `features` being the available features.
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::downlevel_defaults(),
            },
            None,
        )
        .await.ok()?;

    Some((device, queue))
}

pub async fn execute_gpu_inner(gpu_executor: &GpuExecutor, arr: &[i32; N]) -> Option<Vec<i32>> {
    let timer = Instant::now();
    // update the queue with the new data
    gpu_executor.queue.write_buffer(&gpu_executor.storage_buffer, 0, bytemuck::cast_slice(arr));
    println!("write buffer: {:?}", timer.elapsed());

    let timer = Instant::now();
    // recreate the encoder
    let encoder = create_encoder(&gpu_executor.device, &gpu_executor.compute_pipeline, &gpu_executor.bind_group, &gpu_executor.storage_buffer, &gpu_executor.staging_buffer, gpu_executor.size);
    println!("create encoder: {:?}", timer.elapsed());

    let timer = Instant::now();
    // Submits command encoder for processing
    gpu_executor.queue.submit(Some(encoder.finish()));
    println!("submit: {:?}", timer.elapsed());

    let timer = Instant::now();
    // Note that we're not calling `.await` here.
    let buffer_slice = gpu_executor.staging_buffer.slice(..);
    // Sets the buffer up for mapping, sending over the result of the mapping back to us when it is finished.
    let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

    // Poll the device in a blocking manner so that our future resolves.
    // In an actual application, `device.poll(...)` should
    // be called in an event loop or on another thread.
    gpu_executor.device.poll(wgpu::Maintain::Wait);
    println!("wait for answer from gpu: {:?}", timer.elapsed());

    let timer = Instant::now();    // Awaits until `buffer_future` can be read from
    if let Some(Ok(())) = receiver.receive().await {
        // Gets contents of buffer
        let data = buffer_slice.get_mapped_range();
        // Since contents are got in bytes, this converts these bytes back to u32
        let result = bytemuck::cast_slice(&data).to_vec();

        // With the current interface, we have to make sure all mapped views are
        // dropped before we unmap the buffer.
        drop(data);
        gpu_executor.staging_buffer.unmap(); // Unmaps buffer from memory
        // If you are familiar with C++ these 2 lines can be thought of similarly to:
        //   delete myPointer;
        //   myPointer = NULL;
        // It effectively frees the memory

        // Returns data from buffer
        println!("taking the result: {:?}", timer.elapsed());
        Some(result)
    } else {
        panic!("failed to run compute on gpu!")
    }
}

/// **1ms**
pub fn create_encoder(
    device: &Device,
    compute_pipeline: &ComputePipeline,
    bind_group: &BindGroup,
    storage_buffer: &Buffer,
    staging_buffer: &Buffer,
    size: BufferAddress,
) -> CommandEncoder
{
    /// **1ms**
    // A command encoder executes one or many pipelines.
    // It is to WebGPU what a command buffer is to Vulkan.
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
        });
        compute_pass.set_pipeline(compute_pipeline);
        compute_pass.set_bind_group(0, bind_group, &[]);
        compute_pass.dispatch_workgroups(64_u32, 64_u32, 1); // Number of cells to run, the (x,y,z) size of item being processed
    }
    // Sets adds copy operation to command encoder.
    // Will copy data from storage buffer on GPU to staging buffer on CPU.
    encoder.copy_buffer_to_buffer(storage_buffer, 0, staging_buffer, 0, size);
    encoder
}

/// **1ms**
pub fn create_bind_group(device: &Device, storage_buffer: &Buffer, compute_pipeline: &ComputePipeline) -> BindGroup {
    // Instantiates the bind group, once again specifying the binding of buffers.
    let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: storage_buffer.as_entire_binding(),
        }],
    });
    bind_group
}

/// **2ms**
pub fn create_pipeline(device: &Device, shader: &str) -> ComputePipeline {
    /// **1ms**
    // Loads the shader from WGSL
    let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader)),
    });

    /// **1ms**
    // A bind group defines how buffers are accessed by shaders.
    // It is to WebGPU what a descriptor set is to Vulkan.
    // `binding` here refers to the `binding` of a buffer in the shader (`layout(set = 0, binding = 0) buffer`).

    // A pipeline specifies the operation of a shader

    // Instantiates the pipeline.
    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &cs_module,
        entry_point: "main",
    });
    compute_pipeline
}

/// **4ms**
pub fn create_buffers(device: &Device, arr: &[i32; N]) -> (BufferAddress, Buffer, Buffer) {

    // Gets the size in bytes of the buffer.
    let size = std::mem::size_of_val(arr) as BufferAddress;

    // Instantiates buffer without data.
    // `usage` of buffer specifies how it can be used:
    //   `BufferUsages::MAP_READ` allows it to be read (outside the shader).
    //   `BufferUsages::COPY_DST` allows it to be the destination of the copy.
    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Instantiates buffer with data (`arr`).
    // Usage allowing the buffer to be:
    //   A storage buffer (can be bound within a bind group and thus available to a shader).
    //   The destination of a copy.
    //   The source of a copy.
    let storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Storage Buffer"),
        contents: bytemuck::cast_slice(arr),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });

    (size, staging_buffer, storage_buffer)
}
