use bytemuck::{Pod, Zeroable};
use rand::Rng;
use crate::renderer::Renderer;
use crate::glm;

use super::{
    buffer::{Buffer, BufferResourceDescriptor}, pipeline::ShaderResource, texture::*
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

#[readonly::make]
pub struct Taa {
    pub render_texture: Texture,
    pub history_texture: Texture,
    pub velocity_buffer: Buffer<glm::Vec4>,
    pub config_buffer: Buffer<TaaConfig>,
    pub shader_resource: ShaderResource,
    #[readonly]
    pub current_jitter: f32,
}

impl Taa {
    pub fn new(renderer: &Renderer) -> Taa {
        let render_texture = Texture::new(renderer, TextureDescriptor {
            width: renderer.size().width,
            height: renderer.size().height,
            filter: wgpu::FilterMode::Linear,
            dimension: wgpu::TextureDimension::D2,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            depth: None,
            label: "TAA Color",
        });

        let history_texture = Texture::new(renderer, TextureDescriptor {
            width: renderer.size().width,
            height: renderer.size().height,
            filter: wgpu::FilterMode::Linear,
            dimension: wgpu::TextureDimension::D2,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            depth: None,
            label: "TAA History",
        });

        let velocity_buffer = Buffer::new(
            renderer,
            (renderer.size().width * renderer.size().height) as usize,
            wgpu::BufferUsages::STORAGE,
        );

        let config_buffer = Buffer::new(
            renderer, 
            1, 
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );

        let shader_resource = ShaderResource::builder()
            .add_texture(&history_texture, &TextureResourceDescriptor {
                usage: TextureResourceUsage::TEXTURE | TextureResourceUsage::SAMPLER,
                sample_type: Some(wgpu::TextureSampleType::Float { filterable: true }),
            })
            .add_buffer(&velocity_buffer, &BufferResourceDescriptor {
                visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                buffer_type: wgpu::BufferBindingType::Storage { read_only: false },
            })
            .add_buffer(&config_buffer, &BufferResourceDescriptor {
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                buffer_type: wgpu::BufferBindingType::Uniform,
            })
            .build(renderer);

        let current_jitter = 0.0;

        Taa { 
            render_texture, 
            history_texture, 
            velocity_buffer, 
            config_buffer, 
            shader_resource, 
            current_jitter,
        }
    }

    pub fn update(&mut self, renderer: &Renderer) {
        let taa_config = TaaConfig::new(renderer);

        self.current_jitter = taa_config.jitter;
        self.config_buffer.fill_exact(renderer, 0, &[taa_config])
            .expect("Cannot fill TAA buffer");

        let mut rebind_resources = false;

        let render_descr = *self.render_texture.description();
        if render_descr.width != renderer.size().width || render_descr.height != renderer.size().height {
            self.render_texture.resize(renderer, renderer.size());
            rebind_resources = true;
        }

        let history_descr = *self.history_texture.description();
        if history_descr.width != renderer.size().width || history_descr.height != renderer.size().height {
            self.history_texture.resize(renderer, renderer.size());
            rebind_resources = true;
        }

        let viewport_size = renderer.size().width as usize * renderer.size().height as usize;
        if *self.velocity_buffer.capacity() != viewport_size {
            self.velocity_buffer.resize(renderer, viewport_size);
            rebind_resources = true;
        }

        if rebind_resources {
            self.shader_resource = ShaderResource::builder()
                .add_texture(&self.history_texture, &TextureResourceDescriptor {
                    usage: TextureResourceUsage::TEXTURE | TextureResourceUsage::SAMPLER,
                    sample_type: Some(wgpu::TextureSampleType::Float { filterable: true }),
                })
                .add_buffer(&self.velocity_buffer, &BufferResourceDescriptor {
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    buffer_type: wgpu::BufferBindingType::Storage { read_only: false },
                })
                .add_buffer(&self.config_buffer, &BufferResourceDescriptor {
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    buffer_type: wgpu::BufferBindingType::Uniform,
                })
                .build(renderer);
        }
    }
}