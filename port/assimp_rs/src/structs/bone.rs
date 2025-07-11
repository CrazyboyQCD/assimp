use super::{mesh::AiVertexWeight, node::Node, nodes::Index};
use crate::utils::float_precision::Mat4;

#[derive(Debug, Clone, Default)]
pub struct AiBone {
    pub name: String,
    pub armature: Index<Node>,
    pub node: Index<Node>,
    pub weights: Vec<AiVertexWeight>,
    pub offset_matrix: Mat4,
}
