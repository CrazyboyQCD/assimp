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

//! Defines scene c-ffi types for the library

use core::{
    ffi::{CStr, c_char, c_void},
    ptr::{self, NonNull},
};

use crate::ffi::{
    AiStringFFI, animation::AiAnimationFFI, bone::AiBoneFFI, camera::AiCameraFFI,
    light::AiLightFFI, material::AiMaterialFFI, mesh::AiMeshFFI, metadata::AiMetadataFFI,
    node::AiNodeFFI, skeleton::AiSkeletonFFI, texture::AiTextureFFI,
};

#[repr(C)]
pub struct AiSceneFFI {
    /** Any combination of the AI_SCENE_FLAGS_XXX flags. By default
     * this value is 0, no flags are set. Most applications will
     * want to reject all scenes with the AI_SCENE_FLAGS_INCOMPLETE
     * bit set.
     */
    flags: usize,

    /** The root node of the hierarchy.
     *
     * There will always be at least the root node if the import
     * was successful (and no special flags have been set).
     * Presence of further nodes depends on the format and content
     * of the imported file.
     */
    root_node: *mut AiNodeFFI,

    /** The number of meshes in the scene. */
    num_meshes: usize,

    /** The array of meshes.
     *
     * Use the indices given in the aiNode structure to access
     * this array. The array is mNumMeshes in size. If the
     * AI_SCENE_FLAGS_INCOMPLETE flag is not set there will always
     * be at least ONE material.
     */
    meshes: *mut *mut AiMeshFFI,

    /** The number of materials in the scene. */
    num_materials: usize,

    /** The array of materials.
     *
     * Use the index given in each aiMesh structure to access this
     * array. The array is mNumMaterials in size. If the
     * AI_SCENE_FLAGS_INCOMPLETE flag is not set there will always
     * be at least ONE material.
     */
    materials: *mut *mut AiMaterialFFI,

    /** The number of animations in the scene. */
    num_animations: usize,

    /** The array of animations.
     *
     * All animations imported from the given file are listed here.
     * The array is mNumAnimations in size.
     */
    animations: *mut *mut AiAnimationFFI,

    /** The number of textures embedded into the file */
    num_textures: usize,

    /** The array of embedded textures.
     *
     * Not many file formats embed their textures into the file.
     * An example is Quake's MDL format (which is also used by
     * some GameStudio versions)
     */
    textures: *mut *mut AiTextureFFI,

    /** The number of light sources in the scene. Light sources
     * are fully optional, in most cases this attribute will be 0
     */
    num_lights: usize,

    /** The array of light sources.
     *
     * All light sources imported from the given file are
     * listed here. The array is mNumLights in size.
     */
    lights: *mut *mut AiLightFFI,

    /** The number of cameras in the scene. Cameras
     * are fully optional, in most cases this attribute will be 0
     */
    num_cameras: usize,

    /** The array of cameras.
     *
     * All cameras imported from the given file are listed here.
     * The array is mNumCameras in size. The first camera in the
     * array (if existing) is the default camera view into
     * the scene.
     */
    cameras: *mut *mut AiCameraFFI,

    /**
     *  @brief  The global metadata assigned to the scene itself.
     *
     *  This data contains global metadata which belongs to the scene like
     *  unit-conversions, versions, vendors or other model-specific data. This
     *  can be used to store format-specific metadata as well.
     */
    metadata: *mut AiMetadataFFI,

    /** The name of the scene itself.
     */
    name: AiStringFFI,

    /**
     *
     */
    num_skeletons: usize,

    /**
     *
     */
    skeletons: *mut *mut AiSkeletonFFI,
}

impl AiSceneFFI {
    /// Check whether the scene contains meshes
    /// Unless no special scene flags are set this will always be true.
    pub fn has_meshes(&self) -> bool {
        !self.meshes.is_null() && self.num_meshes > 0
    }

    /// Check whether the scene contains materials
    /// Unless no special scene flags are set this will always be true.
    pub fn has_materials(&self) -> bool {
        !self.materials.is_null() && self.num_materials > 0
    }

