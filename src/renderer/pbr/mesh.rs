use bytemuck::{Pod, Zeroable};
use nalgebra_glm as glm;
use serde::{Deserialize, Serialize};

use super::Color;

/// A vertex structure containing position, normal, and color attributes.
#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Zeroable, Pod)]
pub struct Vertex {
    pub position: glm::Vec3,
    pub normal: glm::Vec3,
    pub color: Color,
}

impl Vertex {
    /// Vertex attributes for position, normal, and color.
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
        2 => Float32x3,
    ];

    /// Returns a description of the vertex buffer layout.
    ///
    /// # Returns
    ///
    /// A `wgpu::VertexBufferLayout` describing the layout of the vertex buffer.
    pub fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// A mesh structure containing vertex data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Mesh {
    pub vertex_data: Vec<Vertex>,
}

impl Mesh {
    /// Adds a top face to the mesh at the specified position with the given color.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the face.
    /// * `y` - The y-coordinate of the face.
    /// * `z` - The z-coordinate of the face.
    /// * `color` - The color of the face.
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

    /// Adds a bottom face to the mesh at the specified position with the given color.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the face.
    /// * `y` - The y-coordinate of the face.
    /// * `z` - The z-coordinate of the face.
    /// * `color` - The color of the face.
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

    /// Adds a front face to the mesh at the specified position with the given color.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the face.
    /// * `y` - The y-coordinate of the face.
    /// * `z` - The z-coordinate of the face.
    /// * `color` - The color of the face.
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

    /// Adds a back face to the mesh at the specified position with the given color.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the face.
    /// * `y` - The y-coordinate of the face.
    /// * `z` - The z-coordinate of the face.
    /// * `color` - The color of the face.
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

    /// Adds a left face to the mesh at the specified position with the given color.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the face.
    /// * `y` - The y-coordinate of the face.
    /// * `z` - The z-coordinate of the face.
    /// * `color` - The color of the face.
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

    /// Adds a right face to the mesh at the specified position with the given color.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the face.
    /// * `y` - The y-coordinate of the face.
    /// * `z` - The z-coordinate of the face.
    /// * `color` - The color of the face.
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
