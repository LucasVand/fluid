@group(0) @binding(0) var<uniform> model: mat4x4<f32>;
@group(1) @binding(0) var<uniform> camera: Camera;

struct Camera {
    matrix: mat4x4<f32>,
    position: vec3<f32>,
    _pad: f32,
}

@vertex
fn vs_main(@location(0) pos: vec3<f32>) -> @builtin(position) vec4<f32> {
    let world_pos = model * vec4(pos, 1.0);
    return camera.matrix * world_pos;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4(1.0, 1.0, 1.0, 1.0);
}
