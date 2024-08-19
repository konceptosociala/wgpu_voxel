use bytemuck::{Pod, Zeroable};
use nalgebra_glm as glm;

use crate::renderer::{hal::Padding, InstanceData};

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct RtCamera {
    pub image_width: u32,
    pub image_height: u32,
    pub center: glm::Vec3,
    pub first_pixel: glm::Vec3,
    pub pixel_delta_u: glm::Vec3,
    pub pixel_delta_v: glm::Vec3,
    pub scan_depth: u32,
}

impl RtCamera {
    pub fn new(
        image_width: u32, 
        image_height: u32, 
        scan_depth: u32, 
        jitter: f32,
    ) -> RtCamera {
        let aspect = image_width as f32 / image_height as f32;

        let focal_length = 1.0;
        let viewport_height = 2.0;
        let viewport_width = viewport_height * aspect;
        let center = glm::vec3(0.0, 0.0, 0.0);

        let viewport_u = glm::vec3(viewport_width, 0.0, 0.0);
        let viewport_v = glm::vec3(0.0, -viewport_height, 0.0);

        let pixel_delta_u = viewport_u / image_width as f32;
        let pixel_delta_v = viewport_v / image_height as f32;

        let jitter = jitter / image_height as f32;
        let viewport_upper_left = center - glm::vec3(0.0, 0.0, focal_length) - viewport_u/2.0 - viewport_v/2.0;
        let first_pixel = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v) + glm::vec3(jitter, jitter, 0.);

        RtCamera {
            image_height,
            image_width,
            center,
            first_pixel,
            pixel_delta_u,
            pixel_delta_v,
            scan_depth,
        }
    }
}

impl InstanceData for RtCamera {
    type UniformData = RtCameraUniform;

    fn uniform_data(&mut self) -> Self::UniformData {
        RtCameraUniform {
            image_height: self.image_height,
            image_width: self.image_width,
            center: self.center,
            first_pixel: self.first_pixel,
            pixel_delta_u: self.pixel_delta_u,
            pixel_delta_v: self.pixel_delta_v,
            scan_depth: self.scan_depth,
            ..Default::default()
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Zeroable, Pod)]
pub struct RtCameraUniform {
    image_width: u32,
    image_height: u32,
    _padding0: Padding,
    _padding1: Padding,

    center: glm::Vec3,
    _padding3: Padding,

    first_pixel: glm::Vec3,
    _padding2: Padding,

    pixel_delta_u: glm::Vec3,
    _padding4: Padding,

    pixel_delta_v: glm::Vec3,
    scan_depth: u32,
}