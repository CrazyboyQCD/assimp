use super::anim::AiMeshKey;
use crate::{
    AiReal,
    structs::key::{AiQuatKey, AiVectorKey},
    utils::float_precision::{Quat, Vec3},
};
pub trait Interpolate {
    fn interpolate(&mut self, a: Self, b: Self, d: AiReal);
}

impl Interpolate for Quat {
    #[inline]
    fn interpolate(&mut self, a: Self, b: Self, d: AiReal) {
        *self = a.slerp(b, d);
    }
}

impl Interpolate for u32 {
    #[inline]
    fn interpolate(&mut self, a: Self, b: Self, d: AiReal) {
        *self = if d > 0.5 { b } else { a };
    }
}

impl Interpolate for Vec3 {
    #[inline]
    fn interpolate(&mut self, a: Self, b: Self, d: AiReal) {
        *self = a.lerp(b, d);
    }
}

impl Interpolate for AiVectorKey {
    #[inline]
    fn interpolate(&mut self, a: Self, b: Self, d: AiReal) {
        self.value.interpolate(a.value, b.value, d);
    }
}

impl Interpolate for AiQuatKey {
    #[inline]
    fn interpolate(&mut self, a: Self, b: Self, d: AiReal) {
        self.value.interpolate(a.value, b.value, d);
    }
}

impl Interpolate for AiMeshKey {
    #[inline]
    fn interpolate(&mut self, a: Self, b: Self, d: AiReal) {
        self.value.interpolate(a.value, b.value, d);
    }
}
