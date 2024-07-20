use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use super::chunk::Chunk;

/// A structure representing a block with an active status and a color identifier.
///
/// This struct can be used to define properties for blocks in a [`Chunk`],
/// where each block can be active or inactive (visible or invisible), and has a color associated with it.
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    /// Indicates whether the block is active or not.
    active: bool,
    
    /// The color ID for the block.
    color_id: u8,
}

impl Block {
    /// Constructs a new `Block` instance with the given active status and color identifier.
    ///
    /// # Arguments
    ///
    /// * `active` - A boolean indicating if the block is active.
    /// * `color_id` - A number representing the color identifier for the block.
    ///
    /// # Returns
    ///
    /// A `Block` instance with the specified active status and color identifier.
    pub fn new(active: bool, color_id: u8) -> Block {
        Block { active, color_id }
    }

    /// Retrieves the color identifier for the block.
    ///
    /// # Returns
    ///
    /// A number representing the color identifier of the block.
    pub fn color(&self) -> u8 {
        self.color_id
    }

    /// Sets a new color identifier for the block.
    ///
    /// # Arguments
    ///
    /// * `color_id` - A number representing the new color identifier for the block.
    pub fn set_color(&mut self, color_id: u8) {
        self.color_id = color_id;
    }

    /// Checks if the block is active.
    ///
    /// # Returns
    ///
    /// A boolean indicating if the block is active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Sets the active status of the block.
    ///
    /// # Arguments
    ///
    /// * `active` - A boolean indicating if the block should be set as active.
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}
