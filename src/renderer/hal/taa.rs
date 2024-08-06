use bytemuck::{Pod, Zeroable};
// use derive_getters::Getters;
use rand::Rng;
// use super::{buffer::{Buffer, BufferResource}, texture::{Texture, TextureResource}};

// pub type Colorf32 = nalgebra_glm::Vec4;

// structstruck::strike! {
//     #[strikethrough[derive(Getters)]]
//     pub struct Taa {
//         history_texture: TextureResource,
//         velocity_texture: TextureResource,
//         config_buffer: BufferResource<
//             #[repr(C)]
//             #[derive(Debug, Clone, Copy, Zeroable, Pod)]
//             pub struct TaaConfig {
//                 jitter: f32,
//             }
//         >,
//     }
// }

// #[derive(Getters)]
// pub struct TaaTexture {
//     in_texture: Texture,
//     out_texture: Texture,
//     bind_group_layout: wgpu::BindGroupLayout,
//     bind_group: wgpu::BindGroup,
// }

// impl TaaTexture {
//     pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, label: &'static str) -> TaaTexture {
//         let in_texture = Texture::new(
//             device, 
//             config, 
//             wgpu::FilterMode::Linear, 
//             wgpu::TextureDimension::D2, 
//             wgpu::TextureFormat::Rgba8Unorm,
//             wgpu::TextureUsages::TEXTURE_BINDING 
//                 | wgpu::TextureUsages::COPY_SRC
//                 | wgpu::TextureUsages::STORAGE_BINDING,
//             None,
//             label,
//         );

//         let out_texture = Texture::new(
//             device, 
//             config, 
//             wgpu::FilterMode::Linear, 
//             wgpu::TextureDimension::D2, 
//             wgpu::TextureFormat::Rgba8Unorm,
//             wgpu::TextureUsages::TEXTURE_BINDING 
//                 | wgpu::TextureUsages::COPY_DST,
//             None,
//             label,
//         );

//         let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//             label: Some(format!("{label} texture bind group layout").as_str()),
//             entries: &[
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 0,
//                     visibility: wgpu::ShaderStages::COMPUTE,
//                     ty: wgpu::BindingType::StorageTexture {
//                         access: wgpu::StorageTextureAccess::WriteOnly,
//                         format: wgpu::TextureFormat::Rgba8Unorm,
//                         view_dimension: wgpu::TextureViewDimension::D2,
//                     },
//                     count: None,
//                 },
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 1,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Texture {
//                         sample_type: wgpu::TextureSampleType::Float { filterable: true },
//                         view_dimension: wgpu::TextureViewDimension::D2,
//                         multisampled: false,
//                     },
//                     count: None,
//                 },
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 2,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
//                     count: None,
//                 }
//             ],
//         });

//         let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
//             label: Some(format!("{label} texture bind group").as_str()),
//             layout: &bind_group_layout,
//             entries: &[
//                 wgpu::BindGroupEntry {
//                     binding: 0,
//                     resource: wgpu::BindingResource::TextureView(in_texture.view())
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 1, 
//                     resource: wgpu::BindingResource::TextureView(out_texture.view()),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 2,
//                     resource: wgpu::BindingResource::Sampler(out_texture.sampler()),
//                 }
//             ]
//         });

//         TaaTexture { in_texture, out_texture, bind_group_layout, bind_group }
//     }
// }

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

// impl TaaConfigBuffer {
//     pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> TaaConfigBuffer {
//         let inner = Buffer::new(device, 1, wgpu::BufferUsages::UNIFORM);
//             inner.fill_exact(queue, &[TaaConfig::new()]).unwrap();

//             let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//                 label: Some("TAA config bind group layout"),
//                 entries: &[
//                     wgpu::BindGroupLayoutEntry {
//                         binding: 0,
//                         visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
//                         ty: wgpu::BindingType::Buffer {
//                             ty: wgpu::BufferBindingType::Uniform,
//                             has_dynamic_offset: false,
//                             min_binding_size: None,
//                         },
//                         count: None,
//                     }
//                 ],
//             });

//             let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
//                 label: Some("TAA config bind group"),
//                 layout: &bind_group_layout,
//                 entries: &[wgpu::BindGroupEntry {
//                     binding: 0, 
//                     resource: inner.inner().as_entire_binding(),
//                 }]
//             });

//             TaaConfigBuffer { inner, bind_group_layout, bind_group }
//     }
// }

// impl Taa {
//     pub fn new(
//         device: &wgpu::Device, 
//         queue: &wgpu::Queue, 
//         config: &wgpu::SurfaceConfiguration,
//     ) -> Taa {
//         Taa {
//             history_texture: TaaTexture::new(device, config, "TAA history"),
//             velocity_texture: TaaTexture::new(device, config, "TAA velocity"),
//             config_buffer: TaaConfigBuffer::new(device, queue),
//         }
//     }
// }