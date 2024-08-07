use bitflags::bitflags;
use derive_getters::Getters;

use crate::renderer::{RenderSurface, Renderer};

use super::pipeline::{ShaderBinding, ShaderResource};

pub use wgpu::TextureSampleType;

structstruck::strike! {
    #[derive(Debug, Clone, Copy)]
    pub struct TextureDescriptor {
        pub width: u32,
        pub height: u32,
        pub filter: pub type TextureFilter = wgpu::FilterMode,
        pub dimension: pub type TextureDimension = wgpu::TextureDimension,
        pub usage: pub type TextureUsage = wgpu::TextureUsages,
        pub format: pub type TextureFormat = wgpu::TextureFormat,
        pub depth: Option<u32>,
        pub label: &'static str,
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TextureResourceUsage: u8 {
        const TEXTURE = 1;
        const SAMPLER = 1 << 1;
        const STORAGE = 1 << 2;
    }
}

pub struct TextureResource {
    pub texture: Texture,
    pub resource: ShaderResource,
}

impl ShaderBinding for TextureResource {
    fn get_resource(&self) -> &ShaderResource {
        &self.resource
    }
}

impl TextureResource {
    pub fn new(
        renderer: &Renderer,
        texture: Texture,
        usage: TextureResourceUsage,
        sample_type: Option<wgpu::TextureSampleType>,
    ) -> TextureResource {
        let view_dimension = match texture.description.dimension {
            wgpu::TextureDimension::D1 => wgpu::TextureViewDimension::D1,
            wgpu::TextureDimension::D2 => wgpu::TextureViewDimension::D2,
            wgpu::TextureDimension::D3 => wgpu::TextureViewDimension::D3,
        };

        let bind_group_layout = renderer.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(format!("{} texture bind group layout", texture.description.label).as_str()),
            entries: &usage
                .iter()
                .enumerate()
                .filter_map(|(i, usage)| {
                    match usage {
                        TextureResourceUsage::STORAGE => {
                            Some(wgpu::BindGroupLayoutEntry {
                                binding: i as u32,
                                visibility: wgpu::ShaderStages::COMPUTE,
                                ty: wgpu::BindingType::StorageTexture {
                                    access: wgpu::StorageTextureAccess::WriteOnly,
                                    format: texture.description.format,
                                    view_dimension,
                                },
                                count: None,
                            })
                        },
                        TextureResourceUsage::TEXTURE => {
                            Some(wgpu::BindGroupLayoutEntry {
                                binding: i as u32,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    sample_type: sample_type.unwrap_or_else(|| {
                                        panic!("Must specify sample type for texture with TextureResourceUsage::TEXTURE");
                                    }),
                                    view_dimension,
                                    multisampled: false,
                                },
                                count: None,
                            })
                        },
                        TextureResourceUsage::SAMPLER => {
                            Some(wgpu::BindGroupLayoutEntry {
                                binding: i as u32,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                                count: None,
                            })
                        },
                        _ => None,
                    }
                })
                .collect::<Vec<_>>()
        });

        let bind_group = renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(format!("{} texture bind group", texture.description.label).as_str()),
            layout: &bind_group_layout,
            entries: &usage
                .iter()
                .enumerate()
                .filter_map(|(i, usage)| {
                    match usage {
                        TextureResourceUsage::STORAGE | TextureResourceUsage::TEXTURE => {
                            Some(wgpu::BindGroupEntry {
                                binding: i as u32,
                                resource: wgpu::BindingResource::TextureView(texture.view())
                            },)
                        },
                        TextureResourceUsage::SAMPLER => {
                            Some(wgpu::BindGroupEntry {
                                binding: i as u32,
                                resource: wgpu::BindingResource::Sampler(texture.sampler()),
                            })
                        },
                        _ => None,
                    }
                })
                .collect::<Vec<_>>(),
        });

        TextureResource {
            texture,
            resource: ShaderResource {
                bind_group,
                bind_group_layout
            },
        }
    }
}

/// A structure representing a depth texture, including its view and sampler.
#[derive(Debug, Getters)]
pub struct Texture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    description: TextureDescriptor,
}

impl RenderSurface for Texture {
    fn view(&self) -> &wgpu::TextureView {
        if !self.description.usage.contains(TextureUsage::RENDER_ATTACHMENT) {
            panic!("Texture, used as render surfce, must have RENDER_ATTACHMENT usage");
        }

        &self.view
    }
}

impl Texture {
    /// Creates a new depth texture with the specified device and surface configuration.
    ///
    /// # Arguments
    ///
    /// * `device` - A reference to the wgpu device.
    /// * `config` - A reference to the surface configuration.
    ///
    /// # Returns
    ///
    /// A new instance of `Texture`.
    pub fn new(
        renderer: &Renderer, 
        description: TextureDescriptor,
    ) -> Texture {
        let size = wgpu::Extent3d {
            width: description.width,
            height: description.height,
            depth_or_array_layers: description.depth.unwrap_or(1),
        };

        let texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
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

        let sampler = renderer.device.create_sampler(&wgpu::SamplerDescriptor {
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
