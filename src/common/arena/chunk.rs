use std::alloc::Layout;
use std::cell::Cell;
use std::mem::MaybeUninit;
use std::ptr;

pub(super) struct Chunk<T> {
    /// Pointer which points to the start of a memory block.
    ptr: *mut MaybeUninit<T>,
    /// Capacity of chunk.
    capacity: usize,
    /// Chunk used.
    ///
    /// [`Cell`] for internal mutability of [`Chunk::inc_used`].
    used: Cell<usize>,
}

impl<T> Chunk<T> {
    pub(super) fn new(capacity: usize) -> Self {
        // Ensure the capacity not equal to 0,
        // Because allocating a 0-sized memory will cause a UB
        let capacity = if capacity == 0 { 1 } else { capacity };
        let layout = Layout::array::<MaybeUninit<T>>(capacity).unwrap();
        let ptr = unsafe { std::alloc::alloc(layout) as *mut MaybeUninit<T> };
        if ptr.is_null() {
            // This will directly panic
            std::alloc::handle_alloc_error(layout)
        }
        Chunk {
            ptr,
            capacity,
            used: Cell::new(0),
        }
    }

    pub(super) unsafe fn current_ptr(&self) -> *mut T {
        unsafe { self.slot_ptr(self.used.get()) }
    }

    /// Get raw pointer of index-th element.
    unsafe fn slot_ptr(&self, index: usize) -> *mut T {
        unsafe { self.ptr.add(index) as *mut T }
    }

    pub(super) fn used(&self) -> usize {
        self.used.get()
    }

    pub(super) fn is_full(&self) -> bool {
        self.used.get() >= self.capacity
    }

    pub(super) fn inc_used(&self) {
        self.used.set(self.used() + 1)
    }

    /// Release all `T` elements and deallocate memory.
    unsafe fn destroy(&mut self) {
        let used = self.used();
        for i in 0..used {
            unsafe {
                ptr::drop_in_place(self.slot_ptr(i));
            }
        }
        let layout = Layout::array::<MaybeUninit<T>>(self.capacity).unwrap();
        unsafe {
            std::alloc::dealloc(self.ptr as *mut u8, layout);
        }
    }
}

unsafe impl<#[may_dangle] T> Drop for Chunk<T> {
    fn drop(&mut self) {
        unsafe { self.destroy() }
    }
}

unsafe impl<T: Send> Send for Chunk<T> {}
