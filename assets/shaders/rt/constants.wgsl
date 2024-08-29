// ========= Constants =========

const MAX_TRAVERSAL_STEPS: u32 = 128;

const VOXEL_SIZE: f32 = 1.0 / 8.0;

const HALF_VOXEL_SIZE: f32 = VOXEL_SIZE / 2.0;

const CHUNK_SIZE: u32 = 32;

const CHUNK_ARRAY_SIZE: u32 = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

const CHUNK_MIN: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);

const CHUNK_MAX: vec3<f32> = vec3<f32>(
    f32(CHUNK_SIZE) * VOXEL_SIZE,
    f32(CHUNK_SIZE) * VOXEL_SIZE,
    f32(CHUNK_SIZE) * VOXEL_SIZE,
);