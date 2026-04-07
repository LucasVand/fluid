@group(0) @binding(0) var<storage, read_write> spatial_lookup: array<vec2<u32>>;
@group(0) @binding(1) var<storage, read_write> start_indices: array<u32>;
@group(0) @binding(2) var<storage, read_write> end_indices: array<u32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let n = arrayLength(&spatial_lookup);
    if i >= n {
        return;
    }

    let lookup = spatial_lookup[i];

    if i == 0 {
        start_indices[lookup.x] = 0;
        return;
    }

    let prev = spatial_lookup[i - 1];

    if lookup.x != prev.x {
        start_indices[lookup.x] = i;
        end_indices[prev.x] = i;
    }

    // Last element
    if i == n - 1 {
        end_indices[lookup.x] = n;
    }
}

const MAX = 0xFFFFFFFFu;
@compute @workgroup_size(64) 
fn main_clear(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let n = arrayLength(&spatial_lookup);
    if i >= n {
        return;
    }
    start_indices[i] = MAX;
    end_indices[i] = MAX;
}

