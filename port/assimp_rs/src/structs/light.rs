use super::color::Color3D;
use crate::utils::float_precision::{Vec2, Vec3};

// ---------------------------------------------------------------------------
/** Enumerates all supported types of light sources.
 */
#[derive(Default, Clone, Debug)]
pub enum LightType {
    #[default]
    Undefined = 0x0,

    /// A directional light source has a well-defined direction
    /// but is infinitely far away. That's quite a good
    /// approximation for sun light.
    Directional = 0x1,

    /// A point light source has a well-defined position
    /// in space but no direction - it emits light in all
    /// directions. A normal bulb is a point light.
    Point = 0x2,

    /// A spot light source emits light in a specific
    /// angle. It has a position and a direction it is pointing to.
    /// A good example for a spot light is a light spot in
    /// sport arenas.
    Spot = 0x3,

    /// The generic light level of the world, including the bounces
    /// of all other light sources.
    /// Typically, there's at most one ambient light in a scene.
    /// This light type doesn't have a valid position, direction, or
    /// other properties, just a color.
    Ambient = 0x4,

    /// An area light is a rectangle with predefined size that uniformly
    /// emits light from one of its sides. The position is center of the
    /// rectangle and direction is its normal vector.
    Area = 0x5,
}

#[derive(Default, Clone, Debug)]
pub struct AiLight {
    /** The name of the light source.
     *
     *  There must be a node in the scene-graph with the same name.
     *  This node specifies the position of the light in the scene
     *  hierarchy and can be animated.
     */
    pub name: String,

    /** The type of the light source.
     *
     * aiLightSource_UNDEFINED is not a valid value for this member.
     */
    pub light_type: LightType,

    /** Position of the light source in space. Relative to the
     *  transformation of the node corresponding to the light.
     *
     *  The position is undefined for directional lights.
     */
    pub position: Vec3,

    /** Direction of the light source in space. Relative to the
     *  transformation of the node corresponding to the light.
     *
     *  The direction is undefined for point lights. The vector
     *  may be normalized, but it needn't.
     */
    pub direction: Vec3,

    /** Up direction of the light source in space. Relative to the
     *  transformation of the node corresponding to the light.
     *
     *  The direction is undefined for point lights. The vector
     *  may be normalized, but it needn't.
     */
    pub up: Vec3,

    /** Constant light attenuation factor.
     *
     *  The intensity of the light source at a given distance 'd' from
     *  the light's position is
     *  @code
     *  Atten = 1/( att0 + att1 * d + att2 * d*d)
     *  @endcode
     *  This member corresponds to the att0 variable in the equation.
     *  Naturally undefined for directional lights.
     */
    pub attenuation_constant: f32,

    /** Linear light attenuation factor.
     *
     *  The intensity of the light source at a given distance 'd' from
     *  the light's position is
     *  @code
     *  Atten = 1/( att0 + att1 * d + att2 * d*d)
     *  @endcode
     *  This member corresponds to the att1 variable in the equation.
     *  Naturally undefined for directional lights.
     */
    pub attenuation_linear: f32,

    /** Quadratic light attenuation factor.
     *
     *  The intensity of the light source at a given distance 'd' from
     *  the light's position is
     *  @code
     *  Atten = 1/( att0 + att1 * d + att2 * d*d)
     *  @endcode
     *  This member corresponds to the att2 variable in the equation.
     *  Naturally undefined for directional lights.
     */
    pub attenuation_quadratic: f32,

    /** Diffuse color of the light source
     *
     *  The diffuse light color is multiplied with the diffuse
     *  material color to obtain the final color that contributes
     *  to the diffuse shading term.
     */
    pub color_diffuse: Color3D,

    /** Specular color of the light source
     *
     *  The specular light color is multiplied with the specular
     *  material color to obtain the final color that contributes
     *  to the specular shading term.
     */
    pub color_specular: Color3D,

    /** Ambient color of the light source
     *
     *  The ambient light color is multiplied with the ambient
     *  material color to obtain the final color that contributes
     *  to the ambient shading term. Most renderers will ignore
     *  this value it, is just a remaining of the fixed-function pipeline
     *  that is still supported by quite many file formats.
     */
    pub color_ambient: Color3D,

    /** Inner angle of a spot light's light cone.
     *
     *  The spot light has maximum influence on objects inside this
     *  angle. The angle is given in radians. It is 2PI for point
     *  lights and undefined for directional lights.
     */
    pub angle_inner_cone: f32,

    /** Outer angle of a spot light's light cone.
     *
     *  The spot light does not affect objects outside this angle.
     *  The angle is given in radians. It is 2PI for point lights and
     *  undefined for directional lights. The outer angle must be
     *  greater than or equal to the inner angle.
     *  It is assumed that the application uses a smooth
     *  interpolation between the inner and the outer cone of the
     *  spot light.
     */
    pub angle_outer_cone: f32,

    /** Size of area light source. */
    pub size: Vec2,
}
