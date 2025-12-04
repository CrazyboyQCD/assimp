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

//! Defines the data structures in which the imported animations
//! are returned.

use alloc::{string::String, vec::Vec};

use anim::{AiMeshAnim, AiMeshMorphAnim, AiNodeAnim};

pub mod anim;
pub mod interpolate;

/// An animation consists of key-frame data for a number of nodes.
///
/// For each node affected by the animation a separate series of data is given.
#[derive(Clone, Debug, Default)]
pub struct AiAnimation {
    /// ### The name of the animation.
    ///
    /// If the modeling package this data was
    /// exported from does support only a single animation channel, this
    /// name is usually empty (length is zero).
    pub name: String,

    /// ### Duration of the animation in ticks
    pub duration: f64,

    /// ### Ticks per second.
    ///
    /// Zero (0.000... ticks/second) if not
    /// specified in the imported file
    pub ticks_per_second: f64,

    /// ### Node animation channels.
    ///
    /// Each channel
    /// affects a single node.
    pub channels: Vec<AiNodeAnim>,

    /// ### The mesh animation channels.
    ///
    /// Each channel
    /// affects a single mesh.
    /// The array is m_num_mesh_channels in size
    /// (maybe refine to a derivative of usize?)
    pub mesh_channels: Vec<AiMeshAnim>,

    /// ### The morph mesh animation channels.
    ///
    /// Each channel affects a single mesh.
    /// The array is mNumMorphMeshChannels in size.
    pub morph_mesh_channels: Vec<AiMeshMorphAnim>,
}

/// The interpolation setting of a key
///
/// The interpolation setting of a key is used to determine how the value of the key is interpolated
/// between the previous and next key.
///
/// - Step: The value of the nearest key is used without interpolation
/// - Linear: The value of the nearest two keys is linearly extrapolated for the current time value.
/// - SphericalLinear: The value of the nearest two keys is spherically linearly extrapolated for
///   the current time value.
/// - CubicSpline: The value of the nearest two keys is cubically extrapolated for the current time
///   value.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AiAnimInterpolation {
    /// The value of the nearest key is used without interpolation
    Step,
    /// The value of the nearest two keys is linearly extrapolated for the current time value.
    #[default]
    Linear,
    /// The value of the nearest two keys is spherically linearly extrapolated for the current time
    /// value.
    SphericalLinear,
    /// The value of the nearest two keys is cubically extrapolated for the current time value.
    CubicSpline,
}
