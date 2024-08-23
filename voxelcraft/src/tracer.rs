use tracengine::renderer::{
    hal::{
        buffer::{Buffer, BufferResourceDescriptor},
        pipeline::{include_wgsl, Pipeline, ShaderResource}, 
        taa::Taa, 
        texture::{Texture, TextureDescriptor, TextureResourceDescriptor, TextureResourceUsage}
    }, 
    rt::{
        camera::{RtCamera, RtCameraDescriptor, RtCameraUniform},
        transform::RtTransform,
    }, 
    types::*,
    voxel::{
        chunk::Chunk, 
        model::VoxelModel,
    }, 
    InstanceData, Renderer
};
use tracengine::glm;

use crate::camera::CameraConfiguration;

const CHUNKS_RENDER_DISTANCE: u32 = 3;

const fn chunks_count() -> u32 {
    let distance = [CHUNKS_RENDER_DISTANCE, 1][(CHUNKS_RENDER_DISTANCE < 1) as usize];
    (2 * distance + 1) * (2 * distance + 1)
}

pub struct Tracer {
    pub taa: Taa,

    pub camera_buffer: Buffer<RtCameraUniform>,

    pub color_buffer: Buffer<glm::Vec4>,
    pub depth_buffer: Buffer<f32>,
    pub normal_buffer: Buffer<glm::Vec4>,

    pub palettes_buffer: Buffer<glm::Vec4>,
    pub chunks_3d_texture: Texture,
    pub shader_resource: ShaderResource,

    pub rt_pipeline: Pipeline,
    pub taa_pipeline: Pipeline,

    pub chunk: Chunk,
    pub camera: RtCamera,
    pub tmp_transform: RtTransform,
    pub camera_config: CameraConfiguration,
}

impl Tracer {
    pub fn new(renderer: &mut Renderer) -> Tracer {
        // Init TAA instance
        let taa = Taa::new(renderer);

        // Init color buffer
        let viewport_size = (renderer.size().width * renderer.size().height) as usize;
        let color_buffer = Buffer::new(renderer, viewport_size, BufferUsages::STORAGE);

        // Init depth buffer
        let depth_buffer = Buffer::new(renderer, viewport_size, BufferUsages::STORAGE);

        // Init normal buffer
        let normal_buffer = Buffer::new(renderer, viewport_size, BufferUsages::STORAGE);

        // Init camera
        let camera_config = CameraConfiguration {
            limit: -85.0..85.0,
            ..Default::default()
        };

        let mut camera = RtCamera::new(&RtCameraDescriptor {
            image_width: renderer.size().width,
            image_height: renderer.size().height,
            scan_depth: 2,
            jitter: taa.current_jitter,
        });

        let camera_buffer = Buffer::new(renderer, 1, BufferUsages::UNIFORM | BufferUsages::COPY_DST);
        camera_buffer.fill_exact(renderer, 0, &[camera.uniform_data()]).unwrap();

        // Init chunks
        let chunk = VoxelModel::load_vox("../assets/vox/model2.vox")
            .unwrap()
            .swap_remove(0)
            .into_chunks()
            .swap_remove(0)
            .chunk;

        let chunks_3d_texture = Texture::new(renderer, TextureDescriptor {
            width: Chunk::CHUNK_SIZE as u32,
            height: Chunk::CHUNK_SIZE as u32,
            depth: Some(Chunk::CHUNK_SIZE as u32 * chunks_count()),
            filter: FilterMode::Nearest,
            dimension: TextureDimension::D3,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            format: TextureFormat::Rgba8Uint,
            label: "Chunks",
        });

        let palettes_buffer = Buffer::new(renderer, 256 * chunks_count() as usize, BufferUsages::STORAGE);
        
        chunk.write_to_texture(renderer, 
            &chunks_3d_texture,
            &palettes_buffer,
            0,
        ).unwrap();

        // TODO: local transformations
        // Init transform
        let tmp_transform = RtTransform::default();

        // Init shader resource
        let shader_resource = ShaderResource::builder()
            .add_buffer(&camera_buffer, &BufferResourceDescriptor {
                visibility: ShaderStages::COMPUTE,
                buffer_type: BufferBindingType::Uniform,
            })
            .add_buffer(&color_buffer, &BufferResourceDescriptor {
                visibility: ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
                buffer_type: BufferBindingType::Storage { read_only: false },
            })
            .add_buffer(&normal_buffer, &BufferResourceDescriptor {
                visibility: ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
                buffer_type: BufferBindingType::Storage { read_only: false },
            })
            .add_buffer(&depth_buffer, &BufferResourceDescriptor {
                visibility: ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
                buffer_type: BufferBindingType::Storage { read_only: false },
            })
            .add_buffer(&palettes_buffer, &BufferResourceDescriptor {
                visibility: ShaderStages::COMPUTE,
                buffer_type: BufferBindingType::Storage { read_only: true },
            })
            .add_texture(&chunks_3d_texture, &TextureResourceDescriptor {
                usage: TextureResourceUsage::TEXTURE | TextureResourceUsage::SAMPLER,
                sample_type: Some(TextureSampleType::Uint),
            })
            .build(renderer);

        // Init pipelines
        let rt_pipeline = Pipeline::new_compute(
            renderer, 
            include_wgsl!("../../assets/shaders/rt_shader.wgsl"),
            &[&taa.shader_resource, &shader_resource], 
            "Ray tracing"
        );

        let taa_pipeline = Pipeline::new_render(
            renderer,
            include_wgsl!("../../assets/shaders/taa_shader.wgsl"),
            &[&taa.shader_resource, &shader_resource],
            "TAA",
            false,
        );  

        Tracer {
            taa,
            camera_buffer,
            color_buffer,
            normal_buffer,
            depth_buffer,
            chunks_3d_texture,
            palettes_buffer,
            shader_resource,
            rt_pipeline,
            taa_pipeline,
            chunk,
            camera,
            tmp_transform,
            camera_config,
        }
    }

    pub fn rebind_resources(&mut self, renderer: &mut Renderer) {
        self.shader_resource = ShaderResource::builder()
            .add_buffer(&self.camera_buffer, &BufferResourceDescriptor {
                visibility: ShaderStages::COMPUTE,
                buffer_type: BufferBindingType::Uniform,
            })
            .add_buffer(&self.color_buffer, &BufferResourceDescriptor {
                visibility: ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
                buffer_type: BufferBindingType::Storage { read_only: false },
            })
            .add_buffer(&self.normal_buffer, &BufferResourceDescriptor {
                visibility: ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
                buffer_type: BufferBindingType::Storage { read_only: false },
            })
            .add_buffer(&self.depth_buffer, &BufferResourceDescriptor {
                visibility: ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
                buffer_type: BufferBindingType::Storage { read_only: false },
            })
            .add_buffer(&self.palettes_buffer, &BufferResourceDescriptor {
                visibility: ShaderStages::COMPUTE,
                buffer_type: BufferBindingType::Storage { read_only: true },
            })
            .add_texture(&self.chunks_3d_texture, &TextureResourceDescriptor {
                usage: TextureResourceUsage::TEXTURE | TextureResourceUsage::SAMPLER,
                sample_type: Some(TextureSampleType::Uint),
            })
            .build(renderer);
    }
}