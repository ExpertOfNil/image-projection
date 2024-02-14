// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(1)
var<uniform> projectors: array<CameraUniform, 10>;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    // Calculate vertex position in clip space
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.world_pos = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = vec4<f32>(0.0);
    for (var i = 0; i < 10; i += 1) {
        let proj_coords = projectors[i].view_proj * in.world_pos;
        let ndc = proj_coords.xy / proj_coords.w;
        let new_color = tex_color(ndc);
        color = mix(new_color, color, color.a);
    }
    return color;
}

fn tex_color(ndc: vec2<f32>) -> vec4<f32> {
    let tex_pos = ndc * 0.5 + 0.5;

    if tex_pos.x >= 0.0 && tex_pos.x <= 1.0
        && tex_pos.y >= 0.0 && tex_pos.y <= 1.0
    {
        let tex_coords = vec2<f32>(tex_pos.x, tex_pos.y);
        return textureSample(t_diffuse, s_diffuse, tex_coords);
    }
    return vec4<f32>(0.0);
}


