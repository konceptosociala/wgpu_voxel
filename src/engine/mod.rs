use game_loop::winit::event::WindowEvent;
use hecs::World;

use crate::renderer::{error::RenderError, Renderer};

pub trait Engine {
    fn init(&mut self, world: &mut World);

    fn update(&mut self, world: &mut World);

    fn input(&mut self, event: &WindowEvent) -> bool;

    fn render(&mut self, world: &mut World, renderer: &mut Renderer) -> Result<(), RenderError>;
}