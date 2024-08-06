use wgpu_voxel::{
    engine::Engine, 
    event::WindowEvent, 
    renderer::{
        hal::{
            buffer::{Buffer, BufferResource}, 
            pipeline::{include_wgsl, Pipeline}, 
            taa::TaaConfig
        }, 
        pbr::transform::Transform
    }, 
    Game, PhysicalSize, WindowBuilder
};

#[derive(Default)]
pub struct RayTracer {
    taa_buffer: Option<BufferResource<TaaConfig>>,
    taa_pipeline: Option<Pipeline>,
}

impl Engine for RayTracer {
    fn init(&mut self, _world: &mut hecs::World, renderer: &mut wgpu_voxel::renderer::Renderer) {
        self.taa_buffer = Some(BufferResource::new(
            renderer,
            Buffer::new(renderer, 1, wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST),
            wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT, 
            wgpu::BufferBindingType::Uniform,
        ));

        self.taa_pipeline = Some(Pipeline::new_render(
            renderer,
            include_wgsl!("../src/renderer/shaders/rt_shader.wgsl"),
            &[
                self.taa_buffer.as_ref().unwrap()
            ],
            "Ray tracing",
            false,
        ));
    }

    fn update(&mut self, _world: &mut hecs::World) {
    }

    fn input(&mut self, _event: &WindowEvent, _world: &mut hecs::World) -> bool {
        false
    }

    fn render(&mut self, _world: &mut hecs::World, renderer: &mut wgpu_voxel::renderer::Renderer) -> Result<(), wgpu_voxel::renderer::error::RenderError> {
        let canvas = renderer.canvas()?;
        let mut ctx = renderer.draw_ctx();

        {
            self.taa_buffer.as_ref().unwrap().buffer.fill_exact(renderer, &[TaaConfig::new()])
                .expect("Cannot fill TAA buffer");

            let mut render_pass = ctx.pass(&canvas, renderer.depth_texture());

            render_pass.draw(
                renderer,
                None, 
                &Transform::default(), 
                self.taa_pipeline.as_ref().unwrap(), 
                &[
                    self.taa_buffer.as_ref().unwrap()
                ],
            );
        }

        ctx.submit(canvas, renderer);

        Ok(())
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