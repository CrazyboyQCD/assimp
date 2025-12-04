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

//! Defines the data structures in which the imported materials
//! are returned.

use alloc::vec::Vec;

use crate::AiReal;

pub mod property;
pub mod texture_property;

use property::{AiBasicProperty, AiMaterialProperty, AiProperty};
use texture_property::{
    AiTextureFlags, AiTextureMapMode, AiTextureMapping, AiTextureOp, AiTextureType,
};

/// ## A material is a collection of properties that describe the appearance of a mesh.
#[derive(Clone, Debug, Default)]
pub struct AiMaterial {
    /// The properties of the material.
    pub properties: Vec<AiMaterialProperty>,
}

impl AiMaterial {
    /// Clears the material properties
    pub fn clear(&mut self) {
        self.properties.clear();
    }

    pub fn get_name(&self) -> Option<&str> {
        self.get_property(0, AiProperty::is_material_name_property)
    }

    /// Removes a property from the material by index and texture type
    pub fn remove_property<V: ?Sized>(
        &mut self,
        index: u32,
        r#type: AiTextureType,
        type_match_fn: impl Fn(&AiProperty) -> Option<&V>,
    ) {
        if let Some(index) = self.properties.iter().position(|p| {
            p.index == index && p.r#type == r#type && type_match_fn(&p.property).is_some()
        }) {
            // don't care the order now
            self.properties.swap_remove(index);
        }
    }

    /// Clones the material properties
    pub fn clone_property(&self) -> Vec<AiMaterialProperty> {
        self.properties.clone()
    }

    /// Adds a property to the material by index.
    pub fn add_property(&mut self, property: AiProperty, index: u32) {
        self.add_property_by_texture_type(property, index, AiTextureType::None);
    }

    /// Adds a property to the material by index and texture type
    pub fn add_property_by_texture_type(
        &mut self,
        property: AiProperty,
        index: u32,
        r#type: AiTextureType,
    ) {
        self.properties.push(AiMaterialProperty {
            index,
            r#type,
            property,
        });
    }

    /// Gets a property from the material by index.
    pub fn get_property<V: ?Sized>(
        &self,
        index: u32,
        type_match_fn: impl Fn(&AiProperty) -> Option<&V>,
    ) -> Option<&V> {
        self.get_property_by_texture_type(index, AiTextureType::None, type_match_fn)
    }

    /// Gets a property from the material by index and texture type
    pub fn get_property_by_texture_type<V: ?Sized>(
        &self,
        index: u32,
        r#type: AiTextureType,
        type_match_fn: impl Fn(&AiProperty) -> Option<&V>,
    ) -> Option<&V> {
        for p in self.properties.iter() {
            if (index == u32::MAX || p.index == index && p.r#type == r#type)
                && let Some(v) = type_match_fn(&p.property)
            {
                return Some(v);
            }
        }
        None
    }

    /// Gets a custom property from the material by key and index.
    pub fn get_custom_property(&self, key: &str, index: u32) -> Option<&AiBasicProperty> {
        self.get_custom_property_by_texture_type(key, index, AiTextureType::None)
    }

    /// Gets a custom property from the material by key and index and texture type
    pub fn get_custom_property_by_texture_type(
        &self,
        key: &str,
        index: u32,
        r#type: AiTextureType,
    ) -> Option<&AiBasicProperty> {
        for p in self.properties.iter() {
            if (index == u32::MAX || p.index == index && p.r#type == r#type)
                && let AiProperty::Custom((k, v)) = &p.property
                && key == k
            {
                return Some(v);
            }
        }
        None
    }

    /// Gets a material by texture type and index
    pub fn get_material_by_texture_type(
        &self,
        r#type: AiTextureType,
        index: u32,
    ) -> Option<(
        &str,
        Option<AiReal>,
        Option<AiTextureOp>,
        Option<AiTextureMapping>,
        Option<AiTextureMapMode>,
        Option<AiTextureMapMode>,
        Option<AiTextureFlags>,
    )> {
        let path =
            self.get_property_by_texture_type(index, r#type, AiProperty::is_texture_file_property)?;

        let mapping = self
            .get_property_by_texture_type(index, r#type, AiProperty::is_texture_mapping_uv_property)
            .copied();

        let blend_factor = self
            .get_property_by_texture_type(
                index,
                r#type,
                AiProperty::is_texture_blend_factor_property,
            )
            .copied();

        let op = self
            .get_property_by_texture_type(index, r#type, AiProperty::is_texture_op_property)
            .copied();

        let mapping_mode_u = self
            .get_property_by_texture_type(
                index,
                r#type,
                AiProperty::is_texture_mapping_mode_u_property,
            )
            .copied();
        let mapping_mode_v = self
            .get_property_by_texture_type(
                index,
                r#type,
                AiProperty::is_texture_mapping_mode_v_property,
            )
            .copied();

        let flags = self
            .get_property_by_texture_type(index, r#type, AiProperty::is_texture_flags_property)
            .copied();

        Some((
            path,
            blend_factor,
            op,
            mapping,
            mapping_mode_u,
            mapping_mode_v,
            flags,
        ))
    }
}
