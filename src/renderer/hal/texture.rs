/// A structure representing a depth texture, including its view and sampler.
pub struct Texture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    filter: wgpu::FilterMode,
    dimension: wgpu::TextureDimension,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
    depth: Option<u32>,
    label: &'static str,
}

// TODO: refactor texture construction
#[allow(clippy::too_many_arguments)]
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
        device: &wgpu::Device, 
        config: &wgpu::SurfaceConfiguration,
        filter: wgpu::FilterMode,
        dimension: wgpu::TextureDimension,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        depth: Option<u32>,
        label: &'static str,
    ) -> Texture {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: depth.unwrap_or(1),
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(format!("{label} texture").as_str()),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension,
            format,
            usage,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(format!("{label} texture sampler").as_str()),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: filter,
            min_filter: filter,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Texture { 
            texture, 
            view, 
            sampler,
            filter,
            dimension,
            format,
            usage,
            depth,
            label,
        }
    }

    pub fn recreate(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        *self = Texture::new(
            device, 
            config, 
            self.filter, 
            self.dimension, 
            self.format, 
            self.usage, 
            self.depth,
            self.label,
        )
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
