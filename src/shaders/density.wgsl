const PI = 3.14159265359;
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
var<workgroup> shared_index: array<u32, 64>;

fn smoothing_kernel(radius: f32, dist: f32) -> f32 {
    if dist >= radius {
        return 0.0;
    }
    let volume = PI * radius * radius * radius * radius / 6.0;
    return (radius - dist) * (radius - dist) / volume;
}

fn near_density_smoothing_kernel(radius: f32, dist: f32) -> f32 {
    if dist >= radius {
        return 0.0;
    }
    let volume = PI * radius * radius * radius * radius * radius * radius / 15.0;
    let v = radius - dist;
    return v * v * v / volume;
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
fn process_particle(predicted: vec3<f32>, neighbor_predicted: vec3<f32>, density: ptr<function, f32>,
    near_density: ptr<function, f32>) {
    let dst = distance(neighbor_predicted, predicted);

    let influence = smoothing_kernel(params.smoothing_radius, dst);
    let near_influence = near_density_smoothing_kernel(params.smoothing_radius, dst);

    *density += influence;
    *near_density += near_influence;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(local_invocation_id) local_id: vec3<u32>, @builtin(workgroup_id) workgroup_id: vec3<u32>) {

    let range = cell_ranges[workgroup_id.x];

    let lookup_idx = range.x + local_id.x;

    var loader_only: bool;
    var particle_idx: u32;
    var predicted: vec3<f32>;
    if lookup_idx >= range.y {
        loader_only = true;

        let start_spatial_index = spatial_lookup[range.x];
        predicted = particles[start_spatial_index.y].predicted_position;
        particle_idx = 0;
    } else {
        loader_only = false;

        let lookup = spatial_lookup[lookup_idx];

        particle_idx = lookup.y;
        predicted = particles[particle_idx].predicted_position;
    }

    var density = 0.00001;
    var near_density = 0.00001;

    let cell_count = u32(arrayLength(&start_indices));
    let particle_count = arrayLength(&particles);

    let coords = get_cell_coords(predicted);

    // Check all 27 neighboring cells
    for (var ox: i32 = -1; ox <= 1; ox += 1) {
        for (var oy: i32 = -1; oy <= 1; oy += 1) {
            for (var oz: i32 = -1; oz <= 1; oz += 1) {

                let neighbor_coords = coords + vec3<i32>(ox, oy, oz);
                let neighbor_key = hash_coords(neighbor_coords.x, neighbor_coords.y, neighbor_coords.z, cell_count);

                // process_cell(neighbor_key, particle_idx, predicted, cell_count, &density, &near_density);

                let start = start_indices[neighbor_key];
                let end = end_indices[neighbor_key];
                if start == MAX {
                    continue;
                }

                for (var i: u32 = start; i < end; i += WORKGROUP_SIZE) {

                    let spatial_load_index = i + local_id.x;

                    if spatial_load_index < end {
                        let neighbor_lookup = spatial_lookup[spatial_load_index];
                        let neighbor_idx = neighbor_lookup.y;
                        shared_predicted[local_id.x] = particles[neighbor_idx].predicted_position;
                        shared_index[local_id.x] = neighbor_idx;
                    }

                    workgroupBarrier();

                    let chunk_size = min(WORKGROUP_SIZE, end - i);

                    if !loader_only {
                        for (var j: u32 = 0; j < chunk_size; j++) {
                            process_particle(predicted, shared_predicted[j], &density, &near_density);
                        }
                    }

                    workgroupBarrier();
                }
            }
        }
    }

    if !loader_only {
        particles[particle_idx].density = density;
        particles[particle_idx].near_density = near_density;
    }
}

fn process_cell(
    cell_key: u32,
    particle_idx: u32,
    particle_pos: vec3<f32>,
    cell_count: u32,
    density: ptr<function, f32>,
    near_density: ptr<function, f32>
) {
    let start_index = start_indices[cell_key];
    let end_index = end_indices[cell_key];
    if start_index == MAX {
        return;
    }

    for (var i = start_index; i < end_index; i++) {
        let lookup_entry = spatial_lookup[i];
        let lookup_cell_key = lookup_entry.x;
        let neighbor_idx = lookup_entry.y;

        if neighbor_idx != particle_idx {
            let neighbor = particles[neighbor_idx];
            process_particle(particle_pos, neighbor.predicted_position, density, near_density);
        }
    }
}

fn is_nan(x: f32) -> bool {
    return x != x;
}
