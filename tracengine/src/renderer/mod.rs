use std::sync::Arc;
use bytemuck::Pod;
use error::RenderError;
use hal::{
    buffer::{Buffer, BufferId, InvalidBufferId}, 
    pipeline::{Pipeline, ShaderBinding}, 
    texture::*
};
use game_loop::winit::{
    dpi::PhysicalSize,
    window::Window,
};
use pbr::mesh::Vertex;

pub mod error;
pub mod voxel;
pub mod pbr;
pub mod hal;
pub mod rt;

pub mod types {
    pub use wgpu::{
        BufferUsages,
        ShaderStages,
        BufferBindingType,
        FilterMode,
        TextureDimension,
        TextureUsages,
        TextureFormat,
        TextureSampleType,
        Extent3d,
        ShaderSource,
        TextureView,
    };
}

pub use include_wgsl_oil::include_wgsl_oil;

/// Represents a renderer that handles drawing to a window using wgpu.
pub struct Renderer {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    vertex_buffers: Vec<Buffer<Vertex>>,
    depth_texture: Option<Texture>,
}

impl Renderer {
    /// Creates a new `Renderer` instance.
    ///
    /// # Parameters
    /// - `window`: A shared Window instance that represents the window to render into.
    ///
    /// # Returns
    /// A `Result` containing the `Renderer` instance or an error if creation fails.
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

        let mut renderer = Renderer {
            surface,
            device,
            queue,
            config,
            size,
            window,
            vertex_buffers: vec![],
            depth_texture: None,
        };

        renderer.depth_texture = Some(Texture::new(
            &renderer, 
            TextureDescriptor {
                width: renderer.config.width,
                height: renderer.config.height,
                filter: wgpu::FilterMode::Linear,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                depth: None,
                label: "Depth data",
            },
        ));

