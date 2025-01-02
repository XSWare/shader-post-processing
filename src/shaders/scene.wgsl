const PI: f32 = 3.14159265;

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};

// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct Globals {
    time: f32,
    cursor_x: f32,
    cursor_y: f32,
}
@group(2) @binding(0)
var<uniform> globals: Globals;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vertex(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let image = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let img_avg = average(image.xyz);

    let x_sin = normalized_sin(in.tex_coords.x * 2 * PI);
    let y_sin = normalized_sin(in.tex_coords.y * 2 * PI);
    let rainbow_x = rainbow(in.tex_coords.x * 2 * PI * 1 + (globals.time * 2));
    let rainbow_y = rainbow(in.tex_coords.y * 2 * PI * 1);
    let rainbow_crossed = (rainbow_x + rainbow_y) / 2;
    let srgb = vec3<f32>(x_sin, y_sin, 1.0);
    return vec4<f32>((rainbow_x + img_avg) / 2, 1.0);
}

fn average(srgb: vec3<f32>) -> f32 {
    return (srgb.x + srgb.y + srgb.z) / 3.0;
}

fn rainbow(angle: f32) -> vec3<f32> {
    return vec3<f32>(normalized_sin(angle), normalized_sin(angle + (PI / 3.0)), normalized_sin(angle + (2.0 * PI / 3.0)));
}

fn normalized_sin(angle: f32) -> f32 {
    return (sin(angle) + 1.0) / 2;
}
