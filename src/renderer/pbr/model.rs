use super::mesh::Mesh;

pub struct Model {
    pub(crate) mesh: Mesh,
    pub(crate) buffer: wgpu::Buffer,
}

impl Model {
    pub fn mesh(&self) -> &Mesh { &self.mesh }
}