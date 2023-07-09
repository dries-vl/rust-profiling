use std::alloc::System;

#[global_allocator]
static GLOBAL: System = System;

fn main() {
    let start = &mut 0 as *mut i32;
    println!("Hello, world!");
    let end = &mut 0 as *mut i32;
    println!("Approximate stack usage: {} bytes", (end as usize) - (start as usize));
}
