use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

pub static ALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);

pub struct ProfilingAllocator;

impl ProfilingAllocator {

    pub fn measure_memory<F: FnOnce()>(f: F) -> usize {
        Self::reset_bytes();
        f();
        Self::allocated_bytes()
    }

    pub fn allocated_bytes() -> usize {
        ALLOCATED_BYTES.load(Ordering::SeqCst)
    }

    pub fn reset_bytes() -> () {
        ALLOCATED_BYTES.store(0, Ordering::SeqCst);
    }
}

unsafe impl GlobalAlloc for ProfilingAllocator {

    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            ALLOCATED_BYTES.fetch_add(layout.size(), Ordering::SeqCst);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        ALLOCATED_BYTES.fetch_sub(layout.size(), Ordering::SeqCst);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = System.realloc(ptr, layout, new_size);
        if !new_ptr.is_null() {
            let old_size = layout.size();
            if new_size > old_size {
                ALLOCATED_BYTES.fetch_add(new_size - old_size, Ordering::SeqCst);
            } else {
                ALLOCATED_BYTES.fetch_sub(old_size - new_size, Ordering::SeqCst);
            }
        }
        new_ptr
    }
}