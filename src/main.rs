use crate::heap_profiler::ProfilingAllocator;

mod heap_profiler;
mod stack_profiler;

#[global_allocator]
static GLOBAL: ProfilingAllocator = ProfilingAllocator;

/// | @ this is a comment
/// | # title
/// | **bold text**
/// |
///
/// | Header1 | Header2 |
/// |---------|---------|
/// | abc     | def     |
///
/// - [x] Complete task
/// - [ ] Incomplete task
/// ![Tip](images/tip.png)\
///
/// > ------------------------------------------------------------
/// >  multiline text bla bla bla bla
/// >  bla bla bla bla bla bla bla ... the
/// >                       blank line below is important
/// >   rtsrtrtrt
/// > ------------------------------------------------------------
///
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
fn main() {

    stack_profiler::assert_stack_size(|| test_func(), 2765);
    println!("Stack did not overflow!");

    let bytes = heap_profiler::measure_memory(|| test_func());
    println!("Allocated bytes: {}", bytes);

}

/// >
/// > --------------------------------
/// >
/// >
/// >
/// >
/// >
/// >
/// >
/// > --------------------------------
fn test_func() -> () {
    vec![1; 1000];
    let mut array = [0; 502];
    array[0] = 1;
}
