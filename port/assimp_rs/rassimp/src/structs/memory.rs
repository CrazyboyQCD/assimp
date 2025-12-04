#[derive(Clone, Copy, Debug, Default)]
pub struct AiMemoryInfo {
    pub textures: usize,
    pub materials: usize,
    pub meshes: usize,
    pub nodes: usize,
    pub animations: usize,
    pub cameras: usize,
    pub lights: usize,
    pub total: usize,
}

impl AiMemoryInfo {
    pub fn new(
        textures_info: usize,
        materials_info: usize,
        meshes_info: usize,
        nodes_info: usize,
        animations_info: usize,
        cameras_info: usize,
        lights_info: usize,
        total_info: usize,
    ) -> AiMemoryInfo {
        AiMemoryInfo {
            textures: textures_info,
            materials: materials_info,
            meshes: meshes_info,
            nodes: nodes_info,
            animations: animations_info,
            cameras: cameras_info,
            lights: lights_info,
            total: total_info,
        }
    }
}
