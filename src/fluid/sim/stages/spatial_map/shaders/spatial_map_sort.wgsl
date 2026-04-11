@group(0) @binding(0) var<storage, read_write> spatial_lookup: array<vec2<u32>>;
@group(0) @binding(1) var<storage, read_write> start_indices: array<u32>;

var<push_constant> params: Params;

struct Params {
    j: u32,
    k: u32,
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;

    // These would normally be passed as uniforms
    var k: u32 = params.k;
    var j: u32 = params.j;

    let ixj = i ^ j;

    if ixj > i {
        let ascending = (i & k) == 0u;

        let a = spatial_lookup[i];
        let b = spatial_lookup[ixj];

        var should_swap = false;

        if ascending && a.x > b.x {
            should_swap = true;
        }

        if !ascending && a.x < b.x {
            should_swap = true;
        }

        if should_swap {
            spatial_lookup[i] = b;
            spatial_lookup[ixj] = a;
        }
    }
}
