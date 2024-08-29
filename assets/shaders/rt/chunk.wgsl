// ========= Chunk =========

#import ray.wgsl as Ray
#import box.wgsl as Box
#import voxel.wgsl as Voxel
#import constants.wgsl as Constants
#import utils.wgsl as Utils

@group(1) @binding(6)
var<storage, read> palettes_buffer: array<vec4<f32>>;

@group(1) @binding(7)
var chunks: texture_3d<u32>;

@group(1) @binding(8)
var chunks_sampler: sampler;

fn hit(
    ray: Ray::Ray, 
    box_t_min: f32, 
    box_t_max: f32,
    record: ptr<function, Ray::HitRecord>,
) -> bool {
    var grid_record = Ray::HitRecord();

    var box = Box::Box(Constants::CHUNK_MIN, Constants::CHUNK_MAX);

    if !Box::hit(&box, ray, box_t_min, box_t_max, &grid_record) {
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
    for (var steps = 0u; steps < Constants::MAX_TRAVERSAL_STEPS; steps++) {
        voxel = Voxel::parse(textureLoad(chunks, vec3<i32>(pos), 0));

        if voxel.is_active {
            (*record).t = grid_record.t + max((t_max[axis] - t_delta[axis]) / voxels_per_unit[axis], 0.0);
            (*record).p = Ray::at(ray, (*record).t);
            
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

            if dot(ray.direction, (*record).normal) >= 0.0 {
                (*record).normal = -(*record).normal;
            }

            (*record).voxel_color = palettes_buffer[voxel.color_id];

            return true;
        } 

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
    }

    return false;
}