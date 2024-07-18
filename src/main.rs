use game_loop::winit::{event::KeyEvent, keyboard::{KeyCode, PhysicalKey}};
use nalgebra_glm as glm;
use hecs::World;
use wgpu_voxel::{
    engine::Engine, 
    renderer::{
        error::RenderError, 
        pbr::{
            camera::{Camera, CameraType}, 
            transform::Transform
        }, 
        voxel::{
            chunk::Chunk, 
            model::VoxelModel
        }, 
        Renderable, Renderer
    }, 
    Game, PhysicalSize, WindowBuilder, WindowEvent,
};

struct MyGame;

impl Engine for MyGame {
    fn init(&mut self, world: &mut World, renderer: &mut Renderer) {
        let models = VoxelModel::load_vox("model.vox").unwrap();
        for model in models {
            for mut chunk_bundle in model.into_chunks().into_iter() {
                chunk_bundle.chunk.update(renderer);
                world.spawn(chunk_bundle);
            }
        }

        world.spawn((
            Camera::new(
                CameraType::LookAt, 
                renderer.size.width as f32 / renderer.size.height as f32,
            ),
            Transform::new_from_translation(glm::vec3(0.0, 0.0, -3.0)),
        ));
    }

    fn update(&mut self, _world: &mut World) {
    }

    fn input(&mut self, event: &WindowEvent, world: &mut World) -> bool { 
        for (_, (_, t)) in &mut world.query::<(&Camera, &mut Transform)>() {
            match event {
                WindowEvent::CursorMoved { position, .. } => {
                    
                },
                WindowEvent::MouseWheel { delta, .. } => {

                },
                _ => return false,
            }
        }

        true
    }

    fn render(
        &mut self,
        world: &mut World,
        renderer: &mut Renderer,
    ) -> Result<(), RenderError> {
        let output = renderer.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            for (_, (camera, transform)) in &mut world.query::<(&mut Camera, &Transform)>() {
                camera.set_aspect(renderer.size.width as f32 / renderer.size.height as f32);
                renderer.update_camera(camera, transform);
            }

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: renderer.depth_texture.view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            }); 

            render_pass.set_pipeline(&renderer.render_pipelines.main_pipeline);
            render_pass.set_bind_group(0, renderer.camera_buffer.bind_group(), &[]);
    
            for (_, (chunk, _)) in &mut world.query::<(&Chunk, &Transform)>() {
                render_pass.set_vertex_buffer(0, renderer.vertex_buffers[chunk.vertex_buffer()].inner.slice(..)); 
                render_pass.draw(0..renderer.vertex_buffers[chunk.vertex_buffer()].capacity() as u32, 0..1);
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
