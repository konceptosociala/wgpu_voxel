pub mod state;

use game_loop::winit::event::WindowEvent;
use hecs::World;
use state::StateManager;

use crate::renderer::Renderer;

pub trait Engine {
    fn init(&mut self, world: &mut World, state_manager: &mut StateManager);

    fn update(&mut self, world: &mut World, state_manager: &mut StateManager);

    fn input(&mut self, event: &WindowEvent) -> bool;

    fn render(&mut self, world: &mut World, renderer: &mut Renderer) -> Result<(), wgpu::SurfaceError>;
}