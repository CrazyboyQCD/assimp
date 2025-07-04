use anim::{AiMeshAnim, AiMeshMorphAnim, AiNodeAnim};

pub mod anim;
pub mod interpolate;

#[derive(Debug, Clone, Default)]
pub struct AiAnimation {
    /* The name of the animation. If the modeling package this data was
     * exported from does support only a single animation channel, this
     * name is usually empty (length is zero).
     */
    pub name: String,
    // Duration of the animation in ticks
    pub duration: f64,
    // Ticks per second. Zero (0.000... ticks/second) if not
    // specified in the imported file
    pub ticks_per_second: f64,
    /* Node animation channels. Each channel
    affects a single node.
    */
    pub channels: Vec<AiNodeAnim>,
    /* The mesh animation channels. Each channel
    affects a single mesh.
    The array is m_num_mesh_channels in size
    (maybe refine to a derivative of usize?)
    */
    pub mesh_channels: Vec<AiMeshAnim>,
    /* The morph mesh animation channels. Each channel affects a single mesh.
    The array is mNumMorphMeshChannels in size.
    */
    pub morph_mesh_channels: Vec<AiMeshMorphAnim>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AiAnimInterpolation {
    Step,
    #[default]
    Linear,
    SphericalLinear,
    CubicSpline,
}
