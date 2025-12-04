/*
---------------------------------------------------------------------------
Open Asset Import Library (assimp)
---------------------------------------------------------------------------

Copyright (c) 2006-2025, assimp team

All rights reserved.

Redistribution and use of this software in source and binary forms,
with or without modification, are permitted provided that the following
conditions are met:

* Redistributions of source code must retain the above
  copyright notice, this list of conditions and the
  following disclaimer.

* Redistributions in binary form must reproduce the above
  copyright notice, this list of conditions and the
  following disclaimer in the documentation and/or other
  materials provided with the distribution.

* Neither the name of the assimp team, nor the names of its
  contributors may be used to endorse or promote products
  derived from this software without specific prior
  written permission of the assimp team.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
"AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
---------------------------------------------------------------------------
*/

//! Declares the data structures in which the imported geometry is
//! returned by ASSIMP: [`AiMesh`] data structures.

use alloc::{borrow::ToOwned, string::String, vec::Vec};

#[cfg(feature = "armature_populate")]
use crate::structs::node::AiNode;
use crate::{
    AiMat4, AiReal, AiVec3,
    structs::{aabb::AABB, color::Color4D, index::Index, mesh::primitive_type::AiPrimitiveType},
};

pub mod anim_mesh;
pub mod bone;
pub mod face;
pub mod primitive_type;

use anim_mesh::AiAnimMesh;
use bone::AiBone;
use face::AiFace;

/// ## Maximum number of color sets
pub const AI_MAX_NUMBER_OF_COLOR_SETS: usize = 0x8;

/// ## Maximum number of texture coordinates
pub const AI_MAX_NUMBER_OF_TEXTURECOORDS: usize = 0x8;

/// ## A mesh represents a geometry or model with a single material.
///
/// It usually consists of a number of vertices and a series of primitives/faces
/// referencing the vertices. In addition there might be a series of bones, each
/// of them addressing a number of vertices with a certain weight. Vertex data
/// is presented in channels with each channel containing a single per-vertex
/// information such as a set of texture coordinates or a normal vector.
/// If a data pointer is non-null, the corresponding data stream is present.
///
/// ~From C++-programs you can also use the comfort functions Has*() to
/// test for the presence of various data streams.~
///
/// A Mesh uses only a single material which is referenced by a material ID.
///
/// ### Note:
///[AiScene](crate::structs::scene::AiScene)
/// The mPositions member is usually not optional. However, vertex positions
/// *could* be missing if the
/// [`AiSceneFlags::Incomplete`](crate::structs::scene::AiSceneFlags::Incomplete) flag is set in
/// [`AiScene::flags`](crate::structs::scene::AiScene::flags)
#[derive(Clone, Debug, Default)]
pub struct AiMesh {
    /// ### Name of the mesh.
    ///
    /// Meshes can be named, but this is not a
    /// requirement and leaving this field empty is totally fine.
    /// There are mainly three uses for mesh names:
    ///   - Some formats name nodes and meshes independently.
    ///   - Importers tend to split meshes up to meet the one-material-per-mesh requirement.
    ///     Assigning the same (dummy) name to each of the result meshes aids the caller at
    ///     recovering the original mesh partitioning.
    ///   - Vertex animations refer to meshes by their names.
    pub name: String,

    /// Bitwise combination of the members of the #aiPrimitiveType enum.
    /// This specifies which types of primitives are present in the mesh.
    /// The "SortByPrimitiveType"-Step can be used to make sure the
    /// output meshes consist of one primitive type each.
    pub primitive_types: AiPrimitiveType,

    /// ### Vertex positions.
    ///
    /// This array is always present in a mesh.
    ///
    /// ~The array is mNumVertices in size.~
    pub vertices: Vec<AiVec3>,

