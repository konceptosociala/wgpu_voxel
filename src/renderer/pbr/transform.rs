use bytemuck::{Pod, Zeroable};
use serde::{Serialize, Deserialize};
use nalgebra_glm as glm;

use crate::renderer::InstanceData;

/// A structure representing a transformation in 3D space, including translation, rotation, and scale.
///
/// The transformation is represented by a translation vector, a rotation quaternion, and a uniform scale factor.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// The translation vector of the transformation.
    pub translation: glm::Vec3,
    
    /// The rotation of the transformation, represented as a quaternion.
    pub rotation: glm::Quat,
    
    /// The uniform scale factor of the transformation.
    pub scale: f32,
}

impl Transform {
    /// Constructs a new `Transform` instance with the given translation, rotation, and scale.
    ///
    /// # Arguments
    ///
    /// * `translation` - The translation vector.
    /// * `rotation` - The rotation quaternion.
    /// * `scale` - The uniform scale factor.
    ///
    /// # Returns
    ///
    /// A `Transform` instance with the specified properties.
    pub fn new(translation: glm::Vec3, rotation: glm::Quat, scale: f32) -> Transform {
        Transform { translation, rotation, scale }
    }

    /// Returns a `Transform` instance with default properties (identity transformation).
    ///
    /// # Returns
    ///
    /// A `Transform` instance with zero translation, no rotation, and a scale of 1.0.
    pub fn identity() -> Transform {
        Transform::default()
    }

    /// Constructs a new `Transform` instance with the given translation and default rotation and scale.
    ///
    /// # Arguments
    ///
    /// * `translation` - The translation vector.
    ///
    /// # Returns
    ///
    /// A `Transform` instance with the specified translation and default rotation and scale.
    pub fn new_from_translation(translation: glm::Vec3) -> Transform {
        Transform { translation, ..Default::default() }
    }

    /// Constructs a new `Transform` instance with the given rotation and default translation and scale.
    ///
    /// # Arguments
    ///
    /// * `rotation` - The rotation quaternion.
    ///
    /// # Returns
    ///
    /// A `Transform` instance with the specified rotation and default translation and scale.
    pub fn new_from_rotation(rotation: glm::Quat) -> Transform {
        Transform { rotation, ..Default::default() }
    }

    /// Returns the local x-axis direction vector in world space.
    ///
    /// # Returns
    ///
    /// A `glm::Vec3` representing the local x-axis direction.
    pub fn local_x(&self) -> glm::Vec3 {
        let m = TransformUniform::new(self).transform_matrix;
        
        glm::vec3(
            m[(0, 0)],
            m[(0, 1)], 
            m[(0, 2)]
        )
    }
    
    /// Returns the local y-axis direction vector in world space.
    ///
    /// # Returns
    ///
    /// A `glm::Vec3` representing the local y-axis direction.
    pub fn local_y(&self) -> glm::Vec3 {
        let m = TransformUniform::new(self).transform_matrix;
        
        glm::vec3(
            m[(1, 0)],
            m[(1, 1)], 
            m[(1, 2)]
        )
    }
    
    /// Returns the local z-axis direction vector in world space.
    ///
    /// # Returns
    ///
    /// A `glm::Vec3` representing the local z-axis direction.
    pub fn local_z(&self) -> glm::Vec3 {
        let m = TransformUniform::new(self).transform_matrix;
        
        glm::vec3(
            m[(2, 0)],
            m[(2, 1)], 
            m[(2, 2)]
        )
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            translation: glm::vec3(0.0, 0.0, 0.0),
            rotation: glm::Quat::identity(),
            scale: 1.0,
        }
    }
}

/// A structure representing the transformation matrices used for rendering.
///
/// Contains both the transformation matrix and its inverse.
#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct TransformUniform {
    /// The transformation matrix that combines translation, rotation, and scale.
    pub transform_matrix: glm::Mat4,
    
    /// The inverse of the transformation matrix.
    pub inverse_matrix: glm::Mat4,
}

impl Default for TransformUniform {
    fn default() -> Self {
        TransformUniform {
            transform_matrix: glm::Mat4::identity(),
            inverse_matrix: glm::Mat4::identity().try_inverse().unwrap(),
        }
    }
}

impl TransformUniform {
    /// Creates a new `TransformUniform` instance based on the provided `Transform`.
    ///
    /// # Arguments
    ///
    /// * `transform` - The `Transform` instance used to compute the matrices.
    ///
    /// # Returns
    ///
    /// A `TransformUniform` instance with the computed matrices.
    pub fn new(transform: &Transform) -> TransformUniform {
        let transform_matrix = glm::Mat4::identity()
            * glm::translation(&transform.translation)
            * glm::quat_cast(&transform.rotation)
            * glm::scaling(&glm::vec3(transform.scale, transform.scale, transform.scale));

        let inverse_matrix = transform_matrix.try_inverse().unwrap();
        TransformUniform {
            transform_matrix,
            inverse_matrix,
        }
    }
}

impl InstanceData for Transform {
    type UniformData = TransformUniform;

    fn uniform_data(&mut self) -> Self::UniformData {
        TransformUniform::new(self)
    }
}