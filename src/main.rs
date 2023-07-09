use std::alloc::System;
use jemalloc_ctl::{epoch, stats};

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    println!("Hello, world!");

    // many statistics are cached and only updated when the epoch is advanced.
    epoch::advance().unwrap();

    let allocated = stats::allocated::read().unwrap();
    let resident = stats::resident::read().unwrap();
    println!("{} bytes allocated/{} bytes resident", allocated, resident);
}
