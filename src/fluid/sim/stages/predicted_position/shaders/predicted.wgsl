struct Particle {
    position: vec3<f32>,
    _pad0: f32,
    //
    predicted_position: vec3<f32>,
    _pad1: f32,
    //
    velocity: vec3<f32>,
    _pad2: f32,
    //
    density: f32,
    near_density: f32,
    is_boundry: u32,
    _pad3: f32,
}

struct Params {
    target_density: f32,
    pressure_multiplier: f32,
    near_pressure_multiplier: f32,
    smoothing_radius: f32,
    gravity: f32,
    damping: f32,
    time_step: f32,
    particle_size: f32,
    viscosity_strength: f32,
}

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if idx >= arrayLength(&particles) {
        return;
    }

    var particle = particles[idx];

    // Apply gravity to velocity
    particles[idx].velocity.y -= params.gravity * params.time_step;

    // Calculate predicted position
    particles[idx].predicted_position = particle.position + particle.velocity * (1.0 / 120.0);
}
