struct Light {
    pos: vec4<f32>,
    color: vec4<f32>
};

struct GBuffer {
    values: array<atomic<u32>>
};

struct Screen {
    values: array<atomic<u32>>
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
@group(0) @binding(0) var<storage, read_write> screen : Screen;
@group(1) @binding(0) var<storage, read_write> g_buffer : GBuffer;
@group(2) @binding(0) var<uniform> cam : Camera;
@group(3) @binding(0) var<storage, read> light_buf : array<Light>;

fn w_to_c(v: vec4<f32>) -> vec3<f32> {
    // translates from world coords to camera coords //
    let a = cam.world_to_cam * position * v;
    let b = vec4(a.xy, a.z - pc.cam_z, 1.0) * pc.map_scale;
    let c = (cam.cam_to_screen * b).xyz * pc.pix_x
        + vec3(pc.pix_x / 2., pc.pix_y / 2., 0.0);
    return c;
}

fn depth_buf_to_z(num: u32) -> f32 {
    return f32(num) / 16777215.0
        * (pc.near_plane - pc.far_plane)
        + pc.far_plane;
}

// functions for indexing the g-buffer //
fn depth(x: u32, y: u32) -> u32 { return (x + y * u32(pc.pix_x)) * 3u; }
fn color(x: u32, y: u32) -> u32 { return (x + y * u32(pc.pix_x)) * 3u + 1u; }
fn normal(x: u32, y: u32) -> u32 { return (x + y * u32(pc.pix_x)) * 3u + 2u; }

fn int_to_normal(num: u32) -> vec3<f32> {
    let x = (f32(num >> 16u) / 65535.0) * 2.0 - 1.0;
    let y = (f32(num % (1u << 16u)) / 65535.0) * 2.0 - 1.0;
    let z = sqrt(1.0 - x * x - y * y);
    return vec3<f32>(x, y, z);
}

fn color_to_int(col: vec4<f32>) -> u32 {
    // takes a color vector and converts it to an integer //
    return u32(255.0 * clamp(col.a, 0.0, 1.0))
        + (u32(255.0 * clamp(col.b, 0.0, 1.0)) << 8u)
        + (u32(255.0 * clamp(col.g, 0.0, 1.0)) << 16u)
        + (u32(255.0 * clamp(col.r, 0.0, 1.0)) << 24u);
}

fn int_to_color(in: u32) -> vec4<f32> {
    // takes an integer and converts it to a color vector //
    return vec4<f32>(
        f32(in >> 24u) / 255.0,
        f32((in >> 16u) % 256u) / 255.0,
        f32((in >> 8u) % 256u) / 255.0,
        f32(in % 256u) / 255.0,
    );
}

@compute
@workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let x = global_invocation_id.x;
    let y = global_invocation_id.y;
    let light = global_invocation_id.z;

    if (x >= u32(pc.pix_x))
    || (y >= u32(pc.pix_y))
    || (light > arrayLength(&light_buf)) {
        return;
    }

    let idx = x + y * u32(pc.pix_x);

    if light == 0u {
        let col = int_to_color(
            atomicLoad(&g_buffer.values[color(x, y)])
        );
        atomicStore(
            &screen.values[idx],
            color_to_int(
                vec4(0.05 * col.rgb, col.a)
                + int_to_color(
                    atomicLoad(&screen.values[idx])
                )
            )
        )
    } else {
        let lidx = u32(i32(light) - 1);
        if light_buf[lidx].pos.w < 0.0 {
            return;
        }
        let l_pos = w_to_c(light_buf[lidx].pos);
        let col = light_buf[lidx].color;
        let p = vec3<f32>(
            f32(x),
            f32(y),
            depth_buf_to_z(
                atomicLoad(
                    &g_buffer.values[depth(x, y)]
                )
            )
        );

        let pcol = int_to_color(atomicLoad(
            &g_buffer.values[color(x, y)]
        ));

        let norm = int_to_normal(
            atomicLoad(
                &g_buffer.values[normal(x, y)]
            )
        );

        let ray = (p - l_pos) / (pc.map_scale * pc.pix_x);
        if dot(ray, ray) == 0.0 {
            return;
        }
        let brightness = col.a / (dot(ray, ray) * 100.0);

        atomicStore(
            &screen.values[idx],
            color_to_int(
                brightness * vec4(col.rgb, 1.0) * pcol
                * clamp(-dot(normalize(ray), norm), 0.0, 1.0)
                + int_to_color(
                    atomicLoad(&screen.values[idx])
                )
            )
        )
    }
}