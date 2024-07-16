fn main() -> anyhow::Result<()> {  
    let mut game = Game::new(
        WindowBuilder::new()
            .with_title("Wgpu Voxel")
            .with_inner_size(PhysicalSize::new(800, 600))
    )?;
    game.set_engine(Box::new(MyGame));
    game.run()?;

    Ok(())
}