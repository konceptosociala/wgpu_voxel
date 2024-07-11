use game_loop::winit::{dpi::PhysicalSize, event::WindowEvent, keyboard::{Key, NamedKey}, platform::modifier_supplement::KeyEventExtModifierSupplement};
use wgpu_voxel::{
    engine::{state::StateManager, Engine}, 
    renderer::Renderer, 
    Game, WindowBuilder
};
use hecs::*;

pub struct MyGame {
    color: wgpu::Color,
    index: usize,
}

impl Engine for MyGame {
    fn init(&mut self, _world: &mut World, _state_manager: &mut StateManager) {
    }

    fn update(&mut self, _world: &mut hecs::World, _state_manager: &mut StateManager) {
    }

    fn render(&mut self, _world: &mut hecs::World, renderer: &mut Renderer) -> Result<(), wgpu::SurfaceError> {
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
                            load: wgpu::LoadOp::Clear(self.color),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
    
            render_pass.set_pipeline(&renderer.render_pipelines[self.index]);
            render_pass.draw(0..3, 0..1);
        }

        renderer.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
    
    fn input(&mut self, event: &game_loop::winit::event::WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.color = wgpu::Color {
                    r: (position.x/10000.0).clamp(0.0, 1.0),
                    g: (position.y/10000.0).clamp(0.0, 1.0),
                    b: (position.x/10000.0).clamp(0.0, 1.0),
                    a: (position.y/10000.0).clamp(0.0, 1.0),
                };
    
                return true;
            },
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state.is_pressed()
                    && !event.repeat
                    && event.key_without_modifiers().as_ref() == Key::Named(NamedKey::Space)
                {
                    self.index = 1 - self.index;
                }
            }
            _ => {},
        }

        false
    }
}

fn main() -> anyhow::Result<()> {  
    let mut game = Game::new(
        WindowBuilder::new()
            .with_title("Wgpu Voxel")
            .with_inner_size(PhysicalSize::new(800, 600))
    );
    game.set_engine(Box::new(MyGame {
        color: wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        },
        index: 0,
    }));
    game.run()?;

    Ok(())
}