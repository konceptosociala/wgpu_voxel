pub mod engine;
pub mod game;
pub mod renderer;

use std::sync::Arc;

use game_loop::{game_loop, winit::{
    event::Event, 
    event_loop::EventLoop, 
}};
use hecs::World;

use engine::Engine;
use parking_lot::Mutex;
use renderer::{error::RenderError, Renderer};

pub use game_loop::winit::window::WindowBuilder;
pub use game_loop::winit::dpi::PhysicalSize;
pub use game_loop::winit::event::WindowEvent;

pub struct Game {
    event_loop: Option<EventLoop<()>>,
    renderer: Renderer,
    world: Arc<Mutex<World>>,
    engine: Option<Box<dyn Engine>>,
}

impl Game {
    pub fn new(window: WindowBuilder) -> anyhow::Result<Game> {
        let event_loop = EventLoop::new().unwrap();
        let window = Arc::new(window.build(&event_loop).unwrap());
        let world = Arc::new(Mutex::new(World::new()));

        Ok(Game {
            event_loop: Some(event_loop),
            renderer: pollster::block_on(Renderer::new(window))?,
            world,
            engine: None,
        })
    }

    pub fn set_engine(&mut self, engine: Box<dyn Engine>) {
        self.engine = Some(engine);
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        if self.engine.is_none() {
            panic!("Engine not set!");
        }

        let event_loop = std::mem::take(&mut self.event_loop).unwrap();
        let window = self.renderer.window();

        self.engine.as_mut().unwrap().init(&mut self.world.lock());

        game_loop(
            event_loop, window, self, 240, 0.1,
            |g| {
                g.game.update();
            },
            |g| {
                match g.game.render() {
                    Ok(_) => {},
                    Err(RenderError::Lost) => g.game.renderer.resize(),
                    Err(RenderError::OutOfMemory) => g.exit(),
                    Err(e) => eprintln!("{e}"),
                }
            },
            |g, event| { 
                if let Event::WindowEvent { ref event, .. } = event {
                    if !g.game.input(event) {
                        match event { 
                            WindowEvent::CloseRequested => {
                                g.exit();
                            },
                            WindowEvent::Resized(size) => {
                                g.game.renderer.resize_with(*size);
                            },
                            _ => {},
                        }
                    }
                }
            }
        )?;

        Ok(())
    }

    fn update(&mut self) {
        self.engine.as_mut().unwrap().update(&mut self.world.lock());
    }

    fn render(&mut self) -> Result<(), RenderError> {
        self.engine.as_mut().unwrap().render(&mut self.world.lock(), &mut self.renderer)
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.engine.as_mut().unwrap().input(event)
    }
}