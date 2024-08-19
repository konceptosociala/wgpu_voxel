// Voxel ray tracing shader

#import rt/utils.wgsl as Utils
#import rt/constants.wgsl as Constants
#import rt/box.wgsl as Box
#import rt/ray.wgsl as Ray
#import rt/voxel.wgsl as Voxel
#import rt/chunk.wgsl as Chunk

// ========= Uniforms =========

@group(0) @binding(0)
var<uniform> taa_config: Utils::TaaConfig;

@group(1) @binding(0)
var<uniform> camera: Utils::Camera;

@group(2) @binding(0)
var<storage, read_write> color_buffer: array<vec4<f32>>;

@group(3) @binding(0)
var<storage, read_write> velocity_buffer: array<vec4<f32>>;

@group(4) @binding(0)
var chunks: texture_3d<u32>;

@group(4) @binding(1)
var chunks_sampler: sampler;

@group(5) @binding(0)
var<storage, read> palettes_buffer: array<vec4<f32>>;

var<push_constant> tmp_transform: Utils::Transform;

fn color(ray: Ray::Ray, co: vec2<f32>, scan_depth: u32) -> vec4<f32> {
    var current_ray = ray;
    var current_depth = scan_depth;
    var color = vec4<f32>(1.0);
    var attenuation = 1.0;

    loop {
        if current_depth == 0 {
            return vec4<f32>(0., 0., 0., 1.0);
        }

        var hit_record = Ray::HitRecord();

        if Chunk::hit(current_ray, 0.001, 3.40282347e+38, &hit_record) {
            // let direction = hit_record.normal + normalize(Utils::random_vec_in_unit_sphere(co, taa_config.jitter));
            // current_ray = Ray::Ray(hit_record.p, direction);
            // attenuation *= 0.5;
            // current_depth = current_depth - 1;
            // return vec4<f32>(hit_record.normal * 0.5 + vec3<f32>(0.5), 1.0);
            return hit_record.voxel_color;
        } else {
            let unit_direction = normalize(current_ray.direction);
            let a = 0.5 * (unit_direction.y + 1.0);
            let background_color = (1.0 - a) * vec3<f32>(1.0, 1.0, 1.0) + a * vec3<f32>(0.5, 0.7, 1.0);
            return vec4<f32>(attenuation * background_color, 1.0);
        }
    }

    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}

fn velocity(current_ray: Ray::Ray, co: vec2<f32>) -> vec2<f32> {
    var hit_record = Ray::HitRecord();
    // trrftddergftjedadw gxVRFDCFGHFBF

    if Chunk::hit(current_ray, 0.001, 3.40282347e+38, &hit_record) {
        let p = tmp_transform.prev_matrix * (tmp_transform.inverse_matrix * vec4<f32>(hit_record.p, 1.0));
        return Utils::calc_velocity(vec4<f32>(hit_record.p, 1.0), p);
    }

    return vec2<f32>(0.0);
}

@compute @workgroup_size(1)
fn cs_main(
    @builtin(global_invocation_id) id: vec3<u32>
) {
    var current_ray = Ray::on_coords(id.xy, camera);
    current_ray.origin = (tmp_transform.inverse_matrix * vec4<f32>(current_ray.origin, 1.0)).xyz;
    current_ray.direction = (tmp_transform.inverse_matrix * vec4<f32>(current_ray.direction, 0.0)).xyz;

    // FIXME: valid velocity
    // velocity_buffer[id.x + id.y * taa_config.canvas_width] = vec4<f32>(
    //     velocity(
    //         current_ray,
    //         vec2<f32>(f32(id.x), f32(id.y)),
    //     ), 
    //     0.0,
    //     0.0,
    // );

    let color = color(current_ray, vec2<f32>(id.xy), camera.scan_depth);

    color_buffer[id.x + id.y * taa_config.canvas_width] = color;
}