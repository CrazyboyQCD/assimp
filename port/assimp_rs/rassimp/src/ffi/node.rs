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

//! Defines node c-ffi types for the library

use alloc::{boxed::Box, vec::Vec};
use core::{
    mem::{self, ManuallyDrop},
    ptr::{self, NonNull},
    slice,
};

use crate::{
    ffi::{AiMatrix4x4FFI, metadata::AiMetadataFFI, string::AiStringFFI},
    structs::{index::Index, node::AiNode},
};

/// ## A node in the imported hierarchy.
///
/// Each node has name, a parent node (except for the root node),
/// a transformation relative to its parent and possibly several child nodes.
/// Simple file formats don't support hierarchical structures - for these formats
/// the imported scene does consist of only a single root node without children.
#[repr(C)]
#[derive(Default)]
pub struct AiNodeFFI {
    /// ### The name of the node.
    ///
    /// The name might be empty (length of zero) but all nodes which
    /// need to be referenced by either bones or animations are named.
    ///
    /// Multiple nodes may have the same name, except for nodes which are referenced
    /// by bones (see [`AiBoneFFI`](crate::ffi::bone::AiBoneFFI)
    /// and [`AiMeshFFI::bones`](crate::ffi::mesh::AiMeshFFI::bones)).
    /// Their names *must* be unique.
    ///
    /// Cameras and lights reference a specific node by name - if there
    /// are multiple nodes with this name, they are assigned to each of them.
    /// <br>
    /// There are no limitations with regard to the characters contained in
    /// the name string as it is usually taken directly from the source file.
    ///
    /// Implementations should be able to handle tokens such as whitespace, tabs,
    /// line feeds, quotation marks, ampersands etc.
    ///
    /// ~Sometimes assimp introduces new nodes not present in the source file
    /// into the hierarchy (usually out of necessity because sometimes the
    /// source hierarchy format is simply not compatible). Their names are
    /// surrounded by `<>` e.g. `<DummyRootNode>`.~
    pub name: AiStringFFI,

    /// ### The transformation relative to the node's parent.
    ///
    /// The transformation relative to the node's parent.
    pub transformation: AiMatrix4x4FFI,

    /// ### Parent node.
    ///
    /// nullptr if this node is the root node.
    pub parent: *mut AiNodeFFI,

    /// ### Number of children of this node.
    ///
    /// The number of children of this node.
    pub num_children: usize,

    /// ### The child nodes of this node.
    ///
    /// nullptr if mNumChildren is 0.
    pub children: *mut AiNodeFFI,

    /// ### The number of meshes of this node.
    pub num_meshes: usize,

    /// ### The meshes of this node.
    ///
    /// Each entry is an index into the mesh list of the
    /// [`AiSceneFFI`](crate::ffi::scene::AiSceneFFI).
    pub meshes: *mut u32,

    /// ### The metadata of this node.
    ///
    /// The metadata of this node.
    pub metadata: *mut AiMetadataFFI,
}

impl From<Vec<AiNode>> for AiNodeFFI {
    fn from(nodes: Vec<AiNode>) -> Self {
        let mut root = AiNodeFFI::default();
        Self::create_from_vector(&mut root, nodes);
        root
    }
}

impl From<Vec<AiNode>> for Box<AiNodeFFI> {
    fn from(nodes: Vec<AiNode>) -> Self {
        let mut root = Box::<AiNodeFFI>::default();
        AiNodeFFI::create_from_vector(root.as_mut(), nodes);
        root
    }
}

impl AiNodeFFI {
    /// Constructor for the AiNodeFFI.
    pub fn new() -> Self {
        Self::default()
    }