    /// ### Vertex normals.
    ///
    /// The array contains normalized vectors, empty if not present.
    ///
    /// ~The array is mNumVertices in size.~
    ///
    /// Normals are undefined for
    /// point and line primitives. A mesh consisting of points and
    /// lines only may not have normal vectors. Meshes with mixed
    /// primitive types (i.e. lines and triangles) may have normals,
    /// but the normals for vertices that are only referenced by
    /// point or line primitives are undefined and set to QNaN (WARN:
    /// qNaN compares to inequal to *everything*, even to qNaN itself.
    /// Using code like this to check whether a field is qnan is:
    /// ```text
    /// f.is_nan()
    /// ```
    /// still dangerous because even 1.f == 1.f could evaluate to false! (
    /// remember the subtleties of IEEE754 artithmetics). Use stuff like
    /// `f.classify()` instead.
    ///
    /// ### Note:
    /// Normal vectors computed by Assimp are always unit-length.
    /// However, this needn't apply for normals that have been taken
    /// directly from the model file.
    pub normals: Vec<AiVec3>,

    /// ### Vertex tangents.
    ///
    /// The tangent of a vertex points in the direction of the positive
    /// X texture axis. The array contains normalized vectors, empty if
    /// not present. The array is mNumVertices in size. A mesh consisting
    /// of points and lines only may not have normal vectors. Meshes with
    /// mixed primitive types (i.e. lines and triangles) may have
    /// normals, but the normals for vertices that are only referenced by
    /// point or line primitives are undefined and set to qNaN.  See
    /// the #mNormals member for a detailed discussion of qNaNs.
    ///
    /// ### Note:
    /// If the mesh contains tangents, it automatically also
    /// contains bitangents.
    pub tangents: Vec<AiVec3>,

    /// ### Vertex bitangents.
    ///
    /// The bitangent of a vertex points in the direction of the positive
    /// Y texture axis. The array contains normalized vectors, empty if not
    /// present. The array is mNumVertices in size.
    ///
    /// ### Note:
    /// If the mesh contains tangents, it automatically also contains
    /// bitangents.
    pub bitangents: Vec<AiVec3>,

    /// ### Vertex color sets.
    ///
    /// A mesh may contain 0 to [`AI_MAX_NUMBER_OF_COLOR_SETS`] vertex
    /// colors per vertex. empty if not present.
    ///
    /// ~Each array is `mNumVertices` in size if present.~
    pub colors: [Vec<Color4D>; AI_MAX_NUMBER_OF_COLOR_SETS],

    /// ### Vertex texture coordinates, also known as UV channels.
    ///
    /// A mesh may contain 0 to [`AI_MAX_NUMBER_OF_TEXTURECOORDS`] channels per
    /// vertex. Used and unused (empty) channels may go in any order.
    ///
    /// ~The array is `mNumVertices` in size.~
    pub texture_coords: [Vec<AiVec3>; AI_MAX_NUMBER_OF_TEXTURECOORDS],

    ///
    /// ### Vertex UV stream names.
    ///
    /// ~Pointer to array of size [`AI_MAX_NUMBER_OF_TEXTURECOORDS`].~
    ///
    /// ~The array is `mNumVertices` in size if present.~
    pub texture_coords_names: Option<[String; AI_MAX_NUMBER_OF_TEXTURECOORDS]>,

    /// ### Specifies the number of components for a given UV channel.
    ///
    /// Up to 3 channels are supported (UVW, for accessing volume
    /// or cube maps). If the value is 2 for a given channel n, the
    /// component p.z of `texture_coords[n][p]` is set to 0.0.
    /// If the value is 1 for a given channel, p.y is set to 0.0, too.
    ///
    /// ### Note:
    /// 4D coordinates are not supported
    pub num_of_uv_components: [u32; AI_MAX_NUMBER_OF_TEXTURECOORDS],

    /// ### The faces the mesh is constructed from.
    ///
    /// Each face refers to a number of vertices by their indices.
    /// This array is always present in a mesh~, its size is given
    /// in `mNumFaces`~. If the
    /// [`AiSceneFlags::NonVerboseFormat`](crate::structs::scene::AiSceneFlags::NonVerboseFormat)
    /// is NOT set each face references an unique set of vertices.
    pub faces: Vec<AiFace>,

