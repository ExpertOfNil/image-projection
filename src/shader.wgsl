// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(1)
var<uniform> projector: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    // Calculate vertex position in clip space
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);

    // Transform vertex position from world space to projector space
    let proj_space = projector.view_proj * vec4<f32>(model.position, 1.0);

    // Calculate vertex position in screen space
    let ndc = proj_space.xyz / proj_space.w;
    let tex_pos = ndc * 0.5 + 0.5;
    out.tex_coords = vec2<f32>(tex_pos.x, 1.0 - tex_pos.y);

    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_coords = clamp(in.tex_coords, vec2<f32>(0.0), vec2<f32>(1.0));
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}


