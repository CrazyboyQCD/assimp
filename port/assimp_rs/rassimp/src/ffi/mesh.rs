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

//! A mesh represents a geometry or model with a single material.

use crate::{
    ffi::{
        AiColor4DFFI, AiVector3DFFI, aabb::AiAABBFFI, bone::AiBoneFFI, face::AiFaceFFI,
        string::AiStringFFI,
    },
    structs::mesh::{AI_MAX_NUMBER_OF_COLOR_SETS, AI_MAX_NUMBER_OF_TEXTURECOORDS},
};

/** @brief A mesh represents a geometry or model with a single material.
 *
 * It usually consists of a number of vertices and a series of primitives/faces
 * referencing the vertices. In addition there might be a series of bones, each
 * of them addressing a number of vertices with a certain weight. Vertex data
 * is presented in channels with each channel containing a single per-vertex
 * information such as a set of texture coordinates or a normal vector.
 * If a data pointer is non-null, the corresponding data stream is present.
 * From C++-programs you can also use the comfort functions Has*() to
 * test for the presence of various data streams.
 *
 * A Mesh uses only a single material which is referenced by a material ID.
 * @note The mPositions member is usually not optional. However, vertex positions
 * *could* be missing if the #AI_SCENE_FLAGS_INCOMPLETE flag is set in
 * @code
 * aiScene::mFlags
 * @endcode
 */
pub struct AiMeshFFI {
    /**
     * Bitwise combination of the members of the #aiPrimitiveType enum.
     * This specifies which types of primitives are present in the mesh.
     * The "SortByPrimitiveType"-Step can be used to make sure the
     * output meshes consist of one primitive type each.
     */
    pub primitive_types: u32,

    /**
     * The number of vertices in this mesh.
     * This is also the size of all of the per-vertex data arrays.
     * The maximum value for this member is #AI_MAX_VERTICES.
     */
    pub num_vertices: u32,

    /**
     * The number of primitives (triangles, polygons, lines) in this  mesh.
     * This is also the size of the mFaces array.
     * The maximum value for this member is #AI_MAX_FACES.
     */
    pub num_faces: u32,

    /**
     * @brief Vertex positions.
     *
     * This array is always present in a mesh. The array is
     * mNumVertices in size.
     */
    pub vertices: *mut AiVector3DFFI,

    /**
     * @brief Vertex normals.
     *
     * The array contains normalized vectors, nullptr if not present.
     * The array is mNumVertices in size. Normals are undefined for
     * point and line primitives. A mesh consisting of points and
     * lines only may not have normal vectors. Meshes with mixed
     * primitive types (i.e. lines and triangles) may have normals,
     * but the normals for vertices that are only referenced by
     * point or line primitives are undefined and set to QNaN (WARN:
     * qNaN compares to inequal to *everything*, even to qNaN itself.
     * Using code like this to check whether a field is qnan is:
     * @code
     * #define IS_QNAN(f) (f != f)
     * @endcode
     * still dangerous because even 1.f == 1.f could evaluate to false! (
     * remember the subtleties of IEEE754 artithmetics). Use stuff like
     * @c fpclassify instead.
     * @note Normal vectors computed by Assimp are always unit-length.
     * However, this needn't apply for normals that have been taken
     * directly from the model file.
     */
    pub normals: *mut AiVector3DFFI,

    /**
     * @brief Vertex tangents.
     *
     * The tangent of a vertex points in the direction of the positive
     * X texture axis. The array contains normalized vectors, nullptr if
     * not present. The array is mNumVertices in size. A mesh consisting
     * of points and lines only may not have normal vectors. Meshes with
     * mixed primitive types (i.e. lines and triangles) may have
     * normals, but the normals for vertices that are only referenced by
     * point or line primitives are undefined and set to qNaN.  See
     * the #mNormals member for a detailed discussion of qNaNs.
     * @note If the mesh contains tangents, it automatically also
     * contains bitangents.
     */
    pub tangents: *mut AiVector3DFFI,

    /**
     * @brief Vertex bitangents.
     *
     * The bitangent of a vertex points in the direction of the positive
     * Y texture axis. The array contains normalized vectors, nullptr if not
     * present. The array is mNumVertices in size.
     * @note If the mesh contains tangents, it automatically also contains
     * bitangents.
     */
    pub bitangents: *mut AiVector3DFFI,

    /**
     * @brief Vertex color sets.
     *
     * A mesh may contain 0 to #AI_MAX_NUMBER_OF_COLOR_SETS vertex
     * colors per vertex. nullptr if not present. Each array is
     * mNumVertices in size if present.
     */
    pub colors: [*mut AiColor4DFFI; AI_MAX_NUMBER_OF_COLOR_SETS],

    /**
     * @brief Vertex texture coordinates, also known as UV channels.
     *
     * A mesh may contain 0 to AI_MAX_NUMBER_OF_TEXTURECOORDS channels per
     * vertex. Used and unused (nullptr) channels may go in any order.
     * The array is mNumVertices in size.
     */
    pub texture_coords: [*mut AiVector3DFFI; AI_MAX_NUMBER_OF_TEXTURECOORDS],

    /**
     * @brief Specifies the number of components for a given UV channel.
     *
     * Up to three channels are supported (UVW, for accessing volume
     * or cube maps). If the value is 2 for a given channel n, the
     * component p.z of mTextureCoords[n][p] is set to 0.0f.
     * If the value is 1 for a given channel, p.y is set to 0.0f, too.
     * @note 4D coordinates are not supported
     */
    pub num_uv_components: [u32; AI_MAX_NUMBER_OF_TEXTURECOORDS],

