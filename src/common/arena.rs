use std::alloc::Layout;
use std::cell::{Cell, UnsafeCell};
use std::mem::MaybeUninit;
use std::ptr;

struct Chunk<T> {
    ptr: *mut MaybeUninit<T>,
    capacity: usize,
    used: Cell<usize>,
}

impl<T> Chunk<T> {
    fn new(capacity: usize) -> Self {
        let layout = Layout::array::<MaybeUninit<T>>(capacity).unwrap();
        let ptr = unsafe { std::alloc::alloc(layout) as *mut MaybeUninit<T> };
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        Chunk {
            ptr,
            capacity,
            used: Cell::new(0),
        }
    }

    unsafe fn slot_ptr(&self, index: usize) -> *mut T {
        unsafe { self.ptr.add(index) as *mut T }
    }

    fn used(&self) -> usize {
        self.used.get()
    }

    fn inc_used(&self) {
        self.used.set(self.used() + 1)
    }

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

pub struct Arena<T> {
    chunks: UnsafeCell<Vec<Chunk<T>>>,
    chunk_capacity: usize,
    current_ptr: UnsafeCell<*mut T>,
}

const DEFAULT_CHUNK_SIZE: usize = 4096;

impl<T> Arena<T> {
    pub fn with_chunk_capacity(cap: usize) -> Self {
        let mut chunks = Vec::new();
        let first = Chunk::new(cap);
        let start_ptr = unsafe { first.slot_ptr(0) };
        chunks.push(first);
        Arena {
            chunks: UnsafeCell::new(chunks),
            chunk_capacity: cap,
            current_ptr: UnsafeCell::new(start_ptr),
        }
    }

    fn size_to_capacity(size: usize) -> usize {
        std::cmp::max(size / std::mem::size_of::<T>(), 1)
    }

    pub fn with_chunk_size(size: usize) -> Self {
        Self::with_chunk_capacity(Arena::<T>::size_to_capacity(size))
    }

    pub fn new() -> Self {
        Self::with_chunk_size(DEFAULT_CHUNK_SIZE)
    }

    fn current_chunk(&self) -> &Chunk<T> {
        let chunks = unsafe { &*self.chunks.get() };
        &chunks[chunks.len() - 1]
    }

    fn debug_check_current_ptr(&self) {
        unsafe {
            debug_assert_eq!(
                *self.current_ptr.get(),
                self.current_chunk().slot_ptr(self.current_chunk().used())
            );
        }
    }

    pub fn alloc(&self, value: T) -> &mut T {
        self.debug_check_current_ptr();
        let chunk = self.current_chunk();
        if chunk.used() >= chunk.capacity {
            let new_chunk = Chunk::new(self.chunk_capacity);
            unsafe {
                let new_start = new_chunk.slot_ptr(0);
                (*self.chunks.get()).push(new_chunk);
                *self.current_ptr.get() = new_start;
            }
        }
        self.current_chunk().inc_used();
        let ptr = unsafe { *self.current_ptr.get() };
        unsafe {
            ptr.write(value);
            *self.current_ptr.get() = ptr.add(1);
        }
        unsafe { &mut *ptr }
    }
}

impl<T> Drop for Arena<T> {
    fn drop(&mut self) {
        let chunks = unsafe { &mut *self.chunks.get() };
        for chunk in chunks {
            unsafe {
                chunk.destroy();
            }
        }
    }
}

unsafe impl<T: Send> Send for Arena<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    mod size_to_capacity {
        use super::*;

        type BigDataType = [usize; 100]; // size = 800

        macro_rules! test_size_to_cap {
            ($name:ident, $size:expr, $cap:expr) => {
                #[test]
                fn $name() {
                    assert_eq!(Arena::<BigDataType>::size_to_capacity($size), $cap)
                }
            };
        }

        test_size_to_cap!(size_gt_data, 2000, 2);
        test_size_to_cap!(size_eq_data, 800, 1);
        test_size_to_cap!(size_lt_data, 20, 1);
        test_size_to_cap!(size_eq_0, 0, 1);
    }
}
