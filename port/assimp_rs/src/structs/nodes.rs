//! Dealing with tree structures, inspired by [`gltf-json`](https://github.com/gltf-rs/gltf/blob/main/gltf-json)

use core::{any, marker};

#[derive(Debug)]
pub struct Index<T>(u32, marker::PhantomData<fn() -> T>);

impl<T> Default for Index<T> {
    fn default() -> Self {
        Self::new(0)
    }
}

impl<T> Index<T> {
    pub const GUARD_INDEX: Index<T> = Index::new(0);

    pub fn is_exist(self) -> bool {
        self.value() != 0
    }
    pub fn push(vec: &mut Vec<T>, value: T) -> Index<T> {
        let len = vec.len();
        let Ok(index): Result<u32, _> = len.try_into() else {
            panic!(
                "vector of {ty} has {len} elements, which exceeds the Index limit",
                ty = any::type_name::<T>(),
            );
        };

        vec.push(value);
        Index::new(index)
    }

    pub fn get(self, vec: &[T]) -> Option<&T> {
        if self.value() < vec.len() {
            // SAFETY: self is not guard index and self.value() is guaranteed to be less than vec.len()
            Some(unsafe { self.get_unchecked(vec) })
        } else {
            None
        }
    }

    pub unsafe fn get_unchecked(self, vec: &[T]) -> &T {
        unsafe { vec.get_unchecked(self.value()) }
    }

    pub fn get_mut(self, vec: &mut [T]) -> Option<&mut T> {
        if self.value() < vec.len() {
            // SAFETY: self is not guard index and self.value() is guaranteed to be less than vec.len()
            Some(unsafe { self.get_mut_unchecked(vec) })
        } else {
            None
        }
    }

    pub unsafe fn get_mut_unchecked(self, vec: &mut [T]) -> &mut T {
        unsafe { vec.get_unchecked_mut(self.value()) }
    }
}
impl<T> Index<T> {
    /// Creates a new `Index` representing an offset into an array containing `T`.
    #[inline(always)]
    pub const fn new(value: u32) -> Self {
        Index(value, marker::PhantomData)
    }

    /// Returns the internal offset value.
    #[inline(always)]
    pub const fn value(&self) -> usize {
        self.0 as usize
    }
}

impl<T> Clone for Index<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Index<T> {}

// impl<T> core::ops::Index<Index<T>> for Vec<T> {
//     type Output = T;

//     fn index(&self, index: Index<T>) -> &Self::Output {
//         &self[index.value()]
//     }
// }
