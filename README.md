# WGPU MagicaVoxel Viewer

`wgpu_voxel` is a voxel renderer of MagicaVoxel 3d models, written using Rust with wgpu

## Usage

```
$ wgpu_voxel --path model.vox
```

or

```
$ wgpu_voxel -p model.vox
```


## Crates

- `anyhow` - Provides flexible, easy-to-use error handling.
- `bytemuck` - Facilitates casting between byte slices and Rust types, especially for working with GPU data.
- `clap` - Parses command-line arguments and generates help and usage information.
- `dot_vox` - Loads and parses `.vox` files, used for voxel data.
- `game-loop` - Provides a simple, structured game loop abstraction, integrating with the `winit` event loop.
- `hecs` - A high-performance Entity-Component-System (ECS) library for managing game state and behavior.
- `nalgebra-glm` - A Rust port of the popular GLM mathematics library, providing vector and matrix math, built on-top of `nalgebra` crate
- `pollster` - A small utility to block the current thread until a `Future` completes, simplifying async initialization.
- `pretty-type-name` - Provides pretty-printed type names for better error messages and debugging.
- `serde` - A framework for serializing and deserializing Rust data structures.
- `thiserror` - Simplifies the creation of custom error types in Rust.
- `wgpu` - A cross-platform, safe, and modern graphics API, based on WebGPU, for rendering and computation.
