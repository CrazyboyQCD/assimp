/*
---------------------------------------------------------------------------
Open Asset Import Library (assimp)
---------------------------------------------------------------------------

Copyright (c) 2006-2025, assimp team

All rights reserved.

Redistribution and use of this software in source and binary forms,
with or without modification, are permitted provided that the following
conditions are met:

* Redistributions of source code must retain the above
  copyright notice, this list of conditions and the
  following disclaimer.

* Redistributions in binary form must reproduce the above
  copyright notice, this list of conditions and the
  following disclaimer in the documentation and/or other
  materials provided with the distribution.

* Neither the name of the assimp team, nor the names of its
  contributors may be used to endorse or promote products
  derived from this software without specific prior
  written permission of the assimp team.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
"AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
---------------------------------------------------------------------------
*/

//! In Rust version of assimp, we use array to represent the tree structure to avoid most of unsafe
//! pointer operations and borrowing issues.
//!
//! [`Index`] is a wrapper around `u32` that represents an index of an element in an array.
//!
//! It is used to represent the index of a node in the tree structure.

use alloc::vec::Vec;
use core::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    marker,
};

/// A wrapper around `u32` that represents an index of an element in an array.
///
/// It is used to represent the index of a node in the tree structure.
#[derive(Debug)]
pub struct Index<T>(u32, marker::PhantomData<fn() -> T>);

impl<T> Default for Index<T> {
    fn default() -> Self {
        Self::new(0)
    }
}

impl<T> PartialEq for Index<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

impl<T> Eq for Index<T> {}

impl<T> Hash for Index<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value().hash(state);
    }
}

impl<T> PartialOrd for Index<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Index<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> Index<T> {
    /// Use [`u32::MAX`] as guard index.
    pub const GUARD_INDEX: Index<T> = Index::new(u32::MAX);

    /// Use 0 as root index.
    pub const ROOT_INDEX: Index<T> = Index::new(0);

    /// Check if the index is guard index.
    pub const fn is_guard(&self) -> bool {
        self.0 == u32::MAX
    }

    /// Push a value to the vector and return the index of the value.
    pub fn push(vec: &mut Vec<T>, value: T) -> Index<T> {
        // Length of vector will not exceed u32::MAX before OOM.
        let len = vec.len() as u32;
        vec.push(value);
        Index::new(len)
    }

    /// Get the value at the index.
    pub fn get(self, vec: &[T]) -> Option<&T> {
        vec.get(self.value())
    }

    /// Get the value at the index without bounds checking.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the index does not exceed the length of the vector.
    pub unsafe fn get_unchecked(self, vec: &[T]) -> &T {
        unsafe { vec.get_unchecked(self.value()) }
    }

    /// Get the mutable reference to the value at the index.
    pub fn get_mut(self, vec: &mut [T]) -> Option<&mut T> {
        vec.get_mut(self.value())
    }

    /// Get the mutable reference to the value at the index without bounds checking.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the index does not exceed the length of the vector.
    pub unsafe fn get_mut_unchecked(self, vec: &mut [T]) -> &mut T {
        unsafe { vec.get_unchecked_mut(self.value()) }
    }
}
impl<T> Index<T> {
    /// Creates a new [`Index`] representing an offset into an array containing `T`.
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
