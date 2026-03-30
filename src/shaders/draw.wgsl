
@group(0) @binding(0) var<uniform> params: RenderParams; 
@group(0) @binding(1) var<storage, read> particles: array<Particle>;
@group(0) @binding(2) var<uniform> camera: Camera; 
@group(0) @binding(3) var<uniform> model: mat4x4<f32>; 

struct Particle {
    pos: vec3<f32>,
    vel: vec3<f32>,
}

struct RenderParams {
    color_multiplier: f32,
    color_offset: f32,
    particle_size: f32,
}
struct Camera {
    matrix: mat4x4<f32>,
    position: vec3<f32>,
    _pad: f32,
}

const QUAD: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>(-1.0, -1.0),  // bottom-left
    vec2<f32>(-1.0, 1.0),   // top-left
    vec2<f32>(1.0, -1.0),   // bottom-right
    vec2<f32>(1.0, 1.0),    // top-right
);

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) vel: vec3<f32>,
    @location(1) local_pos: vec2<f32>,
}
struct FsIn {
    @location(0) vel: vec3<f32>,
    @location(1) local_pos: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) v_index: u32, @builtin(instance_index) i_index: u32) -> VsOut {
    let size: f32 = params.particle_size;
    let quad_pos = QUAD[v_index];

    let particle = particles[i_index];

    let offset = size * quad_pos;
    let world_pos = model * vec4(particle.pos + vec3(offset.x, offset.y, 0.0), 1.0);

    let screen_pos = camera.matrix * world_pos;

    var out: VsOut;
    out.pos = screen_pos;
    out.vel = particle.vel;
    out.local_pos = QUAD[v_index];

    return out;
}

    @fragment
fn fs_main(in: FsIn) -> @location(0) vec4<f32> {

    let dist = length(in.local_pos);

    if dist > 1.0 {
        discard;
    }
    let vel = -length(in.vel) * params.color_multiplier + params.color_offset;

    let rgb = hsv_to_rgb(max(vel, 0.0), 0.7, 0.8);

    return vec4(rgb, 1.0);
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> vec3<f32> {
    let c = v * s;
    let h_prime = h * 6.0;
    let x = c * (1.0 - abs((h_prime % 2.0) - 1.0));

    var rgb: vec3<f32>;
    if h_prime < 1.0 {
        rgb = vec3(c, x, 0.0);
    } else if h_prime < 2.0 {
        rgb = vec3(x, c, 0.0);
    } else if h_prime < 3.0 {
        rgb = vec3(0.0, c, x);
    } else if h_prime < 4.0 {
        rgb = vec3(0.0, x, c);
    } else if h_prime < 5.0 {
        rgb = vec3(x, 0.0, c);
    } else {
        rgb = vec3(c, 0.0, x);
    }

    let m = v - c;
    return rgb + vec3(m);
}
