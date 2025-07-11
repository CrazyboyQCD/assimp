use crate::utils::float_precision::Vec3;

pub struct AiRay {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl AiRay {
    pub const fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { origin, direction }
    }
}

impl Default for AiRay {
    fn default() -> Self {
        Self::new(Vec3::ZERO, Vec3::ZERO)
    }
}
