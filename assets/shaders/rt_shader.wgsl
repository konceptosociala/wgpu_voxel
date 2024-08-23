// Voxel ray tracing shader

#import rt/utils.wgsl as Utils
#import rt/constants.wgsl as Constants
#import rt/box.wgsl as Box
#import rt/ray.wgsl as Ray
#import rt/voxel.wgsl as Voxel
#import rt/chunk.wgsl as Chunk

// ========= Uniforms =========

// TAA resource
@group(0) @binding(0)
var history_texture: texture_2d<f32>;

@group(0) @binding(1)
var history_sampler: sampler;

@group(0) @binding(2)
var<storage, read_write> velocity_buffer: array<vec4<f32>>;

@group(0) @binding(3)
var<uniform> taa_config: Utils::TaaConfig;

// Tracer resource
@group(1) @binding(0)
var<uniform> camera: Utils::Camera;

@group(1) @binding(1)
var<storage, read_write> color_buffer: array<vec4<f32>>;

@group(1) @binding(2)
var<storage, read_write> normal_buffer: array<vec4<f32>>;

@group(1) @binding(3)
var<storage, read_write> depth_buffer: array<f32>;

@group(1) @binding(4)
var<storage, read> palettes_buffer: array<vec4<f32>>;

@group(1) @binding(5)
var chunks: texture_3d<u32>;

@group(1) @binding(6)
var chunks_sampler: sampler;

// Push Constants
var<push_constant> tmp_transform: Utils::Transform;

fn render(ray: Ray::Ray, co: vec2<u32>, scan_depth: u32) {
    var current_ray = ray;
    var current_depth = scan_depth;
    var color = vec4<f32>(1.0);
    var attenuation = 1.0;

    let index = co.x + co.y * taa_config.canvas_width;

    velocity_buffer[index] = calc_velocity(
        vec2<f32>(f32(co.x)/f32(taa_config.canvas_width), f32(co.y)/f32(taa_config.canvas_height)), 
        1.0,
    );

    loop {
        if current_depth == 0u {
            color_buffer[index] = vec4<f32>(0.0, 0.0, 0.0, 1.0);
            return;
        }

        var hit_record = Ray::HitRecord();

        var box = Box::Box(vec3<f32>(-3., -3., -3,), vec3<f32>(-6., -6., -6.));

        if Box::hit(&box, current_ray, 0.001, 3.40282347e+38, &hit_record) {
        // if Chunk::hit(current_ray, 0.001, 3.40282347e+38, &hit_record) {
            let direction = hit_record.normal + normalize(Utils::random_vec_in_unit_sphere(vec2<f32>(co), taa_config.jitter));
            current_ray = Ray::Ray(hit_record.p, direction);
            attenuation *= 0.5;
            current_depth -= 1u;
            // color_buffer[index] = hit_record.voxel_color;
            // normal_buffer[index] = vec4<f32>(hit_record.normal, 1.0);
            // depth_buffer[index] = hit_record.t;
            velocity_buffer[index] = calc_velocity(
                vec2<f32>(f32(co.x)/f32(taa_config.canvas_width), f32(co.y)/f32(taa_config.canvas_height)), 
                hit_record.t,
            );
            // return;
        } else {
            let unit_direction = normalize(current_ray.direction);
            let a = 0.5 * (unit_direction.y + 1.0);
            let background_color = (1.0 - a) * vec3<f32>(1.0, 1.0, 1.0) + a * vec3<f32>(0.5, 0.7, 1.0);

            color_buffer[index] = vec4<f32>(attenuation * background_color, 1.0);
            // normal_buffer[index] = vec4<f32>(0.0, 0.0, 0.0, 1.0);
            // depth_buffer[index] = 1.0;
            return;
        }
    }
}

fn calc_velocity(coord : vec2<f32>, depth_sample: f32) -> vec4<f32> {
    if (depth_sample >= 1.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    let pos_clip = vec4(coord.x * 2.0 - 1.0, (1.0 - coord.y) * 2.0 - 1.0, depth_sample, 1.0);
    let pos_world_w = tmp_transform.inverse_matrix * pos_clip;
    let pos_world = pos_world_w / pos_world_w.w;

    let current_pos = pos_clip;
    let previous_pos = tmp_transform.prev_matrix * pos_world;
    let velocity = (current_pos - previous_pos) * 0.5;
    return vec4<f32>(velocity.x, velocity.y, 0.0, 1.0);
}

@compute @workgroup_size(1)
fn cs_main(
    @builtin(global_invocation_id) id: vec3<u32>
) {
    var current_ray = Ray::on_coords(id.xy, camera);
    current_ray.origin = (tmp_transform.inverse_matrix * vec4<f32>(current_ray.origin, 1.0)).xyz;
    current_ray.direction = (tmp_transform.inverse_matrix * vec4<f32>(current_ray.direction, 0.0)).xyz;

    render(current_ray, id.xy, camera.scan_depth);
}