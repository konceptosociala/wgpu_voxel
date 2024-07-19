use bytemuck::{Pod, Zeroable};

pub mod pipeline;
pub mod buffer;
pub mod depth_texture;

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, Zeroable, Pod)]
pub struct Padding {
    _padding: u32,
}