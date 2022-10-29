struct GBuffer {
    values: array<atomic<u32>>
};

struct Vertex {
    pos: vec4<f32>,
    color: vec4<f32>
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

struct Camera {
    world_to_cam: mat4x4<f32>,
    cam_to_screen: mat4x4<f32>,
};

var<push_constant> pc: PushConstants;
@group(0) @binding(0) var<storage, read_write> g_buffer: GBuffer;
@group(1) @binding(0) var<uniform> cam: Camera;
@group(2) @binding(0) var<storage, read> vertices : array<Vertex>;
@group(2) @binding(1) var<storage, read> indices : array<u32>;
@group(2) @binding(2) var<storage, read> position : mat4x4<f32>;

// functions for indexing the g-buffer //
fn depth(x: u32, y: u32) -> u32 { return (x + y * u32(pc.pix_x)) * 3u; }
fn color(x: u32, y: u32) -> u32 { return (x + y * u32(pc.pix_x)) * 3u + 1u; }
fn normal(x: u32, y: u32) -> u32 { return (x + y * u32(pc.pix_x)) * 3u + 2u; }

fn triangle_normal(a: vec3<f32>, b: vec3<f32>, c: vec3<f32>) -> vec3<f32> {
    // computes the normal of a triangle //
    return normalize(cross((b - c), (a - c)));
}

fn color_to_int(col: vec4<f32>) -> u32 {
    // takes a color vector and converts it to an integer //
    return u32(255.0 * clamp(col.a, 0.0, 1.0))
        + (u32(255.0 * clamp(col.b, 0.0, 1.0)) << 8u)
        + (u32(255.0 * clamp(col.g, 0.0, 1.0)) << 16u)
        + (u32(255.0 * clamp(col.r, 0.0, 1.0)) << 24u);
}

fn normal_to_int(norm: vec3<f32>) -> u32 {
    // takes a normal and converts it to an integer //
    return (u32(65535.0 * clamp((norm.x + 1.0) / 2.0, 0.0, 1.0)) << 16u)
        + u32(65535.0 * clamp((norm.y + 1.0) / 2.0, 0.0, 1.0));
}

fn w_to_c(v: vec4<f32>) -> vec3<f32> {
    // translates from world coords to camera coords //
    let a = cam.world_to_cam * position * v;
    let b = vec4(a.xy, a.z - pc.cam_z, 1.0) * pc.map_scale;
    let c = (cam.cam_to_screen * b).xyz * pc.pix_x
        + vec3(pc.pix_x / 2., pc.pix_y / 2., 0.0);
    return c;
}

fn barycentric(
    v1: vec3<f32>,
    v2: vec3<f32>,
    v3: vec3<f32>,
    p: vec2<f32>
) -> vec3<f32> {
    // gives the barycentric coords for p in terms of v1, v2, v3 //
    let u = cross(
        vec3<f32>(v3.x - v1.x, v2.x - v1.x, v1.x - p.x), 
        vec3<f32>(v3.y - v1.y, v2.y - v1.y, v1.y - p.y)
    );

    if (abs(u.z) < 1.0) {
        return vec3<f32>(-1.0, 1.0, 1.0);
    }

    return vec3<f32>(1.0 - (u.x+u.y)/u.z, u.y/u.z, u.x/u.z); 
}

fn get_min_max(v1: vec3<f32>, v2: vec3<f32>, v3: vec3<f32>) -> vec4<f32> {
    // gets the minimum and maximum x and y values //
    var min_max = vec4<f32>();
    min_max.x = max(min(min(v1.x, v2.x), v3.x), 0.0);
    min_max.y = max(min(min(v1.y, v2.y), v3.y), 0.0);
    min_max.z = min(max(max(v1.x, v2.x), v3.x), pc.pix_x - 1.);
    min_max.w = min(max(max(v1.y, v2.y), v3.y), pc.pix_y - 1.);

    return min_max;
}

@compute
@workgroup_size(64)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = arrayLength(&indices);
    let index = global_invocation_id.x * 3u;
    if (index >= total) {
        return;
    }

    let a = w_to_c(vertices[indices[index + 0u]].pos);
    let b = w_to_c(vertices[indices[index + 1u]].pos);
    let c = w_to_c(vertices[indices[index + 2u]].pos);

    let norm = triangle_normal(a, b, c);
    if norm.z < 0.0 {
        return;
    }

    let minmax = get_min_max(a, b, c);
    let startX = u32(minmax.x);
    let startY = u32(minmax.y);
    let endX = u32(minmax.z);
    let endY = u32(minmax.w);

    for (var x: u32 = startX; x <= endX; x = x + 1u) {
        for (var y: u32 = startY; y <= endY; y = y + 1u) {
            let bc = barycentric(a, b, c, vec2(f32(x), f32(y)));
            if (bc.x < -0.0 || bc.y < -0.0 || bc.z < -0.0) {
                continue;
            }

            let d = (bc.x * a.z + bc.y * b.z + bc.z * c.z);
            if d < pc.far_plane || d > pc.near_plane{
                continue;
            }

            let d = u32((d - pc.far_plane)
                / (pc.near_plane - pc.far_plane)
                * 16777215.0);
            
            let col = (
                    bc.x * vertices[indices[index + 0u]].color +
                    bc.y * vertices[indices[index + 1u]].color +
                    bc.z * vertices[indices[index + 2u]].color);

            if d > atomicMax(&g_buffer.values[depth(x, y)], d) {
                atomicStore(
                    &g_buffer.values[color(x, y)],
                    color_to_int(col)
                );
                atomicStore(
                    &g_buffer.values[normal(x, y)],
                    normal_to_int(norm)
                );
            }
        }
    }

};
