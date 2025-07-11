use std::array;

use crate::{
    structs::{
        color::{Color3D, Color4D},
        key::{AiQuatKey, AiVectorKey},
        mesh::{AI_MAX_NUMBER_OF_COLOR_SETS, AI_MAX_NUMBER_OF_TEXTURECOORDS},
        nodes::Index,
    },
    utils::float_precision::{Mat4, Vec2, Vec3},
};

#[derive(Debug, Clone, Default)]
pub struct Face {
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct TexEntry {
    pub name: String,
    pub is_normal_map: bool,
}

impl TexEntry {
    pub fn new(name: String, is_normal_map: bool) -> Self {
        Self {
            name,
            is_normal_map,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Material {
    pub name: String,
    pub is_reference: bool, // if true, name holds a name by which the actual material can be found in the material list
    pub diffuse: Color4D,
    pub specular_exponent: f32,
    pub specular: Color3D,
    pub emissive: Color3D,
    pub textures: Vec<TexEntry>,
    pub scene_index: u32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: String::new(),
            is_reference: false,
            diffuse: Color4D::default(),
            specular_exponent: 0.0,
            specular: Color3D::default(),
            emissive: Color3D::default(),
            textures: Vec::new(),
            scene_index: u32::MAX,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BoneWeight {
    pub vertex: u32,
    pub weight: f32,
}

#[derive(Debug, Clone, Default)]
pub struct Bone {
    pub name: String,
    pub weights: Vec<BoneWeight>,
    pub offset_matrix: Mat4,
}

impl Bone {
    pub fn new(name: String) -> Self {
        Self {
            name,
            weights: Vec::new(),
            offset_matrix: Mat4::ZERO,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub name: String,
    pub positions: Vec<Vec3>,
    pub pos_faces: Vec<Face>,
    pub normals: Vec<Vec3>,
    pub norm_faces: Vec<Face>,
    pub num_textures: u32,
    pub tex_coords: [Vec<Vec2>; AI_MAX_NUMBER_OF_TEXTURECOORDS],
    pub num_color_sets: u32,
    pub colors: [Vec<Color4D>; AI_MAX_NUMBER_OF_COLOR_SETS],

    pub face_materials: Vec<u32>,
    pub materials: Vec<Material>,

    pub bones: Vec<Bone>,
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            name: String::new(),
            positions: Vec::new(),
            pos_faces: Vec::new(),
            normals: Vec::new(),
            norm_faces: Vec::new(),
            num_textures: 0,
            tex_coords: array::from_fn(|_| Vec::new()),
            num_color_sets: 0,
            colors: array::from_fn(|_| Vec::new()),
            face_materials: Vec::new(),
            materials: Vec::new(),
            bones: Vec::new(),
        }
    }
}

impl Mesh {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MatrixKey {
    pub time: f64,
    pub matrix: Mat4,
}

/** Helper structure representing a single animated bone in a XFile */
#[derive(Debug, Clone, Default)]
pub struct AnimBone {
    pub name: String,
    pub pos_keys: Vec<AiVectorKey>, // either three separate key sequences for position, rotation, scaling
    pub rot_keys: Vec<AiQuatKey>,
    pub scale_keys: Vec<AiVectorKey>,
    pub trafo_keys: Vec<MatrixKey>, // or a combined key sequence of transformation matrices.
}

impl AnimBone {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            pos_keys: Vec::new(),
            rot_keys: Vec::new(),
            scale_keys: Vec::new(),
            trafo_keys: Vec::new(),
        }
    }
}

/** Helper structure to represent an animation set in a XFile */
#[derive(Debug, Clone, Default)]
pub struct Animation {
    pub name: String,
    pub anims: Vec<AnimBone>,
}

impl Animation {
    pub fn new(name: String) -> Self {
        Self {
            name,
            anims: Vec::new(),
        }
    }
}

/** Helper structure to represent a XFile frame */
#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub transformation_matrix: Mat4,
    pub parent: Index<Node>,
    pub children: Vec<Index<Node>>,
    pub meshes: Vec<Mesh>,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            name: String::new(),
            transformation_matrix: Mat4::IDENTITY,
            parent: Index::new(0),
            children: Vec::new(),
            meshes: Vec::new(),
        }
    }
}

impl Node {
    pub fn new(parent: Index<Node>) -> Self {
        Self {
            name: String::new(),
            transformation_matrix: Mat4::IDENTITY,
            parent,
            children: Vec::new(),
            meshes: Vec::new(),
        }
    }
}

/** Helper structure analogue to aiScene */
#[derive(Debug, Clone)]
pub struct Scene {
    pub root_node: Option<Index<Node>>,

    pub nodes: Vec<Node>,

    pub global_meshes: Vec<Mesh>, // global meshes found outside of any frames
    pub global_materials: Vec<Material>, // global materials found outside of any meshes.

    pub animations: Vec<Animation>,
    pub anim_ticks_per_second: u32,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            root_node: None,
            nodes: Vec::new(),
            global_meshes: Vec::new(),
            global_materials: Vec::new(),
            animations: Vec::new(),
            anim_ticks_per_second: 0,
        }
    }
}

impl Scene {
    pub fn push_node(&mut self, parent: Index<Node>, node: Node) -> Index<Node> {
        let index = Index::push(&mut self.nodes, node);
        if index.value() == 0 {
            self.root_node = Some(index);
            return index;
        }
        if let Some(parent) = Index::get_mut(parent, &mut self.nodes) {
            parent.children.push(index);
        }
        index
    }
}
