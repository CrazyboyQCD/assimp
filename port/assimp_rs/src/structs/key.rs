use std::cmp::Ordering;

use crate::{
    structs::anim::AiAnimInterpolation,
    utils::float_precision::{Quat, Vec3},
};

/** A time-value pair specifying a certain 3D vector for the given time. */
#[derive(Debug, Clone, Copy, Default)]
pub struct AiVectorKey {
    /** The time of this key */
    pub time: f64,

    /** The value of this key */
    pub value: Vec3,

    /** The interpolation setting of this key */
    pub interpolation: AiAnimInterpolation,
}

impl AiVectorKey {
    pub fn new(time: f64, value: Vec3) -> Self {
        Self {
            time,
            value,
            interpolation: AiAnimInterpolation::default(),
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

/** A time-value pair specifying a rotation for the given time.
 *  Rotations are expressed with quaternions. */
#[derive(Debug, Clone, Copy, Default)]
pub struct AiQuatKey {
    /** The time of this key */
    pub time: f64,

    /** The value of this key */
    pub value: Quat,

    /** The interpolation setting of this key */
    pub interpolation: AiAnimInterpolation,
}

impl AiQuatKey {
    pub fn new(time: f64, value: Quat) -> Self {
        Self {
            time,
            value,
            interpolation: AiAnimInterpolation::default(),
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

#[derive(Debug, Clone, Copy)]
pub struct MeshMorphKeyValues {
    pub value: u32,
    pub weight: f64,
}

/** Binds a morph anim mesh to a specific point in time. */
#[allow(unused)]
#[derive(Debug, Clone, Default)]
pub struct AiMeshMorphKey {
    /** The time of this key */
    pub time: f64,

    /** The values and weights at the time of this key
     *   - values: index of attachment mesh to apply weight at the same position in weights
     *   - weights: weight to apply to the blend shape index at the same position in values
     */
    pub values: Box<[u32]>,
    pub weights: Box<[f64]>,
}

impl AiMeshMorphKey {
    pub fn new(num_values_and_weights: u32) -> Self {
        Self {
            time: 0.0,
            values: vec![0; num_values_and_weights as usize].into_boxed_slice(),
            weights: vec![0.0; num_values_and_weights as usize].into_boxed_slice(),
        }
    }

    pub fn num_values_and_weights(&self) -> usize {
        self.values.len()
    }
}
