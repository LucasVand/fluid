use glam::Vec3;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuSimParams {
    pub target_density: f32,
    pub pressure_multiplier: f32,
    pub near_pressure_multiplier: f32,
    pub smoothing_radius: f32,

    pub gravity: f32,
    pub damping: f32,
    pub time_step: f32,
    pub particle_size: f32,

    pub viscosity_strength: f32,
    pub _pad2: [f32; 3],

    pub bounds_min: Vec3,
    pub _pad0: f32,

    pub bounds_max: Vec3,
    pub _pad1: f32,
}
