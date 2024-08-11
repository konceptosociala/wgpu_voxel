use std::ops::{Deref, DerefMut};

use bytemuck::{Pod, Zeroable};

use crate::{
    glm,
    renderer::{
        pbr::transform::Transform, 
        InstanceData,
    },
};

pub struct RtTransform {
    transform: Transform,
    uniform: RtTransformUniform,
}

impl Default for RtTransform {
    fn default() -> Self {
        let mut transform = Transform::default();
        let uniform = RtTransformUniform {
            inverse_matrix: transform.uniform_data().inverse_matrix,
            prev_matrix: glm::Mat4::identity(),
        };

        RtTransform { transform, uniform }
    }
}

impl Deref for RtTransform {
    type Target = Transform;

    fn deref(&self) -> &Self::Target {
        &self.transform
    }
}

impl DerefMut for RtTransform {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transform
    }
}

impl InstanceData for RtTransform {
    type UniformData = RtTransformUniform;

    fn uniform_data(&mut self) -> Self::UniformData {
        self.uniform.prev_matrix = self.uniform.inverse_matrix.try_inverse().unwrap();
        self.uniform.inverse_matrix = self.transform.uniform_data().inverse_matrix;

        self.uniform
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct RtTransformUniform {
    inverse_matrix: glm::Mat4,
    prev_matrix: glm::Mat4,
}