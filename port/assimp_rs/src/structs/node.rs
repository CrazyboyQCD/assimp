use glam::Mat4;

use crate::{structs::meta::Metadata, structs::nodes::Index};

#[derive(Debug, Clone, Default)]
pub struct Node {
    pub name: String,
    pub transformation_matrix: Mat4,
    pub parent: Index<Node>,
    pub children: Vec<Index<Node>>,
    pub meshes: Vec<u32>,
    pub metadata: Box<Metadata>,
}
