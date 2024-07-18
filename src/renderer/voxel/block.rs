use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    active: bool,
    color_id: u8,
}

impl Block {
    pub fn new(active: bool, color_id: u8) -> Block {
        Block { active, color_id }
    }

    pub fn color(&self) -> u8 {
        self.color_id
    }

    pub fn set_color(&mut self, color_id: u8) {
        self.color_id = color_id;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}