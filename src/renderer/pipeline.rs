use std::ops::Deref;

use super::pbr::mesh::Vertex;

pub struct Pipeline {
    shader: wgpu::ShaderModule,
    layout: wgpu::PipelineLayout,
    pipeline: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn new(
        shader: wgpu::ShaderModuleDescriptor, 
        label: &str, 
        device: &wgpu::Device, 
        config: &wgpu::SurfaceConfiguration,
    ) -> Pipeline {
        let shader = device.create_shader_module(shader);

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(format!("{label} Pipeline Layout").as_str()),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
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
            depth_stencil: None, 
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
}

impl Deref for Pipeline {
    type Target = wgpu::RenderPipeline;

    fn deref(&self) -> &Self::Target {
        &self.pipeline
    }
}

pub struct RenderPipelines {
    pub main_pipeline: Pipeline,
}

impl RenderPipelines {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> RenderPipelines {
        RenderPipelines {
            main_pipeline: Pipeline::new(
                wgpu::include_wgsl!("shaders/main_shader.wgsl"), 
                "Main", 
                device, 
                config
            )
        }
    }
}