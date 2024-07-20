use std::sync::Arc;
use hecs::Bundle;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::renderer::{
    hal::buffer::BufferId,
    pbr::{
        mesh::Mesh,
        transform::Transform,
        Color,
    },
    voxel::block::Block,
    Drawable, Renderer
};

#[derive(Debug, Error)]
#[error("Invalid block coords ({0}, {1}, {2}) in chunk")]
pub struct InvalidBlockCoords(pub usize, pub usize, pub usize);

#[derive(Debug, Serialize, Deserialize)]
pub struct Chunk {
    blocks: [[[Block; Self::CHUNK_SIZE]; Self::CHUNK_SIZE]; Self::CHUNK_SIZE],
    palette: Arc<[Color]>,
    #[serde(skip)]
    vertex_buffer: Option<BufferId>,
}

impl Chunk {
    pub const CHUNK_SIZE: usize = 32;

    pub fn new(palette: Arc<[Color]>) -> Chunk {
        Chunk {
            palette,
            ..Default::default()
        }
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<&Block> {
        self.blocks.get(x).and_then(
            |arr| arr.get(y).and_then(
                |arr| arr.get(z)
            )
        )
    }

    pub fn check_block(&self, x: usize, y: usize, z: usize) -> bool {
        self
            .get_block(x, y, z)
            .filter(|b| b.is_active())
            .is_some()
    }

    pub fn set_block(&mut self, block: Block, x: usize, y: usize, z: usize) -> Result<(), InvalidBlockCoords> {
        if x >= self.blocks.len() || y >= self.blocks[0].len() || z >= self.blocks[0][0].len() {
            return Err(InvalidBlockCoords(x, y, z));
        }

        self.blocks[x][y][z] = block;

        Ok(())
    }

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
            palette: Arc::new([]),
            vertex_buffer: None,
        }
    }
}

#[derive(Bundle, Debug)]
pub struct ChunkBundle {
    pub chunk: Chunk,
    pub transform: Transform,
}