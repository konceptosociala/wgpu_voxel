use std::sync::Arc;
use hecs::Bundle;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use nalgebra_glm as glm;
use crate::renderer::{
    hal::buffer::{Buffer, BufferId},
    pbr::{
        mesh::Mesh,
        transform::Transform,
        Color,
    },
    voxel::block::Block,
    types::*,
    Drawable, Renderer, Texture
};

/// Error returned when block coordinates are invalid within a chunk.
#[derive(Debug, Error)]
#[error("Invalid block coords ({0}, {1}, {2}) in chunk")]
pub struct InvalidBlockCoords(pub usize, pub usize, pub usize);

#[derive(Debug, Error)]
pub enum LoadChunkError {
    #[error("Invalid texture dimension: expected `{expected:?}`, found `{found:?}`")]
    InvalidTextureDimension {
        expected: TextureDimension,
        found: TextureDimension,
    },
    #[error(
        "Invalid texture size: expected `{}x{}`, found `{}x{}`", 
        expected.width, expected.height, 
        found.width, found.height,
    )]
    InvalidTextureSize {
        expected: Extent3d,
        found: Extent3d,
    },
    #[error("Invalid chunks index `{found}` for maximum chunks number `{max}`")]
    InvalidChunksIndex {
        max: u64,
        found: u64,
    },
    #[error("Invalid texture depth: `{0}` is not a multiple of `{}`", Chunk::CHUNK_SIZE)]
    InvalidTextureDepth(u32),
}

/// Represents a 3D chunk of blocks in a voxel-based world.
#[derive(Debug, Serialize, Deserialize)]
pub struct Chunk {
    /// 3D array of blocks within the chunk.
    blocks: [[[Block; Self::CHUNK_SIZE]; Self::CHUNK_SIZE]; Self::CHUNK_SIZE],
    
    /// Color palette used to color the blocks.
    palette: Arc<[Color]>,

    /// Optional buffer ID for the vertex buffer associated with the chunk.
    #[serde(skip)]
    vertex_buffer: Option<BufferId>,
}

impl Chunk {
    /// The size of the chunk in each dimension.
    pub const CHUNK_SIZE: usize = 32;

    /// Creates a new `Chunk` with the given color palette.
    ///
    /// # Arguments
    ///
    /// * `palette` - An `Arc` of `Color` values representing the color palette.
    ///
    /// # Returns
    ///
    /// A `Chunk` instance with the provided palette and default values for other fields.
    pub fn new(palette: Arc<[Color]>) -> Chunk {
        Chunk {
            palette,
            ..Default::default()
        }
    }

    /// Retrieves a reference to a block at the specified coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the block.
    /// * `y` - The y-coordinate of the block.
    /// * `z` - The z-coordinate of the block.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the block if the coordinates are valid, otherwise `None`.
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<&Block> {
        self.blocks.get(x).and_then(
            |arr| arr.get(y).and_then(
                |arr| arr.get(z)
            )
        )
    }

    /// Checks if a block is active at the specified coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the block.
    /// * `y` - The y-coordinate of the block.
    /// * `z` - The z-coordinate of the block.
    ///
    /// # Returns
    ///
    /// `true` if the block is active, otherwise `false`.
    pub fn check_block(&self, x: usize, y: usize, z: usize) -> bool {
        self
            .get_block(x, y, z)
            .filter(|b| b.is_active())
            .is_some()
    }

    /// Sets a block at the specified coordinates.
    ///
    /// # Arguments
    ///
    /// * `block` - The `Block` to set.
    /// * `x` - The x-coordinate of the block.
    /// * `y` - The y-coordinate of the block.
    /// * `z` - The z-coordinate of the block.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure. Returns an `InvalidBlockCoords` error if coordinates are out of bounds.
    pub fn set_block(&mut self, block: Block, x: usize, y: usize, z: usize) -> Result<(), InvalidBlockCoords> {
        if x >= self.blocks.len() || y >= self.blocks[0].len() || z >= self.blocks[0][0].len() {
            return Err(InvalidBlockCoords(x, y, z));
        }

        self.blocks[x][y][z] = block;

        Ok(())
    }

    // FIXME: throws "Copy of (131072 * chunk_index)..(131072 * (chunk_index + 1)) 
    // would end up overrunning the bounds of the Source buffer of size 131072", 
    // if `chunk_index` > 0
    pub fn write_to_texture(
        &self, 
        renderer: &Renderer, 
        chunks_texture: &Texture, 
        palettes_buffer: &Buffer<glm::Vec4>,
        chunk_index: u64,
    ) -> Result<(), LoadChunkError> {
        let descr = chunks_texture.description();

        if descr.dimension != wgpu::TextureDimension::D3 {
            return Err(LoadChunkError::InvalidTextureDimension {
                expected: wgpu::TextureDimension::D3,
                found: descr.dimension,
            })
        }

        if descr.depth.unwrap_or(1) % Chunk::CHUNK_SIZE as u32 != 0 {
            return Err(LoadChunkError::InvalidTextureDepth(descr.depth.unwrap_or(1)));
        }

        if descr.width != Chunk::CHUNK_SIZE as u32 || descr.height != Chunk::CHUNK_SIZE as u32 {
            return Err(LoadChunkError::InvalidTextureSize { 
                expected: wgpu::Extent3d {
                    width: Chunk::CHUNK_SIZE as u32,
                    height: Chunk::CHUNK_SIZE as u32,
                    ..Default::default()
                }, 
                found: chunks_texture.texture().size(),
            });
        }

        let max_chunks = descr.depth.unwrap_or(1) as u64 / Chunk::CHUNK_SIZE as u64;
        if chunk_index >= max_chunks {
            return Err(LoadChunkError::InvalidChunksIndex {
                max: max_chunks,
                found: chunk_index,
            });
        }

        renderer.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: chunks_texture.texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.data_to_u8_slice(),
            wgpu::ImageDataLayout {
                offset: chunk_index * Chunk::CHUNK_SIZE as u64 * Chunk::CHUNK_SIZE as u64 * Chunk::CHUNK_SIZE as u64 * 4,
                bytes_per_row: Some(4 * descr.width),
                rows_per_image: Some(descr.height),
            },
            wgpu::Extent3d {
                width: Chunk::CHUNK_SIZE as u32,
                height: Chunk::CHUNK_SIZE as u32,
                depth_or_array_layers: Chunk::CHUNK_SIZE as u32,
            },
        );