    /// ### The bones of this mesh.
    ///
    /// A bone consists of a name by which it can be found in the
    /// frame hierarchy and a set of vertex weights.
    pub bones: Vec<AiBone>,

    /// ### The material used by this mesh.
    ///
    /// A mesh uses only a single material. If an imported model uses
    /// multiple materials, the import splits up the mesh. Use this value
    /// as index into the scene's material list.
    pub material_index: u32,

    /// ### Attachment meshes for this mesh, for vertex-based animation.
    ///
    /// Attachment meshes carry replacement data for some of the
    /// mesh'es vertex components (usually positions, normals).
    /// Currently known to work with loaders:
    ///
    /// *TODO: Implement loaders*
    ///
    ///  - ~Collada~
    ///
    ///  - ~gltf~
    pub anim_meshes: Vec<AiAnimMesh>,

    /// ### Method of morphing when anim-meshes are specified.
    ///
    /// *See [`MorphingMethod`](crate::structs::mesh::MorphingMethod) to learn more about the
    /// provided morphing targets.*
    pub method: MorphingMethod,

    /// ### The bounding box.
    pub aabb: AABB,
}

impl AiMesh {
    /// Check if the mesh has positions.
    pub fn has_positions(&self) -> bool {
        !self.vertices.is_empty()
    }

    /// Check if the mesh has faces.
    pub fn has_face(&self) -> bool {
        !self.faces.is_empty()
    }

    /// Check if the mesh has normals.
    pub fn has_normals(&self) -> bool {
        !self.normals.is_empty()
    }

    /// Check if the mesh has tangents and bitangents.
    pub fn has_tangents_and_bitangents(&self) -> bool {
        !self.tangents.is_empty() && !self.bitangents.is_empty() && !self.vertices.is_empty()
    }

    /// Check if the mesh has vertex colors.
    pub fn has_vertex_colors(&self, index: usize) -> bool {
        index < AI_MAX_NUMBER_OF_COLOR_SETS && !self.colors[index].is_empty()
    }

    /// Check if the mesh has texture coordinates.
    pub fn has_texture_coords(&self, index: usize) -> bool {
        index < AI_MAX_NUMBER_OF_TEXTURECOORDS && !self.texture_coords[index].is_empty()
    }

    /// Get the number of UV channels.
    pub fn num_of_uv_channels(&self) -> usize {
        let mut cnt = 0;
        for v in self.texture_coords.iter() {
            cnt += (!v.is_empty()) as usize;
        }
        cnt
    }

    /// Get the number of color channels.
    pub fn num_of_color_channels(&self) -> usize {
        let mut cnt = 0;
        for v in self.colors.iter() {
            cnt += (!v.is_empty()) as usize;
        }
        cnt
    }

    /// Check if the mesh has bones.
    pub fn has_bones(&self) -> bool {
        !self.bones.is_empty()
    }

    /// Check if the mesh has texture coordinates names.
    pub fn has_texture_coords_name(&self, index: usize) -> bool {
        if index < AI_MAX_NUMBER_OF_TEXTURECOORDS
            && let Some(names) = &self.texture_coords_names
        {
            return !names[index].is_empty();
        }
        false
    }

    /// Set the texture coordinates name.
    pub fn set_texture_coords_name(&mut self, index: usize, name: &str) {
        if index < AI_MAX_NUMBER_OF_TEXTURECOORDS {
            if let Some(names) = &mut self.texture_coords_names {
                names[index] = name.to_owned();
            } else {
                let mut names: [String; AI_MAX_NUMBER_OF_TEXTURECOORDS] = Default::default();
                names[index] = name.to_owned();
                self.texture_coords_names = Some(names);
            }
        }
    }

