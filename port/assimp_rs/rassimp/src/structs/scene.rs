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

//! Defines the data structures in which the imported scene is returned: [`AiScene`].

use alloc::{string::String, vec::Vec};
use core::{fmt::Debug, mem::size_of};

use crate::{
    AiReal, AiVec3, AiVec4,
    structs::{
        animation::{
            AiAnimation,
            anim::{AiMeshAnim, AiMeshKey, AiMeshMorphAnim, AiNodeAnim},
        },
        camera::AiCamera,
        index::Index,
        key::AiMeshMorphKey,
        light::AiLight,
        material::{
            AiMaterial,
            property::{AiBasicBufferProperty, AiBasicProperty, AiMaterialProperty, AiProperty},
        },
        memory::AiMemoryInfo,
        mesh::{AiMesh, AiVertexWeight, bone::AiBone, face::AiFace},
        metadata::{AiMetadata, AiMetadataEntry},
        node::AiNode,
        texture::{AiTexel, AiTexture},
    },
};

/// ## The root structure of the imported data.
///
/// Everything that was imported from the given file can be accessed from here.
///
/// Objects of this class are generally maintained and owned by `Rassimp`, not
/// by the caller. You shouldn't want to instance it, nor should you ever try to
/// delete a given scene on your own.
#[derive(Clone, Debug, Default)]
pub struct AiScene {
    /// ### Any combination of the [`AiSceneFlags`] flags.
    ///
    /// By default this value is 0, no flags are set. Most applications will
    /// want to reject all scenes with the [`AiSceneFlags::Incomplete`]
    /// bit set.
    pub flags: AiSceneFlags,

    /// ### The root node of the hierarchy.
    ///
    /// There will always be at least the root node if the import
    /// was successful (and no special flags have been set).
    /// Presence of further nodes depends on the format and content
    /// of the imported file.
    pub root: Option<Index<AiNode>>,

    /// ### The nodes of the hierarchy.
    ///
    /// The nodes are the nodes of the hierarchy.
    pub nodes: Vec<AiNode>,

    /// ### The array of meshes.
    ///
    /// Use the indices given in the [`AiNode`] structure to access
    /// this array.
    ///
    /// ~The array is mNumMeshes in size.~
    ///
    /// If the
    /// [`AiSceneFlags::Incomplete`] flag is not set there will always
    /// be at least ONE material.
    pub meshes: Vec<AiMesh>,

    /// ### The array of materials.
    ///
    /// Use the index given in each [`AiMesh`] structure to access this
    /// array.
    ///
    /// ~The array is mNumMaterials in size.~
    ///
    /// If the [`AiSceneFlags::Incomplete`] flag is not set there will always
    /// be at least ONE material.
    pub materials: Vec<AiMaterial>,

    /// ### The array of animations.
    ///
    /// All animations imported from the given file are listed here.
    ///
    /// ~The array is mNumAnimations in size.~
    pub animations: Vec<AiAnimation>,

    /// ### The array of embedded textures.
    ///
    /// Not many file formats embed their textures into the file.
    /// An example is `Quake's MDL` format (which is also used by
    /// some GameStudio versions)
    pub textures: Vec<AiTexture>,

    /// ### The array of light sources.
    ///
    /// All light sources imported from the given file are
    /// listed here.
    ///
    /// ~The array is mNumLights in size.~
    pub lights: Vec<AiLight>,

    /// ### The array of cameras.
    ///
    /// All cameras imported from the given file are listed here.
    ///
    /// ~The array is mNumCameras in size.~
    ///
    /// The first camera in the
    /// array (if existing) is the default camera view into
    /// the scene.
    pub cameras: Vec<AiCamera>,

    /// ### The global metadata assigned to the scene itself.
    ///
    /// This data contains global metadata which belongs to the scene like
    /// unit-conversions, versions, vendors or other model-specific data. This
    /// can be used to store format-specific metadata as well.
    pub metadata: AiMetadata,

    /// ### The name of the scene itself.
    pub name: String,
}

impl AiScene {
    const ROOT_INDEX: usize = 0;
    /// Constructor for the scene
    pub fn new() -> Self {
        Self {
            flags: Default::default(),
            root: None,
            nodes: Vec::new(),
            meshes: Vec::new(),
            materials: Vec::new(),
            animations: Vec::new(),
            textures: Vec::new(),
            lights: Vec::new(),
            cameras: Vec::new(),
            metadata: Default::default(),
            name: String::default(),
        }
    }

