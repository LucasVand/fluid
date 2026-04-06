use std::mem;

use crate::renderer::utils::{
    BindGroupBuilder, BindGroupLayoutBuilder, BufferBuilder, ComputePipelineBuilder,
};
use bytemuck::{Pod, Zeroable, bytes_of};
use eframe::wgpu::*;

pub struct SpatialMapStage {
    pub pipeline: ComputePipeline,
    pub bind_group: BindGroup,

    pub sort_pipeline: ComputePipeline,
    pub sort_bind_group: BindGroup,
    pub sort_uniform: Buffer,

    pub finalize_pipeline: ComputePipeline,
    pub clear_pipeline: ComputePipeline,
    pub finalize_bind_group: BindGroup,
}

impl SpatialMapStage {
    pub fn create(
        device: &Device,
        particles_buffer: &Buffer,
        params_buffer: &Buffer,
        spatial_lookup_buffer: &Buffer,
        start_indices_buffer: &Buffer,
        end_indices_buffer: &Buffer,
    ) -> Self {
        let bind_group_layout = BindGroupLayoutBuilder::new(device)
            .buffer(0, ShaderStages::COMPUTE, false)
            .uniform(1, ShaderStages::COMPUTE)
            .buffer(2, ShaderStages::COMPUTE, false)
            .build("Spatial Map Bind Group Layout");

        let pipeline = ComputePipelineBuilder::new(device)
            .bind_group_layout(&[&bind_group_layout])
            .shader(
                include_str!("../../../shaders/spatial_map_insert.wgsl"),
                "Spatial Map Shader",
            )
            .entry_point("main")
            .build("Spatial Map Insert Pipeline");

        let bind_group = BindGroupBuilder::new(device, &bind_group_layout)
            .buffer(0, particles_buffer)
            .buffer(1, params_buffer)
            .buffer(2, spatial_lookup_buffer)
            .build("Spatial Map Bind Group");

        let (sort_pipeline, sort_bind_group, sort_uniform) =
            Self::sort_pipeline(device, spatial_lookup_buffer, start_indices_buffer);

        let (clear_pipeline, finalize_pipeline, finalize_bind_group) = Self::finalize_pipeline(
            device,
            spatial_lookup_buffer,
            start_indices_buffer,
            end_indices_buffer,
        );

        SpatialMapStage {
            pipeline,
            bind_group,
            sort_bind_group,
            sort_pipeline,
            sort_uniform,
            finalize_pipeline,
            clear_pipeline,
            finalize_bind_group,
        }
    }

    pub fn execute(&self, pass: &mut ComputePass, particle_count: usize) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.dispatch_workgroups(((particle_count as u32 + 63) / 64), 1, 1);

        let n = particle_count;

        // sort pass
        pass.set_bind_group(0, &self.sort_bind_group, &[]);
        pass.set_pipeline(&self.sort_pipeline);

        let mut seen = std::collections::HashSet::new();
        for k in (2..=n).step_by(2).map(|x| x.next_power_of_two()) {
            if seen.contains(&k) {
                continue;
            }
            seen.insert(k);
            let mut j = k / 2;
            while j > 0 {
                pass.set_push_constants(
                    0,
                    bytes_of(&SortParams {
                        j: j as u32,
                        k: k as u32,
                        _pad: [0.0; 2],
                    }),
                );
                pass.dispatch_workgroups(((particle_count as u32 + 63) / 64), 1, 1);

                j /= 2;
            }
        }
        pass.set_bind_group(0, &self.finalize_bind_group, &[]);
        pass.set_pipeline(&self.clear_pipeline);

        pass.dispatch_workgroups(((particle_count as u32 + 63) / 64), 1, 1);

        pass.set_pipeline(&self.finalize_pipeline);
        pass.dispatch_workgroups(((particle_count as u32 + 63) / 64), 1, 1);
    }

    fn sort_pipeline(
        device: &Device,
        spatial_lookup_buffer: &Buffer,
        start_indices_buffer: &Buffer,
    ) -> (ComputePipeline, BindGroup, Buffer) {
        let size = mem::size_of::<SortParams>() as u64;

        let uniform = BufferBuilder::new(device)
            .usages(BufferUsages::UNIFORM | BufferUsages::COPY_DST)
            .size(size)
            .build("Sort Uniform Buffer");

        let bind_group_layout = BindGroupLayoutBuilder::new(device)
            .buffer(0, ShaderStages::COMPUTE, false)
            .buffer(1, ShaderStages::COMPUTE, false)
            .uniform(2, ShaderStages::COMPUTE)
            .build("Spatial Map Bind Group Layout");

        let pipeline = ComputePipelineBuilder::new(device)
            .bind_group_layout(&[&bind_group_layout])
            .shader(
                include_str!("../../../shaders/spatial_map_sort.wgsl"),
                "Spatial Map Shader",
            )
            .push_constant_range(ShaderStages::COMPUTE, 0..16)
            .entry_point("main")
            .build("Spatial Map Pipeline");

        let bind_group = BindGroupBuilder::new(device, &bind_group_layout)
            .buffer(0, spatial_lookup_buffer)
            .buffer(1, start_indices_buffer)
            .buffer(2, &uniform)
            .build("Spatial Map Bind Group");

        (pipeline, bind_group, uniform)
    }
    fn finalize_pipeline(
        device: &Device,
        spatial_lookup_buffer: &Buffer,
        start_indices_buffer: &Buffer,
        end_indices_buffer: &Buffer,
    ) -> (ComputePipeline, ComputePipeline, BindGroup) {
        let bind_group_layout = BindGroupLayoutBuilder::new(device)
            .buffer(0, ShaderStages::COMPUTE, false)
            .buffer(1, ShaderStages::COMPUTE, false)
            .buffer(2, ShaderStages::COMPUTE, false)
            .build("Spatial Map Finalisze Bind Group Layout");

        let pipeline_final = ComputePipelineBuilder::new(device)
            .bind_group_layout(&[&bind_group_layout])
            .shader(
                include_str!("../../../shaders/spatial_map_finalize.wgsl"),
                "Spatial Map Finalize Shader",
            )
            .entry_point("main")
            .build("Spatial Map Finalize Pipeline");

        let pipeline_clear = ComputePipelineBuilder::new(device)
            .bind_group_layout(&[&bind_group_layout])
            .shader(
                include_str!("../../../shaders/spatial_map_finalize.wgsl"),
                "Spatial Map Shader",
            )
            .entry_point("main_clear")
            .build("Spatial Map Clear Pipeline");

        let bind_group = BindGroupBuilder::new(device, &bind_group_layout)
            .buffer(0, spatial_lookup_buffer)
            .buffer(1, start_indices_buffer)
            .buffer(2, end_indices_buffer)
            .build("Spatial Map Finalize Bind Group");

        (pipeline_clear, pipeline_final, bind_group)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SortParams {
    j: u32,
    k: u32,
    _pad: [f32; 2],
}