    /// Check whether the scene contains lights
    pub fn has_lights(&self) -> bool {
        !self.lights.is_null() && self.num_lights > 0
    }

    /// Check whether the scene contains textures
    pub fn has_textures(&self) -> bool {
        !self.textures.is_null() && self.num_textures > 0
    }

    /// Check whether the scene contains cameras
    pub fn has_cameras(&self) -> bool {
        !self.cameras.is_null() && self.num_cameras > 0
    }

    /// Check whether the scene contains animations
    pub fn has_animations(&self) -> bool {
        !self.animations.is_null() && self.num_animations > 0
    }

    /// Check whether the scene contains skeletons
    pub fn has_skeletons(&self) -> bool {
        !self.skeletons.is_null() && self.num_skeletons > 0
    }

    /// Returns a short filename from a full path
    pub fn get_short_filename_from_slice(filename: &[u8]) -> &[u8] {
        for (i, b) in filename.iter().enumerate().rev() {
            if *b == b'/' || *b == b'\\' {
                return filename.get(i + 1..).unwrap_or(filename);
            }
        }
        filename
    }

    /// Returns an embedded texture
    pub unsafe fn get_embedded_texture(
        &mut self,
        filename: *const c_char,
    ) -> Option<&mut AiTextureFFI> {
        unsafe { self.get_embedded_texture_and_index(filename).0 }
    }

    /// Returns an embedded texture and its index
    pub unsafe fn get_embedded_texture_and_index(
        &mut self,
        filename: *const c_char,
    ) -> (Option<&mut AiTextureFFI>, i32) {
        if filename.is_null() {
            return (None, -1);
        }
        // SAFETY: filename is not null
        let filename = unsafe { CStr::from_ptr(filename) };
        let s = filename.to_bytes();
        // lookup using texture ID (if referenced like: "*1", "*2", etc.)
        if let [b'*', rest @ ..] = s {
            if let Ok(s) = core::str::from_utf8(rest)
                && let Ok(index) = s.parse::<i32>()
            {
                if 0 > index || self.num_textures <= index as usize {
                    return (None, -1);
                }
                // SAFETY: index is in range
                let ptr_of_texture_ptr = unsafe { self.textures.add(index as usize) };
                // SAFETY: ptr_of_texture_ptr is not null
                if let Some(texture_ptr) = unsafe { ptr_of_texture_ptr.as_mut() } {
                    // SAFETY: texture_ptr is not null
                    return (unsafe { texture_ptr.as_mut() }, index);
                }
            }
            return (None, -1);
        }
        // lookup using filename
        let short_filename = Self::get_short_filename_from_slice(filename.to_bytes());
        if short_filename.is_empty() {
            return (None, -1);
        }

        for i in 0..self.num_textures {
            // SAFETY: i is in range
            if let Some(ptr_of_texture) = unsafe { self.textures.add(i).as_mut() } {
                // SAFETY: ptr_of_texture is not null
                if let Some(texture) = unsafe { ptr_of_texture.as_mut() } {
                    // SAFETY: texture is not null
                    let short_texture_filename =
                        Self::get_short_filename_from_slice(texture.filename.as_slice());
                    if short_texture_filename == short_filename {
                        return (Some(texture), i as i32);
                    }
                }
            }
        }
        (None, -1)
    }

    /**
     * @brief Will try to locate a bone described by its name.
     *
     * @param name  The name to look for.
     * @return The bone as a pointer.
     */
    pub unsafe fn find_bone(&self, name: &AiStringFFI) -> Option<NonNull<AiBoneFFI>> {
        for m in 0..self.num_meshes {
            if let Some(ptr_of_mesh) = unsafe { self.meshes.add(m).as_mut() }
                && let Some(mesh) = unsafe { ptr_of_mesh.as_mut() }
            {
                for b in 0..mesh.num_bones {
                    if let Some(bone) = unsafe { mesh.bones.add(b as usize).as_mut() }
                        && name == &bone.name
                    {
                        return Some(unsafe { NonNull::new_unchecked(bone) });
                    }
                }
            }
        }
        None
    }
}
