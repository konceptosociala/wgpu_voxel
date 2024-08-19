// TAA Shader

#import rt/utils.wgsl as Utils

// ========= Uniforms =========

@group(0) @binding(0)
var<uniform> taa_config: Utils::TaaConfig;

@group(1) @binding(0)
var history_texture: texture_2d<f32>;

@group(1) @binding(1)
var history_sampler: sampler;

@group(2) @binding(0)
var<storage, read_write> velocity_buffer: array<vec4<f32>>;

@group(3) @binding(0)
var<storage, read_write> color_buffer: array<vec4<f32>>;

// ========= Render =========

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32
) -> @builtin(position) vec4<f32> {
    var x = 0.0;
    var y = 0.0;

    switch vertex_index {
        case 0u: {
            x = -1.0;
            y = -1.0;
        } 
        case 1u: {
            x = 1.0;
            y = -1.0;
        } 
        case 2u: {
            x = -1.0;
            y = 1.0;
        } 
        case 3u: {
            x = 1.0;
            y = -1.0;
        }
        case 4u: {
            x = 1.0;
            y = 1.0;
        } 
        case 5u: {
            x = -1.0;
            y = 1.0;
        }
        default: {}
    };

    return vec4<f32>(
        x + (taa_config.jitter / 1280.0), 
        y + (taa_config.jitter / 720.0), 
        0.0, 
        1.0
    );
}

@fragment
fn fs_main(
    @builtin(position) frag_pos: vec4<f32>,
) -> @location(0) vec4<f32> {

    let color_pos = vec2<u32>(
        u32((frag_pos.x + 1.0)),
        u32((frag_pos.y + 1.0)),
    );

    let history_pos = vec2<f32>(
        frag_pos.x / f32(taa_config.canvas_width),
        frag_pos.y / f32(taa_config.canvas_height),
    );

    let velocity = velocity_buffer[color_pos.x + color_pos.y * taa_config.canvas_width].xy;
    let previous_pixel_pos = history_pos - velocity;

    let current_color = color_buffer[color_pos.x + color_pos.y * taa_config.canvas_width];
    var history_color = textureSample(history_texture, history_sampler, previous_pixel_pos);

    let near_color0 = color_buffer[(color_pos.x + 1) + color_pos.y * taa_config.canvas_width];
    let near_color1 = color_buffer[color_pos.x + (color_pos.y + 1) * taa_config.canvas_width];
    let near_color2 = color_buffer[(color_pos.x - 1) + color_pos.y * taa_config.canvas_width];
    let near_color3 = color_buffer[color_pos.x + (color_pos.y - 1) * taa_config.canvas_width];

    let box_min = min(current_color, min(near_color0, min(near_color1, min(near_color2, near_color3))));
    let box_max = max(current_color, max(near_color0, max(near_color1, max(near_color2, near_color3))));

    history_color = clamp(history_color, box_min, box_max);

    let modulation_factor = 0.9;

    return mix(current_color, history_color, modulation_factor);
    // return current_color;
}
