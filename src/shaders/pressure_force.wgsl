const PI = 3.14159265359;
const MASS = 1.0;
const WORKGROUP_SIZE: u32 = 64;

const MAX: u32 = 0xFFFFFFFFu;

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
@group(0) @binding(4) var<storage, read> end_indices: array<u32>;
@group(0) @binding(5) var<storage, read> cell_ranges: array<vec2<u32>>;

var<workgroup> shared_predicted: array<vec3<f32>, 64>;
var<workgroup> shared_velocity: array<vec3<f32>, 64>;
var<workgroup> shared_near_density: array<f32, 64>;
var<workgroup> shared_density: array<f32, 64>;

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
    let volume = (PI * radius * radius * radius * radius * radius * radius / 15.0);
    return -3.0 * v * v / volume;
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
    let pressure = density_error * params.pressure_multiplier;
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

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(local_invocation_id) local_id: vec3<u32>, @builtin(workgroup_id) workgroup_id: vec3<u32>) {
    let range = cell_ranges[workgroup_id.x];

    let lookup_idx = range.x + local_id.x;

    var loader_only: bool;
    var particle_idx: u32;
    var predicted: vec3<f32>;
    var velocity: vec3<f32>;
    var density: f32;
    var near_density: f32;
    var neighbour_count: u32 = 0;
    if lookup_idx >= range.y {
        loader_only = true;

        let start_spatial_index = spatial_lookup[range.x];
        predicted = particles[start_spatial_index.y].predicted_position;
        particle_idx = 0;
    } else {
        loader_only = false;

        let lookup = spatial_lookup[lookup_idx];

        particle_idx = lookup.y;
        let particle = particles[particle_idx];

        predicted = particle.predicted_position;
        velocity = particle.velocity;
        density = particle.density;
        near_density = particle.near_density;
    }

    var pressure_force = vec3(0.000001);
    var viscosity_force = vec3(0.0);

    let cell_count = u32(arrayLength(&start_indices));
    let particle_count = arrayLength(&particles);

    let coords = get_cell_coords(predicted);

    // Check all 27 neighboring cells
    for (var ox: i32 = -1; ox <= 1; ox += 1) {
        for (var oy: i32 = -1; oy <= 1; oy += 1) {
            for (var oz: i32 = -1; oz <= 1; oz += 1) {
                let neighbor_coords = coords + vec3<i32>(ox, oy, oz);
                let neighbor_key = hash_coords(neighbor_coords.x, neighbor_coords.y, neighbor_coords.z, cell_count);

                let start = start_indices[neighbor_key];
                let end = end_indices[neighbor_key];
                if start == MAX {
                    continue;
                }

                // if !loader_only {
                //     process_cell_forces(neighbor_key, particle_idx, predicted, velocity, density, near_density, cell_count, &pressure_force, &viscosity_force);
                // }

                for (var i: u32 = start; i < end; i += WORKGROUP_SIZE) {

                    let spatial_load_index = i + local_id.x;

                    if spatial_load_index < end {
                        let neighbor_lookup = spatial_lookup[spatial_load_index];
                        let neighbor_idx = neighbor_lookup.y;
                        let neighbor = particles[neighbor_idx];

                        shared_predicted[local_id.x] = neighbor.predicted_position;
                        shared_near_density[local_id.x] = neighbor.near_density;
                        shared_density[local_id.x] = neighbor.density;
                        shared_velocity[local_id.x] = neighbor.velocity;
                    }

                    workgroupBarrier();

                    let chunk_size = min(WORKGROUP_SIZE, end - i);

                    if !loader_only {
                        for (var j: u32 = 0; j < chunk_size; j++) {
                            process_particle(&pressure_force,
                                &viscosity_force,
                                predicted,
                                velocity,
                                density,
                                near_density,
                                shared_predicted[j],
                                shared_velocity[j],
                                shared_density[j],
                                shared_near_density[j],
                                &neighbour_count);
                        }
                    }

                    workgroupBarrier();
                }
            }
        }
    }

    if !loader_only {
        let inv_density = 1.0 / max(particles[particle_idx].density, 0.001);
        particles[particle_idx].velocity += ((pressure_force * inv_density) + viscosity_force * params.viscosity_strength);
        // Airborne drag: damp spray/isolated particles to prevent them flying off
        if neighbour_count < 8 {
            let drag = 1.0f - 1.5f * params.time_step;
            particles[particle_idx].velocity.x *= drag;
        }
    }
}

fn process_particle(pressure_force: ptr<function, vec3<f32>>,
    viscosity_force: ptr<function, vec3<f32>>,
    predicted: vec3<f32>,
    velocity: vec3<f32>,
    density: f32,
    near_density: f32,
    neighbor_predicted: vec3<f32>,
    neighbor_velocity: vec3<f32>,
    neighbor_density: f32,
    neighbor_near_density: f32,
    neighbour_count: ptr<function, u32>) {

    let dst = distance(neighbor_predicted, predicted);
    if dst == 0.0 {
        return;
    }

    *neighbour_count += 1;
    let dir = (neighbor_predicted - predicted) / dst;
    let slope = smoothing_kernel_derivative(params.smoothing_radius, dst);
    let slope_near = near_density_smoothing_kernel_derivative(params.smoothing_radius, dst);

    // Pressure force
    let neighbor_pressure = convert_density_to_pressure(neighbor_density, neighbor_near_density);
    let self_pressure = convert_density_to_pressure(density, near_density);
    let shared_pressure = (neighbor_pressure + self_pressure) * 0.5;

    // let density_product = neighbor_density * density;
    *pressure_force += shared_pressure.x * dir * slope * MASS / density;

    // let near_density_product = neighbor_near_density * near_density;
    *pressure_force += shared_pressure.y * dir * slope_near * MASS / near_density;

    // Viscosity force
    let influence = viscosity_smoothing_kernel(params.smoothing_radius, dst);
    *viscosity_force += (neighbor_velocity - velocity) * influence;
}
