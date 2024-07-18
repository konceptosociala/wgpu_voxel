use bytemuck::{Pod, Zeroable};
use nalgebra_glm as glm;
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b }
    }
}

impl From<dot_vox::Color> for Color {
    fn from(value: dot_vox::Color) -> Color {
        Color {
            r: value.r as f32 / 255.0,
            g: value.g as f32 / 255.0,
            b: value.b as f32 / 255.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Vertex {
    pub position: glm::Vec3,
    pub color: Color,
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
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