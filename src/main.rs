use std::path::PathBuf;

use clap::Parser;
use game_loop::winit::event::MouseScrollDelta;
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

#[derive(Clone, Default, Debug)]
struct CameraConfiguration {
    limit: (f32, f32),
    target_x: f32,
    target_y: f32,
    latest_pos: glm::Vec2,
}

struct VoxelViewer {
    camera_config: CameraConfiguration,
    model_path: PathBuf,
}

impl Engine for VoxelViewer {
    fn init(&mut self, world: &mut World, renderer: &mut Renderer) {
        let models = VoxelModel::load_vox(&self.model_path).unwrap();
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
            Transform::new_from_translation(glm::vec3(0.0, 0.0, -75.0)),
        ));
    }

    fn update(&mut self, _world: &mut World) {}

    fn input(&mut self, event: &WindowEvent, world: &mut World) -> bool { 
        for (_, (_, camera_transform)) in &mut world.query::<(&Camera, &mut Transform)>() {
            match event {
                WindowEvent::CursorMoved { position, .. } => {
                    let (delta_x, delta_y) = {
                        if self.camera_config.latest_pos == glm::Vec3::default() {
                            (0.0, 0.0)
                        } else {
                            (
                                position.x as f32 - self.camera_config.latest_pos.x,
                                position.y as f32 - self.camera_config.latest_pos.y,
                            )
                        }
                    };

                    self.camera_config.latest_pos = glm::vec2(position.x as f32, position.y as f32);

                    let local_x = camera_transform.local_x();

                    let (tx, ty) = (self.camera_config.target_x, self.camera_config.target_y);
                    
                    self.camera_config.target_x += delta_y * 0.005;
                    self.camera_config.target_y -= delta_x * 0.005;

                    self.camera_config.target_x = self.camera_config.target_x.clamp(
                        self.camera_config.limit.0.to_radians(), 
                        self.camera_config.limit.1.to_radians(),
                    );

                    camera_transform.rotation *=
                        glm::quat_angle_axis(self.camera_config.target_x - tx, &local_x) *
                        glm::quat_angle_axis(self.camera_config.target_y - ty, &glm::Vec3::y());
                },
                WindowEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(_, delta), .. } => {
                    camera_transform.translation.z = (camera_transform.translation.z + delta).min(-0.5);
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

            for (_, (camera, transform)) in &mut world.query::<(&mut Camera, &Transform)>() {
                camera.set_aspect(renderer.size.width as f32 / renderer.size.height as f32);
                renderer.update_camera(camera, transform);
            }
    
            for (_, (chunk, transform)) in &mut world.query::<(&Chunk, &Transform)>() {
                render_pass.set_pipeline(&renderer.render_pipelines.main_pipeline);
                render_pass.set_bind_group(0, renderer.camera_buffer.bind_group(), &[]);
                render_pass.set_push_constants(
                    wgpu::ShaderStages::VERTEX,
                    0,
                    bytemuck::cast_slice(&[transform.uniform()]),
                );
                render_pass.set_vertex_buffer(0, renderer.vertex_buffers[chunk.vertex_buffer()].inner.slice(..)); 
                render_pass.draw(0..renderer.vertex_buffers[chunk.vertex_buffer()].capacity() as u32, 0..1);
            }
        }

        renderer.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = "MagicaVoxel model viewer written with wgpu")]
pub struct Args {
    #[arg(short, long)]
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut game = Game::new(
        WindowBuilder::new()
            .with_title("Wgpu Voxel")
            .with_inner_size(PhysicalSize::new(800, 600)),
    )?;
    game.set_engine(Box::new(VoxelViewer {
        camera_config: CameraConfiguration {
            limit: (-85.0, 85.0),
            ..Default::default()
        },
        model_path: args.path,
    }));
    game.run()?;

    Ok(())
}
