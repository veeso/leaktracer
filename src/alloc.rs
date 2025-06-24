use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::AtomicUsize;

/// This module provides a custom Allocator ([`GlobalAlloc`]) that tracks to log the memory allocations and stores the
/// allocation information for each module in the program.
///
/// ## Example
///
/// ```rust
/// use leaktracer::LeaktracerAllocator;
///
/// #[global_allocator]
/// static ALLOCATOR: LeaktracerAllocator = LeaktracerAllocator::init();
/// ```
pub struct LeaktracerAllocator {
    allocated: AtomicUsize,
}

impl LeaktracerAllocator {
    /// Creates a new instance of the [`LeaktracerAllocator`].
    pub const fn init() -> Self {
        LeaktracerAllocator {
            allocated: AtomicUsize::new(0),
        }
    }

    /// Returns the total number of bytes allocated by the allocator up to this point.
    pub fn allocated(&self) -> usize {
        self.allocated.load(std::sync::atomic::Ordering::Relaxed)
    }
}

unsafe impl GlobalAlloc for LeaktracerAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            self.allocated
                .fetch_add(layout.size(), std::sync::atomic::Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if !ptr.is_null() {
            self.allocated
                .fetch_sub(layout.size(), std::sync::atomic::Ordering::Relaxed);
        }
        unsafe { System.dealloc(ptr, layout) };
    }
}
