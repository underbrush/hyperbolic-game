struct ColorBuffer {
    values: array<u32>
};

struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

struct PushConstants {
    pix_x: f32,
    pix_y: f32,
    border_x: f32,
    border_y: f32,
    map_scale: f32,
    cam_z: f32,
    near_plane: f32,
    far_plane: f32,
};

var<push_constant> pc: PushConstants;

@vertex
fn vs_main(@builtin(vertex_index) VertexIndex : u32) -> VertexOutput {
    var pos = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0)
    );

    var output : VertexOutput;
    output.position = vec4<f32>(
        (2.0 * pos[VertexIndex].x - 1.0) * (1.0 - pc.border_x),
        (1.0 - 2.0 * pos[VertexIndex].y) * (1.0 - pc.border_y),
        0.0, 1.0);
    output.tex_coord = pos[VertexIndex];
    return output;
}

@group(0) @binding(0) var<storage, read_write> r_color: ColorBuffer;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let x = floor(in.tex_coord.x * pc.pix_x);
    let y = floor(in.tex_coord.y * pc.pix_y);

    let index = u32(x + (y * pc.pix_x));
    let luminance = f32(r_color.values[index] % 256u) / 255.0;
    return vec4(
        f32(r_color.values[index] >> 24u) / 255.0,
        f32((r_color.values[index] >> 16u) % 256u) / 255.0,
        f32((r_color.values[index] >> 8u) % 256u) / 255.0,
        1.0);
}