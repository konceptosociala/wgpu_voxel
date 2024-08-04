use bytemuck::{Pod, Zeroable};

pub mod pipeline;
pub mod buffer;
pub mod texture;
pub mod taa;

/// A structure used for padding to align data to specific byte boundaries.
/// 
/// This struct is marked with `#[repr(C)]` to ensure a C-compatible memory layout,
/// and it derives common traits such as `Default`, `Debug`, `Clone`, `Copy`, 
/// `Zeroable`, and `Pod` for convenience and safety in GPU memory operations.
#[repr(C)]
#[derive(Default, Debug, Clone, Copy, Zeroable, Pod)]
pub struct Padding {
    _padding: u32,
}
