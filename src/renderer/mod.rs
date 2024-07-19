use std::sync::Arc;
use hal::{buffer::{Buffer, BufferId, InvalidBufferId}, depth_texture::DepthTexture};
use game_loop::winit::{
    dpi::PhysicalSize, 
    window::Window,
};
use pbr::{camera::{Camera, CameraBuffer}, mesh::Vertex, transform::Transform};
use hal::pipeline::RenderPipelines;

pub mod error;
pub mod voxel;
pub mod pbr;
pub mod hal;

#[allow(dead_code)]
pub struct Renderer {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    pub vertex_buffers: Vec<Buffer<Vertex>>,
    pub camera_buffer: CameraBuffer,
    pub render_pipelines: RenderPipelines,
    pub depth_texture: DepthTexture,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Renderer> {
        let size = window.inner_size();

        let instance = Self::init_instance();
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = Self::init_adapter(instance, &surface).await;
        let (device, queue) = Self::init_device(&adapter).await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = Self::init_config(surface_format, size, surface_caps);

        let camera_buffer = CameraBuffer::new(&device, &queue);
        let render_pipelines = RenderPipelines::new(&device, &config, &[
            camera_buffer.bind_group_layout(),
        ]);

        let depth_texture = DepthTexture::new(&device, &config);

        Ok(Renderer {
            surface,
            device,
            queue,
            config,
            size,
            window,
            vertex_buffers: vec![],
            camera_buffer,
            render_pipelines,
            depth_texture,
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
        self.depth_texture = DepthTexture::new(&self.device, &self.config);
    }

    pub fn create_vertex_buffer(&mut self, capacity: usize) -> BufferId {
        let id = self.vertex_buffers.len();

        self.vertex_buffers.push(Buffer::new(
            &self.device, 
            capacity, 
            wgpu::BufferUsages::VERTEX,
        ));

        id
    }

    pub fn update_vertex_buffer(&mut self, id: BufferId, data: &[Vertex]) -> Result<(), InvalidBufferId> {
        self.vertex_buffers
            .get_mut(id)
            .ok_or(InvalidBufferId(id))?
            .fill(&self.device, &self.queue, data);

        Ok(())
    }

    pub fn update_camera(&self, camera: &Camera, transform: &Transform) {
        self.camera_buffer.update(camera, transform, &self.queue);
    }

    async fn init_device(adapter: &wgpu::Adapter) -> Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError> {
        adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::PUSH_CONSTANTS,
                required_limits: wgpu::Limits {
                    max_push_constant_size: 128,
                    ..Default::default()
                },
                label: Some("Logical device"),
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
}

pub trait Renderable {
    fn update(&mut self, renderer: &mut Renderer);

    fn vertex_buffer(&self) -> BufferId;
}