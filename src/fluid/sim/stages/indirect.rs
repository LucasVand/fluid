use crate::renderer::utils::{BindGroupBuilder, BindGroupLayoutBuilder, ComputePipelineBuilder};
use eframe::wgpu::*;

pub struct IndirectStage {
    pub pipeline: ComputePipeline,
    pub bind_group: BindGroup,
}

impl IndirectStage {
    pub fn create(
        device: &Device,
        spatial_lookup_buffer: &Buffer,
        cell_ranges_buffer: &Buffer,
        indirect_buffer: &Buffer,
    ) -> Self {
        let bgl = BindGroupLayoutBuilder::new(device)
            .buffer(0, ShaderStages::COMPUTE, true)
            .buffer(1, ShaderStages::COMPUTE, false)
            .buffer(2, ShaderStages::COMPUTE, false)
            .build("Indirect Bind Group");

        let pipeline = ComputePipelineBuilder::new(device)
            .shader(
                include_str!("../../../shaders/calculate_indirect.wgsl"),
                "Indirect Shader",
            )
            .bind_group_layout(&[&bgl])
            .entry_point("main")
            .build("Indirect Pipeline");

        let bind_group = BindGroupBuilder::new(device, &bgl)
            .buffer(0, spatial_lookup_buffer)
            .buffer(1, cell_ranges_buffer)
            .buffer(2, indirect_buffer)
            .build("Indirect Bind Group");

        Self {
            pipeline,
            bind_group,
        }
    }

    pub fn execute(&self, pass: &mut ComputePass) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.dispatch_workgroups(1, 1, 1);
    }
}
