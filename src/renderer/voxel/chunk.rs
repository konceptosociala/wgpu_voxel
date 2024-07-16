use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::renderer::pbr::mesh::{Mesh, Vertex};

use super::block::Block;

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
}

impl Chunk {
    pub const CHUNK_SIZE: usize = 32;

    pub fn new() -> Chunk {
        Chunk::default()
    }

    pub fn load_vox(path: impl AsRef<Path>) -> anyhow::Result<Vec<Chunk>> {
        let data = dot_vox::load(path.as_ref().to_str().unwrap()).map_err(|e| e.to_owned());

        
    }

    pub fn generate_mesh(&self) -> Mesh {
        todo!()
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Chunk {
            blocks: [[[Block::default(); Self::CHUNK_SIZE]; Self::CHUNK_SIZE]; Self::CHUNK_SIZE],
        }
    }
}