        // TODO: Chunks 3d texture and palettes buffer to single type

        palettes_buffer.fill_exact(
            renderer, 
            chunk_index,
            &self.palette
                .iter()
                .map(|c| glm::vec4(c.r, c.g, c.b, 1.0))
                .collect::<Vec<_>>()
        ).unwrap();

        Ok(())
    }

    fn data_to_u8_slice(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.blocks.len() * 4);
        
        for z in 0..Self::CHUNK_SIZE {
            for y in 0..Self::CHUNK_SIZE {
                for x in 0..Self::CHUNK_SIZE {
                    let block = self.get_block(x, y, z).unwrap();
                    bytes.push(block.is_active() as u8);
                    bytes.push(block.color());
                    bytes.push(0);
                    bytes.push(0);
                }
            }
        }
        
        bytes
    }

    /// Generates a `Mesh` for the chunk based on its blocks.
    ///
    /// # Returns
    ///
    /// A `Mesh` representing the chunk.
    pub fn generate_mesh(&self) -> Mesh {
        let mut mesh = Mesh::default();

        for x in 0..Self::CHUNK_SIZE {
            for y in 0..Self::CHUNK_SIZE {
                for z in 0..Self::CHUNK_SIZE {
                    let block = self.get_block(x, y, z).unwrap();

                    if !block.is_active() {
                        continue;
                    }

                    let color = *self.palette
                        .get(block.color() as usize)
                        .unwrap_or_else(|| {
                            panic!(
                                "Wrong palette index `{}` in palette with size `{}`", 
                                block.color(), 
                                self.palette.len()
                            );
                        });

                    // Front face
                    if z == 0 || !self.check_block(x, y, z - 1) {
                        mesh.add_front_face(x, y, z, color)
                    }

                    // Back face
                    if !self.check_block(x, y, z + 1) {
                        mesh.add_back_face(x, y, z, color);
                    }

                    // Left face
                    if x == 0 || !self.check_block(x - 1, y, z) {
                        mesh.add_left_face(x, y, z, color);
                    }

                    // Right face
                    if !self.check_block(x + 1, y, z) {
                        mesh.add_right_face(x, y, z, color);
                    }

                    // Bottom face
                    if y == 0 || !self.check_block(x, y - 1, z) {
                        mesh.add_bottom_face(x, y, z, color);
                    }

                    // Top face
                    if !self.check_block(x, y + 1, z) {
                        mesh.add_top_face(x, y, z, color);
                    }
                }
            }
        }

        mesh
    }
}

impl Drawable for Chunk {
    /// Updates the vertex buffer with the new mesh data.
    ///
    /// # Arguments
    ///
    /// * `renderer` - The `Renderer` instance used to manage rendering resources.
    ///
    /// # Panics
    ///
    /// Panics if `update()` has not been called and `vertex_buffer` is `None`.
    fn update(&mut self, renderer: &mut Renderer) {
        let mesh = self.generate_mesh();

        if self.vertex_buffer.is_none() {
            self.vertex_buffer = Some(renderer.create_vertex_buffer(mesh.vertex_data.len()));
        }

        renderer.update_vertex_buffer(self.vertex_buffer.unwrap(), &mesh.vertex_data)
            .expect("Cannot call update() on chunk");
    }

    /// Retrieves the vertex buffer ID for the chunk.
    ///
    /// # Returns
    ///
    /// The `BufferId` for the vertex buffer.
    ///
    /// # Panics
    ///
    /// Panics if `vertex_buffer` is `None`, indicating that `update()` has not been called.
    fn vertex_buffer(&self) -> BufferId {
        self.vertex_buffer
            .expect("Chunk is not set up with update()")
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Chunk {
            blocks: [[[Block::default(); Self::CHUNK_SIZE]; Self::CHUNK_SIZE]; Self::CHUNK_SIZE],
            palette: Arc::new([]),
            vertex_buffer: None,
        }
    }
}

/// A bundle containing a `Chunk` and its associated `Transform`.
#[derive(Bundle, Debug)]
pub struct ChunkBundle {
    /// The `Chunk` instance.
    pub chunk: Chunk,
    
    /// The `Transform` associated with the chunk.
    pub transform: Transform,
}