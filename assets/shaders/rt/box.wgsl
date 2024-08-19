// ========= Box =========

#import ray.wgsl as Ray

struct Box {
    start: vec3<f32>,
    end: vec3<f32>,
}

fn hit(
    box: ptr<function, Box>, 
    ray: Ray::Ray, 
    t_min: f32, 
    t_max: f32,
    record: ptr<function, Ray::HitRecord>,
) -> bool {
    let start = (*box).start;
    let end = (*box).end;

    let tx1 = (start.x - ray.origin.x) / ray.direction.x;
    let tx2 = (end.x - ray.origin.x) / ray.direction.x;
    let ty1 = (start.y - ray.origin.y) / ray.direction.y;
    let ty2 = (end.y - ray.origin.y) / ray.direction.y;
    let tz1 = (start.z - ray.origin.z) / ray.direction.z;
    let tz2 = (end.z - ray.origin.z) / ray.direction.z;

    let t_near = max(min(tx1, tx2), max(min(ty1, ty2), min(tz1, tz2)));
    let t_far = min(max(tx1, tx2), min(max(ty1, ty2), max(tz1, tz2)));

    if t_near > t_far || t_far < t_min || t_near > t_max {
        return false;
    }

    (*record).t = t_near;
    (*record).p = Ray::at(ray, (*record).t);
    
    let center = (end + start) * 0.5;    
    (*record).normal = normalize(vec3<f32>((*record).p.x - center.x, (*record).p.y - center.y, (*record).p.z - center.z));

    return true;
}