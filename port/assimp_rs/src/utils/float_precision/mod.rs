#[cfg(feature = "double_precision")]
mod precision {
    pub use glam::DMat3 as Mat3;
    pub use glam::DMat4 as Mat4;
    pub use glam::DQuat as Quat;
    pub use glam::DVec2 as Vec2;
    pub use glam::DVec3 as Vec3;
    pub use glam::DVec4 as Vec4;
}
#[cfg(not(feature = "double_precision"))]
mod precision {
    pub use glam::Mat3;
    pub use glam::Mat4;
    pub use glam::Quat;
    pub use glam::Vec2;
    pub use glam::Vec3;
    pub use glam::Vec4;
}

pub use precision::*;
