use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() {
    let start = &mut 0 as *mut i32;
    println!("Hello, world!");

    test_stack();

    let end = &mut 0 as *mut i32;
    println!("Approximate stack usage: {} bytes", (end as usize) - (start as usize));
}

fn test_stack() {
    let stack_size = 128 * 1024; // 128 kb
    let builder = std::thread::Builder::new().stack_size(stack_size);

    let handle = builder.spawn(|| {
        test_func();
    });

    println!("Start of the program");
    match handle.unwrap().join() {
        Ok(_) => println!("Function executed successfully within stack size."),
        Err(_) => println!("Stack overflowed!"),
    }
    println!("end of the program");
}

fn test_func() -> [i32; 16400] {
    let mut array= [0; 16400];
    array[0] = 1;
    array
}