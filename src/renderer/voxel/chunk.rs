use std::sync::Arc;

use hecs::Bundle;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::renderer::{
    voxel::block::Block,
    pbr::{
        mesh::Vertex, 
        transform::Transform,
    },
};

#[derive(Debug, Error)]
#[error("Invalid block coords ({0}, {1}, {2}) in chunk")]
pub struct InvalidBlockCoords(usize, usize, usize);

const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [0.5, 0.0, 0.5] }, // A
    Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.5, 0.0, 0.5] }, // B
    Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.5, 0.0, 0.5] }, // C
    Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.5, 0.0, 0.5] }, // D
    Vertex { position: [0.44147372, 0.2347359, 0.0], color: [0.5, 0.0, 0.5] }, // E
];

const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];

#[derive(Debug, Serialize, Deserialize)]
pub struct Chunk {
    blocks: [[[Block; Self::CHUNK_SIZE]; Self::CHUNK_SIZE]; Self::CHUNK_SIZE],
    #[serde(skip)]
    vertex_buffer: Option<Arc<wgpu::Buffer>>,
    #[serde(skip)]
    index_buffer: Option<Arc<wgpu::Buffer>>,
}

impl Chunk {
    pub const CHUNK_SIZE: usize = 32;

    pub fn new() -> Chunk {
        Chunk::default()
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<Block> {
        self.blocks.get(x).and_then(
            |arr| arr.get(y).and_then(
                |arr| arr.get(z)
            )
        ).copied()
    }

    pub fn set_block(&mut self, block: Block, x: usize, y: usize, z: usize) -> Result<(), InvalidBlockCoords> {
        if x >= self.blocks.len() || y >= self.blocks[0].len() || z >= self.blocks[0][0].len() {
            return Err(InvalidBlockCoords(x, y, z));
        }

        self.blocks[x][y][z] = block;

        Ok(())
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Chunk {
            blocks: [[[Block::default(); Self::CHUNK_SIZE]; Self::CHUNK_SIZE]; Self::CHUNK_SIZE],
            vertex_buffer: None,
            index_buffer: None,
        }
    }
}

#[derive(Bundle)]
pub struct ChunkBundle {
    pub chunk: Chunk,
    pub transform: Transform,
}