use bitflags::bitflags;
use derive_getters::Getters;
use game_loop::winit::dpi::PhysicalSize;

use crate::renderer::{RenderSurface, Renderer};
use crate::renderer::types::*;

#[derive(Debug, Clone, Copy)]
pub struct TextureDescriptor {
    pub width: u32,
    pub height: u32,
    pub depth: Option<u32>,
    pub filter: FilterMode,
    pub dimension: TextureDimension,
    pub usage: TextureUsages,
    pub format: TextureFormat,
    pub label: &'static str,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TextureResourceUsage: u8 {
        const TEXTURE = 1;
        const SAMPLER = 1 << 1;
        const STORAGE = 1 << 2;
    }
}

pub struct TextureResourceDescriptor {
    pub usage: TextureResourceUsage,
    pub sample_type: Option<TextureSampleType>,
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
    fn view(&self) -> &TextureView {
        if !self.description.usage.contains(wgpu::TextureUsages::RENDER_ATTACHMENT) {
            panic!("Texture, used as render surface, must have RENDER_ATTACHMENT usage");
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

    pub fn resize(&mut self, renderer: &Renderer, size: PhysicalSize<u32>) {
        let mut descr = self.description;
        descr.width = size.width;
        descr.height = size.height;
        *self = Texture::new(renderer, descr);
    }
}
