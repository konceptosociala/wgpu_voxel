use std::{collections::HashMap, ops::Deref};

use crate::renderer::pbr::mesh::Vertex;

pub struct Pipeline {
    shader: wgpu::ShaderModule,
    layout: wgpu::PipelineLayout,
    pipeline: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn new(
        shader: wgpu::ShaderModuleDescriptor<'_>, 
        label: &str, 
        device: &wgpu::Device, 
        config: &wgpu::SurfaceConfiguration,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> Pipeline {
        let shader = device.create_shader_module(shader);

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(format!("{label} Pipeline Layout").as_str()),
            bind_group_layouts,
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX,
                range: 0..128,
            }],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(format!("{label} Pipeline").as_str()),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", 
                buffers: &[Vertex::desc()], 
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
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

        Pipeline {
            shader,
            layout,
            pipeline,
        }
    }
    
    pub fn shader(&self) -> &wgpu::ShaderModule {
        &self.shader
    }
    
    pub fn layout(&self) -> &wgpu::PipelineLayout {
        &self.layout
    }
}

impl Deref for Pipeline {
    type Target = wgpu::RenderPipeline;

    fn deref(&self) -> &Self::Target {
        &self.pipeline
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum PipelineKey {
    MainPipeline,
}

pub struct RenderPipelines {
    pipelines: HashMap<PipelineKey, Pipeline>,
}

impl RenderPipelines {
    pub fn new(
        device: &wgpu::Device, 
        config: &wgpu::SurfaceConfiguration,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> RenderPipelines {
        RenderPipelines {
            pipelines: HashMap::from([
                (
                    PipelineKey::MainPipeline,
                    Pipeline::new(
                        wgpu::include_wgsl!("../shaders/main_shader.wgsl"), 
                        "Main", 
                        device, 
                        config,
                        bind_group_layouts,
                    ),
                )
            ])
        }
    }

    pub fn get(&self, key: &PipelineKey) -> &Pipeline {
        self.pipelines.get(key).unwrap()
    }
}