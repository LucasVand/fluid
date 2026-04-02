struct Particle {
    position: vec3<f32>,
    _pad0: f32,
    predicted_position: vec3<f32>,
    _pad1: f32,
    velocity: vec3<f32>,
    _pad2: f32,
    density: f32,
    near_density: f32,
    _pad3: vec2<f32>,
}

struct Params {
    target_density: f32,
    pressure_multiplier: f32,
    near_pressure_multiplier: f32,
    smoothing_radius: f32,
    // 16 
    gravity: f32,
    damping: f32,
    time_step: f32,
    particle_size: f32,
    // 16
    viscosity_strength: f32,
    // 12 bytes of padding here lol
    // 16
    bounds_min: vec3<f32>,
    _pad0: f32,
    // 16
    bounds_max: vec3<f32>,
    _pad1: f32,
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
    let half_size = params.particle_size * 0.5;
    particle.velocity += vec3(0.0, -1.0, 0.0) * params.gravity;

    particle.position += particle.velocity * params.time_step;

    // Collision detection and response

    // X axis
    if particle.position.x - half_size < params.bounds_min.x {
        particle.position.x = params.bounds_min.x + half_size;
        particle.velocity.x *= -1.0 * params.damping;
    }
    if particle.position.x + half_size > params.bounds_max.x {
        particle.position.x = params.bounds_max.x - half_size;
        particle.velocity.x *= -1.0 * params.damping;
    }

    // Y axis
    if particle.position.y - half_size < params.bounds_min.y {
        particle.position.y = params.bounds_min.y + half_size;
        particle.velocity.y *= -1.0 * params.damping;
    }
    if particle.position.y + half_size > params.bounds_max.y {
        particle.position.y = params.bounds_max.y - half_size;
        particle.velocity.y *= -1.0 * params.damping;
    }

    // Z axis
    if particle.position.z - half_size < params.bounds_min.z {
        particle.position.z = params.bounds_min.z + half_size;
        particle.velocity.z *= -1.0 * params.damping;
    }
    if particle.position.z + half_size > params.bounds_max.z {
        particle.position.z = params.bounds_max.z - half_size;
        particle.velocity.z *= -1.0 * params.damping;
    }

    particles[idx] = particle;
}
