use crate::structs::key::{AiMeshMorphKey, AiQuatKey, AiVectorKey};

// ---------------------------------------------------------------------------
/** Binds a anim-mesh to a specific point in time. */
#[derive(Debug, Clone, Default)]
pub struct AiMeshKey {
    /** The time of this key */
    pub time: f64,

    /** Index into the aiMesh::mAnimMeshes array of the
     *  mesh corresponding to the #aiMeshAnim hosting this
     *  key frame. The referenced anim mesh is evaluated
     *  according to the rules defined in the docs for #aiAnimMesh.*/
    pub value: u32,
}

// ---------------------------------------------------------------------------
/** Defines how an animation channel behaves outside the defined time
 *  range. This corresponds to aiNodeAnim::mPreState and
 *  aiNodeAnim::mPostState.*/
#[derive(Debug, Clone, Copy, Default)]
pub enum AiAnimBehaviour {
    /** The value from the default node transformation is taken*/
    #[default]
    Default = 0x0,

    /** The nearest key value is used without interpolation */
    Constant = 0x1,

    /** The value of the nearest two keys is linearly
     *  extrapolated for the current time value.*/
    Linear = 0x2,

    /** The animation is repeated.
     *
     *  If the animation key go from n to m and the current
     *  time is t, use the value at (t-n) % (|m-n|).*/
    Repeat = 0x3,
}

#[derive(Debug, Clone, Default)]
pub struct AiNodeAnim {
    /** The name of the node affected by this animation. The node
     *  must exist and it must be unique.*/
    pub node_name: Box<str>,

    /** The position keys of this animation channel. Positions are
     * specified as 3D vector. The array is mNumPositionKeys in size.
     *
     * If there are position keys, there will also be at least one
     * scaling and one rotation key.*/
    pub position_keys: Vec<AiVectorKey>,

    /** The rotation keys of this animation channel. Rotations are
     *  given as quaternions,  which are 4D vectors. The array is
     *  mNumRotationKeys in size.
     *
     * If there are rotation keys, there will also be at least one
     * scaling and one position key. */
    pub rotation_keys: Vec<AiQuatKey>,

    /** The scaling keys of this animation channel. Scalings are
     *  specified as 3D vector. The array is mNumScalingKeys in size.
     *
     * If there are scaling keys, there will also be at least one
     * position and one rotation key.*/
    pub scaling_keys: Vec<AiVectorKey>,

    /** Defines how the animation behaves before the first
     *  key is encountered.
     *
     *  The default value is aiAnimBehaviour_DEFAULT (the original
     *  transformation matrix of the affected node is used).*/
    pub pre_state: AiAnimBehaviour,

    /** Defines how the animation behaves after the last
     *  key was processed.
     *
     *  The default value is aiAnimBehaviour_DEFAULT (the original
     *  transformation matrix of the affected node is taken).*/
    pub post_state: AiAnimBehaviour,
}

#[derive(Debug, Clone, Default)]
pub struct AiMeshAnim {
    pub name: Box<str>,
    pub key_frames: Vec<AiMeshKey>,
}

#[derive(Debug, Clone, Default)]
pub struct AiMeshMorphAnim {
    pub name: Box<str>,
    pub key_frames: Vec<AiMeshMorphKey>,
}
