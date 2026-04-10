const MAX: u32 = 0xFFFFFFFFu;
const WORKGROUP_SIZE: u32 = 64;

@group(0) @binding(0) var<storage, read> spatial_lookup: array<vec2<u32>>;

//TODO: Bug might not be real but on the debug it says were missing ranges 0-64 always?? 

@group(0) @binding(1) var<storage, read_write> cell_ranges: array<vec2<u32>>;
@group(0) @binding(2) var<storage, read_write> indirectBuffer: array<atomic<u32>>;
@group(0) @binding(3) var<storage, read> start_indices: array<u32>;
@group(0) @binding(4) var<storage, read> end_indices: array<u32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let gloabl_id = id.x;

    let n = arrayLength(&spatial_lookup);
    if gloabl_id >= n {
        return;
    }

    if gloabl_id == 0 {
        atomicStore(&indirectBuffer[1], 1u);
        atomicStore(&indirectBuffer[2], 1u);
    }

    let start_idx = start_indices[gloabl_id];
    let end_idx = end_indices[gloabl_id];

    if start_idx == MAX || end_idx == MAX {
        return;
    }

    var current = start_idx;

    loop {
        var next = current + WORKGROUP_SIZE;
        if next > end_idx {
            next = end_idx;
        }

        let idx = atomicAdd(&indirectBuffer[0], 1u);

        cell_ranges[idx] = vec2(current, next);

        if next >= end_idx {
            break;
        }

        current = next;
    }
}



