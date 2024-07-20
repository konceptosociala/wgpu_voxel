use std::{collections::HashMap, path::Path, sync::Arc};
use nalgebra_glm as glm;
use serde::{Deserialize, Serialize};

use crate::renderer::{error::RenderError, pbr::{transform::Transform, Color}};

use super::{block::Block, chunk::{Chunk, ChunkBundle}};

/// Represents the size of a voxel model in 3D space.
#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Size {
    x: usize,
    y: usize,
    z: usize,
}

impl From<dot_vox::Size> for Size {
    /// Converts a `dot_vox::Size` into a `Size` instance.
    ///
    /// # Arguments
    ///
    /// * `value` - The `dot_vox::Size` to convert.
    ///
    /// # Returns
    ///
    /// A `Size` instance with the dimensions from `value`.
    fn from(value: dot_vox::Size) -> Self {
        Size {
            x: value.x as usize,
            y: value.z as usize,  // Note: `y` is swapped with `z` to match the correct axes.
            z: value.y as usize,  // Note: `z` is swapped with `y` to match the correct axes.
        }
    }
}

/// Represents a voxel model with blocks and a color palette.
#[derive(Clone, Debug)]
pub struct VoxelModel {
    /// A mapping of voxel coordinates to `Block` instances.
    blocks: HashMap<(u8, u8, u8), Block>,
    
    /// Color palette used to color the blocks.
    palette: Arc<[Color]>,
    
    /// Dimensions of the voxel model.
    size: Size,
}

impl VoxelModel {
    /// Loads a voxel model from a `.vox` file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `.vox` file.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `VoxelModel` instances or an error if loading fails.
    pub fn load_vox(path: impl AsRef<Path>) -> Result<Vec<VoxelModel>, RenderError> {
        // Load the `.vox` file using the `dot_vox` crate.
        let data = dot_vox::load(path.as_ref().to_str().unwrap())
            .map_err(RenderError::LoadVoxError)?;

        // Convert the palette from `dot_vox` to `Color`.
        let palette: Arc<[Color]> = data.palette
            .iter()
            .map(|c| Color::from(*c))
            .collect();
        
        // Create a vector of `VoxelModel` instances from the loaded data.
        Ok(data.models
            .iter()
            .map(|m| {
                let size = Size::from(m.size);
                let mut blocks = HashMap::new();

                // Initialize blocks for the entire size of the model.
                for x in 0..size.x as u8 {
                    for y in 0..size.y as u8 {
                        for z in 0..size.z as u8 {
                            blocks.insert((x, y, z), Block::new(false, 0));
                        }
                    }
                }

                // Set the active state and color of each voxel based on the model data.
                for voxel in m.voxels.iter() {
                    let block = blocks.get_mut(&(voxel.x, voxel.z, voxel.y)).unwrap();
                    block.set_active(true);
                    block.set_color(voxel.i);
                }

                VoxelModel {
                    blocks,
                    palette: palette.clone(),
                    size,
                }
            })
            .collect())
    }

    /// Converts the voxel model into chunks.
    ///
    /// # Returns
    ///
    /// A vector of `ChunkBundle` instances, each containing a chunk and its transform.
    pub fn into_chunks(self) -> Vec<ChunkBundle> {
        // Calculate the number of chunks needed in each dimension.
        let chunks_x_size = self.size.x / Chunk::CHUNK_SIZE + 1;
        let chunks_y_size = self.size.y / Chunk::CHUNK_SIZE + 1;
        let chunks_z_size = self.size.z / Chunk::CHUNK_SIZE + 1;

        // Initialize chunks with calculated capacity.
        let mut chunks = HashMap::<(u8, u8, u8), Chunk>::with_capacity(
            chunks_x_size * chunks_y_size * chunks_z_size
        );

        // Create chunks for each coordinate position.
        for x in 0..chunks_x_size {
            for y in 0..chunks_y_size {
                for z in 0..chunks_z_size {
                    chunks.insert((x as u8, y as u8, z as u8), Chunk::new(self.palette.clone()));
                }
            }
        }

        // Place each block into the appropriate chunk.
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
        
        // Convert chunks into `ChunkBundle` instances with proper transforms.
        chunks
            .into_iter()
            .map(|(key, chunk)| {
                ChunkBundle {
                    chunk,
                    transform: Transform {
                        translation: glm::vec3(
                            ((key.0 as usize) * Chunk::CHUNK_SIZE) as f32 - (self.size.x / 2) as f32,
                            ((key.1 as usize) * Chunk::CHUNK_SIZE) as f32 - (self.size.y / 2) as f32,
                            ((key.2 as usize) * Chunk::CHUNK_SIZE) as f32 - (self.size.z / 2) as f32,
                        ),
                        ..Default::default()
                    }
                }
            })
            .collect()
    }
}
