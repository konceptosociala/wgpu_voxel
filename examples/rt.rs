use wgpu_voxel::{
    engine::Engine, 
    event::WindowEvent, 
    renderer::{
        error::RenderError, hal::{
            buffer::{Buffer, BufferResource}, pipeline::{include_wgsl, Pipeline},
            taa::Taa
        }, 
        pbr::transform::Transform, 
        Renderer
    }, 
    Game, PhysicalSize, WindowBuilder
};
use nalgebra_glm as glm;

#[derive(Default)]
pub struct RayTracer {
    taa: Option<Taa>,
    color_buffer: Option<BufferResource<glm::Vec4>>,
    rt_pipeline: Option<Pipeline>,
    taa_pipeline: Option<Pipeline>,
}

impl Engine for RayTracer {
    fn init(&mut self, _world: &mut hecs::World, renderer: &mut Renderer) {
        let taa = Taa::new(renderer);

        self.color_buffer = Some(BufferResource::new(
            renderer, 
            Buffer::new(
                renderer, 
                (renderer.size().width * renderer.size().height) as usize, 
                wgpu::BufferUsages::STORAGE
            ),
            wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT, 
            wgpu::BufferBindingType::Storage { read_only: false },
        ));

        self.rt_pipeline = Some(Pipeline::new_compute(
            renderer, 
            include_wgsl!("../src/renderer/shaders/rt_shader.wgsl"),
            &[
                &taa.config_buffer,
                &taa.velocity_buffer,
                self.color_buffer.as_ref().unwrap(),
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

        let Some(taa) = self.taa.as_mut() else { unreachable!() };

        taa.update(renderer);

        let cb = self.color_buffer.as_mut().unwrap();

        let viewport_size = renderer.size().width as usize * renderer.size().height as usize;
        if *cb.buffer.capacity() != viewport_size {
            cb.resize(renderer, viewport_size);
        }

        {
            let mut compute_pass = ctx.compute_pass();

            compute_pass.compute(
                self.rt_pipeline.as_ref().unwrap(), 
                &[
                    &taa.config_buffer,
                    &taa.velocity_buffer,
                    self.color_buffer.as_ref().unwrap(),
                ], 
                renderer.size(),
            );
        }

        {
            let mut render_pass = ctx.render_pass(&taa.render_texture, renderer.depth_texture());

            render_pass.draw(
                renderer,
                None, 
                &Transform::default(), 
                self.taa_pipeline.as_ref().unwrap(), 
                &[
                    &taa.config_buffer,
                    &taa.history_texture,
                    &taa.velocity_buffer,
                    self.color_buffer.as_ref().unwrap(),
                ],
            );
        }

        ctx.copy_texture(
            &taa.render_texture, 
            &taa.history_texture.texture, 
        );

        {
            let mut render_pass = ctx.render_pass(&canvas, renderer.depth_texture());

            render_pass.draw(
                renderer,
                None, 
                &Transform::default(), 
                self.taa_pipeline.as_ref().unwrap(), 
                &[
                    &taa.config_buffer,
                    &taa.history_texture,
                    &taa.velocity_buffer,
                    self.color_buffer.as_ref().unwrap(),
                ],
            );
        }

        ctx.apply(canvas, renderer);

        Ok(())
    }

    fn update(&mut self, _world: &mut hecs::World) {}

    fn input(&mut self, _event: &WindowEvent, _world: &mut hecs::World) -> bool { false }
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