        Ok(renderer)
    }

    /// Retrieves the current canvas for drawing.
    ///
    /// # Returns
    /// A `Result` containing the `Canvas` or an error if retrieval fails.
    pub fn canvas(&self) -> Result<Canvas, RenderError> {
        let texture = self.surface.get_current_texture()?;
        let view = texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(Canvas { texture, view })
    }

    /// Creates a new drawing context for issuing draw commands.
    ///
    /// # Returns
    /// A `DrawContext` that can be used for issuing draw commands.
    pub fn draw_ctx(&self) -> DrawContext {
        DrawContext {
            encoder: self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default()),
        }
    }

    /// Retrieves the window associated with the renderer.
    ///
    /// # Returns
    /// An `Arc<Window>` representing the window.
    pub fn window(&self) -> Arc<Window> {
        self.window.clone()
    }

    /// Resizes the renderer to the current window size.
    pub fn resize(&mut self) {
        self.resize_with(self.size);
    }

    /// Resizes the renderer to a specified size.
    ///
    /// # Parameters
    /// - `new_size`: The new size for the renderer.
    pub fn resize_with(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 { return }

        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);

        if let Some(depth_texture) = &self.depth_texture {
            let mut depth_descr = *depth_texture.description();
            depth_descr.width = self.config.width;
            depth_descr.height = self.config.height;
            self.depth_texture = Some(Texture::new(self, depth_descr));
        }
    }

    /// Creates a new vertex buffer with a specified capacity.
    ///
    /// # Parameters
    /// - `capacity`: The capacity of the vertex buffer.
    ///
    /// # Returns
    /// The ID of the newly created vertex buffer.
    pub fn create_vertex_buffer(&mut self, capacity: usize) -> BufferId {
        let id = self.vertex_buffers.len();

        self.vertex_buffers.push(Buffer::new(
            self,
            capacity,
            wgpu::BufferUsages::VERTEX,
        ));

        BufferId(id)
    }

    /// Updates the data in an existing vertex buffer.
    ///
    /// # Parameters
    /// - `id`: The ID of the vertex buffer to update.
    /// - `data`: The new vertex data.
    ///
    /// # Returns
    /// A `Result` indicating success or failure. 
    pub fn update_vertex_buffer(&mut self, id: BufferId, data: &[Vertex]) -> Result<(), InvalidBufferId> {
        if self.vertex_buffers
            .get(id.0)
            .ok_or(InvalidBufferId(id))?
            .fill_exact(self, 0, data).is_err() 
            {
                let mut buffer = self.vertex_buffers.get(id.0).unwrap().clone();
                buffer.fill(self, 0, data);
                *self.vertex_buffers.get_mut(id.0).unwrap() = buffer;
            }

        Ok(())
    }

    /// Retrieves the current size of the renderer.
    ///
    /// # Returns
    /// The current size as a `PhysicalSize<u32>`.
    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    /// Retrieves the depth texture used for depth testing.
    ///
    /// # Returns
    /// A reference to the `Texture`.
    pub fn depth_texture(&self) -> Option<&Texture> {
        self.depth_texture.as_ref()
    }

    async fn init_device(adapter: &wgpu::Adapter) -> Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError> {
        adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::PUSH_CONSTANTS,
                required_limits: wgpu::Limits {
                    max_push_constant_size: 128,
                    max_bind_groups: 6,
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

/// Represents a drawing context used for issuing draw commands.
pub struct DrawContext {
    encoder: wgpu::CommandEncoder,
}

impl DrawContext {
    /// Begins a new render pass with the specified canvas and depth texture.
    ///
    /// # Parameters
    /// - `canvas`: The canvas to render to.
    /// - `depth_texture`: The depth texture to use for depth testing.
    ///
    /// # Returns
    /// A `RenderPass` instance for issuing draw commands.
    pub fn render_pass<'a>(
        &'a mut self,
        canvas: &'a impl RenderSurface,
        depth_texture: Option<&'a Texture>,
    ) -> RenderPass<'a> {
        let pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: canvas.view(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: depth_texture.map(|t| {
                wgpu::RenderPassDepthStencilAttachment {
                    view: t.view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        RenderPass { pass }
    }

    pub fn compute_pass(&mut self) -> ComputePass<'_> {
        let pass = self.encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute pass"),
            timestamp_writes: None,
        });

        ComputePass { pass }
    }

    pub fn clear_buffer<T>(&mut self, buffer: &Buffer<T>) {
        self.encoder.clear_buffer(buffer.inner(), 0, None);
    }

    pub fn copy_texture(&mut self, from: &Texture, to: &Texture) {
        self.encoder.copy_texture_to_texture(
            wgpu::ImageCopyTexture {
                texture: from.texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyTexture {
                texture: to.texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: to.description().width,
                height: to.description().height,
                depth_or_array_layers: 1,
            }
        );
    }

    /// Applies the drawing commands and presents the canvas.
    ///
    /// # Parameters
    /// - `canvas`: The canvas to present.
    /// - `renderer`: The renderer instance used to submit commands.
    pub fn apply(self, canvas: Canvas, renderer: &Renderer) {        
        renderer.queue.submit(std::iter::once(self.encoder.finish()));
        canvas.texture.present();
    }
}

pub struct ComputePass<'a> {
    pass: wgpu::ComputePass<'a>,
}

impl<'a> ComputePass<'a> {
    pub fn compute<T: Pod>(
        &mut self,
        instance_data: Option<&mut dyn InstanceData<UniformData = T>>,
        pipeline: &'a Pipeline,
        shader_bindings: &[&'a dyn ShaderBinding],
        size: PhysicalSize<u32>,
    ) {
        if let Pipeline::Compute(p) = pipeline {
            self.pass.set_pipeline(p);
        } else {
            panic!("Cannot use render pipeline in compute() command");
        }

        for (i, binding) in shader_bindings.iter().enumerate() {
            self.pass.set_bind_group(i as u32, &binding.get_resource().bind_group, &[]);
        }

        if let Some(instance_data) = instance_data {
            self.pass.set_push_constants(
                0,
                bytemuck::cast_slice(&[instance_data.uniform_data()]),
            );
        }

        self.pass.dispatch_workgroups(size.width, size.height, 1);
    }
}

/// Represents a render pass used for drawing.
pub struct RenderPass<'a> {
    pass: wgpu::RenderPass<'a>
}

impl<'a> RenderPass<'a> {
    pub fn draw<T: Pod>(
        &mut self,
        renderer: &'a Renderer,
        drawable: Option<&dyn Drawable>,
        instance_data: Option<&mut dyn InstanceData<UniformData = T>>,
        pipeline: &'a Pipeline,
        shader_bindings: &[&'a dyn ShaderBinding],
    ) {
        if let Pipeline::Render(p) = pipeline {
            self.pass.set_pipeline(p);
        } else {
            panic!("Cannot use compute pipeline in draw() command");
        }

        for (i, binding) in shader_bindings.iter().enumerate() {
            self.pass.set_bind_group(i as u32, &binding.get_resource().bind_group, &[]);
        }

        if let Some(instance_data) = instance_data {
            self.pass.set_push_constants(
                wgpu::ShaderStages::VERTEX,
                0,
                bytemuck::cast_slice(&[instance_data.uniform_data()]),
            );
        }
        
        if let Some(drawable) = drawable {
            self.pass.set_vertex_buffer(0, renderer.vertex_buffers[drawable.vertex_buffer().0].inner().slice(..)); 
            self.pass.draw(0..*renderer.vertex_buffers[drawable.vertex_buffer().0].capacity() as u32, 0..1);
        } else {
            self.pass.draw(0..6, 0..1);
        }
    }
}

pub trait RenderSurface {
    fn view(&self) -> &wgpu::TextureView;
}

/// Represents the canvas used for rendering.
pub struct Canvas {
    texture: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
}

impl RenderSurface for Canvas {
    fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}

/// Trait for drawable objects.
pub trait Drawable {
    /// Updates the drawable object's renderer data
    ///
    /// # Parameters
    /// - `renderer`: The renderer instance used to update the drawable.
    fn update(&mut self, renderer: &mut Renderer);

    /// Retrieves the ID of the vertex buffer used by the drawable.
    ///
    /// # Returns
    /// The ID of the vertex buffer.
    fn vertex_buffer(&self) -> BufferId;
}

pub trait InstanceData {
    type UniformData: Pod;

    fn uniform_data(&mut self) -> Self::UniformData;
}