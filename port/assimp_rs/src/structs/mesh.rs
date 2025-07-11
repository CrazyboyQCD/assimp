use super::{aabb::AABB, bone::AiBone, color::Color4D, face::AiFace, node::Node, nodes::Index};
use crate::utils::float_precision::{Mat4, Vec3};

pub const AI_MAX_NUMBER_OF_COLOR_SETS: usize = 0x8;
pub const AI_MAX_NUMBER_OF_TEXTURECOORDS: usize = 0x8;

#[derive(Debug, Clone, Default)]
pub struct AiMesh {
    pub name: String,
    pub primitive_type: u32,
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub tangents: Vec<Vec3>,
    pub bitangents: Vec<Vec3>,
    pub colors: Box<[Vec<Color4D>; AI_MAX_NUMBER_OF_COLOR_SETS]>,
    pub texture_coords: Box<[Vec<Vec3>; AI_MAX_NUMBER_OF_TEXTURECOORDS]>,
    pub texture_coords_names: Option<Box<[String; AI_MAX_NUMBER_OF_TEXTURECOORDS]>>,
    pub num_of_uv_components: Box<[u32; AI_MAX_NUMBER_OF_TEXTURECOORDS]>,
    pub faces: Vec<AiFace>,
    pub bones: Vec<AiBone>,
    pub material_index: u32,
    pub anim_meshes: Vec<AnimMesh>,
    pub method: MorphingMethod,
    pub aabb: AABB,
}

impl AiMesh {
    pub fn has_positions(&self) -> bool {
        !self.vertices.is_empty()
    }

    pub fn has_face(&self) -> bool {
        !self.faces.is_empty()
    }

    pub fn has_normals(&self) -> bool {
        !self.normals.is_empty()
    }

    pub fn has_tangents_and_bitangents(&self) -> bool {
        !self.tangents.is_empty() && !self.bitangents.is_empty() && !self.vertices.is_empty()
    }

    pub fn has_vertex_colors(&self, index: usize) -> bool {
        index < AI_MAX_NUMBER_OF_COLOR_SETS && !self.colors[index].is_empty()
    }

    pub fn has_texture_coords(&self, index: usize) -> bool {
        index < AI_MAX_NUMBER_OF_TEXTURECOORDS && !self.texture_coords[index].is_empty()
    }

    pub fn num_of_uv_channels(&self) -> usize {
        let mut cnt = 0;
        for v in self.texture_coords.iter() {
            cnt += (!v.is_empty()) as usize;
        }
        cnt
    }

    pub fn num_of_color_channels(&self) -> usize {
        let mut cnt = 0;
        for v in self.colors.iter() {
            cnt += (!v.is_empty()) as usize;
        }
        cnt
    }

    pub fn has_bones(&self) -> bool {
        !self.bones.is_empty()
    }

    pub fn has_texture_coords_name(&self, index: usize) -> bool {
        if index < AI_MAX_NUMBER_OF_TEXTURECOORDS {
            if let Some(names) = &self.texture_coords_names {
                return !names[index].is_empty();
            }
        }
        false
    }

    pub fn set_texture_coords_name(&mut self, index: usize, name: &str) {
        if index < AI_MAX_NUMBER_OF_TEXTURECOORDS {
            if let Some(names) = &mut self.texture_coords_names {
                names[index] = name.to_owned();
            } else {
                let mut names: Box<[String; AI_MAX_NUMBER_OF_TEXTURECOORDS]> = Box::default();
                names[index] = name.to_owned();
                self.texture_coords_names = Some(names);
            }
        }
    }

    pub fn get_texture_coords_name(&self, index: usize) -> Option<&str> {
        if index < AI_MAX_NUMBER_OF_TEXTURECOORDS {
            if let Some(names) = &self.texture_coords_names {
                return Some(names[index].as_ref());
            }
        }
        None
    }
}

#[derive(Debug, Clone, Default)]
pub struct AiVertexWeight {
    /// Index of the vertex which is influenced by the bone.
    pub vertex_id: u32,

    /// The strength of the influence in the range (0...1).
    ///
    /// The influence from all bones at one vertex amounts to 1.
    pub weight: f32,
}

#[derive(Debug, Clone, Default)]
pub struct AnimMesh {
    /// Anim Mesh name
    pub name: String,

    /** Replacement for aiMesh::mVertices. If this array is non-nullptr,

    *  it *must* contain mNumVertices entries. The corresponding
    *  array in the host mesh must be non-nullptr as well - animation
    *  meshes may neither add or nor remove vertex components (if
    *  a replacement array is nullptr and the corresponding source
    *  array is not, the source data is taken instead)*/
    pub vertices: Box<[Vec3]>,

