use wgpu_voxel::{
    engine::Engine, 
    event::{MouseScrollDelta, WindowEvent}, 
    renderer::{
        error::RenderError, hal::{
            buffer::{Buffer, BufferResource}, pipeline::{include_wgsl, Pipeline},
            taa::Taa, texture::{Texture, TextureDescriptor, TextureResource, TextureResourceUsage}
        }, rt::transform::RtTransform, voxel::{chunk::Chunk, model::VoxelModel}, Renderer
    }, 
    Game, PhysicalSize, WindowBuilder
};

use nalgebra_glm as glm;

const CHUNKS_RENDER_DISTANCE: u32 = 3;

const fn chunks_count() -> u32 {
    let distance = [CHUNKS_RENDER_DISTANCE, 1][(CHUNKS_RENDER_DISTANCE < 1) as usize];
    (2 * distance + 1) * (2 * distance + 1)
}

#[derive(Clone, Default, Debug)]
struct CameraConfiguration {
    limit: (f32, f32),
    target_x: f32,
    target_y: f32,
    latest_pos: glm::Vec2,
}

#[derive(Default)]
pub struct RayTracer {
    taa: Option<Taa>,
    color_buffer: Option<BufferResource<glm::Vec4>>,
    chunks_3d_texture: Option<TextureResource>,
    palettes: Option<BufferResource<glm::Vec4>>,
    rt_pipeline: Option<Pipeline>,
    taa_pipeline: Option<Pipeline>,
    chunk: Option<Chunk>,
    tmp_transform: RtTransform,
    camera_config: CameraConfiguration,
}

impl Engine for RayTracer {
    fn init(&mut self, _world: &mut hecs::World, renderer: &mut Renderer) {
        self.chunk = Some(
            VoxelModel::load_vox("model2.vox")
                .unwrap()
                .swap_remove(0)
                .into_chunks()
                .swap_remove(0)
                .chunk
        );

        let viewport_size = (renderer.size().width * renderer.size().height) as usize;

        let taa = Taa::new(renderer);

        self.camera_config.limit = (-85.0, 85.0);

        self.color_buffer = Some(BufferResource::new(
            renderer, 
            Buffer::new(renderer, viewport_size, wgpu::BufferUsages::STORAGE),
            wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT, 
            wgpu::BufferBindingType::Storage { read_only: false },
        ));

        self.chunks_3d_texture = Some(TextureResource::new(
            renderer,
            Texture::new(renderer, TextureDescriptor {
                width: Chunk::CHUNK_SIZE as u32,
                height: Chunk::CHUNK_SIZE as u32,
                depth: Some(Chunk::CHUNK_SIZE as u32 * chunks_count()),
                filter: wgpu::FilterMode::Nearest,
                dimension: wgpu::TextureDimension::D3,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                format: wgpu::TextureFormat::Rgba8Uint,
                label: "Chunks",
            }),
            TextureResourceUsage::TEXTURE | TextureResourceUsage::SAMPLER,
            Some(wgpu::TextureSampleType::Uint),
        ));

        self.palettes = Some(BufferResource::new(
            renderer,
            Buffer::new(renderer, 256 * chunks_count() as usize, wgpu::BufferUsages::STORAGE),
            wgpu::ShaderStages::COMPUTE,
            wgpu::BufferBindingType::Storage { read_only: true }, 
        ));

        self.chunk.as_ref().unwrap().write_to_texture(
            renderer, 
            &self.chunks_3d_texture.as_ref().unwrap().texture,
            &self.palettes.as_ref().unwrap().buffer,
            0,
        ).unwrap();

        self.rt_pipeline = Some(Pipeline::new_compute(
            renderer, 
            include_wgsl!("../src/renderer/shaders/rt_shader.wgsl"),
            &[
                &taa.config_buffer,
                self.color_buffer.as_ref().unwrap(),
                &taa.velocity_buffer,
                self.chunks_3d_texture.as_ref().unwrap(),
                self.palettes.as_ref().unwrap(),
            ], 
            "Ray tracing"
        ));

        self.taa_pipeline = Some(Pipeline::new_render(
            renderer,
            include_wgsl!("../src/renderer/shaders/taa_shader.wgsl"),
            &[
                &taa.config_buffer,
                &taa.history_texture,
                &taa.velocity_buffer,
                self.color_buffer.as_ref().unwrap(),
            ],
            "TAA",
            false,
        ));

        self.taa = Some(taa);
    }

