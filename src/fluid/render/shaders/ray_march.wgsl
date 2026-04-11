@group(0) @binding(0) var<uniform> model: Model; 
@group(0) @binding(1) var<uniform> params: RenderParams; 
@group(0) @binding(2) var<uniform> camera: Camera;
@group(0) @binding(3) var density_map: texture_storage_3d<r32float, read>;

struct Model {
    matrix: mat4x4<f32>,
    inv_matrix: mat4x4<f32>,
}

struct RenderParams {
    color_multiplier: f32,
    color_offset: f32,
    particle_size: f32,
    pad: f32,
    bounds_min: vec3<f32>,
    pad1: f32,
    bounds_max: vec3<f32>,
    pad2: f32,
    scattering: vec3<f32>,
    pad3: f32,
    density_multiplier: f32,
}

struct Camera {
    matrix: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
    inv_view: mat4x4<f32>,
    position: vec3<f32>,
    _pad: f32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Generate fullscreen quad from vertex index
    // 0,1,2 = first triangle, 3,2,1 = second triangle
    var pos: vec2<f32>;
    var uv: vec2<f32>;

    switch vertex_index {
        case 0u: { pos = vec2<f32>(-1.0, -1.0); uv = vec2<f32>(0.0, 1.0); }
        case 1u: { pos = vec2<f32>(1.0, -1.0); uv = vec2<f32>(1.0, 1.0); }
        case 2u: { pos = vec2<f32>(-1.0, 1.0); uv = vec2<f32>(0.0, 0.0); }
        case 3u: { pos = vec2<f32>(1.0, -1.0); uv = vec2<f32>(1.0, 1.0); }
        case 4u: { pos = vec2<f32>(1.0, 1.0); uv = vec2<f32>(1.0, 0.0); }
        default: { pos = vec2<f32>(-1.0, 1.0); uv = vec2<f32>(0.0, 0.0); }
    }

    return VertexOutput(
        vec4<f32>(pos, 0.0, 1.0),
        uv,
    );
}

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    // 1. UV → NDC
    var ndc = uv * 2.0 - vec2<f32>(1.0);

    // Flip Y if needed
    ndc = vec2<f32>(ndc.x, -ndc.y);

    // 2. NDC → View space
    var clip = vec4<f32>(ndc, 1.0, 1.0);
    var view = camera.inv_proj * clip;
    view /= view.w;

    // 3. View → World
    let world_dir = normalize((camera.inv_view * vec4<f32>(view.xyz, 0.0)).xyz);

    // 4. Ray
    let ray_origin = camera.position;
    let ray_dir = world_dir;

    // 5. Transform to object space
    let local_origin = (model.inv_matrix * vec4<f32>(ray_origin, 1.0)).xyz;
    let local_dir = normalize((model.inv_matrix * vec4<f32>(ray_dir, 0.0)).xyz);

    // 6. Unit cube centered at origin
    let bounds_min = vec3<f32>(-0.5);
    let bounds_max = vec3<f32>(0.5);

    let dst = ray_box_intersection(params.bounds_min, params.bounds_max, local_origin, local_dir);
    let dst_through = max(0.0, dst.y - dst.x);

    let entry = local_origin + local_dir * dst.x;

    if dst_through == 0.0 {
        return vec4(0.0);
    }

    var density = 0.0;
    var light: vec3<f32> = vec3(0.0);

    let scattering = params.scattering;

    let sun_strength = 1.557;
    let sun_dir = vec3(sun_strength);

    const STEP: f32 = 2;
    for (var i: f32 = 0; i < dst_through; i += STEP) {
        let sample_pos = entry + local_dir * i;
        let sample = sample_density_map(sample_pos) * params.density_multiplier * STEP;
        density += sample;

        let density_along_sun_ray = calculate_along_ray(sample_pos, sun_dir, 5.0);
        let transmitted_sun_light = exp(-density_along_sun_ray * scattering);

        let in_light = transmitted_sun_light * sample * scattering;

        let view_ray_transmitted = exp(-density * scattering);

        light += in_light * view_ray_transmitted;
    }

    let c = light;

    return vec4<f32>(c, 1.0);
}

fn ray_box_intersection(box_min: vec3f, box_max: vec3f, ray_origin: vec3f, ray_dir: vec3f) -> vec2f {
    // 1. Calculate the inverse direction to avoid division and handle parallel rays
    let inv_dir = 1.0 / ray_dir;

    // 2. Calculate the distances to the min and max planes for each axis
    let t0 = (box_min - ray_origin) * inv_dir;
    let t1 = (box_max - ray_origin) * inv_dir;

    // 3. Find the near and far distances for each slab
    let tmin_v = min(t0, t1);
    let tmax_v = max(t0, t1);

    // 4. Find the largest near distance and smallest far distance
    let t_near = max(max(tmin_v.x, tmin_v.y), tmin_v.z);
    let t_far = min(min(tmax_v.x, tmax_v.y), tmax_v.z);

    // If t_near > t_far, the ray missed the box. 
    // You can handle this by returning a specific value (like -1.0)
    // or by checking the condition in your main code.
    return vec2f(t_near, t_far);

    // CASE 1: ray intersects box from outside (0 <= dstA <= dstB)
    // dstA is dst to nearest intersection, dstB dst to far intersection

    // CASE 2: ray intersects box from inside (dstA < 0 < dstB)
    // dstA is the dst to intersection behind the ray, dstB is dst to forward intersection

    // CASE 3: ray misses box (dstA > dstB)
}

fn calculate_along_ray(ray_origin: vec3<f32>, ray_dir: vec3<f32>, step: f32) -> f32 {
    let dst = ray_box_intersection(params.bounds_min, params.bounds_max, ray_origin, ray_dir);
    let dst_through = max(0.0, dst.y - dst.x);
    let inside = dst.x < 0.0 && 0.0 < dst.y;

    let start_pos = select(ray_origin + ray_dir * dst.x, ray_origin, inside);

    let factored = select(dst_through, dst.y, inside);

    var density = 0.0;

    for (var i: f32 = 0; i < factored; i += step) {
        let sample_pos = start_pos + ray_dir * i;
        let sample = sample_density_map(sample_pos);
        density += sample;
    }

    return density;
}
fn sample_density_map(pos: vec3<f32>) -> f32 {
    let texture_size = textureDimensions(density_map);

    let bounds_size = params.bounds_max - params.bounds_min;
    let normalized = (pos + bounds_size * 0.5) / bounds_size;

    let uvw: vec3<u32> = vec3<u32>(normalized * vec3<f32>(texture_size));

    // const epsilon = 0.0001;
    // let isEdge: bool = all(uvw >= vec3(1 - epsilon)) || all(uvw <= vec3(epsilon));
    //
    // if isEdge {
    //     return -1.0;
    // };

    let texel = textureLoad(density_map, uvw);
    return texel.r;
}
