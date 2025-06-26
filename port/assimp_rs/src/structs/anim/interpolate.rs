use super::anim::AiMeshKey;
use crate::structs::key::{AiQuatKey, AiVectorKey};
use glam::{Quat, Vec3};
pub trait Interpolate {
    fn interpolate(&mut self, a: Self, b: Self, d: f32);
}

impl Interpolate for Quat {
    #[inline]
    fn interpolate(&mut self, a: Self, b: Self, d: f32) {
        *self = a.slerp(b, d);
    }
}

impl Interpolate for u32 {
    #[inline]
    fn interpolate(&mut self, a: Self, b: Self, d: f32) {
        *self = if d > 0.5 { b } else { a };
    }
}

impl Interpolate for Vec3 {
    #[inline]
    fn interpolate(&mut self, a: Self, b: Self, d: f32) {
        *self = a.lerp(b, d);
    }
}

impl Interpolate for AiVectorKey {
    #[inline]
    fn interpolate(&mut self, a: Self, b: Self, d: f32) {
        self.value.interpolate(a.value, b.value, d);
    }
}

impl Interpolate for AiQuatKey {
    #[inline]
    fn interpolate(&mut self, a: Self, b: Self, d: f32) {
        self.value.interpolate(a.value, b.value, d);
    }
}

impl Interpolate for AiMeshKey {
    #[inline]
    fn interpolate(&mut self, a: Self, b: Self, d: f32) {
        self.value.interpolate(a.value, b.value, d);
    }
}
