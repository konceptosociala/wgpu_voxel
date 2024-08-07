use bytemuck::{Pod, Zeroable};
use rand::Rng;
use crate::renderer::Renderer;

use super::{
    buffer::{Buffer, BufferResource}, pipeline::ShaderBinding, texture::*
};

pub type Colorf32 = nalgebra_glm::Vec4;

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct TaaConfig {
    jitter: f32,
}

impl TaaConfig {
    #[allow(clippy::new_without_default)]
    pub fn new() -> TaaConfig {
        let mut rng = rand::thread_rng();
        TaaConfig {
            jitter: rng.gen_range(-1.0..=1.0)
        }
    }
}

pub struct Taa {
    pub render_texture: Texture,
    pub history_texture: TextureResource,
    pub velocity_texture: TextureResource,
    pub config_buffer: BufferResource<TaaConfig>,
}

impl Taa {
    pub fn new(renderer: &Renderer) -> Taa {
        Taa {
            render_texture: Texture::new(renderer, TextureDescriptor {
                width: renderer.size().width,
                height: renderer.size().height,
                filter: TextureFilter::Linear,
                dimension: TextureDimension::D2,
                usage: TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
                format: TextureFormat::Bgra8UnormSrgb,
                depth: None,
                label: "TAA Color",
            }), 
            history_texture: TextureResource::new(
                renderer, 
                Texture::new(renderer, TextureDescriptor {
                    width: renderer.size().width,
                    height: renderer.size().height,
                    filter: TextureFilter::Linear,
                    dimension: TextureDimension::D2,
                    usage: TextureUsage::TEXTURE_BINDING | TextureUsage::COPY_DST,
                    format: TextureFormat::Bgra8UnormSrgb,
                    depth: None,
                    label: "TAA History",
                }), 
                TextureResourceUsage::TEXTURE | TextureResourceUsage::SAMPLER, 
                Some(TextureSampleType::Float { filterable: true }),
            ),
            velocity_texture: TextureResource::new(
                renderer, 
                Texture::new(renderer, TextureDescriptor {
                    width: renderer.size().width,
                    height: renderer.size().height,
                    filter: TextureFilter::Linear,
                    dimension: TextureDimension::D2,
                    usage: TextureUsage::all() - TextureUsage::RENDER_ATTACHMENT,
                    format: TextureFormat::Rgba16Float,
                    depth: None,
                    label: "TAA Velocity",
                }), 
                TextureResourceUsage::STORAGE, 
                Some(TextureSampleType::Float { filterable: true }),
            ),
            config_buffer: BufferResource::new(
                renderer,
                Buffer::new(renderer, 1, wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST),
                wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE, 
                wgpu::BufferBindingType::Uniform,
            ),
        }
    }

    pub fn update(&mut self, renderer: &Renderer) {
        self.config_buffer.buffer.fill_exact(renderer, &[TaaConfig::new()])
            .expect("Cannot fill TAA buffer");

        let mut render_descr = *self.render_texture.description();
        if render_descr.width != renderer.size().width || render_descr.height != renderer.size().height {
            render_descr.width = renderer.size().width;
            render_descr.height = renderer.size().height;
            self.render_texture = Texture::new(renderer, render_descr);
        }

        let mut history_descr = *self.history_texture.texture.description();
        if history_descr.width != renderer.size().width || history_descr.height != renderer.size().height {
            history_descr.width = renderer.size().width;
            history_descr.height = renderer.size().height;
            self.history_texture = TextureResource::new(
                renderer, 
                Texture::new(renderer, history_descr), 
                TextureResourceUsage::TEXTURE | TextureResourceUsage::SAMPLER, 
                Some(TextureSampleType::Float { filterable: true }),
            );
        }
    }

    pub fn resources(&self) -> Vec<&dyn ShaderBinding> {
        vec![
            &self.config_buffer,
            &self.history_texture,
            &self.velocity_texture,
        ]
    }
}