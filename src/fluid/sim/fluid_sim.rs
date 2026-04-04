use std::mem;

use crate::fluid::fluid_params::FluidParams;
use crate::fluid::model_context::FluidModelContext;
use crate::fluid::particle::{GpuParticle, Particle};
use crate::fluid::sim::gpu_sim_params::GpuSimParams;
use crate::fluid::sim::stages::density::DensityStage;
use crate::fluid::sim::stages::predicted_position::PredictedPositionStage;
use crate::fluid::sim::stages::pressure_force::PressureForceStage;
use crate::fluid::sim::stages::update_position::UpdatePositionStage;
use crate::renderer::renderable::{RenderCC, RenderContext};
use crate::renderer::utils::BufferBuilder;
use crate::spatial_map::SpatialMap;
use eframe::wgpu::wgt::PollType;
use eframe::wgpu::*;

pub struct FluidSim {
    pub device: Device,
    pub queue: Queue,
    pub particles_buffer: Buffer,
    pub particles_staging: Buffer,
    pub params_buffer: Buffer,
    pub spatial_lookup_buffer: Buffer,
    pub start_indices_buffer: Buffer,
    pub particle_count: usize,
    pub predicted_stage: PredictedPositionStage,
    pub density_stage: DensityStage,
    pub pressure_force_stage: PressureForceStage,
    pub update_position_stage: UpdatePositionStage,
    pub spatial_map: SpatialMap,
}

impl FluidSim {
    pub fn new(rcc: &RenderCC, mcc: &FluidModelContext) -> Self {
        let device = rcc.device;
        let particle_count = mcc.particles.len();

        let particles_staging = BufferBuilder::new(device)
            .size((std::mem::size_of::<GpuParticle>() * particle_count) as u64)
            .usages(BufferUsages::COPY_DST | BufferUsages::MAP_READ)
            .build("Particles Staging Buffer");

        let params: GpuSimParams = (&mcc.params).into();
        let params_buffer = BufferBuilder::new(device)
            .contents(&params)
            .usages(BufferUsages::UNIFORM | BufferUsages::COPY_DST)
            .build("Params Buffer");

        let spatial_lookup_buffer = BufferBuilder::new(device)
            .size((std::mem::size_of::<(u32, u32)>() * particle_count) as u64)
            .usages(BufferUsages::STORAGE | BufferUsages::COPY_DST)
            .build("Spatial Lookup Buffer");

        let start_indices_buffer = BufferBuilder::new(device)
            .size((std::mem::size_of::<u32>() * particle_count) as u64)
            .usages(BufferUsages::STORAGE | BufferUsages::COPY_DST)
            .build("Start Indices Buffer");

        let predicted_stage =
            PredictedPositionStage::create(device, &mcc.particles_buf, &params_buffer);
        let density_stage = DensityStage::create(
            device,
            &mcc.particles_buf,
            &params_buffer,
            &spatial_lookup_buffer,
            &start_indices_buffer,
        );
        let pressure_force_stage = PressureForceStage::create(
            device,
            &mcc.particles_buf,
            &params_buffer,
            &spatial_lookup_buffer,
            &start_indices_buffer,
        );
        let update_position_stage =
            UpdatePositionStage::create(device, &mcc.particles_buf, &params_buffer);

        FluidSim {
            device: device.clone(),
            queue: rcc.queue.clone(),
            particles_buffer: mcc.particles_buf.clone(),
            particles_staging,
            params_buffer,
            spatial_lookup_buffer,
            start_indices_buffer,
            particle_count,
            predicted_stage,
            density_stage,
            pressure_force_stage,
            update_position_stage,
            spatial_map: SpatialMap::new(mcc.params.smoothing_radius, mcc.particles.len()),
        }
    }

    fn from_cpu_particles(particles: &[Particle]) -> Vec<GpuParticle> {
        particles.iter().map(GpuParticle::from).collect()
    }

    fn to_cpu_particles(gpu_particles: &[GpuParticle]) -> Vec<Particle> {
        gpu_particles.iter().map(Particle::from).collect()
    }

    pub fn update_params(&self, params: &FluidParams) {
        let params: GpuSimParams = params.into();

        self.queue
            .write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));
    }

    pub fn upload_particles(&self, particles: &[Particle]) {
        let gpu_particles: Vec<GpuParticle> = particles.iter().map(GpuParticle::from).collect();

        self.queue.write_buffer(
            &self.particles_buffer,
            0,
            bytemuck::cast_slice(&gpu_particles),
        );
    }

    fn upload_spatial_map(&self, spatial_map: &SpatialMap) {
        let lookup_u32s: Vec<u32> = spatial_map
            .spacial_lookup
            .iter()
            .flat_map(|(a, b)| vec![*a as u32, *b as u32])
            .collect();

        self.queue.write_buffer(
            &self.spatial_lookup_buffer,
            0,
            bytemuck::cast_slice(&lookup_u32s),
        );

        let start_indices_u32: Vec<u32> = spatial_map
            .start_indices
            .iter()
            .map(|&idx| {
                if idx == usize::MAX {
                    u32::MAX
                } else {
                    idx as u32
                }
            })
            .collect();

        self.queue.write_buffer(
            &self.start_indices_buffer,
            0,
            bytemuck::cast_slice(&start_indices_u32),
        );
    }

    fn download_particles(&self) -> Vec<GpuParticle> {
        let mut command_encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Particles Readback Encoder"),
            });

        command_encoder.copy_buffer_to_buffer(
            &self.particles_buffer,
            0,
            &self.particles_staging,
            0,
            (std::mem::size_of::<GpuParticle>() * self.particle_count) as u64,
        );

        self.queue.submit(std::iter::once(command_encoder.finish()));

        let staging_slice = self.particles_staging.slice(..);
        staging_slice.map_async(MapMode::Read, |_| {});

        let _ = self.device.poll(PollType::wait_indefinitely());

        let data = staging_slice.get_mapped_range();
        let particles: Vec<GpuParticle> = bytemuck::cast_slice(&data).to_vec();

        drop(data);
        self.particles_staging.unmap();

        particles
    }

    pub fn update(&mut self, rc: &RenderContext, mcc: &mut FluidModelContext) {
        self.upload_particles(&mcc.particles);

        let mut command_encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Particle Update Encoder"),
            });

        {
            let mut compute_pass = command_encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Predicted Position Pass"),
                timestamp_writes: None,
            });

            self.predicted_stage
                .execute(&mut compute_pass, self.particle_count);
        }

        let par: Vec<GpuParticle> = self.download_particles();

        par.iter().enumerate().for_each(|(i, p)| {
            self.spatial_map.insert(i, p.position);
        });
        self.spatial_map.finalize();
        self.upload_spatial_map(&self.spatial_map);

        {
            let mut compute_pass = command_encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Density Pass"),
                timestamp_writes: None,
            });

            self.density_stage
                .execute(&mut compute_pass, self.particle_count);
        }

        {
            let mut compute_pass = command_encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Pressure Force Pass"),
                timestamp_writes: None,
            });

            self.pressure_force_stage
                .execute(&mut compute_pass, self.particle_count);
        }

        {
            let mut compute_pass = command_encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Update Position Pass"),
                timestamp_writes: None,
            });

            self.update_position_stage
                .execute(&mut compute_pass, self.particle_count);
        }

        self.queue.submit(Some(command_encoder.finish()));

        let updated_gpu = self.download_particles();
        let cpu_particles = Self::to_cpu_particles(&updated_gpu);

        mcc.particles = cpu_particles;
    }
}
