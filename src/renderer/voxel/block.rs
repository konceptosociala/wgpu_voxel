use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    active: bool,
    material_id: u8,
}

impl Block {
    pub fn new(active: bool, material_id: u8) -> Block {
        Block { active, material_id }
    }

    pub fn material_id(&self) -> u8 {
        self.material_id
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}