    /// Checks if the scene is empty
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
            && self.nodes.is_empty()
            && self.meshes.is_empty()
            && self.materials.is_empty()
            && self.animations.is_empty()
            && self.textures.is_empty()
            && self.lights.is_empty()
            && self.cameras.is_empty()
            && self.metadata.is_empty()
            && self.name.is_empty()
    }

    pub fn root(&self) -> Option<&AiNode> {
        self.nodes.get(Self::ROOT_INDEX)
    }

    /// Gets a node by index
    pub fn get_node_by_index(&self, index: Index<AiNode>) -> Option<&AiNode> {
        self.nodes.get(index.value())
    }

    /// Gets a node by index mutably
    pub fn get_node_by_index_mut(&mut self, index: Index<AiNode>) -> Option<&mut AiNode> {
        self.nodes.get_mut(index.value())
    }

    /// Finds a node by name
    pub fn find_node_by_name(&self, name: &str, index: Index<AiNode>) -> Option<Index<AiNode>> {
        let node = self.get_node_by_index(index)?;
        if node.name == name {
            Some(index)
        } else {
            for child in &node.children {
                if let Some(result) = self.find_node_by_name(name, *child) {
                    return Some(result);
                }
            }
            None
        }
    }

    /// Adds children to a node
    pub fn add_children(
        &mut self,
        parent: Index<AiNode>,
        children: Vec<AiNode>,
    ) -> Option<Vec<AiNode>> {
        let parent_index = parent.value();
        if parent_index == 0 || parent_index >= self.nodes.len() {
            return Some(children);
        };
        let len = children.len();
        if len > 0 {
            let current_len = self.nodes.len();
            self.nodes.extend(children);
            let parent_node = self.get_node_by_index_mut(parent)?;
            parent_node
                .children
                .extend((current_len..current_len + len).map(|i| Index::new(i as u32)));
        }
        None
    }

    pub fn get_memory_requirements(&self) -> AiMemoryInfo {
        let mut total = size_of::<AiScene>();

        // add all nodes
        let mut nodes = size_of::<AiNode>() * self.nodes.capacity();
        for node in &self.nodes {
            nodes += node.name.capacity();
            nodes += size_of::<Index<AiNode>>() * node.children.capacity();
            nodes += size_of::<u32>() * node.meshes.capacity();
            if let Some(metadata) = &node.metadata {
                fn get_ai_metadata_memory_requirements(metadata: &AiMetadata) -> usize {
                    let mut size = 0;
                    // heap size of indices and entry bucket hash value, not accurate but close
                    // enough
                    const INDICES_SIZE: usize = size_of::<u8>();
                    const BUCKET_SIZE: usize =
                        size_of::<usize>() + size_of::<String>() + size_of::<AiMetadataEntry>();
                    size += size_of::<usize>() + (INDICES_SIZE + BUCKET_SIZE) * metadata.capacity();
                    for (s, entry) in metadata.iter() {
                        size += s.capacity();
                        size += match entry {
                            AiMetadataEntry::String(s) => s.capacity(),
                            AiMetadataEntry::Metadata(nested) => {
                                get_ai_metadata_memory_requirements(nested)
                            }
                            _ => 0,
                        }
                    }
                    size
                }
                nodes += get_ai_metadata_memory_requirements(metadata);
            }
        }
        total += nodes;

        // add all meshes
        let mut meshes = size_of::<AiMesh>() * self.meshes.capacity();
        for mesh in &self.meshes {
            meshes += mesh.name.capacity();
            meshes += size_of::<AiVec3>() * mesh.vertices.capacity();
            meshes += size_of::<AiVec3>() * mesh.normals.capacity();
            meshes += size_of::<AiVec3>() * mesh.tangents.capacity();
            meshes += size_of::<AiVec3>() * mesh.bitangents.capacity();

            for color in &mesh.colors {
                meshes += size_of::<AiVec4>() * color.capacity();
            }

            for texture_coord in &mesh.texture_coords {
                meshes += size_of::<AiVec3>() * texture_coord.capacity();
            }

            if let Some(texture_coords_names) = &mesh.texture_coords_names {
                for name in texture_coords_names {
                    meshes += name.capacity();
                }
            }

            meshes += size_of::<AiFace>() * mesh.faces.capacity();
            for f in &mesh.faces {
                meshes += size_of::<u32>() * f.indices.capacity();
            }

            meshes += size_of::<AiBone>() * mesh.bones.capacity();
            for p in &mesh.bones {
                meshes += p.name.capacity();
                meshes += size_of::<AiVertexWeight>() * p.weights.capacity();
            }
        }
        total += meshes;

        // add all embedded textures
        let mut textures = size_of::<AiTexture>() * self.textures.capacity();
        for texture in &self.textures {
            textures += size_of::<AiTexel>() * texture.pc_data.capacity();
            textures += texture.file_name.capacity();
        }
        total += textures;

        // add all animations
        let mut animations = size_of::<AiAnimation>() * self.animations.capacity();
        for animation in &self.animations {
            // add all bone anims
            animations += size_of::<AiNodeAnim>() * animation.channels.capacity();
            for channel in &animation.channels {
                animations += channel.node_name.capacity();
                animations += size_of::<AiNodeAnim>() * channel.position_keys.capacity();
                animations += size_of::<AiNodeAnim>() * channel.scaling_keys.capacity();
                animations += size_of::<AiNodeAnim>() * channel.rotation_keys.capacity();
            }

            animations += size_of::<AiMeshAnim>() * animation.mesh_channels.capacity();
            for channel in &animation.mesh_channels {
                animations += channel.name.capacity();
                animations += size_of::<AiMeshKey>() * channel.key_frames.capacity();
            }

            animations += size_of::<AiMeshMorphAnim>() * animation.morph_mesh_channels.capacity();
            for channel in &animation.morph_mesh_channels {
                animations += channel.name.capacity();
                animations += size_of::<AiMeshMorphKey>() * channel.key_frames.capacity();
            }
        }
        total += animations;

        // add all cameras and all lights
        let mut cameras = size_of::<AiCamera>() * self.cameras.capacity();
        for camera in &self.cameras {
            cameras += camera.name.capacity();
        }
        total += cameras;

        let mut lights = size_of::<AiLight>() * self.lights.capacity();
        for light in &self.lights {
            lights += light.name.capacity();
        }
        total += lights;

        // add all materials
        let mut materials = size_of::<AiMaterial>() * self.materials.capacity();
        for material in &self.materials {
            materials += size_of::<AiMaterialProperty>() * material.properties.capacity();
            for property in &material.properties {
                if let AiProperty::Custom((s, p)) = &property.property {
                    materials += s.capacity();
                    if let AiBasicProperty::Buffer(buf) = p {
                        match buf {
                            AiBasicBufferProperty::StringBuffer(strings) => {
                                for s in strings {
                                    materials += s.capacity();
                                }
                            }
                            AiBasicBufferProperty::NormalBuffer(nums) => {
                                materials += size_of::<u8>() * nums.capacity();
                            }
                            AiBasicBufferProperty::IntBuffer(nums) => {
                                materials += size_of::<i32>() * nums.capacity();
                            }
                            AiBasicBufferProperty::FloatBuffer(nums) => {
                                materials += size_of::<AiReal>() * nums.capacity();
                            }
                            AiBasicBufferProperty::Vec3Buffer(vec3s) => {
                                materials += size_of::<AiVec3>() * vec3s.capacity();
                            }
                            AiBasicBufferProperty::Vec4Buffer(vec4s) => {
                                materials += size_of::<AiVec4>() * vec4s.capacity();
                            }
                        }
                    } else if let AiBasicProperty::String(s) = p {
                        materials += s.capacity();
                    }
                } else if let Some(s) = property.property.get_inner_string() {
                    materials += s.capacity();
                }
            }
        }
        total += materials;
        AiMemoryInfo::new(
            textures, materials, meshes, nodes, animations, cameras, lights, total,
        )
    }
}

