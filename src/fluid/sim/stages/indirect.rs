use crate::renderer::utils::{BindGroupBuilder, BindGroupLayoutBuilder, ComputePipelineBuilder};
use bytemuck::{Pod, Zeroable};
use eframe::wgpu::*;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
struct CellRange {
    start: u32,
    end: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
struct SpatialLookupEntry {
    cell_key: u32,
    particle_idx: u32,
}

pub struct IndirectStage {
    pub pipeline: ComputePipeline,
    pub bind_group: BindGroup,
    pub particle_count: usize,
}

impl IndirectStage {
    pub fn create(
        device: &Device,
        spatial_lookup_buffer: &Buffer,
        cell_ranges_buffer: &Buffer,
        indirect_buffer: &Buffer,
        start_indices_buffer: &Buffer,
        end_indices_buffer: &Buffer,
        particle_count: usize,
    ) -> Self {
        let bgl = BindGroupLayoutBuilder::new(device)
            .buffer(0, ShaderStages::COMPUTE, true)
            .buffer(1, ShaderStages::COMPUTE, false)
            .buffer(2, ShaderStages::COMPUTE, false)
            .buffer(3, ShaderStages::COMPUTE, true)
            .buffer(4, ShaderStages::COMPUTE, true)
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
            .buffer(3, start_indices_buffer)
            .buffer(4, end_indices_buffer)
            .build("Indirect Bind Group");

        Self {
            pipeline,
            bind_group,
            particle_count,
        }
    }

    pub fn execute(&self, pass: &mut ComputePass) {
        let workgroup_count = self.particle_count.div_ceil(256) as u32;
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.dispatch_workgroups(workgroup_count, 1, 1);
    }
}
