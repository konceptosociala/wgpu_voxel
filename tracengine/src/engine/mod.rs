use game_loop::winit::event::WindowEvent;
use hecs::World;
use crate::renderer::{error::RenderError, Renderer};

/// The `Engine` trait defines the core functionality required for initializing,
/// updating, handling input, and rendering within the voxel viewer application.
pub trait Engine {
    /// Initializes the engine with the given world and renderer.
    ///
    /// # Arguments
    ///
    /// * `world` - A mutable reference to the game world.
    /// * `renderer` - A mutable reference to the renderer.
    fn init(&mut self, world: &mut World, renderer: &mut Renderer);

    /// Updates the engine state with the given world.
    ///
    /// # Arguments
    ///
    /// * `world` - A mutable reference to the game world.
    fn update(&mut self, world: &mut World);

    /// Handles input events for the engine.
    ///
    /// # Arguments
    ///
    /// * `event` - A reference to the window event.
    /// * `world` - A mutable reference to the game world.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the event was handled.
    fn input(&mut self, event: &WindowEvent, world: &mut World) -> bool;

    /// Renders the current state of the engine.
    ///
    /// # Arguments
    ///
    /// * `world` - A mutable reference to the game world.
    /// * `renderer` - A mutable reference to the renderer.
    ///
    /// # Errors
    ///
    /// Returns a `RenderError` if the rendering process fails.
    fn render(&mut self, world: &mut World, renderer: &mut Renderer) -> Result<(), RenderError>;
}
