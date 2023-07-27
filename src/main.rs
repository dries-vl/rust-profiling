use crate::heap_profiler::ProfilingAllocator;

mod heap_profiler;
mod stack_profiler;

#[global_allocator]
static GLOBAL: ProfilingAllocator = ProfilingAllocator;

fn main() {

    stack_profiler::assert_stack_size(|| test_func(), 2765);
    println!("Stack did not overflow!");

    let bytes = heap_profiler::measure_memory(|| test_func());
    println!("Allocated bytes: {}", bytes);

}

fn test_func() -> () {
    vec![1; 1000];
    let mut array = [0; 502];
    array[0] = 1;
}
