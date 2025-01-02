const PI: f32 = 3.14159265;

struct Globals {
    time: f32,
    cursor_x: f32,
    cursor_y: f32,
}
@group(0) @binding(0)
var<uniform> globals: Globals;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vertex(
    @builtin(vertex_index) id: u32,
) -> VertexOutput {
    var pos = array(
        vec2f(1.0, 1.0),
        vec2f(-1.0, 1.0),
        vec2f(-1.0, -1.0),
        vec2f(1.0, 1.0),
        vec2f(-1.0, -1.0),
        vec2f(1.0, -1.0),
    );

    var uv = array(
        vec2f(1.0, 0.0),
        vec2f(0.0, 0.0),
        vec2f(0.0, 1.0),
        vec2f(1.0, 0.0),
        vec2f(0.0, 1.0),
        vec2f(1.0, 1.0),
    );

    var out: VertexOutput;
    out.clip_position = vec4<f32>(pos[id], 0.0, 1.0);
    out.uv = vec2<f32>(uv[id]);
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let x_sin = sin(in.uv.y * 20 + globals.time * 5) / 100.;
    let x = x_sin * step(in.uv.x + x_sin, (globals.cursor_x + 1) / 2);
    let tex_x = in.uv.x + x;
    return textureSample(t_diffuse, s_diffuse, vec2<f32>(tex_x, in.uv.y));
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
