use crate::renderer::{pbr::mesh::Vertex, Renderer};

#[macro_export]
macro_rules! include_wgsl {
    ($token:tt) => {
        {
            #[$crate::renderer::include_wgsl_oil($token)]
            mod shader {}

            $crate::renderer::hal::pipeline::Shader {
                label: Some($token),
                source: $crate::renderer::types::ShaderSource::Wgsl(shader::SOURCE.into()),
            }
        }
    };
}

use bytemuck::Pod;
pub use include_wgsl;

use super::{buffer::{Buffer, BufferResourceDescriptor}, texture::{Texture, TextureResourceDescriptor, TextureResourceUsage}};

pub type Shader = wgpu::ShaderModuleDescriptor<'static>;

pub struct ShaderResourceBuilder<'a> {
    label: Option<String>,
    bind_group_layout_entries: Vec<wgpu::BindGroupLayoutEntry>,
    bind_group_entries: Vec<wgpu::BindGroupEntry<'a>>,
}

impl<'a> ShaderResourceBuilder<'a> {
    pub fn set_label(&mut self, label: String) -> &mut Self {
        self.label = Some(label);
        self
    }

    pub fn add_buffer<T: Pod>(
        &mut self,
        buffer: &'a Buffer<T>,
        descriptor: &BufferResourceDescriptor,
    ) -> &mut Self {
        self.bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.bind_group_layout_entries.len() as u32,
            visibility: descriptor.visibility,
            ty: wgpu::BindingType::Buffer {
                ty: descriptor.buffer_type,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });

        self.bind_group_entries.push(wgpu::BindGroupEntry {
            binding: self.bind_group_entries.len() as u32,
            resource: buffer.inner().as_entire_binding(),
        });

        self
    }

    pub fn add_texture(
        &mut self,
        texture: &'a Texture,
        descriptor: &TextureResourceDescriptor,
    ) -> &mut Self {
        let view_dimension = match texture.description().dimension {
            wgpu::TextureDimension::D1 => wgpu::TextureViewDimension::D1,
            wgpu::TextureDimension::D2 => wgpu::TextureViewDimension::D2,
            wgpu::TextureDimension::D3 => wgpu::TextureViewDimension::D3,
        };

        let bind_group_layout_entries = descriptor.usage
            .iter()
            .enumerate()
            .filter_map(|(i, usage)| {
                match usage {
                    TextureResourceUsage::TEXTURE => {
                        Some(wgpu::BindGroupLayoutEntry {
                            binding: (self.bind_group_layout_entries.len() + i) as u32,
                            visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Texture {
                                sample_type: descriptor.sample_type.unwrap_or_else(|| {
                                    panic!("Must specify sample type for texture with TextureResourceUsage::TEXTURE");
                                }),
                                view_dimension,
                                multisampled: false,
                            },
                            count: None,
                        })
                    },
                    TextureResourceUsage::SAMPLER => {
                        Some(wgpu::BindGroupLayoutEntry {
                            binding: (self.bind_group_layout_entries.len() + i) as u32,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        })
                    },
                    TextureResourceUsage::STORAGE => {
                        Some(wgpu::BindGroupLayoutEntry {
                            binding: (self.bind_group_layout_entries.len() + i) as u32,
                            visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::WriteOnly,
                                format: texture.description().format,
                                view_dimension,
                            },
                            count: None,
                        })
                    },
                    _ => None,
                }
            })
            .collect::<Vec<_>>();

        let bind_group_entries = descriptor.usage
            .iter()
            .enumerate()
            .filter_map(|(i, usage)| {
                match usage {
                    TextureResourceUsage::STORAGE | TextureResourceUsage::TEXTURE => {
                        Some(wgpu::BindGroupEntry {
                            binding: (self.bind_group_entries.len() + i) as u32,
                            resource: wgpu::BindingResource::TextureView(texture.view())
                        },)
                    },
                    TextureResourceUsage::SAMPLER => {
                        Some(wgpu::BindGroupEntry {
                            binding: (self.bind_group_entries.len() + i) as u32,
                            resource: wgpu::BindingResource::Sampler(texture.sampler()),
                        })
                    },
                    _ => None,
                }
            })
            .collect::<Vec<_>>();

        self.bind_group_layout_entries.extend(bind_group_layout_entries);
        self.bind_group_entries.extend(bind_group_entries);

        self
    }

    pub fn build(&self, renderer: &Renderer) -> ShaderResource {
        let bind_group_layout = renderer.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: self.label
                .as_ref()
                .map(|label| format!("{label} bind group layout"))
                .as_deref(),
            entries: &self.bind_group_layout_entries,
        });

        let bind_group = renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: self.label
                .as_ref()
                .map(|label| format!("{label} bind group"))
                .as_deref(),
            layout: &bind_group_layout,
            entries: &self.bind_group_entries,
        });

        ShaderResource { bind_group_layout, bind_group }
    }
}

#[derive(Debug)]
pub struct ShaderResource {
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) bind_group: wgpu::BindGroup,
}

impl ShaderResource {
    pub fn builder<'a>() -> ShaderResourceBuilder<'a> {
        ShaderResourceBuilder {
            bind_group_layout_entries: vec![],
            bind_group_entries: vec![],
            label: None,
        }
    }
}

pub trait ShaderBinding {
    
}

pub enum Pipeline {
    Render(wgpu::RenderPipeline),
    Compute(wgpu::ComputePipeline),
}

impl Pipeline {
    pub fn new_render(
        renderer: &Renderer,
        shader: Shader,  
        bindings: &[&ShaderResource],
        label: &str,
        use_vertices: bool,
    ) -> Pipeline {
        let shader = renderer.device.create_shader_module(shader);

        let layout = renderer.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(format!("{label} Pipeline Layout").as_str()),
            bind_group_layouts: &bindings
                .to_vec()
                .iter()
                .map(|b| &b.bind_group_layout)
                .collect::<Vec<_>>(),
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX,
                range: 0..128,
            }],
        });

        let buffers = if use_vertices {
            vec![Vertex::vertex_buffer_layout()]
        } else {
            vec![]
        };

        let pipeline = renderer.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(format!("{label} Pipeline").as_str()),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", 
                buffers: &buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: renderer.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, 
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, 
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1, 
                mask: !0, 
                alpha_to_coverage_enabled: false, 
            },
            multiview: None, 
        });

        Pipeline::Render(pipeline)
    }

    pub fn new_compute(
        renderer: &Renderer,
        shader: Shader,  
        bindings: &[&ShaderResource],
        label: &str,
    ) -> Pipeline {
        let shader = renderer.device.create_shader_module(shader);

        let layout = renderer.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(format!("{label} Pipeline Layout").as_str()),
            bind_group_layouts: &bindings
                .to_vec()
                .iter()
                .map(|b| &b.bind_group_layout)
                .collect::<Vec<_>>(),
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::COMPUTE,
                range: 0..128,
            }],
        });

        let pipeline = renderer.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(format!("{label} Pipeline layout").as_str()),
            layout: Some(&layout),
            module: &shader,
            entry_point: "cs_main",
        });

        Pipeline::Compute(pipeline)
    }
}