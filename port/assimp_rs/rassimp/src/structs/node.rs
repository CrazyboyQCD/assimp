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

//! Defines the data structures in which the imported scene is returned: [`AiNode`].

use alloc::{string::String, vec::Vec};

use crate::{
    AiMat4,
    structs::{index::Index, metadata::AiMetadata},
};

/// ## A node in the imported hierarchy.
///
/// Each node has name, a parent node (except for the root node),
/// a transformation relative to its parent and possibly several child nodes.
/// Simple file formats don't support hierarchical structures - for these formats
/// the imported scene does consist of only a single root node without children.
#[derive(Clone, Debug)]
pub struct AiNode {
    /// ### The name of the node.
    ///
    /// The name might be empty (length of zero) but all nodes which
    /// need to be referenced by either bones or animations are named.
    ///
    /// Multiple nodes may have the same name, except for nodes which are referenced
    /// by bones (see [`AiBone`](crate::structs::bone::AiBone)
    /// and [`AiMesh::bones`](crate::structs::mesh::AiMesh::bones)).
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
    pub name: String,

    /// ### The transformation of the node.
    ///
    /// The transformation of the node relative to its parent.
    pub transformation: AiMat4,

    /// ### Parent node index.
    ///
    /// [`u32::MAX`] if this node is the root node.
    pub parent: Index<AiNode>,

    /// ### Children nodes indices.
    ///
    /// The children nodes of the node.
    pub children: Vec<Index<AiNode>>,

    /// ### Meshes indices.
    ///
    /// The meshes indices of the node. Each entry is an index into the
    /// mesh list of the [`AiScene`].
    pub meshes: Vec<u32>,

    /// ### Metadata.
    ///
    /// Metadata associated with this node or [`None`] if there is no metadata.
    /// Whether any metadata is generated depends on the source file format.
    ///
    /// ~See the [`ImporterNotes`](crate::io::importer_notes) page for more information
    /// on every source file format. Importers that don't document any metadata
    /// don't write any.~
    pub metadata: Option<AiMetadata>,
}

impl AiNode {
    pub fn from_name(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    pub const fn is_root(&self) -> bool {
        self.parent.is_guard()
    }
}

impl Default for AiNode {
    fn default() -> Self {
        Self {
            name: String::default(),
            transformation: AiMat4::default(),
            parent: Index::GUARD_INDEX,
            children: Vec::new(),
            meshes: Vec::new(),
            metadata: None,
        }
    }
}
