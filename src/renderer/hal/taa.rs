use bytemuck::{Pod, Zeroable};
use rand::Rng;
use crate::renderer::Renderer;
use crate::glm;

use super::{
    buffer::{Buffer, BufferResource},
    texture::*
};

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct TaaConfig {
    canvas_width: u32,
    canvas_height: u32,
    jitter: f32,
}

impl TaaConfig {
    #[allow(clippy::new_without_default)]
    pub fn new(renderer: &Renderer) -> TaaConfig {
        let mut rng = rand::thread_rng();
        TaaConfig {
            canvas_width: renderer.size().width,
            canvas_height: renderer.size().height,
            jitter: rng.gen_range(-1.0..=1.0)
        }
    }
}

pub struct Taa {
    pub render_texture: Texture,
    pub history_texture: TextureResource,
    pub velocity_buffer: BufferResource<glm::Vec4>,
    pub config_buffer: BufferResource<TaaConfig>,
}

impl Taa {
    pub fn new(renderer: &Renderer) -> Taa {
        Taa {
            render_texture: Texture::new(renderer, TextureDescriptor {
                width: renderer.size().width,
                height: renderer.size().height,
                filter: wgpu::FilterMode::Linear,
                dimension: wgpu::TextureDimension::D2,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                depth: None,
                label: "TAA Color",
            }),
            history_texture: TextureResource::new(
                renderer,
                Texture::new(renderer, TextureDescriptor {
                    width: renderer.size().width,
                    height: renderer.size().height,
                    filter: wgpu::FilterMode::Linear,
                    dimension: wgpu::TextureDimension::D2,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    depth: None,
                    label: "TAA History",
                }),
                TextureResourceUsage::TEXTURE | TextureResourceUsage::SAMPLER,
                Some(TextureSampleType::Float { filterable: true }),
            ),
            velocity_buffer: BufferResource::new(
                renderer,
                Buffer::new(
                    renderer,
                    (renderer.size().width * renderer.size().height) as usize,
                    wgpu::BufferUsages::STORAGE
                ),
                wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                wgpu::BufferBindingType::Storage { read_only: false },
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
        self.config_buffer.buffer.fill_exact(renderer, 0, &[TaaConfig::new(renderer)])
            .expect("Cannot fill TAA buffer");

        let render_descr = *self.render_texture.description();
        if render_descr.width != renderer.size().width || render_descr.height != renderer.size().height {
            self.render_texture.resize(renderer, renderer.size());
        }

        let history_descr = *self.history_texture.texture.description();
        if history_descr.width != renderer.size().width || history_descr.height != renderer.size().height {
            self.history_texture.resize(renderer, renderer.size());
        }

        let viewport_size = renderer.size().width as usize * renderer.size().height as usize;
        if *self.velocity_buffer.buffer.capacity() != viewport_size {
            self.velocity_buffer.resize(renderer, viewport_size);
        }
    }
}