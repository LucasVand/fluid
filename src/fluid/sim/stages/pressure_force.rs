use crate::renderer::utils::{BindGroupBuilder, BindGroupLayoutBuilder, ComputePipelineBuilder};
use eframe::wgpu::*;

pub struct PressureForceStage {
    pub pipeline: ComputePipeline,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl PressureForceStage {
    pub fn create(
        device: &Device,
        particles_buffer: &Buffer,
        params_buffer: &Buffer,
        spatial_lookup_buffer: &Buffer,
        start_indices_buffer: &Buffer,
    ) -> Self {
        let bind_group_layout = BindGroupLayoutBuilder::new(device)
            .buffer(0, ShaderStages::COMPUTE, false)
            .uniform(1, ShaderStages::COMPUTE)
            .buffer(2, ShaderStages::COMPUTE, true)
            .buffer(3, ShaderStages::COMPUTE, true)
            .build("Pressure Force Bind Group Layout");

        let pipeline = ComputePipelineBuilder::new(device)
            .bind_group_layout(&[&bind_group_layout])
            .shader(
                include_str!("../../../shaders/pressure_force.wgsl"),
                "Pressure Force Shader",
            )
            .entry_point("main")
            .build("Pressure Force Pipeline");

        let bind_group = BindGroupBuilder::new(device, &bind_group_layout)
            .buffer(0, particles_buffer)
            .buffer(1, params_buffer)
            .buffer(2, spatial_lookup_buffer)
            .buffer(3, start_indices_buffer)
            .build("Pressure Force Bind Group");

        PressureForceStage {
            pipeline,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn execute(&self, compute_pass: &mut ComputePass, particle_count: usize) {
        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);
        compute_pass.dispatch_workgroups(((particle_count as u32 + 63) / 64), 1, 1);
    }
}
