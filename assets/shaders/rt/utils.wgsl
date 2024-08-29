// ========= Utils =========

struct TaaConfig {
    canvas_width: u32,
    canvas_height: u32,
    jitter: f32,
};

struct Transform {
    inverse_matrix: mat4x4<f32>,
    prev_matrix: mat4x4<f32>,
};

struct Camera {
    image_width: u32,
    image_height: u32,
    center: vec3<f32>,
    first_pixel: vec3<f32>,
    pixel_delta_u: vec3<f32>,
    pixel_delta_v: vec3<f32>,
    scan_depth: u32,
};

fn random_vec_in_unit_sphere(co: vec2<f32>, jitter: f32) -> vec3<f32> {
    for (var j = 0.0; j < 1.0; j += 0.1) {
        let vector = random_vec_range(co, -1.0, 1.0, jitter + j);
        if vec_len_squared(vector) < 1.0 {
            return vector;
        }
    }

    return vec3<f32>(0.0);
}

fn random_vec_range(co: vec2<f32>, min: f32, max: f32, jitter: f32) -> vec3<f32> {
    return vec3<f32>(
        rand_range(co - 1.0, min, max, jitter),
        rand_range(co + 2.0, min, max, jitter),
        rand_range(co + 5.0, min, max, jitter)
    );
}

fn vec_len_squared(vector: vec3<f32>) -> f32 {
    return vector.x*vector.x + vector.y*vector.y + vector.z*vector.z;
}

fn rand(co: vec2<f32>, jitter: f32) -> f32 {
    return fract(sin(dot(co * jitter, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn rand_range(co: vec2<f32>, min: f32, max: f32, jitter: f32) -> f32 {
    return min + (max - min) * rand(co, jitter);
}

fn calc_velocity(new_pos: vec4<f32>, old_pos: vec4<f32>) -> vec2<f32> {
    var new_pos2 = new_pos;
    var old_pos2 = old_pos;

    old_pos2 /= old_pos2.w;
    old_pos2.x = (old_pos2.x+1.0)/2.0;
    old_pos2.y = (old_pos2.y+1.0)/2.0;
    old_pos2.y = 1 - old_pos2.y;
    
    new_pos2 /= new_pos2.w;
    new_pos2.x = (new_pos2.x+1)/2.0;
    new_pos2.y = (new_pos2.y+1)/2.0;
    new_pos2.y = 1 - new_pos2.y;
    
    return (new_pos2 - old_pos2).xy;
}

fn vec_sign(vector: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        sign(vector.x), 
        sign(vector.y), 
        sign(vector.z),
    );
}

fn sign(value: f32) -> f32 {
    if value > 0.0 {
        return 1.0;
    } else if value < 0.0 {
        return -1.0;
    } else {
        return 0.0;
    }
}