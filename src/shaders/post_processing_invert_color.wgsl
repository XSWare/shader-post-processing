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
    let pixel = textureSample(t_diffuse, s_diffuse, in.uv);
    if in.uv.x > (globals.cursor_x + 1) / 2 {
        return pixel;
    } else {
        let inverted = invert_color(pixel.xyz);
        return vec4<f32>(inverted, 1.0);
    }
}

fn invert_color(color: vec3<f32>) -> vec3<f32> {
    return 1 - color;
}