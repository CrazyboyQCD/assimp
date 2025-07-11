#[cfg(feature = "double_precision")]
mod precision {
    pub type AiReal = f64;
    pub use glam::{
        DMat3 as Mat3, DMat4 as Mat4, DQuat as Quat, DVec2 as Vec2, DVec3 as Vec3, DVec4 as Vec4,
    };
    pub const PRECISION: usize = 17;
}
#[cfg(not(feature = "double_precision"))]
mod precision {
    pub type AiReal = f32;
    pub use glam::{Mat3, Mat4, Quat, Vec2, Vec3, Vec4};
    pub const PRECISION: usize = 9;
}

pub use precision::*;