    fn create_from_vector(root: &mut AiNodeFFI, mut nodes: Vec<AiNode>) {
        fn inner(current: &mut AiNodeFFI, children: Vec<Index<AiNode>>, nodes: &mut Vec<AiNode>) {
            let mut children_ffi = Vec::new();
            for child in children {
                if let Some(child) = child.get_mut(nodes) {
                    let AiNode {
                        name,
                        transformation,
                        children,
                        meshes,
                        metadata,
                        ..
                    } = mem::take(child);
                    let mut child_ffi = AiNodeFFI::new();
                    child_ffi.name = name.into();
                    child_ffi.transformation = transformation.into();
                    child_ffi.parent = current;
                    let meshes = meshes.into_iter().collect::<Box<[u32]>>();
                    child_ffi.num_meshes = meshes.len();
                    child_ffi.meshes = Box::into_raw(meshes).cast();
                    child_ffi.metadata = if let Some(metadata) = metadata {
                        Box::into_raw(Box::new(metadata.into()))
                    } else {
                        ptr::null_mut()
                    };
                    inner(&mut child_ffi, children, nodes);
                    children_ffi.push(child_ffi);
                }
            }
            let mut children_ffi = ManuallyDrop::new(children_ffi.into_boxed_slice());
            current.num_children = children_ffi.len();
            current.children = children_ffi.as_mut_ptr();
        }
        let children = if let Some(old_root) = nodes.get_mut(0) {
            let AiNode {
                name,
                transformation,
                children,
                meshes,
                metadata,
                ..
            } = mem::take(old_root);
            let mut dummy = Box::<AiNodeFFI>::default();
            unsafe { dummy.name.append(c"Dummy".as_ptr().cast()) };
            root.parent = Box::into_raw(dummy);
            root.name = name.into();
            root.transformation = transformation.into();
            let meshes = meshes.into_iter().collect::<Box<[u32]>>();
            root.num_meshes = meshes.len();
            root.meshes = Box::into_raw(meshes).cast();
            root.metadata = if let Some(metadata) = metadata {
                Box::into_raw(Box::new(metadata.into()))
            } else {
                ptr::null_mut()
            };
            children
        } else {
            return;
        };
        inner(root, children, &mut nodes);
    }
}

impl Drop for AiNodeFFI {
    // Should be right since tree doesn't have rings.
    fn drop(&mut self) {
        fn inner(children: &mut [AiNodeFFI]) {
            for child in children {
                // Release meshes
                if !child.meshes.is_null() {
                    let s = unsafe { slice::from_raw_parts_mut(child.meshes, child.num_meshes) };
                    let _: Box<[u32]> = unsafe { Box::from_raw(s) };
                    child.meshes = ptr::null_mut();
                    child.num_meshes = 0;
                }
                // Release children
                if !child.children.is_null() {
                    let s =
                        unsafe { slice::from_raw_parts_mut(child.children, child.num_children) };
                    inner(s);
                    let _: Box<[AiNodeFFI]> = unsafe { Box::from_raw(s) };
                    child.children = ptr::null_mut();
                    child.num_children = 0;
                }
            }
        }

        // Release meshes
        if !self.meshes.is_null() {
            let s = unsafe { slice::from_raw_parts_mut(self.meshes, self.num_meshes) };
            let _: Box<[u32]> = unsafe { Box::from_raw(s) };
            self.meshes = ptr::null_mut();
            self.num_meshes = 0;
        }

        // Release children
        if !self.children.is_null() {
            let s = unsafe { slice::from_raw_parts_mut(self.children, self.num_children) };
            inner(s);
            let _: Box<[AiNodeFFI]> = unsafe { Box::from_raw(s) };
            self.children = ptr::null_mut();
            self.num_children = 0;
        }
    }
}

/// ## Release the AiNodeFFI from the root.
///
/// Pass mutable reference of the raw pointer and set it to null to avoid double free.
///
/// # Safety
///
/// Caller must make Sure that the pointer is passed from the original rust allocation.
pub unsafe extern "C" fn release_ai_node_from_root_rs(root: *mut *mut AiNodeFFI) {
    if let Some(root) = unsafe { root.as_mut() } {
        let ptr = mem::take(root);
        if let Some(ptr) = NonNull::new(ptr) {
            let mut root = unsafe { Box::from_raw(ptr.as_ptr()) };
            if let Some(parent) = NonNull::new(root.parent) {
                let _ = unsafe { Box::from_raw(parent.as_ptr()) };
                root.parent = ptr::null_mut();
            }
        }
    }
}
