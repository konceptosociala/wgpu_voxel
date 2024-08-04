use bytemuck::{Pod, Zeroable};
use derive_getters::Getters;
use nalgebra_glm as glm;
use serde::{Deserialize, Serialize};
use crate::renderer::hal::{buffer::Buffer, Padding};
use super::transform::Transform;

/// A matrix to convert OpenGL coordinate system to WGPU coordinate system.
pub const OPENGL_TO_WGPU_MATRIX: glm::Mat4 = glm::Mat4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

/// Enumeration of different types of cameras.
#[derive(Clone, Default, Debug, Hash, PartialEq, Serialize, Deserialize)]
pub enum CameraType {
    /// First person camera type.
    #[default]
    FirstPerson,
    /// Look-at camera type.
    LookAt,
}

/// Represents a camera in the scene, holding information about its type and projection parameters.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    camera_type: CameraType,
    aspect: f32,
    fovy: f32,
    near: f32,
    far: f32,
}

impl Camera {
    /// Creates a new camera with the specified type and aspect ratio.
    ///
    /// # Arguments
    ///
    /// * `camera_type` - The type of the camera.
    /// * `aspect` - The aspect ratio of the camera's view.
    ///
    /// # Returns
    ///
    /// A new instance of `Camera`.
    pub fn new(camera_type: CameraType, aspect: f32) -> Camera {
        Camera {
            camera_type,
            aspect,
            fovy: 45.0,
            near: 0.1,
            far: 100.0,
        }
    }

    /// Builds the view-projection matrix for the camera based on its transform.
    ///
    /// # Arguments
    ///
    /// * `transform` - The transform of the camera.
    ///
    /// # Returns
    ///
    /// The view-projection matrix.
    pub fn build_view_projection(&self, transform: &Transform) -> glm::Mat4 {
        let rotation_matrix = glm::quat_cast(&transform.rotation);
        let translation_matrix = glm::translation(&transform.translation);

        let view = match self.camera_type {
            CameraType::FirstPerson => rotation_matrix * translation_matrix,
            CameraType::LookAt => translation_matrix * rotation_matrix,
        };

        let projection = glm::perspective(self.aspect, self.fovy, self.near, self.far);

        OPENGL_TO_WGPU_MATRIX * projection * view
    }

    /// Sets the aspect ratio of the camera's view.
    ///
    /// # Arguments
    ///
    /// * `aspect` - The new aspect ratio.
    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
}

/// Uniform data structure for the camera, used for passing camera information to the GPU.
#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct CameraUniform {
    position: glm::Vec3,
    _padding: Padding,
    view_projection: glm::Mat4,
}

impl Default for CameraUniform {
    fn default() -> Self {
        CameraUniform {
            position: glm::Vec3::identity(),
            view_projection: glm::Mat4::identity(),
            _padding: Padding::default(),
        }
    }
}

impl CameraUniform {
    /// Creates a new `CameraUniform` from a given camera and transform.
    ///
    /// # Arguments
    ///
    /// * `camera` - The camera from which to create the uniform.
    /// * `transform` - The transform of the camera.
    ///
    /// # Returns
    ///
    /// A new instance of `CameraUniform`.
    pub fn new(camera: &Camera, transform: &Transform) -> CameraUniform {
        CameraUniform {
            position: transform.translation,
            view_projection: camera.build_view_projection(transform),
            _padding: Padding::default(),
        }
    }
}

/// A buffer that holds camera uniforms and provides bind group layouts and bind groups for rendering.
#[derive(Getters)]
pub struct CameraBuffer {
    buffer: Buffer<CameraUniform>,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl CameraBuffer {
    /// Creates a new `CameraBuffer` and initializes it with default camera uniform data.
    ///
    /// # Arguments
    ///
    /// * `device` - The device to use for creating resources.
    /// * `queue` - The queue to use for submitting commands.
    ///
    /// # Returns
    ///
    /// A new instance of `CameraBuffer`.
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> CameraBuffer {
        let mut buffer = Buffer::new(
            device, 
            1, 
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        );
        buffer.fill(device, queue, &[CameraUniform::default()]);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0, 
                resource: buffer.inner().as_entire_binding(),
            }]
        });
        
        CameraBuffer {
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    /// Updates the camera buffer with new camera and transform data.
    ///
    /// # Arguments
    ///
    /// * `camera` - The camera to update.
    /// * `transform` - The transform of the camera.
    /// * `queue` - The queue to use for submitting commands.
    pub fn update(
        &self, 
        camera: &Camera,
        transform: &Transform,
        queue: &wgpu::Queue
    ) {
        self.buffer.fill_exact(queue, &[CameraUniform::new(camera, transform)])
            .expect("Can't fill camera buffer");
    }
}
