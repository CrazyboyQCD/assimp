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

//! Assimp Library
#![allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::unit_arg,
    clippy::write_with_newline
)]
#![warn(
    clippy::alloc_instead_of_core,
    clippy::std_instead_of_alloc,
    clippy::std_instead_of_core
)]
#![deny(unused_must_use)]
// #![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
#[cfg(not(any(feature = "glam_no_std_libm", feature = "foldhash")))]
compile_error!("glam_no_std_libm and foldhash features should be enabled in no_std environment");

#[cfg(feature = "compression")]
#[cfg(not(any(feature = "zlib_c_allocator", feature = "zlib_rust_allocator")))]
compile_error!(
    "Either zlib_c_allocator or zlib_rust_allocator feature should be enabled when compression feature is enabled"
);

#[cfg(feature = "compression")]
#[cfg(all(feature = "zlib_c_allocator", feature = "zlib_rust_allocator"))]
compile_error!(
    "zlib_c_allocator and zlib_rust_allocator feature should not be enabled at the same time"
);

#[cfg(not(feature = "std"))]
#[cfg(feature = "compression")]
#[cfg(feature = "zlib_rust_allocator")]
compile_error!(
    "zlib_rust_allocator feature should not be enabled in no_std environment, becuase this will bring back std dependency"
);

#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod assets;
pub mod ffi;
pub mod io;
pub mod structs;
pub mod utils;

pub use self::{
    ffi::{
        ai_release_export_data_blob_rs, release_ai_metadata_rs, release_ai_node_from_root_rs,
        release_ai_string_rs,
    },
    io::utils::float_precision::{AiMat3, AiMat4, AiQuat, AiReal, AiVec2, AiVec3, AiVec4},
};