    /** Replacement for aiMesh::mNormals.  */
    pub normals: Box<[Vec3]>,

    /** Replacement for aiMesh::mTangents. */
    pub tangents: Box<[Vec3]>,

    /** Replacement for aiMesh::mBitangents. */
    pub bitangents: Box<[Vec3]>,

    /** Replacement for aiMesh::mColors */
    pub colors: Box<[Box<[Color4D]>; AI_MAX_NUMBER_OF_COLOR_SETS]>,

    /** Replacement for aiMesh::mTextureCoords */
    pub texture_coords: Box<[Vec<Vec3>; AI_MAX_NUMBER_OF_TEXTURECOORDS]>,

    /** The number of vertices in the aiAnimMesh, and thus the length of all
     * the member arrays.
     *
     * This has always the same value as the mNumVertices property in the
     * corresponding aiMesh. It is duplicated here merely to make the length
     * of the member arrays accessible even if the aiMesh is not known, e.g.
     * from language bindings.
     */
    pub num_of_vertices: u32,

    /**
     * Weight of the AnimMesh.
     */
    pub weight: f32,
}

// ---------------------------------------------------------------------------
/** @brief Enumerates the methods of mesh morphing supported by Assimp.
 */
#[repr(u32)]
#[derive(Debug, Clone, Default)]
pub enum MorphingMethod {
    /** Morphing method to be determined */
    #[default]
    Unknown = 0x0,

    /** Interpolation between morph targets */
    VertexBlend = 0x1,

    /** Normalized morphing between morph targets  */
    MorphNormalized = 0x2,

    /** Relative morphing between morph targets  */
    MorphRelative = 0x3,
}

/**
 * @brief  A skeleton bone represents a single bone is a skeleton structure.
 *
 * Skeleton-Animations can be represented via a skeleton struct, which describes
 * a hierarchical tree assembled from skeleton bones. A bone is linked to a mesh.
 * The bone knows its parent bone. If there is no parent bone the parent id is
 * marked with -1.
 * The skeleton-bone stores a pointer to its used armature. If there is no
 * armature this value if set to nullptr.
 * A skeleton bone stores its offset-matrix, which is the absolute transformation
 * for the bone. The bone stores the locale transformation to its parent as well.
 * You can compute the offset matrix by multiplying the hierarchy like:
 * Tree: s1 -> s2 -> s3
 * Offset-Matrix s3 = locale-s3 * locale-s2 * locale-s1
 */
#[derive(Debug, Clone, Default)]
pub struct SkeletonBone {
    /// The parent bone index, is -1 one if this bone represents the root bone.
    pub parent: i32,

    /// @brief The bone armature node - used for skeleton conversion
    /// you must enable aiProcess_PopulateArmatureData to populate this
    pub armature: Index<Node>,

    /// @brief The bone node in the scene - used for skeleton conversion
    /// you must enable aiProcess_PopulateArmatureData to populate this
    pub node: Index<Node>,

    /// The mesh index, which will get influenced by the weight.
    pub mesh_id: Index<AiMesh>,

    /// The influence weights of this bone, by vertex index.
    pub weights: Box<[AiVertexWeight]>,

    /** Matrix that transforms from bone space to mesh space in bind pose.
     *
     * This matrix describes the position of the mesh
     * in the local space of this bone when the skeleton was bound.
     * Thus it can be used directly to determine a desired vertex position,
     * given the world-space transform of the bone when animated,
     * and the position of the vertex in mesh space.
     *
     * It is sometimes called an inverse-bind matrix,
     * or inverse bind pose matrix.
     */
    pub offset_matrix: Mat4,

    /// Matrix that transforms the locale bone in bind pose.
    pub local_matrix: Mat4,
}

/**
 * @brief A skeleton represents the bone hierarchy of an animation.
 *
 * Skeleton animations can be described as a tree of bones:
 *                  root
 *                    |
 *                  node1
 *                  /   \
 *               node3  node4
 * If you want to calculate the transformation of node three you need to compute the
 * transformation hierarchy for the transformation chain of node3:
 * root->node1->node3
 * Each node is represented as a skeleton instance.
 */
pub struct Skeleton {
    /**
     *  @brief The name of the skeleton instance.
     */
    pub name: Box<str>,

    /**
     *  @brief The bone instance in the skeleton.
     */
    pub bones: Box<[SkeletonBone]>,
}
