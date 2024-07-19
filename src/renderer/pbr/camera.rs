use bytemuck::{Pod, Zeroable};
use nalgebra_glm as glm;
use serde::{Deserialize, Serialize};

use crate::renderer::hal::buffer::Buffer;

use super::transform::Transform;

pub const OPENGL_TO_WGPU_MATRIX: glm::Mat4 = glm::Mat4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

#[derive(Clone, Default, Debug, Hash, PartialEq, Serialize, Deserialize)]
pub enum CameraType {
    #[default]
    FirstPerson,
    LookAt,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    camera_type: CameraType,
    aspect: f32,
    fovy: f32,
    near: f32,
    far: f32,
}

impl Camera {
    pub fn new(camera_type: CameraType, aspect: f32) -> Camera {
        Camera {
            camera_type,
            aspect,
            fovy: 45.0,
            near: 0.1,
            far: 100.0,
        }
    }

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
    
    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct CameraUniform {
    view_projection: glm::Mat4,
}

impl Default for CameraUniform {
    fn default() -> Self {
        CameraUniform {
            view_projection: glm::Mat4::identity(),
        }
    }
}

impl CameraUniform {
    pub fn new(camera: &Camera, transform: &Transform) -> CameraUniform {
        CameraUniform {
            view_projection: camera.build_view_projection(transform),
        }
    }
}

pub struct CameraBuffer {
    buffer: Buffer<CameraUniform>,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl CameraBuffer {
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

    pub fn update(
        &self, 
        camera: &Camera,
        transform: &Transform,
        queue: &wgpu::Queue
    ) {
        self.buffer.fill_exact(queue, &[CameraUniform::new(camera, transform)])
            .expect("Can't fill camera buffer");
    }
    
    pub fn buffer(&self) -> &Buffer<CameraUniform> {
        &self.buffer
    }
    
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }
    
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}