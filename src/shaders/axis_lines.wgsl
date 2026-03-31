@group(0) @binding(0) var<uniform> model: mat4x4<f32>;
@group(1) @binding(0) var<uniform> camera: Camera;

struct Camera {
    matrix: mat4x4<f32>,
    position: vec3<f32>,
    _pad: f32,
}

@vertex
fn vs_main(
    @location(0) pos: vec3<f32>,
    @location(1) color: vec3<f32>,
) -> VertexOutput {
    let world_pos = model * vec4(pos, 1.0);
    let clip_pos = camera.matrix * world_pos;
    return VertexOutput(clip_pos, color);
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(in.color, 1.0);
}
