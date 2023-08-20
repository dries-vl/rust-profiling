#![feature(portable_simd)]

use std::cmp::min;
use std::simd::{i32x64, u64x8};
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
///
const N: usize = 1_000_000;

fn main() {
    // rust-gpu -> spir-v file
    const SHADER: &[u8] = include_bytes!(env!("rust_gpu.spv"));

    // random array
    let mut rng = rand::thread_rng();
    let mut arr = vec![0_u64; N].into_boxed_slice();
    let mut arr_2 = Box::new([0_u64; N]);
    for i in 0..N {
        arr[i] = rng.gen::<u64>();
        arr_2[i] = rng.gen::<u64>();
    }

    let alignment = std::mem::align_of::<Box<[u64; N]>>();
    println!("alignment: {:?}", alignment);

    // convert array to simd array
    let mut simd_arr = Box::new([u64x8::splat(0); N / 8]);
    arr.chunks_exact(8)
        .enumerate()
        .for_each(|(index, chunk)| {
            simd_arr[index] = u64x8::from_slice(chunk);
        });
    let mut simd_arr_2 = Box::new([u64x8::splat(0); N / 8]);
    arr_2.chunks_exact(8)
        .enumerate()
        .for_each(|(index, chunk)| {
            simd_arr_2[index] = u64x8::from_slice(chunk);
        });

    // time the algorithms
    let now = Instant::now();
    let mut result = Box::new([u64x8::splat(0); N / 8]);
    for i in 0..(N / 8) {
        result[i] = some_function(simd_arr[i], simd_arr_2[i]);
    }
    println!("time: {:?}", now.elapsed());

    let now = Instant::now();
    let mut result_2 = Box::new([u64x8::splat(0); N / 8]);
    for b in 0..(N / 8 / 1000) {
        for i in 0..1000 {
            result_2[b * 1000 + i] = some_function(simd_arr[b * 1000 + i], simd_arr_2[b * 1000 + i]);
        }
    }
    println!("time: {:?}", now.elapsed());
}


fn some_function(a: u64x8, b: u64x8) -> u64x8 {
    a + b
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