bitflags::bitflags! {
    /// Flags which are combinated in [`AiScene::flags`] to store
    /// auxiliary information about the imported scene.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct AiSceneFlags: u32 {
        /// Specifies that the scene data structure that was imported is not complete.
        /// This flag bypasses some internal validations and allows the import
        /// of animation skeletons, material libraries or camera animation paths
        /// using Assimp. Most applications won't support such data.
        const Incomplete = 1 << 0;

        /// This flag is set by the validation postprocess-step (aiPostProcess_ValidateDS)
        /// if the validation is successful. In a validated scene you can be sure that
        /// any cross references in the data structure (e.g. vertex indices) are valid.
        const Validated = 1 << 1;

        /// This flag is set by the validation postprocess-step (aiPostProcess_ValidateDS)
        /// if the validation is successful but some issues have been found.
        /// This can for example mean that a texture that does not exist is referenced
        /// by a material or that the bone weights for a vertex don't sum to 1.0 ... .
        /// In most cases you should still be able to use the import. This flag could
        /// be useful for applications which don't capture Assimp's log output.
        const Validation_Warning = 1 << 2;

        /// This flag is currently only set by the aiProcess_JoinIdenticalVertices step.
        /// It indicates that the vertices of the output meshes aren't in the internal
        /// verbose format anymore. In the verbose format all vertices are unique,
        /// no vertex is ever referenced by more than one face.
        const Non_Verbose_Format = 1 << 3;

        /// Denotes pure height-map terrain data. Pure terrains usually consist of quads,
        /// sometimes triangles, in a regular grid. The x,y coordinates of all vertex
        /// positions refer to the x,y coordinates on the terrain height map, the z-axis
        /// stores the elevation at a specific point.
        ///
        /// TER (Terragen) and HMP (3D Game Studio) are height map formats.
        ///
        /// Assimp is probably not the best choice for loading *huge* terrains -
        /// fully triangulated data takes extremely much free store and should be avoided
        /// as long as possible (typically you'll do the triangulation when you actually
        /// need to render it).
        const Terrain = 1 << 4;

        /// Specifies that the scene data can be shared between structures. For example:
        /// one vertex in few faces. [`AiSceneFlags::NonVerboseFormat`] can not be
        /// used for this because [`AiSceneFlags::NonVerboseFormat`] has internal
        /// meaning about postprocessing steps.
        const AllowShared = 1 << 5;
    }
}
