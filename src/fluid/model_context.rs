use eframe::wgpu::Buffer;

use crate::{
    fluid::{fluid_params::FluidParams, particle::Particle},
    renderer::utils::box3d::Box3d,
};

pub struct FluidModelContext {
    pub particles: Vec<Particle>,
    pub params: FluidParams,

    pub bounds: Box3d,

    pub model_buf: Buffer,
    pub particles_buf: Buffer,
}
