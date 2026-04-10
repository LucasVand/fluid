use eframe::wgpu::{Buffer, BufferUsages};
use glam::Vec3;

use crate::{
    fluid::{
        fluid::Fluid,
        fluid_params::FluidParams,
        fluid_spawner::create_box,
        particle::{GpuParticle, Particle},
    },
    renderer::{
        renderable::RenderCC,
        utils::{BufferBuilder, box3d::Box3d},
    },
};

pub struct FluidModelContext {
    pub particles: Vec<Particle>,
    pub params: FluidParams,

    pub bounds: Box3d,

    pub model_buf: Buffer,
    pub particles_buf: Buffer,
}

impl FluidModelContext {
    pub fn new(rcc: &RenderCC) -> Self {
        let size = 80.0;
        let bounds = Box3d::from_center(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(size * 4.0, size * 1.5, size),
        );

        let model_mat = Fluid::model_matrix(Vec3::ZERO, Vec3::ZERO, 0.1);

        let bytes: &[u8] = &bytemuck::cast_slice(&model_mat);

        let model_buf = BufferBuilder::new(rcc.device)
            .contents_slice(bytes)
            .usages(BufferUsages::UNIFORM | BufferUsages::COPY_SRC)
            .build("Model Buf");

        let particles: Vec<Particle> = create_box(2_usize.pow(16), bounds);

        let gpu_particles: Vec<GpuParticle> = particles.iter().map(|p| p.into()).collect();

        let particles_buf = BufferBuilder::new(rcc.device)
            .contents_slice(&bytemuck::cast_slice(&gpu_particles))
            .usages(BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST)
            .build("Particles Buffer");

        FluidModelContext {
            particles: particles,
            params: FluidParams {
                target_density: 0.10,
                pressure_multiplier: 7000.0,
                near_pressure_multiplier: 10.0,
                smoothing_radius: 20.0,
                gravity: 2050.0,
                damping: 0.95,
                time_step: 1.0 / 60.0,
                particle_size: 2.0,
                viscosity_strength: 0.3,
                color_multiplier: 0.001,
                color_offset: 0.60,
                bounds: bounds,
                is_running: false,
            },
            bounds: bounds,
            model_buf: model_buf,
            particles_buf: particles_buf,
        }
    }
}
