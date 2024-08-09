// Voxel ray tracing shader

struct TaaConfig {
    canvas_width: u32,
    canvas_height: u32,
    jitter: f32,
};

// ========= Uniforms =========

@group(0) @binding(0)
var<uniform> taa_config: TaaConfig;

@group(1) @binding(0)
var<storage, read_write> velocity_buffer: array<vec4<f32>>;

@group(2) @binding(0)
var<storage, read_write> color_buffer: array<vec4<f32>>;

// ========= Utils =========

alias Color = vec3<f32>;

fn random_vec_in_unit_sphere(co: vec2<f32>) -> vec3<f32> {
    while (true) {
        let vector = random_vec_range(co, -1.0, 1.0);
        if vec_len_squared(vector) < 1.0 {
            return vector;
        }
    }

    return vec3<f32>(0.0);
}

fn random_vec_range(co: vec2<f32>, min: f32, max: f32) -> vec3<f32> {
    return vec3<f32>(
        rand_range(co - 1.0, min, max),
        rand_range(co + 2.0, min, max),
        rand_range(co + 5.0, min, max)
    );
}

fn vec_len_squared(vector: vec3<f32>) -> f32 {
    return vector.x*vector.x + vector.y*vector.y + vector.z*vector.z;
}

fn rand(co: vec2<f32>) -> f32 {
    return fract(sin(dot(co * taa_config.jitter, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn rand_range(co: vec2<f32>, min: f32, max: f32) -> f32 {
    return min + (max-min)*rand(co);
}

// ========= Cube =========

struct Cube {
    pos: vec3<f32>,
}

fn cube_hit(
    cube: ptr<function, Cube>, 
    ray: Ray, 
    t_min: f32, 
    t_max: f32,
    record: ptr<function, HitRecord>,
) -> bool {
    let start = (*cube).pos - vec3<f32>(0.0625, 0.0625, 0.0625);
    let end = (*cube).pos + vec3<f32>(0.0625, 0.0625, 0.0625);

    let tx1 = (start.x - ray.origin.x) / ray.direction.x;
    let tx2 = (end.x - ray.origin.x) / ray.direction.x;
    let ty1 = (start.y - ray.origin.y) / ray.direction.y;
    let ty2 = (end.y - ray.origin.y) / ray.direction.y;
    let tz1 = (start.z - ray.origin.z) / ray.direction.z;
    let tz2 = (end.z - ray.origin.z) / ray.direction.z;

    let t_near = max(min(tx1, tx2), max(min(ty1, ty2), min(tz1, tz2)));
    let t_far = min(max(tx1, tx2), min(max(ty1, ty2), max(tz1, tz2)));

    if t_near > t_far || t_far < t_min {
        return false;
    }

    (*record).t = t_near;
    (*record).p = ray_at(ray, (*record).t);
    
    let center = (end + start) * 0.5;
    let outward_normal = vec3<f32>((*record).p.x - center.x, (*record).p.y - center.y, (*record).p.z - center.z);
    hit_record_set_face_normal(record, ray, outward_normal);

    return true;
}

fn cube_array_hit(
    cubes: ptr<function, array<Cube, 3>>, 
    ray: Ray, 
    t_min: f32, 
    t_max: f32,
    record: ptr<function, HitRecord>,
) -> bool {
    var temp = HitRecord();
    var hit_anything = false;
    var closest_so_far = t_max;

    for (var i = 0; i < 3; i++) {
        var cube = (*cubes)[i];
        if cube_hit(&cube, ray, t_min, closest_so_far, &temp) {
            hit_anything = true;
            closest_so_far = temp.t;
            *record = temp;
        }
    }

    return hit_anything;
}

// ========= Camera =========

struct Camera {
    image_width: u32,
    image_height: u32,
    center: vec3<f32>,
    first_pixel: vec3<f32>,
    pixel_delta_u: vec3<f32>,
    pixel_delta_v: vec3<f32>,
    scan_depth: u32,
}

fn camera_new(
    image_width: u32, 
    image_height: u32, 
    scan_depth: u32, 
) -> Camera {
    let aspect = f32(image_width) / f32(image_height);

    let focal_length = 1.0;
    let viewport_height = 2.0;
    let viewport_width = viewport_height * aspect;
    let center = vec3<f32>(0.0, 0.0, 0.0);    

    let viewport_u = vec3<f32>(viewport_width, 0.0, 0.0);
    let viewport_v = vec3<f32>(0.0, -viewport_height, 0.0);

    let pixel_delta_u = viewport_u / f32(image_width);
    let pixel_delta_v = viewport_v / f32(image_height);

    let viewport_upper_left = center - vec3<f32>(0.0, 0.0, focal_length) - viewport_u/2.0 - viewport_v/2.0;
    let first_pixel = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v) + (taa_config.jitter / f32(taa_config.canvas_height));

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

fn camera_render(camera: Camera, cubes: ptr<function, array<Cube, 3>>, pos: vec2<u32>) -> Color {
    let ray = ray_on_coords(pos, camera);

    return ray_color(ray, vec2<f32>(f32(pos.x), f32(pos.y)), camera.scan_depth, cubes);
}

// ========= Ray =========

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

fn ray_on_coords(pos: vec2<u32>, camera: Camera) -> Ray {
    let pixel_sample = camera.first_pixel
        + (f32(pos.x) * camera.pixel_delta_u)
        + (f32(pos.y) * camera.pixel_delta_v);

    let ray_direction = pixel_sample - camera.center;

    return Ray(camera.center, ray_direction);
}

fn ray_at(ray: Ray, t: f32) -> vec3<f32> {
    return ray.origin + t * ray.direction;
}

fn ray_color(ray: Ray, co: vec2<f32>, scan_depth: u32, cubes: ptr<function, array<Cube, 3>>) -> Color {
    var current_ray = ray;
    var current_depth = scan_depth;
    var color = Color(1.0, 1.0, 1.0);
    var attenuation = 1.0;

    loop {
        if current_depth == 0 {
            return Color(0., 0., 0.);
        }

        var hit_record = HitRecord();

        if cube_array_hit(cubes, current_ray, 0.001, 3.40282347e+38, &hit_record) {
            let direction = hit_record.normal + normalize(random_vec_in_unit_sphere(co));
            current_ray = Ray(hit_record.p, direction);
            attenuation *= 0.7;
            current_depth = current_depth - 1;
        } else {
            let unit_direction = normalize(current_ray.direction);
            let a = 0.5 * (unit_direction.y + 1.0);
            let background_color = (1.0 - a) * Color(1.0, 1.0, 1.0) + a * Color(0.5, 0.7, 1.0);
            return attenuation * background_color;
        }
    }

    return Color(0.0, 0.0, 0.0);
}

// ========= HitRecord =========

struct HitRecord {
    p: vec3<f32>,
    t: f32,
    normal: vec3<f32>,
    front_face: bool,
}

fn hit_record_set_face_normal(record: ptr<function, HitRecord>, ray: Ray, outward_normal: vec3<f32>) {
    (*record).front_face = dot(ray.direction, outward_normal) < 0.0;
    if (*record).front_face {
        (*record).normal = outward_normal;
    } else {
        (*record).normal = -outward_normal;
    }
}

// ========= Compute =========

@compute @workgroup_size(1)
fn cs_main(
    @builtin(global_invocation_id) id: vec3<u32>
) {
    let camera = camera_new(taa_config.canvas_width, taa_config.canvas_height, 10u);

    var cubes = array(
        Cube(vec3<f32>(-0.14, -0.125, -0.25)),
        Cube(vec3<f32>(0.14, -0.125, -0.25)),
        Cube(vec3<f32>(0.0, -0.125, -0.25)),
    );

    color_buffer[id.x + id.y * taa_config.canvas_width] = vec4<f32>(camera_render(camera, &cubes, id.xy), 1.0);
}