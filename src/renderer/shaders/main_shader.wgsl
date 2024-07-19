// Camera
struct CameraUniform {
    view_projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// Transform
struct TransformUniform {
    transform_matrix: mat4x4<f32>,
    inverse_matrix: mat4x4<f32>,
};

var<push_constant> transform: TransformUniform;

// Vertex data
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    input: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;  
    var world_pos: vec4<f32> = transform.transform_matrix * vec4<f32>(input.position, 1.0);

    out.clip_position = camera.view_projection * world_pos;
    out.color = input.color;
    return out;
}

@fragment
fn fs_main(output: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(output.color, 1.0);
}