struct TaaConfig {
    jitter: f32,
};

@group(1) @binding(0)
var<uniform> taa_config: TaaConfig;

@group(2) @binding(1)
var history_texture_out: texture_2d<f32>;

@group(2) @binding(2)
var history_sampler: sampler;

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
    return textureSample(history_texture_out, history_sampler, pos);
}
