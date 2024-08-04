use std::collections::HashMap;
use derive_getters::Getters;

use crate::renderer::pbr::mesh::Vertex;

/// Represents a GPU rendering pipeline, including the shader module,
/// pipeline layout, and render pipeline.
#[derive(Getters)]
pub struct Pipeline {
    inner: wgpu::RenderPipeline,
    shader: wgpu::ShaderModule,
    layout: wgpu::PipelineLayout,
}

impl Pipeline {
    /// Creates a new rendering pipeline with the specified shader, label, device,
    /// surface configuration, and bind group layouts.
    ///
    /// # Arguments
    ///
    /// * `shader` - The descriptor for the shader module.
    /// * `label` - A label for the pipeline.
    /// * `device` - A reference to the wgpu device.
    /// * `config` - A reference to the surface configuration.
    /// * `bind_group_layouts` - A slice of references to bind group layouts.
    ///
    /// # Returns
    ///
    /// A new instance of `Pipeline`.
    pub fn new(
        shader: wgpu::ShaderModuleDescriptor<'_>, 
        label: &str, 
        device: &wgpu::Device, 
        config: &wgpu::SurfaceConfiguration,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        buffers: &[wgpu::VertexBufferLayout<'_>],
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
                buffers, 
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
            inner: pipeline,
        }
    }
}

/// Enum representing the keys for different rendering pipelines.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum PipelineKey {
    MainPipeline,
    RtPipeline,
}

/// A collection of rendering pipelines identified by `PipelineKey`.
pub struct RenderPipelines {
    pipelines: HashMap<PipelineKey, Pipeline>,
}

impl RenderPipelines {
    /// Creates a new set of rendering pipelines with the specified device, surface configuration,
    /// and bind group layouts.
    ///
    /// # Arguments
    ///
    /// * `device` - A reference to the wgpu device.
    /// * `config` - A reference to the surface configuration.
    /// * `bind_group_layouts` - A slice of references to bind group layouts.
    ///
    /// # Returns
    ///
    /// A new instance of `RenderPipelines`.
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
                        &[Vertex::desc()],
                    ),
                ),
                (
                    PipelineKey::RtPipeline,
                    Pipeline::new(
                        wgpu::include_wgsl!("../shaders/rt_shader.wgsl"), 
                        "Rt", 
                        device, 
                        config,
                        bind_group_layouts,
                        &[],
                    ),
                ),
            ])
        }
    }

    /// Retrieves a reference to the pipeline associated with the given key.
    ///
    /// # Arguments
    ///
    /// * `key` - A reference to the `PipelineKey` identifying the desired pipeline.
    ///
    /// # Returns
    ///
    /// A reference to the corresponding `Pipeline`.
    pub fn get(&self, key: &PipelineKey) -> &Pipeline {
        self.pipelines.get(key).unwrap()
    }
}
