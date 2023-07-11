use std::alloc::System;
use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() {
    let start = &mut 0 as *mut i32;
    println!("Hello, world!");
    let end = &mut 0 as *mut i32;
    println!("Approximate stack usage: {} bytes", (end as usize) - (start as usize));
}
