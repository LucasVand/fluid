const PI = 3.14159265359;
const MASS = 1.0;

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
    gravity: f32,
    damping: f32,
    time_step: f32,
    particle_size: f32,
    viscosity_strength: f32,
}

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<uniform> params: Params;
@group(0) @binding(2) var<storage, read> spatial_lookup: array<vec2<u32>>;
@group(0) @binding(3) var<storage, read> start_indices: array<u32>;

fn smoothing_kernel_derivative(radius: f32, dist: f32) -> f32 {
    if dist >= radius {
        return 0.0;
    }
    let scale = 12.0 / (PI * radius * radius * radius * radius);
    return (dist - radius) * scale;
}

fn near_density_smoothing_kernel_derivative(radius: f32, dist: f32) -> f32 {
    if dist >= radius {
        return 0.0;
    }
    let v = radius - dist;
    return -3.0 * v * v / (PI * radius * radius * radius * radius * radius * radius / 15.0);
}

fn viscosity_smoothing_kernel(radius: f32, dist: f32) -> f32 {
    if dist >= radius {
        return 0.0;
    }
    let volume = PI * radius * radius * radius * radius * radius * radius * radius * radius / 4.0;
    let value = max(0.0, radius * radius - dist * dist);
    return value * value * value / volume;
}

fn convert_density_to_pressure(density: f32, near_density: f32) -> vec2<f32> {
    let density_error = density - params.target_density;
    let pressure = density_error * params.pressure_multiplier * 170.0;
    let near_pressure = near_density * params.near_pressure_multiplier;
    return vec2<f32>(pressure, near_pressure);
}

fn hash_coords(c_x: i32, c_y: i32, c_z: i32, cell_count: u32) -> u32 {
    let P1: i32 = 15823;
    let P2: i32 = 9739333;
    let P3: i32 = 786433;

    let hash = P1 * c_x + P2 * c_y + P3 * c_z;
    return u32(hash) % cell_count;
}

fn get_cell_coords(pos: vec3<f32>) -> vec3<i32> {
    let cell_size = params.smoothing_radius;
    return vec3<i32>(
        i32(pos.x / cell_size),
        i32(pos.y / cell_size),
        i32(pos.z / cell_size)
    );
}

fn process_cell_forces(
    cell_key: u32,
    particle_idx: u32,
    particle_pos: vec3<f32>,
    particle_vel: vec3<f32>,
    particle_density: f32,
    particle_near_density: f32,
    cell_count: u32,
    pressure_force: ptr<function, vec3<f32>>,
    viscosity_force: ptr<function, vec3<f32>>
) {
    if cell_key >= cell_count {
        return;
    }

    let start_index = start_indices[cell_key];
    if start_index == 0xFFFFFFFFu {
        return;
    }
    // TODO: yoooo the spacial map look up is wrong because it does not account for the tuple

    var i = start_index;
    while i < arrayLength(&spatial_lookup) {
        let lookup_entry = spatial_lookup[i];
        let lookup_cell_key = lookup_entry.x;
        let neighbor_idx = lookup_entry.y;

        if lookup_cell_key != cell_key {
            break;
        }

        if neighbor_idx != particle_idx {
            let neighbor = particles[neighbor_idx];
            let dst = distance(neighbor.predicted_position, particle_pos);

            if dst > 0.0 {
                let dir = (neighbor.predicted_position - particle_pos) / dst;
                let slope = smoothing_kernel_derivative(params.smoothing_radius, dst);
                let slope_near = near_density_smoothing_kernel_derivative(params.smoothing_radius, dst);

                // Pressure force
                let neighbor_pressure = convert_density_to_pressure(neighbor.density, neighbor.near_density);
                let self_pressure = convert_density_to_pressure(particle_density, particle_near_density);
                let shared_pressure = (neighbor_pressure + self_pressure) * 0.5;

                let density_product = neighbor.density * particle_density;
                if density_product > 0.00001 {
                    *pressure_force += shared_pressure.x * dir * slope * MASS / density_product;
                }

                let near_density_product = neighbor.near_density * particle_near_density;
                if near_density_product > 0.00001 {
                    *pressure_force += shared_pressure.y * dir * slope_near * MASS / near_density_product;
                }

                // Viscosity force
                let influence = viscosity_smoothing_kernel(params.smoothing_radius, dst);
                *viscosity_force += (neighbor.velocity - particle_vel) * influence;
            }
        }

        i += 1u;
    }
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if idx >= arrayLength(&particles) {
        return;
    }

    var particle = particles[idx];
    var pressure_force = vec3<f32>(0.0);
    var viscosity_force = vec3<f32>(0.0);

    let cell_count = u32(arrayLength(&start_indices));
    let coords = get_cell_coords(particle.predicted_position);

    // Check all 27 neighboring cells
    for (var ox: i32 = -1; ox <= 1; ox += 1) {
        for (var oy: i32 = -1; oy <= 1; oy += 1) {
            for (var oz: i32 = -1; oz <= 1; oz += 1) {
                let neighbor_coords = coords + vec3<i32>(ox, oy, oz);
                let neighbor_key = hash_coords(neighbor_coords.x, neighbor_coords.y, neighbor_coords.z, cell_count);

                process_cell_forces(neighbor_key, idx, particle.predicted_position, particle.velocity, particle.density, particle.near_density, cell_count, &pressure_force, &viscosity_force);
            }
        }
    }

    particle.velocity += (pressure_force + viscosity_force * params.viscosity_strength) * params.time_step;
    particles[idx] = particle;
}
