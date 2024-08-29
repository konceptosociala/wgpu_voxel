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
var<storage, read_write> background_buffer: array<vec4<f32>>;

@group(1) @binding(3)
var<storage, read_write> normal_buffer: array<vec4<f32>>;

@group(1) @binding(4)
var<storage, read_write> depth_buffer: array<f32>;

@group(1) @binding(5)
var<storage, read_write> depth2_buffer: array<f32>;

@group(1) @binding(6)
var<storage, read> palettes_buffer: array<vec4<f32>>;

@group(1) @binding(7)
var chunks: texture_3d<u32>;

@group(1) @binding(8)
var chunks_sampler: sampler;

// Push Constants
var<push_constant> tmp_transform: Utils::Transform;

fn box_array_hit(
    ray: Ray::Ray, 
    t_min: f32, 
    t_max: f32,
    record: ptr<function, Ray::HitRecord>,
) -> bool {
    var boxes = array(
        Box::Box(
            vec3<f32>(
                -Constants::HALF_VOXEL_SIZE + Constants::VOXEL_SIZE,
                -Constants::HALF_VOXEL_SIZE,
                -Constants::HALF_VOXEL_SIZE,
            ),
            vec3<f32>(
                Constants::HALF_VOXEL_SIZE + Constants::VOXEL_SIZE,
                Constants::HALF_VOXEL_SIZE,
                Constants::HALF_VOXEL_SIZE,
            ),
        ),
        Box::Box(
            vec3<f32>(
                -Constants::HALF_VOXEL_SIZE - Constants::VOXEL_SIZE,
                -Constants::HALF_VOXEL_SIZE,
                -Constants::HALF_VOXEL_SIZE,
            ),
            vec3<f32>(
                Constants::HALF_VOXEL_SIZE - Constants::VOXEL_SIZE,
                Constants::HALF_VOXEL_SIZE,
                Constants::HALF_VOXEL_SIZE,
            ),
        ),
    );

    var temp = Ray::HitRecord();
    var hit_anything = false;
    var closest_so_far = t_max;

    for (var i = 0; i < 2; i++) {
        var box = boxes[i];
        if Box::hit(&box, ray, t_min, closest_so_far, &temp) {
            hit_anything = true;
            closest_so_far = temp.t;
            *record = temp;
        }
    }

    return hit_anything;
}

fn render(ray: Ray::Ray, co: vec2<u32>, scan_depth: u32) {
    var current_ray = ray;
    var current_depth = scan_depth;
    var attenuation = 1.0;

    let index = co.x + co.y * taa_config.canvas_width;
    let coords = vec2<f32>(f32(co.x)/f32(taa_config.canvas_width), f32(co.y)/f32(taa_config.canvas_height));

    let unit_direction = normalize(current_ray.direction);
    let a = 0.5 * (unit_direction.y + 1.0);
    let background_color = (1.0 - a) * vec3<f32>(1.0) + a * vec3<f32>(0.5, 0.7, 1.0);

    loop {
        if current_depth == 0u {
            color_buffer[index] = vec4<f32>(vec3<f32>(0.0), 1.0);
            return;
        }

        var hit_record = Ray::HitRecord();

        // For box array tracing
        // if box_array_hit(current_ray, 0.001, 3.40282347e+38, &hit_record) {

        // For voxel tracing
        if Chunk::hit(current_ray, 0.001, 3.40282347e+38, &hit_record) {   

            let direction = hit_record.normal + Utils::random_vec_in_unit_sphere(vec2<f32>(co), taa_config.jitter);
            current_ray = Ray::Ray(hit_record.p, direction);
            attenuation *= 0.5;
            current_depth -= 1u;
        } else {
            color_buffer[index] = vec4<f32>(background_color  * attenuation, 1.0);
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