pub mod tracer;
pub mod camera;
pub mod app;

use tracengine::{Game, PhysicalSize, WindowBuilder};
use app::VoxelCraft;

fn main() -> anyhow::Result<()> {
    let mut game = Game::new(
        WindowBuilder::new()
            .with_title("Ray tracing")
            .with_inner_size(PhysicalSize::new(1280, 720)),
    )?;
    game.set_engine(VoxelCraft::default());
    game.run()?;

    Ok(())
}