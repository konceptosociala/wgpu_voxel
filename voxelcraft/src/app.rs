use tracengine::{
    engine::Engine, 
    event::{MouseScrollDelta, WindowEvent}, 
    glm, 
    renderer::{
        error::RenderError, rt::camera::RtCamera, InstanceData, Renderer
    }, 
    World
};

use crate::tracer::Tracer;

#[derive(Default)]
pub struct VoxelCraft {
    tracer: Option<Tracer>,
}

impl Engine for VoxelCraft {
    fn init(&mut self, _: &mut World, renderer: &mut Renderer) {
        self.tracer = Some(Tracer::new(renderer));
    }

    fn input(&mut self, event: &WindowEvent, _: &mut World) -> bool {
        let tracer = self.tracer.as_mut().unwrap();

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let (delta_x, delta_y) = {
                    if tracer.camera_config.latest_pos == glm::Vec2::default() {
                        (0.0, 0.0)
                    } else {
                        (
                            position.x as f32 - tracer.camera_config.latest_pos.x,
                            position.y as f32 - tracer.camera_config.latest_pos.y,
                        )
                    }
                };

                tracer.camera_config.latest_pos = glm::vec2(position.x as f32, position.y as f32);

                let local_x = tracer.tmp_transform.local_x();
                let (tx, ty) = (tracer.camera_config.target_x, tracer.camera_config.target_y);

                tracer.camera_config.target_x += delta_y * 0.005;
                tracer.camera_config.target_y -= delta_x * 0.005;

                tracer.camera_config.target_x = tracer.camera_config.target_x.clamp(
                    tracer.camera_config.limit.start.to_radians(),
                    tracer.camera_config.limit.end.to_radians(),
                );

                tracer.tmp_transform.rotation *=
                    glm::quat_angle_axis(tracer.camera_config.target_x - tx, &local_x) *
                    glm::quat_angle_axis(tracer.camera_config.target_y - ty, &glm::Vec3::y());
            },
            WindowEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(_, delta), .. } => {
                tracer.tmp_transform.translation.z += delta * 0.01;
            },
            _ => return false,
        }

        false
    }

    fn render(&mut self, _: &mut World, renderer: &mut Renderer) -> Result<(), RenderError> {
        let canvas = renderer.canvas()?;
        let mut ctx = renderer.draw_ctx();

        let tracer = self.tracer.as_mut().unwrap();

        tracer.taa.update(renderer);

        let viewport_size = renderer.size().width as usize * renderer.size().height as usize;
        if *tracer.color_buffer.buffer.capacity() != viewport_size {
            tracer.color_buffer.resize(renderer, viewport_size);
        }

        if tracer.camera.image_width != renderer.size().width || tracer.camera.image_height != renderer.size().height {
            tracer.camera = RtCamera::new(
                renderer.size().width, 
                renderer.size().height, 
                2, 
                tracer.taa.current_jitter(),
            );
            tracer.camera_buffer.buffer.fill_exact(renderer, 0, &[tracer.camera.uniform_data()]).unwrap();
            println!("camera");
        }

        {
            let mut compute_pass = ctx.compute_pass();

            compute_pass.compute(
                Some(&mut tracer.tmp_transform),
                &tracer.rt_pipeline, 
                &[
                    &tracer.taa.config_buffer,
                    &tracer.camera_buffer,
                    &tracer.color_buffer,
                    &tracer.taa.velocity_buffer,
                    &tracer.chunks_3d_texture,
                    &tracer.palettes,
                ], 
                renderer.size(),
            );
        }

        {
            let mut render_pass = ctx.render_pass(&tracer.taa.render_texture, renderer.depth_texture());

            render_pass.draw::<()>(
                renderer,
                None, 
                None, 
                &tracer.taa_pipeline, 
                &[
                    &tracer.taa.config_buffer,
                    &tracer.taa.history_texture,
                    &tracer.taa.velocity_buffer,
                    &tracer.color_buffer,
                ],
            );
        }

        ctx.copy_texture(
            &tracer.taa.render_texture, 
            &tracer.taa.history_texture.texture, 
        );

        {
            let mut render_pass = ctx.render_pass(&canvas, renderer.depth_texture());

            render_pass.draw::<()>(
                renderer,
                None, 
                None, 
                &tracer.taa_pipeline, 
                &[
                    &tracer.taa.config_buffer,
                    &tracer.taa.history_texture,
                    &tracer.taa.velocity_buffer,
                    &tracer.color_buffer,
                ],
            );
        }

        ctx.apply(canvas, renderer);

        Ok(())
    }

    fn update(&mut self, _: &mut World) {}
}