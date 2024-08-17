// ========= Chunk =========

#import ray.wgsl as Ray
#import box.wgsl as Box
#import voxel.wgsl as Voxel
#import constants.wgsl as Constants
#import utils.wgsl as Utils

@group(3) @binding(0)
var chunks: texture_3d<u32>;

@group(3) @binding(0)
var chunks_sampler: sampler;

@group(4) @binding(0)
var<storage, read> palettes_buffer: array<vec4<f32>>;

fn hit(
    ray: Ray::Ray, 
    box_t_min: f32, 
    box_t_max: f32,
    record: ptr<function, Ray::HitRecord>,
) -> bool {
    var grid_record = Ray::HitRecord();

    var box = Box::Box(Constants::CHUNK_MIN, Constants::CHUNK_MAX);

    if !Box::hit(
        &box, 
        ray, 
        box_t_min, 
        box_t_max,
        &grid_record,
    ) {
        return false;
    }

    let voxels_per_unit = (Constants::CHUNK_MAX - Constants::CHUNK_MIN) * f32(Constants::CHUNK_SIZE);
    
    var entry_pos = ((ray.origin + ray.direction * (grid_record.t + 0.0001)) - Constants::CHUNK_MIN) * voxels_per_unit;

    let step = Utils::vec_sign(ray.direction);
    let t_delta = abs(1.0 / ray.direction);

    var pos = clamp(floor(entry_pos), vec3<f32>(0.0), vec3<f32>(f32(Constants::CHUNK_SIZE)));
    var t_max = (pos - entry_pos + max(step, vec3<f32>(0.0))) / ray.direction;

    var voxel = Voxel::Voxel(false, 0);

    var axis = 0;
    loop {
        if t_max.x < t_max.y { 
            if t_max.x < t_max.z {
                pos.x += step.x;
                if pos.x < 0.0 || pos.x >= f32(Constants::CHUNK_SIZE) { return false; }

                axis = 0;
                t_max.x += t_delta.x;
            } else {
                pos.z += step.z; 
                if pos.z < 0.0 || pos.z >= f32(Constants::CHUNK_SIZE) { return false; }

                axis = 2;
                t_max.z += t_delta.z;
            } 
        } else { 
            if t_max.y < t_max.z { 
                pos.y += step.y; 
                if pos.y < 0.0 || pos.y >= f32(Constants::CHUNK_SIZE) { return false; } 

                axis = 1;
                t_max.y += t_delta.y; 
            } else { 
                pos.z += step.z; 
                if pos.z < 0.0 || pos.z >= f32(Constants::CHUNK_SIZE) { return false; } 

                axis = 2;
                t_max.z += t_delta.z; 
            }
        } 

        voxel = Voxel::parse(textureLoad(
            chunks, 
            vec3<i32>(
                i32(pos.x), 
                i32(pos.y), 
                i32(pos.z),
            ), 
            0,
        ));

        if voxel.is_active {
            (*record).t = grid_record.t + (t_max[axis] - t_delta[axis]) / voxels_per_unit[axis];
            (*record).p = Ray::at(ray, (*record).t);

            let center = (vec3<f32>(
                pos.x, 
                pos.y, 
                pos.z,
            ) + vec3<f32>(
                pos.x + Constants::VOXEL_SIZE, 
                pos.y + Constants::VOXEL_SIZE, 
                pos.z + Constants::VOXEL_SIZE,
            )) * 0.5; 

            // FIXME: per face cube normal
            switch axis {
                case 0: {
                    (*record).normal = vec3<f32>(1.0, 0.0, 0.0);
                }
                case 1: {
                    (*record).normal = vec3<f32>(0.0, 1.0, 0.0);
                }
                case 2: {
                    (*record).normal = vec3<f32>(0.0, 0.0, 1.0);
                }
                default: {}
            }

            (*record).voxel_color = palettes_buffer[voxel.color_id];

            return true;
        } 
    }

    return false;
}