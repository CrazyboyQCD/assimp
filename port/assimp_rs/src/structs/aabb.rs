use crate::utils::float_precision::Vec3;

#[derive(Debug, Clone, Default)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }
}
