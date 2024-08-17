// ========= Camera =========

#import ray.wgsl as Ray
#import utils.wgsl as Utils

// TODO: uniform camera
struct Camera {
    image_width: u32,
    image_height: u32,
    center: vec3<f32>,
    first_pixel: vec3<f32>,
    pixel_delta_u: vec3<f32>,
    pixel_delta_v: vec3<f32>,
    scan_depth: u32,
}

fn init(
    image_width: u32, 
    image_height: u32, 
    scan_depth: u32, 
    jitter: f32,
) -> Camera {
    let aspect = f32(image_width) / f32(image_height);

    let focal_length = 1.0;
    let viewport_height = 2.0;
    let viewport_width = viewport_height * aspect;
    let center = vec3<f32>(0.0, 0.0, 0.5);

    let viewport_u = vec3<f32>(viewport_width, 0.0, 0.0);
    let viewport_v = vec3<f32>(0.0, -viewport_height, 0.0);

    let pixel_delta_u = viewport_u / f32(image_width);
    let pixel_delta_v = viewport_v / f32(image_height);

    let viewport_upper_left = center - vec3<f32>(0.0, 0.0, focal_length) - viewport_u/2.0 - viewport_v/2.0;
    let first_pixel = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v) + (jitter / f32(image_height));

    var camera = Camera();
    camera.image_height = image_height;
    camera.image_width = image_width;
    camera.center = center;
    camera.first_pixel = first_pixel;
    camera.pixel_delta_u = pixel_delta_u;
    camera.pixel_delta_v = pixel_delta_v;
    camera.scan_depth = scan_depth;
    
    return camera;
}