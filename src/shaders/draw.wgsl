@group(0) @binding(2) var<uniform> params: RenderParams; 
@group(0) @binding(1) var<storage, read> particles: array<Particle>;
@group(0) @binding(0) var<uniform> model: mat4x4<f32>; 
@group(0) @binding(3) var<uniform> camera: Camera;

struct Particle {
    position: vec3<f32>,
    _pad0: f32,
    predicted_position: vec3<f32>,
    _pad1: f32,
    velocity: vec3<f32>,
    _pad2: f32,
    density: f32,
    near_density: f32,
    is_boundry: u32,
    _pad3: f32,
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

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) vel: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_pos: vec3<f32>,
    @location(3) is_boundry: u32,
    @location(4) index: u32,
}

const LIGHT_POS: vec3<f32> = vec3<f32>(100.0, 100.0, 100.0);
const AMBIENT: f32 = 0.4;

@vertex
fn vs_main(
    @location(0) vertex_pos: vec3<f32>,
    @location(1) vertex_normal: vec3<f32>,
    @builtin(instance_index) i_index: u32,
) -> VsOut {
    let particle = particles[i_index];
    let scale = params.particle_size;

    let scaled_pos = vertex_pos * scale;
    let world_pos_calc = model * vec4(particle.position + scaled_pos, 1.0);
    let screen_pos = camera.matrix * world_pos_calc;

    let world_normal = normalize((model * vec4(vertex_normal, 0.0)).xyz);

    var out: VsOut;
    out.pos = screen_pos;
    out.vel = particle.velocity;
    out.world_normal = world_normal;
    out.world_pos = world_pos_calc.xyz;
    out.is_boundry = particle.is_boundry;
    out.index = i_index;

    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {

    if in.is_boundry == 1 {
        discard;
    }

    let vel = -length(in.vel) * params.color_multiplier + params.color_offset;
    let rgb = hsv_to_rgb(max(vel, 0.0), 0.7, 0.8);

    let light_dir = normalize(LIGHT_POS - in.world_pos);
    let diffuse = max(0.0, dot(in.world_normal, light_dir));
    let lighting = AMBIENT + diffuse * (1.0 - AMBIENT);

    return vec4(rgb * lighting, 1.0);
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
