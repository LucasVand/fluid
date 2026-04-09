use crate::fluid::fluid_params::FluidParams;
use crate::fluid::model_context::FluidModelContext;
use crate::fluid::particle::{GpuParticle, Particle};
use crate::fluid::sim::gpu_sim_params::GpuSimParams;
use crate::fluid::sim::stages::density::DensityStage;
use crate::fluid::sim::stages::indirect::IndirectStage;
use crate::fluid::sim::stages::predicted_position::PredictedPositionStage;
use crate::fluid::sim::stages::pressure_force::PressureForceStage;
use crate::fluid::sim::stages::spatial_map::SpatialMapStage;
use crate::fluid::sim::stages::update_position::UpdatePositionStage;
use crate::renderer::renderable::{RenderCC, RenderContext};
use crate::renderer::utils::{BufferBuilder, CommandEncoderBuilder};
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
    pub end_indices_buffer: Buffer,
    pub indirect_buffer: Buffer,
    pub particle_count: usize,
    pub predicted_stage: PredictedPositionStage,
    pub density_stage: DensityStage,
    pub pressure_force_stage: PressureForceStage,
    pub update_position_stage: UpdatePositionStage,
    pub spatial_map_stage: SpatialMapStage,
    pub indirect_stage: IndirectStage,
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

        let spatial_lookup_size = (std::mem::size_of::<(u32, u32)>() * particle_count) as u64;

        let spatial_lookup_buffer = BufferBuilder::new(device)
            .size(spatial_lookup_size)
            .usages(BufferUsages::STORAGE | BufferUsages::COPY_DST)
            .build("Spatial Lookup Buffer");

        let indices_size = (std::mem::size_of::<u32>() * particle_count) as u64;
        let start_indices_buffer = BufferBuilder::new(device)
            .size(indices_size)
            .usages(BufferUsages::STORAGE | BufferUsages::COPY_DST)
            .build("Start Indices Buffer");

        let end_indices_buffer = BufferBuilder::new(device)
            .size(indices_size)
            .usages(BufferUsages::STORAGE | BufferUsages::COPY_DST)
            .build("End Indices Buffer");

        // TODO: Calculate size based on the dimesions of the bounding box
        let cell_ranges_size = (std::mem::size_of::<(u32, u32)>() * particle_count) as u64;
        let cell_ranges_buffer = BufferBuilder::new(device)
            .size(cell_ranges_size)
            .usages(BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC)
            .build("Cell Ranges Buffer");

        let indirect_buffer = BufferBuilder::new(device)
            .size(4 * 4)
            .usages(BufferUsages::STORAGE | BufferUsages::INDIRECT | BufferUsages::COPY_SRC)
            .build("Indirect Buffer");

        let predicted_stage =
            PredictedPositionStage::create(device, &mcc.particles_buf, &params_buffer);

        let spatial_map_stage = SpatialMapStage::create(
            device,
            &mcc.particles_buf,
            &params_buffer,
            &spatial_lookup_buffer,
            &start_indices_buffer,
            &end_indices_buffer,
        );

        let indirect_stage = IndirectStage::create(
            device,
            &spatial_lookup_buffer,
            &cell_ranges_buffer,
            &indirect_buffer,
            particle_count,
        );

        let density_stage = DensityStage::create(
            device,
            &mcc.particles_buf,
            &params_buffer,
            &spatial_lookup_buffer,
            &start_indices_buffer,
            &end_indices_buffer,
            &cell_ranges_buffer,
        );

        let pressure_force_stage = PressureForceStage::create(
            device,
            &mcc.particles_buf,
            &params_buffer,
            &spatial_lookup_buffer,
            &start_indices_buffer,
            &end_indices_buffer,
            &cell_ranges_buffer,
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
            end_indices_buffer,
            particle_count,
            predicted_stage,
            density_stage,
            pressure_force_stage,
            update_position_stage,
            spatial_map_stage,
            indirect_stage,
            indirect_buffer,
        }
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

    pub fn update(&mut self, _rc: &RenderContext, _mcc: &mut FluidModelContext) {
        let mut encoder = CommandEncoderBuilder::new(&self.device)
            .label("Fluid Simulation")
            .build();

        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Predicted Position Pass"),
                timestamp_writes: None,
            });
            self.predicted_stage
                .execute(&mut compute_pass, self.particle_count);
        }

        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Spatial Map Pass"),
                timestamp_writes: None,
            });
            self.spatial_map_stage
                .execute(&mut compute_pass, self.particle_count);
        }

        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Indirect Pass"),
                timestamp_writes: None,
            });
            self.indirect_stage.execute(&mut compute_pass);
        }

        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Density Pass"),
                timestamp_writes: None,
            });
            self.density_stage
                .execute(&mut compute_pass, &self.indirect_buffer);
        }

        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Pressure Force Pass"),
                timestamp_writes: None,
            });
            self.pressure_force_stage
                .execute(&mut compute_pass, &self.indirect_buffer);
        }

        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Update Position Pass"),
                timestamp_writes: None,
            });
            self.update_position_stage
                .execute(&mut compute_pass, self.particle_count);
        }

        self.queue.submit(Some(encoder.finish()));
        let _ = self.device.poll(PollType::wait_indefinitely());

        // self.indirect_stage.debug_print_ranges(&self.queue);
    }
}
