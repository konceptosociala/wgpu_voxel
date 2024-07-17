use hecs::Bundle;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::renderer::{
    buffer::BufferId, 
    pbr::{
        mesh::{Mesh, Vertex}, 
        transform::Transform,
    }, 
    voxel::block::Block, 
    Renderable, Renderer
};

#[derive(Debug, Error)]
#[error("Invalid block coords ({0}, {1}, {2}) in chunk")]
pub struct InvalidBlockCoords(pub usize, pub usize, pub usize);

const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [0.5, 0.0, 0.5] }, // A
    Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.5, 0.0, 0.5] }, // B
    Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.5, 0.0, 0.5] }, // C
];

#[derive(Debug, Serialize, Deserialize)]
pub struct Chunk {
    blocks: [[[Block; Self::CHUNK_SIZE]; Self::CHUNK_SIZE]; Self::CHUNK_SIZE],
    #[serde(skip)]
    vertex_buffer: Option<BufferId>,
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

    // TODO: generate mesh
    pub fn generate_mesh(&self) -> Mesh {
        Mesh { vertex_data: VERTICES.to_vec() }
    }
}

impl Renderable for Chunk {
    fn update(&mut self, renderer: &mut Renderer) {
        let mesh = self.generate_mesh();

        if self.vertex_buffer.is_none() {
            self.vertex_buffer = Some(renderer.create_vertex_buffer(mesh.vertex_data.len()));
        }

        renderer.update_vertex_buffer(self.vertex_buffer(), &mesh.vertex_data)
            .expect("Cannot call update() on chunk");
    }

    fn vertex_buffer(&self) -> BufferId {
        self.vertex_buffer.expect("Chunk is not set up with update()")
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Chunk {
            blocks: [[[Block::default(); Self::CHUNK_SIZE]; Self::CHUNK_SIZE]; Self::CHUNK_SIZE],
            vertex_buffer: None,
        }
    }
}

#[derive(Bundle, Debug)]
pub struct ChunkBundle {
    pub chunk: Chunk,
    pub transform: Transform,
}