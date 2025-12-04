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

//! Defines encoding error for the library

use core::char::DecodeUtf16Error;

use thiserror::Error;

/// Encoding conversion errors
#[derive(Debug, Error)]
pub enum EncodingError {
    #[error("Not valid UTF-32 length: {0}")]
    NotValidUtf32Length(usize),

    #[error("Not valid UTF-32 BE")]
    NotValidUtf32Be,

    #[error("Not valid UTF-32 LE")]
    NotValidUtf32Le,

    #[error("Not valid UTF-16 length: {0}")]
    NotValidUtf16Length(usize),

    #[error("Not valid UTF-16 BE: {0}")]
    NotValidUtf16Be(DecodeUtf16Error),

    #[error("Not valid UTF-16 LE: {0}")]
    NotValidUtf16Le(DecodeUtf16Error),

    #[error("Not valid UTF-8")]
    NotValidUtf8,

    #[error("Not valid code point: {0}")]
    NotValidCodePoint(u32),

    #[error("Unknown encoding")]
    UnknownEncoding,

    #[error("UTF8 code {0} {1} can not be converted into ISA-8859-1.")]
    NotValidUtf8ToIso8859_1(u8, u8),

    #[error("UTF8 code but only one character remaining")]
    NotValidUtf8OnlyOneCharacterRemaining,
}
