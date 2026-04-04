use std::mem;

use crate::fluid_sim::Particle;
use crate::renderer::utils::BufferBuilder;
use crate::sim::{
    DensityStage, GpuParticle, GpuSimParams, PredictedPositionStage, PressureForceStage,
    UpdatePositionStage,
};
use crate::spatial_map::SpatialMap;
use eframe::CreationContext;
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
}

impl FluidSim {
    pub fn new(
        cc: &CreationContext<'_>,
        particles: &[Particle],
        target_density: f32,
        pressure_multiplier: f32,
        near_pressure_multiplier: f32,
        smoothing_radius: f32,
        gravity: f32,
        damping: f32,
        time_step: f32,
        bounds_min: glam::Vec3,
        bounds_max: glam::Vec3,
        particle_size: f32,
        viscosity_strength: f32,
    ) -> Self {
        let state = cc.wgpu_render_state.as_ref().unwrap();
        let device = &state.device;
        let particle_count = particles.len();

        let particles_buffer = BufferBuilder::new(device)
            .contents_slice(bytemuck::cast_slice(&Self::from_cpu_particles(particles)))
            .usages(BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST)
            .build("Particles Buffer");

        let particles_staging = BufferBuilder::new(device)
            .size((std::mem::size_of::<GpuParticle>() * particle_count) as u64)
            .usages(BufferUsages::COPY_DST | BufferUsages::MAP_READ)
            .build("Particles Staging Buffer");

        let params = GpuSimParams {
            target_density,
            pressure_multiplier,
            near_pressure_multiplier,
            smoothing_radius,
            gravity,
            damping,
            time_step,
            particle_size,
            viscosity_strength,
            _pad2: [0.0; 3],
            bounds_min,
            _pad0: 0.0,
            bounds_max,
            _pad1: 0.0,
        };
        let params_size = mem::size_of::<GpuSimParams>();
        println!("{}", params_size);

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
            PredictedPositionStage::create(device, &particles_buffer, &params_buffer);
        let density_stage = DensityStage::create(
            device,
            &particles_buffer,
            &params_buffer,
            &spatial_lookup_buffer,
            &start_indices_buffer,
        );
        let pressure_force_stage = PressureForceStage::create(
            device,
            &particles_buffer,
            &params_buffer,
            &spatial_lookup_buffer,
            &start_indices_buffer,
        );
        let update_position_stage =
            UpdatePositionStage::create(device, &particles_buffer, &params_buffer);

        FluidSim {
            device: state.device.clone(),
            queue: state.queue.clone(),
            particles_buffer,
            particles_staging,
            params_buffer,
            spatial_lookup_buffer,
            start_indices_buffer,
            particle_count,
            predicted_stage,
            density_stage,
            pressure_force_stage,
            update_position_stage,
        }
    }

    pub fn from_cpu_particles(particles: &[Particle]) -> Vec<GpuParticle> {
        particles.iter().map(GpuParticle::from).collect()
    }

    pub fn to_cpu_particles(gpu_particles: &[GpuParticle]) -> Vec<Particle> {
        gpu_particles.iter().map(Particle::from).collect()
    }

    pub fn update_params_from_sim(&self, sim: &crate::fluid_sim::FluidSim) {
        let params = GpuSimParams {
            target_density: sim.target_density,
            pressure_multiplier: sim.pressure_multiplier,
            near_pressure_multiplier: sim.near_pressure_multiplier,
            smoothing_radius: sim.smoothing_radius,
            gravity: sim.gravity,
            damping: 0.7,
            time_step: (1.0 / 120.0),
            particle_size: sim.particle_size,
            viscosity_strength: sim.viscosity_strength,
            _pad2: [0.0; 3],
            bounds_min: sim.bounds.min,
            _pad0: 0.0,
            bounds_max: sim.bounds.max,
            _pad1: 0.0,
        };

        self.queue
            .write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));
    }

    pub fn upload_particles(&self, gpu_particles: &[GpuParticle]) {
        self.queue.write_buffer(
            &self.particles_buffer,
            0,
            bytemuck::cast_slice(gpu_particles),
        );
    }

    pub fn upload_spatial_map(&self, spatial_map: &SpatialMap) {
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

    pub fn download_particles(&self) -> Vec<GpuParticle> {
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

    pub fn update(&self, particles: &[Particle], spatial_map: &mut SpatialMap) -> Vec<Particle> {
        let gpu_particles = Self::from_cpu_particles(particles);
        self.upload_particles(&gpu_particles);

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

        let par = self.download_particles();
        par.iter()
            .map(|p| {
                let particle: Particle = p.into();
                particle
            })
            .enumerate()
            .for_each(|(i, p)| {
                spatial_map.insert(i, p.pos);
            });
        spatial_map.finalize();
        self.upload_spatial_map(spatial_map);

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

        self.queue.submit(std::iter::once(command_encoder.finish()));

        let updated_gpu = self.download_particles();
        Self::to_cpu_particles(&updated_gpu)
    }
}
