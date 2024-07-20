/// A structure representing a depth texture, including its view and sampler.
pub struct DepthTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl DepthTexture {
    /// Creates a new depth texture with the specified device and surface configuration.
    ///
    /// # Arguments
    ///
    /// * `device` - A reference to the wgpu device.
    /// * `config` - A reference to the surface configuration.
    ///
    /// # Returns
    ///
    /// A new instance of `DepthTexture`.
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> DepthTexture {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Depth texture sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        DepthTexture { 
            texture, 
            view, 
            sampler,
        }
    }

    /// Returns a reference to the inner wgpu texture.
    ///
    /// # Returns
    ///
    /// A reference to the `wgpu::Texture`.
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    /// Returns a reference to the texture view.
    ///
    /// # Returns
    ///
    /// A reference to the `wgpu::TextureView`.
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// Returns a reference to the sampler.
    ///
    /// # Returns
    ///
    /// A reference to the `wgpu::Sampler`.
    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }
}
