use wgpu_voxel::{engine::Engine, event::{KeyEvent, WindowEvent}, renderer::{hal::{pipeline::PipelineKey, taa::TaaConfig}, pbr::transform::Transform}, Game, PhysicalSize, WindowBuilder};

pub struct RayTracer;

impl Engine for RayTracer {
    fn init(&mut self, world: &mut hecs::World, _renderer: &mut wgpu_voxel::renderer::Renderer) {
    }

    fn update(&mut self, _world: &mut hecs::World) {
    }

    fn input(&mut self, event: &WindowEvent, world: &mut hecs::World) -> bool {
        false
    }

    fn render(&mut self, world: &mut hecs::World, renderer: &mut wgpu_voxel::renderer::Renderer) -> Result<(), wgpu_voxel::renderer::error::RenderError> {
        let canvas = renderer.canvas()?;
        let mut ctx = renderer.draw_ctx();

        {
            renderer.update_taa_config(TaaConfig::new());
            let mut render_pass = ctx.pass(&canvas, renderer.depth_texture());

            render_pass.draw(&(), &Transform::default(), &PipelineKey::RtPipeline, renderer);
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
    game.set_engine(RayTracer);
    game.run()?;

    Ok(())
}