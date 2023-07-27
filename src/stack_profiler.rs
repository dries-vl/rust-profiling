pub fn assert_stack_size<F: FnOnce() + Send + 'static>(f: F, stack_size: usize) -> () {
    let thread = std::thread::Builder::new().stack_size(stack_size);

    thread.spawn(|| f).unwrap().join().unwrap();

}
