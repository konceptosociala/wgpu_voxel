use wgpu_voxel::{
    engine::Engine, 
    event::WindowEvent, 
    renderer::{
        error::RenderError, hal::{
            pipeline::{include_wgsl, Pipeline}, 
            taa::Taa,
        }, pbr::transform::Transform, Renderer
    }, 
    Game, PhysicalSize, WindowBuilder
};

#[derive(Default)]
pub struct RayTracer {
    taa: Option<Taa>,
    rt_pipeline: Option<Pipeline>,
    taa_pipeline: Option<Pipeline>,
}

impl Engine for RayTracer {
    fn init(&mut self, _world: &mut hecs::World, renderer: &mut Renderer) {
        let taa = Taa::new(renderer);

        self.rt_pipeline = Some(Pipeline::new_render(
            renderer, 
            include_wgsl!("../src/renderer/shaders/rt_shader.wgsl"),
            &taa.resources(), 
            "Ray tracing",
            false,
        ));

        self.taa_pipeline = Some(Pipeline::new_render(
            renderer,
            include_wgsl!("../src/renderer/shaders/taa_shader.wgsl"),
            &taa.resources(),
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

        {
            let mut render_pass = ctx.render_pass(&taa.render_texture, renderer.depth_texture());

            render_pass.draw(
                renderer,
                None, 
                &Transform::default(), 
                self.taa_pipeline.as_ref().unwrap(), 
                &taa.resources(),
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
                &taa.resources(),
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