use std::ops::Range;

use tracengine::glm;

#[derive(Clone, Default, Debug)]
pub struct CameraConfiguration {
    pub limit: Range<f32>,
    pub target_x: f32,
    pub target_y: f32,
    pub latest_pos: glm::Vec2,
}