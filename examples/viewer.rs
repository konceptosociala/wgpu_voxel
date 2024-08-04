use std::path::PathBuf;
use clap::Parser;
use wgpu_voxel::{
    engine::Engine,
    event::{MouseScrollDelta, WindowEvent},
    glm,
    renderer::{
        error::RenderError, hal::pipeline::PipelineKey, pbr::{
            camera::{Camera, CameraType},
            transform::Transform
        },
        voxel::{
            chunk::Chunk,
            model::VoxelModel
        },
        Drawable, Renderer
    },
    Game, PhysicalSize, WindowBuilder, World
};

/// Configuration for the camera, including rotation limits and target positions.
#[derive(Clone, Default, Debug)]
struct CameraConfiguration {
    limit: (f32, f32),
    target_x: f32,
    target_y: f32,
    latest_pos: glm::Vec2,
}

/// Engine implementation for viewing voxel models.
struct VoxelViewer {
    camera_config: CameraConfiguration,
    model_path: PathBuf,
}

impl Engine for VoxelViewer {
    fn init(&mut self, world: &mut World, renderer: &mut Renderer) {
        let models = VoxelModel::load_vox(&self.model_path).unwrap_or_else(|e| {
            panic!("Cannot load model `{}: {}", &self.model_path.to_str().unwrap(), e);
        });

        for model in models {
            for mut chunk_bundle in model.into_chunks().into_iter() {
                chunk_bundle.chunk.update(renderer);
                world.spawn(chunk_bundle);
            }
        }

        world.spawn((
            Camera::new(
                CameraType::LookAt,
                renderer.size().width as f32 / renderer.size().height as f32,
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
                    camera_transform.translation.z = (camera_transform.translation.z + delta).min(-16.0);
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
        let canvas = renderer.canvas()?;
        let mut ctx = renderer.draw_ctx();

        {
            let mut render_pass = ctx.pass(&canvas, renderer.depth_texture());

            for (_, (camera, transform)) in &mut world.query::<(&mut Camera, &Transform)>() {
                renderer.update_camera(camera, transform);
            }

            for (_, (chunk, transform)) in &mut world.query::<(&Chunk, &Transform)>() {
                render_pass.draw(chunk, transform, &PipelineKey::MainPipeline, renderer);
            }
        }

        ctx.submit(canvas, renderer);

        Ok(())
    }
}

/// Command-line arguments for the application.
#[derive(Parser, Debug)]
#[command(version, about, long_about = "MagicaVoxel model viewer written with wgpu")]
pub struct Args {
    #[arg(short, long)]
    path: PathBuf,
}

/// Entry point of the application.
fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut game = Game::new(
        WindowBuilder::new()
            .with_title("Wgpu Voxel")
            .with_inner_size(PhysicalSize::new(800, 600)),
    )?;
    game.set_engine(VoxelViewer {
        camera_config: CameraConfiguration {
            limit: (-85.0, 85.0),
            ..Default::default()
        },
        model_path: args.path,
    });
    game.run()?;

    Ok(())
}
