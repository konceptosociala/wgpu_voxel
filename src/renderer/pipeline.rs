use wgpu::include_wgsl;

pub struct Shaders {
    main_shader: wgpu::ShaderModule,
}

impl Shaders {
    pub fn new(device: wgpu::Device) -> Shaders {
        Shaders {
            main_shader: device.create_shader_module(include_wgsl!("shaders/main_shader.wgsl"))
        }
    }
}

pub struct RenderPipelines {
    main_pipeline: wgpu::RenderPipeline,
}

impl RenderPipelines {
    pub fn new(device: wgpu::Device) -> RenderPipelines {
        RenderPipelines {
            main_pipeline: 
        }
    }
}