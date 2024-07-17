use std::{collections::HashMap, path::Path};
use nalgebra_glm as glm;

use crate::renderer::pbr::{model::Model, transform::Transform};

use super::{block::Block, chunk::{Chunk, ChunkBundle}};

#[derive(Clone, Debug, Default)]
pub struct VoxelModel {
    blocks: HashMap<(u8, u8, u8), Block>,
    size: (u8, u8, u8),
}

impl VoxelModel {
    pub fn load_vox(path: impl AsRef<Path>) -> Result<Vec<VoxelModel>, String> {
        let data = dot_vox::load(path.as_ref().to_str().unwrap()).map_err(|e| e.to_owned())?;
        
        Ok(data.models
            .iter()
            .map(|m| {
                let size = (m.size.x as u8, m.size.y as u8, m.size.z as u8);
                let mut blocks = HashMap::new();

                for x in 0..size.0 {
                    for y in 0..size.1 {
                        for z in 0..size.2 {
                            blocks.insert((x, y, z), Block::new(false, 0));
                        }
                    }
                }

                for voxel in m.voxels.iter() {
                    let block = blocks.get_mut(&(voxel.x, voxel.y, voxel.z)).unwrap();
                    block.set_active(true);
                    block.set_material(voxel.i);
                }

                VoxelModel {
                    blocks,
                    size,
                }
            })
            .collect())
    }

    pub fn into_chunks(self) -> Vec<ChunkBundle> {
        let chunks_x_size = self.size.0 as usize / Chunk::CHUNK_SIZE + 1;
        let chunks_y_size = self.size.1 as usize / Chunk::CHUNK_SIZE + 1;
        let chunks_z_size = self.size.2 as usize / Chunk::CHUNK_SIZE + 1;

        let mut chunks = HashMap::<(u8, u8, u8), Chunk>::with_capacity(
            chunks_x_size * chunks_y_size * chunks_z_size
        );

        for x in 0..chunks_x_size {
            for y in 0..chunks_y_size {
                for z in 0..chunks_z_size {
                    chunks.insert((x as u8, y as u8, z as u8), Chunk::new());
                }
            }
        }

        for ((x, y, z), block) in self.blocks {
            let chunk_x = x as usize / Chunk::CHUNK_SIZE;
            let chunk_y = y as usize / Chunk::CHUNK_SIZE;
            let chunk_z = z as usize / Chunk::CHUNK_SIZE;

            let block_x = x as usize - (chunk_x * Chunk::CHUNK_SIZE);
            let block_y = y as usize - (chunk_y * Chunk::CHUNK_SIZE);
            let block_z = z as usize - (chunk_z * Chunk::CHUNK_SIZE);

            chunks
                .get_mut(&(chunk_x as u8, chunk_y as u8, chunk_z as u8)).unwrap()
                .set_block(block, block_x, block_y, block_z).unwrap();
        }
        
        chunks
            .into_iter()
            .map(|(key, chunk)| {
                ChunkBundle {
                    chunk,
                    transform: Transform {
                        translation: glm::vec3(
                            (32 * key.0) as f32,
                            (32 * key.1) as f32,
                            (32 * key.2) as f32,
                        ),
                        ..Default::default()
                    }
                }
            })
            .collect()
    }

    pub fn into_model(self) -> Model {
        todo!();
    }
}