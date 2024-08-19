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

pub use include_wgsl;

pub type Shader = wgpu::ShaderModuleDescriptor<'static>;

pub trait ShaderBinding {
    fn get_resource(&self) -> &ShaderResource;
}

#[derive(Debug)]
pub struct ShaderResource {
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) bind_group: wgpu::BindGroup,
}

pub enum Pipeline {
    Render(wgpu::RenderPipeline),
    Compute(wgpu::ComputePipeline),
}

impl Pipeline {
    pub fn new_render(
        renderer: &Renderer,
        shader: Shader,  
        bindings: &[&dyn ShaderBinding],
        label: &str,
        use_vertices: bool,
    ) -> Pipeline {
        let shader = renderer.device.create_shader_module(shader);

        let layout = renderer.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(format!("{label} Pipeline Layout").as_str()),
            bind_group_layouts: &bindings
                .to_vec()
                .iter()
                .map(|b| &b.get_resource().bind_group_layout)
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
        bindings: &[&dyn ShaderBinding],
        label: &str,
    ) -> Pipeline {
        let shader = renderer.device.create_shader_module(shader);

        let layout = renderer.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(format!("{label} Pipeline Layout").as_str()),
            bind_group_layouts: &bindings
                .to_vec()
                .iter()
                .map(|b| &b.get_resource().bind_group_layout)
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