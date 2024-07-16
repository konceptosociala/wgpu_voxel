use hecs::World;
use wgpu_voxel::{
    engine::Engine, 
    renderer::{
        error::RenderError, pbr::transform::Transform, voxel::{chunk::Chunk, model::VoxelModel}, Renderer
    }, 
    Game, PhysicalSize, WindowBuilder, WindowEvent,
};

struct MyGame;

impl Engine for MyGame {
    fn init(&mut self, world: &mut World) {
        let models = VoxelModel::load_vox("model.vox").unwrap();
        for model in models {
            for chunk_bundle in model.into_chunks().into_iter() {
                world.spawn(chunk_bundle);
            }
        }
    }

    fn update(&mut self, _world: &mut World) {}

    fn input(&mut self, _event: &WindowEvent) -> bool { false }

    fn render(
        &mut self,
        world: &mut World,
        renderer: &mut Renderer,
    ) -> Result<(), RenderError> {
        let output = renderer.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
    
            for (_, (chunk, transform)) in world.query::<(&Chunk, &Transform)>().iter() {

            }
        }

        renderer.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let mut game = Game::new(
        WindowBuilder::new()
            .with_title("Wgpu Voxel")
            .with_inner_size(PhysicalSize::new(800, 600)),
    )?;
    game.set_engine(Box::new(MyGame));
    game.run()?;

    Ok(())
}
