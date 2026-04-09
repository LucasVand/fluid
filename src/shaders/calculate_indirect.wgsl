const MAX: u32 = 0xFFFFFFFFu;

struct DispatchIndirectArgs {
    x: u32,
    y: u32,
    z: u32,
};

@group(0) @binding(0) var<storage, read> spatial_lookup: array<vec2<u32>>;

//TODO: use start and end indicies to parrallelize this

@group(0) @binding(1) var<storage, read_write> cell_ranges: array<vec2<u32>>;
@group(0) @binding(2) var<storage, read_write> indirectBuffer: array<DispatchIndirectArgs>;

@compute @workgroup_size(32)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let gloabl_id = id.x;

    if gloabl_id != 0 {
        return;
    }

    let n = arrayLength(&spatial_lookup);

    let workgroup_size: u32 = 64;
    var count: u32 = 0;
    var start: u32 = 0;
    var current_cell: u32 = 0;
    var prev_key: u32 = spatial_lookup[0].x;

    for (var i: u32 = 0; i < n; i++) {
        // if count is equal to workgroup size or we are at a break between cells
        if count >= workgroup_size || prev_key != spatial_lookup[i].x {
            cell_ranges[current_cell] = vec2(start, i);
            start = i;
            current_cell += 1;
            count = 0;
        }
        count += 1;

        prev_key = spatial_lookup[i].x;
    }

    cell_ranges[current_cell] = vec2(start, n);
    current_cell += 1;

    indirectBuffer[0].x = current_cell;
    indirectBuffer[0].y = 1;
    indirectBuffer[0].z = 1;
}
