use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

pub mod mesh;
pub mod transform;
pub mod camera;

/// A structure representing a color with red, green, and blue components.
///
/// The color components are floating-point values ranging from 0.0 to 1.0.
#[repr(C)]
#[derive(Default, Copy, Clone, Debug, Serialize, Deserialize, Zeroable, Pod)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
}

impl Color {
    /// Constructs a new `Color` instance with the given red, green, and blue components.
    ///
    /// # Arguments
    ///
    /// * `r` - The red component of the color (0.0 to 1.0).
    /// * `g` - The green component of the color (0.0 to 1.0).
    /// * `b` - The blue component of the color (0.0 to 1.0).
    ///
    /// # Returns
    ///
    /// A `Color` instance with the specified components.
    pub const fn new(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b }
    }
}

impl From<dot_vox::Color> for Color {
    /// Converts a `dot_vox::Color` instance to a `Color` instance.
    ///
    /// The conversion scales the RGB values from a range of 0-255 to 0.0-1.0.
    ///
    /// # Arguments
    ///
    /// * `value` - The `dot_vox::Color` instance to convert.
    ///
    /// # Returns
    ///
    /// A `Color` instance with the corresponding RGB values scaled to 0.0-1.0.
    fn from(value: dot_vox::Color) -> Color {
        Color {
            r: value.r as f32 / 255.0,
            g: value.g as f32 / 255.0,
            b: value.b as f32 / 255.0,
        }
    }
}