    /**
     * @brief The faces the mesh is constructed from.
     *
     * Each face refers to a number of vertices by their indices.
     * This array is always present in a mesh, its size is given
     *  in mNumFaces. If the #AI_SCENE_FLAGS_NON_VERBOSE_FORMAT
     * is NOT set each face references an unique set of vertices.
     */
    pub faces: *mut AiFaceFFI,

    /**
     * The number of bones this mesh contains. Can be 0, in which case the mBones array is
     * nullptr.
     */
    pub num_bones: u32,

    /**
     * @brief The bones of this mesh.
     *
     * A bone consists of a name by which it can be found in the
     * frame hierarchy and a set of vertex weights.
     */
    pub bones: *mut AiBoneFFI,

    /**
     * @brief The material used by this mesh.
     *
     * A mesh uses only a single material. If an imported model uses
     * multiple materials, the import splits up the mesh. Use this value
     * as index into the scene's material list.
     */
    pub material_index: u32,

    /**
     *  Name of the mesh. Meshes can be named, but this is not a
     *  requirement and leaving this field empty is totally fine.
     *  There are mainly three uses for mesh names:
     *   - some formats name nodes and meshes independently.
     *   - importers tend to split meshes up to meet the one-material-per-mesh requirement.
     *     Assigning the same (dummy) name to each of the result meshes aids the caller at
     *     recovering the original mesh partitioning.
     *   - Vertex animations refer to meshes by their names.
     */
    pub name: AiStringFFI,

    /**
     * The number of attachment meshes.
     * Currently known to work with loaders:
     * - Collada
     * - gltf
     */
    pub num_anim_meshes: u32,

    /**
     * Attachment meshes for this mesh, for vertex-based animation.
     * Attachment meshes carry replacement data for some of the
     * mesh'es vertex components (usually positions, normals).
     * Currently known to work with loaders:
     * - Collada
     * - gltf
     */
    pub anim_meshes: *mut AiAnimMeshFFI,

    /**
     *  Method of morphing when anim-meshes are specified.
     *  @see aiMorphingMethod to learn more about the provided morphing targets.
     */
    pub method: AiMorphingMethodFFI,

    /**
     *  The bounding box.
     */
    pub aabb: AiAABBFFI,

    /**
     * Vertex UV stream names. Pointer to array of size AI_MAX_NUMBER_OF_TEXTURECOORDS
     */
    pub texture_coords_names: *mut AiStringFFI,
}

/** @brief An AnimMesh is an attachment to an #aiMesh stores per-vertex
 *  animations for a particular frame.
 *
 *  You may think of an #aiAnimMesh as a `patch` for the host mesh, which
 *  replaces only certain vertex data streams at a particular time.
 *  Each mesh stores n attached attached meshes (#aiMesh::mAnimMeshes).
 *  The actual relationship between the time line and anim meshes is
 *  established by #aiMeshAnim, which references singular mesh attachments
 *  by their ID and binds them to a time offset.
 */
#[repr(C)]
pub struct AiAnimMeshFFI {
    /**Anim Mesh name */
    pub name: AiStringFFI,

    /** Replacement for aiMesh::mVertices. If this array is non-nullptr,
     *  it *must* contain mNumVertices entries. The corresponding
     *  array in the host mesh must be non-nullptr as well - animation
     *  meshes may neither add or nor remove vertex components (if
     *  a replacement array is nullptr and the corresponding source
     *  array is not, the source data is taken instead) */
    pub vertices: *mut AiVector3DFFI,

    /** Replacement for aiMesh::mNormals. */
    pub normals: *mut AiVector3DFFI,

    /** Replacement for aiMesh::mTangents. */
    pub tangents: *mut AiVector3DFFI,

    /** Replacement for aiMesh::mBitangents. */
    pub bitangents: *mut AiVector3DFFI,

    /** Replacement for aiMesh::mColors */
    pub colors: [*mut AiColor4DFFI; AI_MAX_NUMBER_OF_COLOR_SETS],

    /** Replacement for aiMesh::mTextureCoords */
    pub texture_coords: [*mut AiVector3DFFI; AI_MAX_NUMBER_OF_TEXTURECOORDS],

    /** The number of vertices in the aiAnimMesh, and thus the length of all
     * the member arrays.
     *
     * This has always the same value as the mNumVertices property in the
     * corresponding aiMesh. It is duplicated here merely to make the length
     * of the member arrays accessible even if the aiMesh is not known, e.g.
     * from language bindings.
     */
    pub num_vertices: u32,

    /**
     * Weight of the AnimMesh.
     */
    pub weight: f32,
}

/** @brief Enumerates the methods of mesh morphing supported by Assimp.
 */
#[cfg_attr(not(feature = "swig"), repr(C))]
#[cfg_attr(feature = "swig", repr(C, u32))]
pub enum AiMorphingMethodFFI {
    /** Morphing method to be determined */
    Unknown = 0x0,

    /** Interpolation between morph targets */
    VertexBlend = 0x1,

    /** Normalized morphing between morph targets */
    MorphNormalized = 0x2,

    /** Relative morphing between morph targets */
    MorphRelative = 0x3,
}
