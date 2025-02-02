const PI: f32 = 3.14159265;

struct Globals {
    time: f32,
    cursor_x: f32,
    cursor_y: f32,
    // WebGL needs 16 byte alignment 
    padding: f32,
}
@group(0) @binding(0)
var<uniform> globals: Globals;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vertex(
    @builtin(vertex_index) id: u32,
) -> VertexOutput {
    // vertices describe a rectangle that covers the complete screen
    var pos = array(
        vec2f(1.0, 1.0),
        vec2f(-1.0, 1.0),
        vec2f(-1.0, -1.0),
        vec2f(1.0, 1.0),
        vec2f(-1.0, -1.0),
        vec2f(1.0, -1.0),
    );

    // provide a texture mapping that covers the rectangle created above
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

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let center = vec2<f32>((globals.cursor_x + 1) / 2, (globals.cursor_y + 1) / 2);
    let r_vec = in.uv - center;
    let dis = distance(in.uv, center);
    let fall_off_distance = 0.1;
    let fall_off_factor = clamp((0.1 - dis / 2), 0.0, 1.0);
    let radius_offset = fall_off_factor * (sin((dis * PI * 50) - globals.time * 10) + 1) / 2;
    let out_radius = dis + radius_offset;
    let out_vec = center + normalize(r_vec) * out_radius;
    return textureSample(t_diffuse, s_diffuse, out_vec);
}

fn larp(distance: f32, limit: f32) -> f32 {
    return distance;
}