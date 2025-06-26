use glam::Vec2;

/// @brief Defines how an UV channel is transformed.
///
/// This is just a helper structure for the #AI_MATKEY_UVTRANSFORM key.
/// See its documentation for more details.
///
/// Typically you'll want to build a matrix of this information. However,
/// we keep separate scaling/translation/rotation values to make it
/// easier to process and optimize UV transformations internally.
///
#[derive(Default, Clone, Debug)]
pub struct AiUVTransform {
    /// Translation on the u and v axes.
    ///
    /// The default value is (0|0).
    pub translation: Vec2,

    /// Scaling on the u and v axes.
    ///
    /// The default value is (1|1).
    pub scaling: Vec2,

    /// Rotation - in counter-clockwise direction.
    ///
    /// The rotation angle is specified in radians. The
    /// rotation center is 0.5f|0.5f. The default value
    /// 0.f.
    pub rotation: f32,
}
