use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};

use crate::common::arena::chunk::Chunk;

mod chunk;

pub struct Arena<T> {
    /// Chunks collection.
    ///
    /// Should be in **first** field place to make sure it is **first** dropped,
    /// so that no dangling pointer is produced, i.e. [`Arena::current_ptr`].
    chunks: UnsafeCell<Vec<Chunk<T>>>,
    chunk_capacity: usize,
}

const DEFAULT_CHUNK_SIZE: usize = 4096;

impl<T> Arena<T> {
    pub fn with_chunk_capacity(cap: usize) -> Self {
        Arena {
            chunks: UnsafeCell::new(vec![Chunk::new(cap)]),
            chunk_capacity: cap,
        }
    }

    /// Convert provided chunk size to capacity.
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
        let len = chunks.len();
        &chunks[len - 1]
    }

    pub fn alloc(&self, value: T) -> ArenaPtrMut<'_, T> {
        if self.current_chunk().is_full() {
            let new_chunk = Chunk::new(self.chunk_capacity);
            unsafe {
                (*self.chunks.get()).push(new_chunk);
            }
        }
        let chunk = self.current_chunk();
        let ptr = unsafe { &mut *chunk.current_ptr() };
        *ptr = value;
        chunk.inc_used();
        ArenaPtrMut(ptr)
    }
}

unsafe impl<T: Send> Send for Arena<T> {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArenaPtr<'arena, T>(&'arena T);

impl<'arena, T> Deref for ArenaPtr<'arena, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

#[derive(Debug, PartialEq)]
pub struct ArenaPtrMut<'arena, T>(&'arena mut T);

impl<'arena, T> Deref for ArenaPtrMut<'arena, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'arena, T> DerefMut for ArenaPtrMut<'arena, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl<'arena, T> From<ArenaPtrMut<'arena, T>> for ArenaPtr<'arena, T> {
    fn from(value: ArenaPtrMut<'arena, T>) -> Self {
        Self(value.0)
    }
}

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