    fn render(&mut self, _world: &mut hecs::World, renderer: &mut Renderer) -> Result<(), RenderError> {
        let canvas = renderer.canvas()?;
        let mut ctx = renderer.draw_ctx();

        let taa = self.taa.as_mut().unwrap();
        let color_buffer = self.color_buffer.as_mut().unwrap();

        taa.update(renderer);

        let viewport_size = renderer.size().width as usize * renderer.size().height as usize;
        if *color_buffer.buffer.capacity() != viewport_size {
            color_buffer.resize(renderer, viewport_size);
        }

        {
            let mut compute_pass = ctx.compute_pass();

            compute_pass.compute(
                Some(&mut self.tmp_transform),
                self.rt_pipeline.as_ref().unwrap(), 
                &[
                    &taa.config_buffer,
                    color_buffer,
                    &taa.velocity_buffer,
                    self.chunks_3d_texture.as_ref().unwrap(),
                    self.palettes.as_ref().unwrap(),
                ], 
                renderer.size(),
            );
        }

        {
            let mut render_pass = ctx.render_pass(&taa.render_texture, renderer.depth_texture());

            render_pass.draw::<()>(
                renderer,
                None, 
                None, 
                self.taa_pipeline.as_ref().unwrap(), 
                &[
                    &taa.config_buffer,
                    &taa.history_texture,
                    &taa.velocity_buffer,
                    color_buffer,
                ],
            );
        }

        ctx.copy_texture(
            &taa.render_texture, 
            &taa.history_texture.texture, 
        );

        {
            let mut render_pass = ctx.render_pass(&canvas, renderer.depth_texture());

            render_pass.draw::<()>(
                renderer,
                None, 
                None, 
                self.taa_pipeline.as_ref().unwrap(), 
                &[
                    &taa.config_buffer,
                    &taa.history_texture,
                    &taa.velocity_buffer,
                    color_buffer,
                ],
            );
        }

        ctx.apply(canvas, renderer);

        Ok(())
    }

    fn update(&mut self, _world: &mut hecs::World) {
    }

    fn input(&mut self, event: &WindowEvent, _world: &mut hecs::World) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let (delta_x, delta_y) = {
                    if self.camera_config.latest_pos == glm::Vec2::default() {
                        (0.0, 0.0)
                    } else {
                        (
                            position.x as f32 - self.camera_config.latest_pos.x,
                            position.y as f32 - self.camera_config.latest_pos.y,
                        )
                    }
                };

                self.camera_config.latest_pos = glm::vec2(position.x as f32, position.y as f32);

                let local_x = self.tmp_transform.local_x();
                let (tx, ty) = (self.camera_config.target_x, self.camera_config.target_y);

                self.camera_config.target_x += delta_y * 0.005;
                self.camera_config.target_y -= delta_x * 0.005;

                self.camera_config.target_x = self.camera_config.target_x.clamp(
                    self.camera_config.limit.0.to_radians(),
                    self.camera_config.limit.1.to_radians(),
                );

                self.tmp_transform.rotation *=
                    glm::quat_angle_axis(self.camera_config.target_x - tx, &local_x) *
                    glm::quat_angle_axis(self.camera_config.target_y - ty, &glm::Vec3::y());
            },
            WindowEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(_, delta), .. } => {
                self.tmp_transform.translation.z += delta * 0.01;
            },
            _ => return false,
        }

        false
    }
}

fn main() -> anyhow::Result<()> {
    let mut game = Game::new(
        WindowBuilder::new()
            .with_title("Ray tracing")
            .with_inner_size(PhysicalSize::new(1280, 720)),
    )?;
    game.set_engine(RayTracer::default());
    game.run()?;

    Ok(())
}