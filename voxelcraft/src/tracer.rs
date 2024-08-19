use tracengine::renderer::{
    hal::{
        buffer::{Buffer, BufferResource}, 
        pipeline::{include_wgsl, Pipeline}, 
        taa::Taa, 
        texture::{Texture, TextureDescriptor, TextureResource, TextureResourceUsage}
    }, rt::{
        camera::{RtCamera, RtCameraUniform},
        transform::RtTransform,
    }, types::*, voxel::{
        chunk::Chunk, 
        model::VoxelModel,
    }, InstanceData, Renderer
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
    pub camera_buffer: BufferResource<RtCameraUniform>,
    pub color_buffer: BufferResource<glm::Vec4>,
    pub chunks_3d_texture: TextureResource,
    pub palettes: BufferResource<glm::Vec4>,
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
        let color_buffer = BufferResource::new(
            renderer,
            Buffer::new(renderer, viewport_size, BufferUsages::STORAGE),
            ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
            BufferBindingType::Storage { read_only: false },
        );

        // Init camera
        let camera_config = CameraConfiguration {
            limit: -85.0..85.0,
            ..Default::default()
        };

        let mut camera = RtCamera::new(
            renderer.size().width, 
            renderer.size().height, 
            10,
            taa.current_jitter(),
        );

        let camera_buffer = BufferResource::new(
            renderer,
            Buffer::new(renderer, 1, BufferUsages::UNIFORM | BufferUsages::COPY_DST),
            ShaderStages::COMPUTE,
            BufferBindingType::Uniform,
        );
        camera_buffer.buffer.fill_exact(renderer, 0, &[camera.uniform_data()]).unwrap();

        // Init chunks
        let chunk = VoxelModel::load_vox("../assets/vox/model2.vox")
            .unwrap()
            .swap_remove(0)
            .into_chunks()
            .swap_remove(0)
            .chunk;

        let chunks_3d_texture = TextureResource::new(
            renderer,
            Texture::new(renderer, TextureDescriptor {
                width: Chunk::CHUNK_SIZE as u32,
                height: Chunk::CHUNK_SIZE as u32,
                depth: Some(Chunk::CHUNK_SIZE as u32 * chunks_count()),
                filter: FilterMode::Nearest,
                dimension: TextureDimension::D3,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                format: TextureFormat::Rgba8Uint,
                label: "Chunks",
            }),
            TextureResourceUsage::TEXTURE | TextureResourceUsage::SAMPLER,
            Some(TextureSampleType::Uint),
        );

        let palettes = BufferResource::new(
            renderer,
            Buffer::new(renderer, 256 * chunks_count() as usize, BufferUsages::STORAGE),
            ShaderStages::COMPUTE,
            BufferBindingType::Storage { read_only: true },
        );

        chunk.write_to_texture(
            renderer, 
            &chunks_3d_texture.texture,
            &palettes.buffer,
            0,
        ).unwrap();

        // Init pipelines
        let rt_pipeline = Pipeline::new_compute(
            renderer, 
            include_wgsl!("../../assets/shaders/rt_shader.wgsl"),
            &[
                &taa.config_buffer,
                &camera_buffer,
                &color_buffer,
                &taa.velocity_buffer,
                &chunks_3d_texture,
                &palettes,
            ], 
            "Ray tracing"
        );

        let taa_pipeline = Pipeline::new_render(
            renderer,
            include_wgsl!("../../assets/shaders/taa_shader.wgsl"),
            &[
                &taa.config_buffer,
                &taa.history_texture,
                &taa.velocity_buffer,
                &color_buffer,
            ],
            "TAA",
            false,
        );

        // TODO: local transformations
        // Init transform
        let tmp_transform = RtTransform::default();

        Tracer {
            taa,
            camera_buffer,
            color_buffer,
            chunks_3d_texture,
            palettes,
            rt_pipeline,
            taa_pipeline,
            chunk,
            camera,
            tmp_transform,
            camera_config,
        }
    }
}