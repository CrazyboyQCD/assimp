use crate::utils::float_precision::Vec3;

// ---------------------------------------------------------------------------
/** Helper structure to describe a virtual camera.
 *
 * Cameras have a representation in the node graph and can be animated.
 * An important aspect is that the camera itself is also part of the
 * scene-graph. This means, any values such as the look-at vector are not
 * *absolute*, they're <b>relative</b> to the coordinate system defined
 * by the node which corresponds to the camera. This allows for camera
 * animations. For static cameras parameters like the 'look-at' or 'up' vectors
 * are usually specified directly in aiCamera, but beware, they could also
 * be encoded in the node transformation. The following (pseudo)code sample
 * shows how to do it: <br><br>
 * @code
 * // Get the camera matrix for a camera at a specific time
 * // if the node hierarchy for the camera does not contain
 * // at least one animated node this is a static computation
 * get-camera-matrix (node sceneRoot, camera cam) : matrix
 * {
 *    node   cnd = find-node-for-camera(cam)
 *    matrix cmt = identity()
 *
 *    // as usual - get the absolute camera transformation for this frame
 *    for each node nd in hierarchy from sceneRoot to cnd
 *      matrix cur
 *      if (is-animated(nd))
 *         cur = eval-animation(nd)
 *      else cur = nd->mTransformation;
 *      cmt = mult-matrices( cmt, cur )
 *    end for
 *
 *    // now multiply with the camera's own local transform
 *    cam = mult-matrices (cam, get-camera-matrix(cmt) )
 * }
 * @endcode
 *
 * @note some file formats (such as 3DS, ASE) export a "target point" -
 * the point the camera is looking at (it can even be animated). Assimp
 * writes the target point as a subnode of the camera's main node,
 * called "<camName>.Target". However this is just additional information
 * then the transformation tracks of the camera main node make the
 * camera already look in the right direction.
 *
*/
#[derive(Default, Clone, Debug)]
pub struct AiCamera {
    /** The name of the camera.
     *
     *  There must be a node in the scenegraph with the same name.
     *  This node specifies the position of the camera in the scene
     *  hierarchy and can be animated.
     */
    pub name: Box<str>,

    /** Position of the camera relative to the coordinate space
     *  defined by the corresponding node.
     *
     *  The default value is 0|0|0.
     */
    pub position: Vec3,

    /** 'Up' - vector of the camera coordinate system relative to
     *  the coordinate space defined by the corresponding node.
     *
     *  The 'right' vector of the camera coordinate system is
     *  the cross product of  the up and lookAt vectors.
     *  The default value is 0|1|0. The vector
     *  may be normalized, but it needn't.
     */
    pub up: Vec3,

    /** 'LookAt' - vector of the camera coordinate system relative to
     *  the coordinate space defined by the corresponding node.
     *
     *  This is the viewing direction of the user.
     *  The default value is 0|0|1. The vector
     *  may be normalized, but it needn't.
     */
    pub look_at: Vec3,

    /** Horizontal field of view angle, in radians.
     *
     *  The field of view angle is the angle between the center
     *  line of the screen and the left or right border.
     *  The default value is 1/4PI.
     */
    pub horizontal_fov: f32,

    /** Distance of the near clipping plane from the camera.
     *
     * The value may not be 0.f (for arithmetic reasons to prevent
     * a division through zero). The default value is 0.1f.
     */
    pub clip_plane_near: f32,

    /** Distance of the far clipping plane from the camera.
     *
     * The far clipping plane must, of course, be further away than the
     * near clipping plane. The default value is 1000.f. The ratio
     * between the near and the far plane should not be too
     * large (between 1000-10000 should be ok) to avoid floating-point
     * inaccuracies which could lead to z-fighting.
     */
    pub clip_plane_far: f32,

    /** Screen aspect ratio.
     *
     * This is the ration between the width and the height of the
     * screen. Typical values are 4/3, 1/2 or 1/1. This value is
     * 0 if the aspect ratio is not defined in the source file.
     * 0 is also the default value.
     */
    pub aspect: f32,

    /** Half horizontal orthographic width, in scene units.
     *
     *  The orthographic width specifies the half width of the
     *  orthographic view box. If non-zero the camera is
     *  orthographic and the mAspect should define to the
     *  ratio between the orthographic width and height
     *  and mHorizontalFOV should be set to 0.
     *  The default value is 0 (not orthographic).
     */
    pub orthographic_width: f32,
}
