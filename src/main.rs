#![feature(portable_simd)]

use std::cmp::min;
use std::simd::{i32x64, i64x8};
use std::time::Instant;

use rand::Rng;
use rayon::prelude::*;

use crate::compute_shader::GpuExecutor;

mod compute_shader;

/// <div style="
///         background: #F2F2F2;
///         color: black;
///         border: 3px solid #535353;
///         margin: 0px auto;
///         width: 456px;
///         padding: 10px;
///         border-radius: 10px;">
///
///     This is inside a **pretty** box.
///
/// </div>
///
///
///
const N: usize = 1000_000_000;

fn main() {
    // rust-gpu -> spir-v file
    const SHADER: &[u8] = include_bytes!(env!("rust_gpu.spv"));

    // random array
    let mut rng = rand::thread_rng();
    let mut arr = Box::new([0_i32; N]);

    for i in 0..N {
        arr[i] = rng.gen::<i32>();
    }

    // simd sum
    let mut simd_arr = Box::new([i32x64::splat(0); N / 64]);
    arr.chunks_exact(64)
        .enumerate()
        .for_each(|(index, chunk)| {
            simd_arr[index] = i32x64::from_slice(chunk);
        });

    let now = Instant::now();
    let seq_result = simd_sum(&simd_arr);
    println!("Simd sum result: {:?}, time: {:?}", seq_result, now.elapsed());

    // standard sum
    let now = Instant::now();
    let seq_result = parallel_chunked_sum(&arr);
    println!("Sum result: {}, time: {:?}", seq_result, now.elapsed());


    // let gpu_executor = create_gpu_executor(&mut arr);
    //
    // let instant = Instant::now();
    // pollster::block_on(compute_shader::execute_gpu_inner(
    //     &gpu_executor,
    //     &*arr,
    // ));
    // println!("compute shader time: {:?}", instant.elapsed().as_millis());
}


fn some_function(a: i64x8, b: i64x8) -> i64x8 {
    return a + b;
}

/// **950ms**
/// u16 about 40% faster
fn sequential_sum(arr: &[i32; N]) -> i32 {
    arr.iter()
        .enumerate()
        .map(|(i, e)| e + arr[min(i - 1, N - 1)] + arr[min(i + 1, N - 1)])
        .sum()
}

/// no, especially when parallel too, speedup for small or simple loops
/// sequential **500ms**  --really basic loops like only sum is auto-vectorized already--
/// parallel   **120ms**
/// not worth it for say, 1-3ms range, because overhead; same for parallel ones
/// super worth it for 100ms range, especially with par_iter
fn simd_sum(simd_arr: &[i32x64; N / 64]) -> i32 {
    simd_arr.par_iter()
        .enumerate()
        .map(|(i, e)| e + simd_arr[min(i - 1, N / 64 - 1)] + simd_arr[min(i + 1, N / 64 - 1)])
        .sum::<i32x64>()
        .to_array()
        .iter()
        .sum()
}

/// **170ms**
/// speedup; worth it for more than 20ms range
fn parallel_sum(arr: &[i32; N]) -> i32 {
    arr.par_iter()
        .enumerate()
        .map(|(i, e)| e + arr[min(i - 1, N - 1)] + arr[min(i + 1, N - 1)])
        .sum()
}

/// **200ms** --slighty less overhead than par_iter--
/// 100 chunks is about optimal
/// speedup; also good for 5-10ms range
fn parallel_chunked_sum(arr: &[i32; N]) -> i32 {
    let chunk_size = N / 100;
    arr.par_chunks(chunk_size)
        .map(|chunk| chunk.iter()
            .enumerate()
            .map(|(i, e)| e + chunk[min(i - 1, chunk_size - 1)] + chunk[min(i + 1, chunk_size - 1)])
            .sum::<i32>()
        )
        .sum()
}

fn create_gpu_executor(arr: &mut [i32; N]) -> GpuExecutor {
// create gpu device
    let (device, queue) = pollster::block_on(compute_shader::setup_gpu())
        .expect("Could not connect to the gpu");
    // create buffers
    let (size, staging_buffer, storage_buffer) = compute_shader::create_buffers(&device, arr);
    // create pipeline from shader
    let compute_pipeline = compute_shader::create_pipeline(&device, include_str!("shaders/sum.wgsl"));
    // bind group
    let bind_group = compute_shader::create_bind_group(&device, &storage_buffer, &compute_pipeline);
    //
    let gpu_executor = compute_shader::GpuExecutor {
        device: device,
        queue: queue,
        compute_pipeline: compute_pipeline,
        size: size,
        staging_buffer: staging_buffer,
        storage_buffer: storage_buffer,
        bind_group: bind_group,
    };
    gpu_executor
}

