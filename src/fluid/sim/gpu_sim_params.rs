use crate::fluid::fluid_params::FluidParams;
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

impl From<&FluidParams> for GpuSimParams {
    fn from(value: &FluidParams) -> Self {
        let params = GpuSimParams {
            target_density: value.target_density,
            pressure_multiplier: value.pressure_multiplier,
            near_pressure_multiplier: value.near_pressure_multiplier,
            smoothing_radius: value.smoothing_radius,
            gravity: value.gravity,
            damping: 0.7,
            time_step: (1.0 / 120.0),
            particle_size: value.particle_size,
            viscosity_strength: value.viscosity_strength,
            _pad2: [0.0; 3],
            bounds_min: value.bounds.min,
            _pad0: 0.0,
            bounds_max: value.bounds.max,
            _pad1: 0.0,
        };

        params
    }
}
