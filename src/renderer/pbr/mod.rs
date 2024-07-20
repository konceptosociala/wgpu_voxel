use serde::{Deserialize, Serialize};

pub mod mesh;
pub mod transform;
pub mod camera;

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