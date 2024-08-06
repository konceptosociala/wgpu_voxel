use derive_getters::Getters;

use super::pipeline::{ShaderBinding, ShaderResource};

structstruck::strike! {
    #[derive(Debug, Clone, Copy)]
    pub struct TextureDescriptor {
        pub width: u32,
        pub height: u32,
        pub filter: pub type TextureFilter = wgpu::FilterMode,
        pub dimension: pub type TextureDimension = wgpu::TextureDimension,
        pub format: pub type TextureFormat = wgpu::TextureFormat,
        pub usage: pub type TextureUsage = wgpu::TextureUsages,
        pub depth: Option<u32>,
        pub label: &'static str,
    }
}

pub struct TextureResource {
    texture: Texture,
    resource: ShaderResource,
}

impl ShaderBinding for TextureResource {
    fn get_resource(&self) -> &ShaderResource {
        &self.resource
    }
}

// impl TextureResource {
//     pub fn new(
//         texture: Texture,
//     ) -> TextureResource {

//     }
// }

/// A structure representing a depth texture, including its view and sampler.
#[derive(Debug, Getters)]
pub struct Texture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    description: TextureDescriptor,
}

impl Texture {
    /// Creates a new depth texture with the specified device and surface configuration.
    ///
    /// # Arguments
    ///
    /// * `device` - A reference to the wgpu device.
    /// * `config` - A reference to the surface configuration.
    ///depth_texture
    /// # Returns
    ///
    /// A new instance of `Texture`.
    pub fn new(
        device: &wgpu::Device, 
        description: TextureDescriptor,
    ) -> Texture {
        let size = wgpu::Extent3d {
            width: description.width,
            height: description.height,
            depth_or_array_layers: description.depth.unwrap_or(1),
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(format!("{} texture", description.label).as_str()),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: description.dimension,
            format: description.format,
            usage: description.usage,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(format!("{} texture sampler", description.label).as_str()),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: description.filter,
            min_filter: description.filter,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Texture { 
            texture, 
            view, 
            sampler,
            description,
        }
    }
}
