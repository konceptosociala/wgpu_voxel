use std::{collections::{hash_map::Entry, HashMap}, sync::Arc};
use bytemuck::{Pod, Zeroable};
use error::RenderError;
use game_loop::winit::{
    dpi::PhysicalSize, 
    window::Window,
};
use pbr::{mesh::{Mesh, Vertex}, model::Model};
use pipeline::RenderPipelines;
use wgpu::util::DeviceExt;

pub mod error;
pub mod voxel;
pub mod pbr;
pub mod pipeline;

#[allow(dead_code)]
pub struct Renderer {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    render_pipelines: RenderPipelines,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Renderer> {
        let size = window.inner_size();

        let instance = init_instance();
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = init_adapter(instance, &surface).await;
        let (device, queue) = init_device(&adapter).await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = init_config(surface_format, size, surface_caps);

        Ok(Renderer {
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipelines: HashMap::new(),
        })
    }

    pub fn window(&self) -> Arc<Window> {
        self.window.clone()
    }

    pub fn resize(&mut self) {
        self.resize_with(self.size);
    }

    pub fn resize_with(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 { return }

        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn create_model(&mut self, mesh: Mesh) -> Model {
        Model {
            
        }
    }

    pub fn update_model(&mut self, model: &mut Model, new_mesh: Mesh) {

    }

    pub fn create_vertex_buffer(&self, data: &[Vertex]) -> wgpu::Buffer {
        self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex buffer"),
                contents: bytemuck::cast_slice(data),
                usage: wgpu::BufferUsages::VERTEX,
            }  
        )
    }

    pub fn create_index_buffer(&self, data: &[u16]) -> wgpu::Buffer {
        self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index buffer"),
                contents: bytemuck::cast_slice(data),
                usage: wgpu::BufferUsages::INDEX,
            }  
        )
    }

    pub fn render(&mut self) -> Result<(), RenderError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
    
            for object in objects {
                render_pass.set_vertex_buffer(0, object.vertex_buffer().slice(..));
                render_pass.set_index_buffer(object.index_buffer().slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..object.index_count(), 0, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

async fn init_device(adapter: &wgpu::Adapter) -> Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError> {
    adapter.request_device(
        &wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: None,
        },
        None,
    ).await
}

async fn init_adapter(instance: wgpu::Instance, surface: &wgpu::Surface<'static>) -> wgpu::Adapter {
    instance.request_adapter(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        }
    ).await.unwrap()
}

fn init_instance() -> wgpu::Instance {
    wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..Default::default()
    })
}

fn init_config(surface_format: wgpu::TextureFormat, size: PhysicalSize<u32>, surface_caps: wgpu::SurfaceCapabilities) -> wgpu::SurfaceConfiguration {
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    }
}