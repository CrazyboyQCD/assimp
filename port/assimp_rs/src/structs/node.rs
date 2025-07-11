use crate::{
    structs::{meta::Metadata, nodes::Index},
    utils::float_precision::Mat4,
};

#[derive(Debug, Clone, Default)]
pub struct Node {
    pub name: String,
    pub transformation_matrix: Mat4,
    pub parent: Index<Node>,
    pub children: Vec<Index<Node>>,
    pub meshes: Vec<u32>,
    pub metadata: Box<Metadata>,
}
