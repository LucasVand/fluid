use crate::renderer::utils::{BindGroupBuilder, BindGroupLayoutBuilder, ComputePipelineBuilder};
use eframe::wgpu::*;

pub struct PredictedPositionStage {
    pub pipeline: ComputePipeline,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl PredictedPositionStage {
    pub fn create(device: &Device, particles_buffer: &Buffer, params_buffer: &Buffer) -> Self {
        let bind_group_layout = BindGroupLayoutBuilder::new(device)
            .buffer(0, ShaderStages::COMPUTE, false)
            .uniform(1, ShaderStages::COMPUTE)
            .build("Predicted Position Bind Group Layout");

        let pipeline = ComputePipelineBuilder::new(device)
            .bind_group_layout(&[&bind_group_layout])
            .shader(
                include_str!("../../../shaders/predicted.wgsl"),
                "Predicted Position Shader",
            )
            .entry_point("main")
            .build("Predicted Position Pipeline");

        let bind_group = BindGroupBuilder::new(device, &bind_group_layout)
            .buffer(0, particles_buffer)
            .buffer(1, params_buffer)
            .build("Predicted Position Bind Group");

        PredictedPositionStage {
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
