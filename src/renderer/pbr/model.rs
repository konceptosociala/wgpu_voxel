use crate::renderer::{
    hal::buffer::BufferId,
    Renderable,
};

use super::mesh::Mesh;

pub struct Model {
    mesh: Mesh,
    vertex_buffer: Option<BufferId>,
}

impl Model {
    pub fn mesh(&self) -> &Mesh { &self.mesh }
}