    /// Get the texture coordinates name.
    pub fn get_texture_coords_name(&self, index: usize) -> Option<&str> {
        if index < AI_MAX_NUMBER_OF_TEXTURECOORDS
            && let Some(names) = &self.texture_coords_names
        {
            Some(names[index].as_ref())
        } else {
            None
        }
    }
}

/// ## A vertex weight represents the influence of a bone on a vertex.
#[derive(Clone, Debug, Default)]
pub struct AiVertexWeight {
    /// Index of the vertex which is influenced by the bone.
    pub vertex_id: u32,

    /// The strength of the influence in the range (0...1).
    ///
    /// The influence from all bones at one vertex amounts to 1.
    pub weight: AiReal,
}

/// ## Enumerates the methods of mesh morphing supported by Assimp.
#[derive(Clone, Debug, Default)]
pub enum MorphingMethod {
    /// Morphing method to be determined
    #[default]
    Unknown = 0x0,

    /// Interpolation between morph targets
    VertexBlend = 0x1,

    /// Normalized morphing between morph targets
    MorphNormalized = 0x2,

    /// Relative morphing between morph targets
    MorphRelative = 0x3,
}

/// ## A skeleton bone represents a single bone is a skeleton structure.
///
/// Skeleton-Animations can be represented via a skeleton struct, which describes
/// a hierarchical tree assembled from skeleton bones. A bone is linked to a mesh.
///
/// The bone knows its parent bone. If there is no parent bone the parent id is
/// marked with -1.
///
/// The skeleton-bone stores a pointer to its used armature. If there is no
/// armature this value if set to nullptr.
///
/// A skeleton bone stores its offset-matrix, which is the absolute transformation
/// for the bone. The bone stores the locale transformation to its parent as well.
///
/// You can compute the offset matrix by multiplying the hierarchy like:
/// ```text
/// Tree: s1 -> s2 -> s3
/// Offset-Matrix s3 = locale-s3 * locale-s2 * locale-s1
/// ```
#[derive(Clone, Debug, Default)]
pub struct SkeletonBone {
    /// The parent bone index, is -1 one if this bone represents the root bone.
    pub parent: i32,

    #[cfg(feature = "armature_populate")]
    /// The bone armature node - used for skeleton conversion
    /// you must enable aiProcess_PopulateArmatureData to populate this
    pub armature: Index<AiNode>,

    #[cfg(feature = "armature_populate")]
    /// The bone node in the scene - used for skeleton conversion
    /// you must enable aiProcess_PopulateArmatureData to populate this
    pub node: Index<AiNode>,

    /// The mesh index, which will get influenced by the weight.
    pub mesh_id: Index<AiMesh>,

    /// The influence weights of this bone, by vertex index.
    pub weights: Vec<AiVertexWeight>,

    /// Matrix that transforms from bone space to mesh space in bind pose.
    ///
    /// This matrix describes the position of the mesh
    /// in the local space of this bone when the skeleton was bound.
    /// Thus it can be used directly to determine a desired vertex position,
    /// given the world-space transform of the bone when animated,
    /// and the position of the vertex in mesh space.
    ///
    /// It is sometimes called an inverse-bind matrix,
    /// or inverse bind pose matrix.
    pub offset_matrix: AiMat4,

    /// Matrix that transforms the locale bone in bind pose.
    pub local_matrix: AiMat4,
}

/// ## A skeleton represents the bone hierarchy of an animation.
///
/// Skeleton animations can be described as a tree of bones:
/// ```text
///                  root
///                    |
///                  node1
///                  /   \
///               node3  node4
/// ```
/// If you want to calculate the transformation of node three you need to compute the
/// transformation hierarchy for the transformation chain of node3:
/// ```text
/// root->node1->node3
/// ```
/// Each node is represented as a skeleton instance.
pub struct Skeleton {
    /// The name of the skeleton instance.
    pub name: String,

    /// The bone instance in the skeleton.
    pub bones: Vec<SkeletonBone>,
}
