use bytemuck::{Pod, Zeroable};
use nalgebra_glm as glm;
use serde::{Deserialize, Serialize};

use super::Color;

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Vertex {
    pub position: glm::Vec3,
    pub normal: glm::Vec3,
    pub color: Color,
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
        2 => Float32x3,
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

unsafe impl Zeroable for Vertex {}
unsafe impl Pod for Vertex {}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Mesh {
    pub vertex_data: Vec<Vertex>,
}

impl Mesh {
    pub fn add_top_face(&mut self, x: usize, y: usize, z: usize, color: Color) {
        let normal = glm::vec3(0.0, 1.0, 0.0);

        self.vertex_data.extend(&[
            Vertex { position: glm::vec3(x as f32, y as f32 + 1.0, z as f32), normal, color },
            Vertex { position: glm::vec3(x as f32, y as f32 + 1.0, z as f32 + 1.0), normal, color },
            Vertex { position: glm::vec3(x as f32 + 1.0, y as f32 + 1.0, z as f32), normal, color },
            Vertex { position: glm::vec3(x as f32 + 1.0, y as f32 + 1.0, z as f32), normal, color },
            Vertex { position: glm::vec3(x as f32, y as f32 + 1.0, z as f32 + 1.0), normal, color },
            Vertex { position: glm::vec3(x as f32 + 1.0, y as f32 + 1.0, z as f32 + 1.0), normal, color },
        ]);
    }

    pub fn add_bottom_face(&mut self, x: usize, y: usize, z: usize, color: Color) {
        let normal = glm::vec3(0.0, -1.0, 0.0);

        self.vertex_data.extend(&[
            Vertex { position: glm::vec3(x as f32, y as f32, z as f32), normal, color },
            Vertex { position: glm::vec3(x as f32 + 1.0, y as f32, z as f32), normal, color },
            Vertex { position: glm::vec3(x as f32, y as f32, z as f32 + 1.0), normal, color },
            Vertex { position: glm::vec3(x as f32 + 1.0, y as f32, z as f32), normal, color },
            Vertex { position: glm::vec3(x as f32 + 1.0, y as f32, z as f32 + 1.0), normal, color },
            Vertex { position: glm::vec3(x as f32, y as f32, z as f32 + 1.0), normal, color },
        ]);
    }

    pub fn add_front_face(&mut self, x: usize, y: usize, z: usize, color: Color) {
        let normal = glm::vec3(0.0, 0.0, 1.0);

        self.vertex_data.extend(&[
            Vertex { position: glm::vec3(x as f32, y as f32, z as f32), normal, color },
            Vertex { position: glm::vec3(x as f32, y as f32 + 1.0, z as f32), normal, color },
            Vertex { position: glm::vec3(x as f32 + 1.0, y as f32, z as f32), normal, color },
            Vertex { position: glm::vec3(x as f32 + 1.0, y as f32, z as f32), normal, color },
            Vertex { position: glm::vec3(x as f32, y as f32 + 1.0, z as f32), normal, color },
            Vertex { position: glm::vec3(x as f32 + 1.0, y as f32 + 1.0, z as f32), normal, color },
        ]);
    }

    pub fn add_back_face(&mut self, x: usize, y: usize, z: usize, color: Color) {
        let normal = glm::vec3(0.0, 0.0, -1.0);

        self.vertex_data.extend(&[
            Vertex { position: glm::vec3(x as f32, y as f32, z as f32 + 1.0), normal, color },
            Vertex { position: glm::vec3(x as f32 + 1.0, y as f32, z as f32 + 1.0), normal, color },
            Vertex { position: glm::vec3(x as f32, y as f32 + 1.0, z as f32 + 1.0), normal, color },
            Vertex { position: glm::vec3(x as f32 + 1.0, y as f32, z as f32 + 1.0), normal, color },
            Vertex { position: glm::vec3(x as f32 + 1.0, y as f32 + 1.0, z as f32 + 1.0), normal, color },
            Vertex { position: glm::vec3(x as f32, y as f32 + 1.0, z as f32 + 1.0), normal, color },
        ]);
    }

    pub fn add_left_face(&mut self, x: usize, y: usize, z: usize, color: Color) {
        let normal = glm::vec3(-1.0, 0.0, 0.0);

        self.vertex_data.extend(&[
            Vertex { position: glm::vec3(x as f32, y as f32, z as f32), normal, color },
            Vertex { position: glm::vec3(x as f32, y as f32, z as f32 + 1.0), normal, color },
            Vertex { position: glm::vec3(x as f32, y as f32 + 1.0, z as f32), normal, color },
            Vertex { position: glm::vec3(x as f32, y as f32 + 1.0, z as f32), normal, color },
            Vertex { position: glm::vec3(x as f32, y as f32, z as f32 + 1.0), normal, color },
            Vertex { position: glm::vec3(x as f32, y as f32 + 1.0, z as f32 + 1.0), normal, color },
        ]);
    }

    pub fn add_right_face(&mut self, x: usize, y: usize, z: usize, color: Color) {
        let normal = glm::vec3(1.0, 0.0, 0.0);

        self.vertex_data.extend(&[
            Vertex { position: glm::vec3(x as f32 + 1.0, y as f32, z as f32), normal, color },
            Vertex { position: glm::vec3(x as f32 + 1.0, y as f32 + 1.0, z as f32), normal, color },
            Vertex { position: glm::vec3(x as f32 + 1.0, y as f32, z as f32 + 1.0), normal, color },
            Vertex { position: glm::vec3(x as f32 + 1.0, y as f32, z as f32 + 1.0), normal, color },
            Vertex { position: glm::vec3(x as f32 + 1.0, y as f32 + 1.0, z as f32), normal, color },
            Vertex { position: glm::vec3(x as f32 + 1.0, y as f32 + 1.0, z as f32 + 1.0), normal, color },
        ]);
    }
}