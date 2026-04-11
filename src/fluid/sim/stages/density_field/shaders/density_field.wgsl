const PI = 3.14159265359;
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
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
    bounds_min: vec3<f32>,
    _pad3: f32,
    bounds_max: vec3<f32>,
    _pad4: f32,
}

@group(0) @binding(0) var<storage, read> particles: array<Particle>;
@group(0) @binding(1) var<uniform> params: Params;
@group(0) @binding(2) var<storage, read> spatial_lookup: array<vec2<u32>>;
@group(0) @binding(3) var<storage, read> start_indices: array<u32>;
@group(0) @binding(4) var<storage, read> end_indices: array<u32>;
@group(0) @binding(5) var density_texture: texture_storage_3d<r32float, read_write>;

fn smoothing_kernel(radius: f32, dist: f32) -> f32 {
    if dist >= radius {
        return 0.0;
    }
    let volume = PI * radius * radius * radius * radius / 6.0;
    return (radius - dist) * (radius - dist) / volume;
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

@compute @workgroup_size(8, 8, 4)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let texture_size = textureDimensions(density_texture);

    // Check bounds
    if global_id.x >= texture_size.x || global_id.y >= texture_size.y || global_id.z >= texture_size.z {
        return;
    }

    // Map thread ID to world position within bounding box
    let normalized = vec3<f32>(global_id) / vec3<f32>(texture_size);
    let pos = params.bounds_min + normalized * (params.bounds_max - params.bounds_min);

    var density = 0.00000;

    let cell_count = u32(arrayLength(&start_indices));
    let coords = get_cell_coords(pos);
    let smoothing_radius_sq = params.smoothing_radius * params.smoothing_radius;

    // Search radius in cells (1 cell = smoothing_radius)
    let search_radius: i32 = 1i;

    // Check all neighboring cells within smoothing radius
    for (var dx = -search_radius; dx <= search_radius; dx++) {
        for (var dy = -search_radius; dy <= search_radius; dy++) {
            for (var dz = -search_radius; dz <= search_radius; dz++) {
                let neighbor_coords = coords + vec3<i32>(dx, dy, dz);
                let neighbor_key = hash_coords(neighbor_coords.x, neighbor_coords.y, neighbor_coords.z, cell_count);

                let start = start_indices[neighbor_key];
                let end = end_indices[neighbor_key];

                if start == MAX {
                    continue;
                }

                // Loop through particles in this cell
                for (var i = start; i < end; i++) {
                    let lookup = spatial_lookup[i];
                    let particle_idx = lookup.y;
                    let particle = particles[particle_idx];
                    let dst_sq = dot(particle.predicted_position, pos);

                    if dst_sq < smoothing_radius_sq {
                        let diff = particle.predicted_position - pos;
                        let dst = distance(diff, diff);

                        let influence = smoothing_kernel(params.smoothing_radius, dst);
                        density += influence;
                    }
                }
            }
        }
    }

    // Write density to 3D texture
    textureStore(density_texture, global_id, vec4<f32>(density, 0.0, 0.0, 0.0));
}
