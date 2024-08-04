use bytemuck::{Pod, Zeroable};
use derive_getters::Getters;
use rand::Rng;
use super::{buffer::Buffer, texture::Texture};

structstruck::strike! {
    #[strikethrough[derive(Getters)]]
    pub struct Taa {
        history_texture: Texture,
        velocity_texture: Texture,
        config_buffer: pub struct TaaConfigBuffer {
            inner: Buffer<
                #[repr(C)]
                #[derive(Debug, Clone, Copy, Zeroable, Pod)]
                pub struct TaaConfig {
                    jitter: f32,
                }
            >,
            bind_group_layout: wgpu::BindGroupLayout,
            bind_group: wgpu::BindGroup,
        },
    }
}

impl TaaConfig {
    #[allow(clippy::new_without_default)]
    pub fn new() -> TaaConfig {
        let mut rng = rand::thread_rng();
        TaaConfig {
            jitter: rng.gen_range(0.0..=1.)
        }
    }
}

impl Taa {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) -> Taa {        
        let history_texture = Texture::new(
            device, 
            config, 
            wgpu::FilterMode::Linear, 
            wgpu::TextureDimension::D2, 
            wgpu::TextureFormat::Rgba32Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING, 
            None,
            "TAA history",
        );

        let velocity_texture = Texture::new(
            device,
            config,
            wgpu::FilterMode::Linear,
            wgpu::TextureDimension::D2,
            wgpu::TextureFormat::Rgba32Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            None,
            "TAA velocity",
        );

        let buffer = Buffer::new(device, 1, wgpu::BufferUsages::UNIFORM);
        buffer.fill_exact(queue, &[TaaConfig::new()]).unwrap();

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("TAA config bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("TAA config bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0, 
                resource: buffer.inner().as_entire_binding(),
            }]
        });

        let config_buffer = TaaConfigBuffer { inner: buffer, bind_group_layout, bind_group };

        Taa {
            history_texture,
            velocity_texture,
            config_buffer,
        }
    }
}