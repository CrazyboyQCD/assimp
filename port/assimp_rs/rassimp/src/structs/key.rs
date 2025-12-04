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
//! are returned: [`AiVectorKey`], [`AiQuatKey`], [`AiMeshMorphKey`].

use alloc::vec::Vec;
use core::cmp::Ordering;

use crate::{AiQuat, AiVec3, structs::animation::AiAnimInterpolation};

/// A time-value pair specifying a certain 3D vector for the given time.
#[derive(Clone, Copy, Debug, Default)]
pub struct AiVectorKey {
    /// The time of this key
    pub time: f64,

    /// The value of this key
    pub value: AiVec3,

    /// The interpolation setting of this key
    pub interpolation: AiAnimInterpolation,
}

impl AiVectorKey {
    /// Constructor for the vector key.
    pub const fn new(time: f64, value: AiVec3) -> Self {
        Self {
            time,
            value,
            interpolation: AiAnimInterpolation::Linear,
        }
    }
}

impl PartialEq for AiVectorKey {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for AiVectorKey {}

impl Ord for AiVectorKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time
            .partial_cmp(&other.time)
            // Treat NaN as greater than any other value
            .unwrap_or(Ordering::Greater)
    }
}

impl PartialOrd for AiVectorKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A time-value pair specifying a rotation for the given time.
///
/// Rotations are expressed with quaternions.
#[derive(Clone, Copy, Debug, Default)]
pub struct AiQuatKey {
    /// The time of this key
    pub time: f64,

    /// The value of this key
    pub value: AiQuat,

    /// The interpolation setting of this key
    pub interpolation: AiAnimInterpolation,
}

impl AiQuatKey {
    /// Constructor for the quaternion key.
    pub const fn new(time: f64, value: AiQuat) -> Self {
        Self {
            time,
            value,
            interpolation: AiAnimInterpolation::Linear,
        }
    }
}

impl PartialEq for AiQuatKey {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for AiQuatKey {}

impl Ord for AiQuatKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time
            .partial_cmp(&other.time)
            .unwrap_or(Ordering::Greater)
    }
}

impl PartialOrd for AiQuatKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A time-value pair specifying a certain 3D vector for the given time.
#[derive(Clone, Copy, Debug)]
pub struct MeshMorphKeyValues {
    /// The value of the key.
    pub value: u32,

    /// The weight of the key.
    pub weight: f64,
}

/// Binds a morph anim mesh to a specific point in time.
#[allow(unused)]
#[derive(Clone, Debug, Default)]
pub struct AiMeshMorphKey {
    /// The time of this key
    pub time: f64,

    /// The values at the time of this key
    ///
    /// - values: index of attachment mesh to apply weight at the same position in weights
    pub values: Vec<u32>,

    /// The weights at the time of this key
    ///
    /// - weights: weight to apply to the blend shape index at the same position in values
    pub weights: Vec<f64>,
}

impl AiMeshMorphKey {
    /// Constructor for the mesh morph key.
    pub fn new(num_values_and_weights: u32) -> Self {
        Self {
            time: 0.0,
            values: vec![0; num_values_and_weights as usize],
            weights: vec![0.0; num_values_and_weights as usize],
        }
    }

    /// Returns the number of values and weights at the time of this key
    pub const fn num_values_and_weights(&self) -> usize {
        self.values.len()
    }
}
