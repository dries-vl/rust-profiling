use crate::heap_profiler::ProfilingAllocator;

mod heap_profiler;

#[global_allocator]
static GLOBAL: ProfilingAllocator = ProfilingAllocator;

fn main() {
    test_stack();

    let bytes = ProfilingAllocator::measure_memory(test_stack);
    println!("Allocated bytes: {}", bytes);

    ProfilingAllocator::reset_bytes();
    let _vec_2: Vec<u8> = vec![0; 1000];
    println!("Allocated bytes: {}", ProfilingAllocator::allocated_bytes());
}

fn test_stack() {
    let stack_size = 2765; // plateau of min stack size in bytes
    let builder = std::thread::Builder::new().stack_size(stack_size);
    let _vec_2: Vec<u8> = vec![0; 1000];

    let handle = builder.spawn(|| {
        test_func();
    });

    match handle.unwrap().join() {
        Ok(_) => println!("Function executed successfully within stack size."),
        Err(_) => println!("Stack overflowed!"),
    }
}

fn test_func() -> () {
    let mut array = [0; 502];
    array[0] = 1;
}
