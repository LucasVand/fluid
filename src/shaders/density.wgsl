const PI = 3.14159265359;

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

fn process_cell(
    cell_key: u32,
    particle_idx: u32,
    particle_pos: vec3<f32>,
    cell_count: u32,
    density: ptr<function, f32>,
    near_density: ptr<function, f32>
) {
    if cell_key >= cell_count {
        return;
    }

    let start_index = start_indices[cell_key];
    if start_index == 0xFFFFFFFFu {
        return;
    }

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

            let influence = smoothing_kernel(params.smoothing_radius, dst);
            let near_influence = near_density_smoothing_kernel(params.smoothing_radius, dst);

            *density += influence;
            *near_density += near_influence;
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
    var density = 0.00001;
    var near_density = 0.00001;

    let cell_count = u32(arrayLength(&start_indices));
    let coords = get_cell_coords(particle.predicted_position);

    // Check all 27 neighboring cells
    for (var ox: i32 = -1; ox <= 1; ox += 1) {
        for (var oy: i32 = -1; oy <= 1; oy += 1) {
            for (var oz: i32 = -1; oz <= 1; oz += 1) {
                let neighbor_coords = coords + vec3<i32>(ox, oy, oz);
                let neighbor_key = hash_coords(neighbor_coords.x, neighbor_coords.y, neighbor_coords.z, cell_count);

                process_cell(neighbor_key, idx, particle.predicted_position, cell_count, &density, &near_density);
            }
        }
    }

    particles[idx].density = density;
    particles[idx].near_density = near_density;
}
