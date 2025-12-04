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

//! Defines string c-ffi types for the library

use alloc::{borrow::ToOwned, boxed::Box, string::String};
use core::{
    ffi::{CStr, c_char},
    mem,
    ptr::NonNull,
    slice,
};

const AI_MAXLEN: usize = 1024;

/// ## A string in the FFI.
#[repr(C)]
pub struct AiStringFFI {
    /// The length of the string.
    pub length: u32, // std::pat::pattern_type!(u32 is 0..1024),
    /// The data of the string.
    pub data: [u8; AI_MAXLEN],
}

impl Default for AiStringFFI {
    fn default() -> Self {
        let mut data = [27; AI_MAXLEN];
        data[0] = 0;
        Self { length: 0, data }
    }
}

impl Clone for AiStringFFI {
    fn clone(&self) -> Self {
        Self {
            length: self.length,
            data: self.data,
        }
    }
}

impl AiStringFFI {
    /// ## Create a new AiStringFFI from another AiStringFFI.
    pub fn new_from_other(other: &Self) -> Self {
        let length = (other.length as usize).min(AI_MAXLEN - 1);
        let mut data = [0; AI_MAXLEN];
        data[..length].copy_from_slice(&other.data[..length]);
        Self {
            length: length as u32,
            data,
        }
    }

    /// ## Set the AiStringFFI from a C string.
    pub unsafe fn set(&mut self, sz: *const c_char, len: usize) {
        if sz.is_null() {
            return;
        }
        let s = unsafe { slice::from_raw_parts(sz as *const u8, len) };
        let len = s.iter().position(|b| *b == 0).unwrap_or(len);
        let length = len.min(AI_MAXLEN - 1);
        self.length = length as u32;
        self.data[..length].copy_from_slice(&s[..length]);
    }

    /// ## Append a C string to the AiStringFFI.
    pub unsafe fn append(&mut self, s: *const c_char) {
        let s = unsafe {
            if s.is_null() {
                c""
            } else {
                CStr::from_ptr(s.cast())
            }
        };
        let s = s.to_bytes();
        let len = s.len();
        let new_length = len + self.length as usize;
        if new_length >= AI_MAXLEN {
            return;
        }
        self.data[self.length as usize..new_length].copy_from_slice(s);
        self.length = new_length as u32;
    }

    /// ## Clear the AiStringFFI.
    pub fn clear(&mut self) {
        self.length = 0;
        self.data[0] = 0;
        self.data[1..].fill(27);
    }

    /// ## Check if the AiStringFFI is empty.
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// ## Check if the AiStringFFI is null.
    pub fn is_null(&self) -> bool {
        self.length == 0
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data[..self.length as usize]
    }
}

impl From<&str> for AiStringFFI {
    fn from(value: &str) -> Self {
        value.as_bytes().into()
    }
}

impl From<&String> for AiStringFFI {
    fn from(value: &String) -> Self {
        value.as_bytes().into()
    }
}

impl From<String> for AiStringFFI {
    fn from(value: String) -> Self {
        value.as_bytes().into()
    }
}

impl From<&[u8]> for AiStringFFI {
    fn from(value: &[u8]) -> Self {
        let length = value.len().min(AI_MAXLEN - 1);
        let mut data = [0; AI_MAXLEN];
        data[..length].copy_from_slice(&value[..length]);
        Self {
            length: length as u32,
            data,
        }
    }
}

impl PartialEq<AiStringFFI> for AiStringFFI {
    fn eq(&self, other: &AiStringFFI) -> bool {
        if self.length == other.length {
            self.data[..self.length as usize] == other.data[..other.length as usize]
        } else {
            false
        }
    }
}

impl TryFrom<&AiStringFFI> for String {
    type Error = core::str::Utf8Error;
    fn try_from(value: &AiStringFFI) -> Result<Self, Self::Error> {
        match str::from_utf8(&value.data[..value.length as usize]) {
            Ok(s) => Ok(s.to_owned()),
            Err(e) => Err(e),
        }
    }
}

impl<'a> TryFrom<&'a AiStringFFI> for &'a str {
    type Error = core::str::Utf8Error;
    fn try_from(value: &'a AiStringFFI) -> Result<Self, Self::Error> {
        match str::from_utf8(&value.data[..value.length as usize]) {
            Ok(s) => Ok(s),
            Err(e) => Err(e),
        }
    }
}

/// ## Release the AiStringFFI.
///
/// Pass mutable reference of the raw pointer and set it to null to avoid double free.
///
/// # Safety
///
/// Caller must make Sure that the pointer is passed from the original rust allocation.
pub unsafe extern "C" fn release_ai_string_rs(value: *mut *mut AiStringFFI) {
    if let Some(value) = unsafe { value.as_mut() } {
        let ptr = mem::take(value);
        if let Some(ptr) = NonNull::new(ptr) {
            let _ = unsafe { Box::from_raw(ptr.as_ptr()) };
        }
    }
}
