use game_loop::winit::event::WindowEvent;
use hecs::World;

use crate::renderer::{error::RenderError, Renderer};

pub trait Engine {
    fn init(&mut self, world: &mut World, renderer: &mut Renderer);

    fn update(&mut self, world: &mut World);

    fn input(&mut self, event: &WindowEvent, world: &mut World) -> bool;

    fn render(&mut self, world: &mut World, renderer: &mut Renderer) -> Result<(), RenderError>;
}