use crate::utils::float_precision::Mat4;

use super::{mesh::AiVertexWeight, node::Node, nodes::Index};

#[derive(Debug, Clone, Default)]
pub struct AiBone {
    pub name: Box<str>,
    pub armature: Index<Node>,
    pub node: Index<Node>,
    pub weights: Box<[AiVertexWeight]>,
    pub offset_matrix: Mat4,
}
