use std::{fmt::Debug, ops::Range};

use crate::{
    structs::{
        anim::AiAnimation, camera::AiCamera, light::AiLight, material::AiMaterial, mesh::AiMesh,
        meta::Metadata, nodes::Index, texture::AiTexture,
    },
    utils::float_precision::Mat4,
};
#[derive(Default, Clone, Debug)]
pub struct AiNode {
    pub name: String,
    pub transformation: Mat4,
    pub parent: Index<AiNode>,
    pub children: Vec<Index<AiNode>>,
    pub meshes: Range<u32>,
    pub metadata: Box<Metadata>,
}
#[derive(Default, Clone, Debug)]
pub struct AiScene {
    pub root: Option<Index<AiNode>>,
    pub nodes: Vec<AiNode>,
    pub meshes: Vec<AiMesh>,
    pub materials: Vec<AiMaterial>,
    pub animations: Vec<AiAnimation>,
    pub textures: Vec<AiTexture>,
    pub lights: Vec<AiLight>,
    pub cameras: Vec<AiCamera>,
    pub metadata: Box<Metadata>,
    pub name: Box<str>,
}

impl AiScene {
    pub fn new() -> Self {
        Self {
            root: None,
            nodes: Vec::new(),
            meshes: Vec::new(),
            materials: Vec::new(),
            animations: Vec::new(),
            textures: Vec::new(),
            lights: Vec::new(),
            cameras: Vec::new(),
            metadata: Box::default(),
            name: Box::default(),
        }
    }

    pub fn get_node_by_index(&self, index: Index<AiNode>) -> Option<&AiNode> {
        self.nodes.get(index.value())
    }

    pub fn get_node_by_index_mut(&mut self, index: Index<AiNode>) -> Option<&mut AiNode> {
        self.nodes.get_mut(index.value())
    }

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

    pub fn add_children(
        &mut self,
        parent: Index<AiNode>,
        children: Vec<AiNode>,
    ) -> Option<Vec<AiNode>> {
        let index = parent.value();
        if index == 0 || index >= self.nodes.len() {
            return Some(children);
        };
        let len = children.len();
        if len > 0 {
            let current_len = self.nodes.len();
            self.nodes.extend(children);
            let parent_node = self.get_node_by_index_mut(parent)?;
            parent_node.children.extend(
                (current_len..current_len + len)
                    .map(|i| Index::new(i as u32))
                    .into_iter(),
            );
        }
        None
    }
}
