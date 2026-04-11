use crate::renderer::utils::{BindGroupBuilder, BindGroupLayoutBuilder, ComputePipelineBuilder};
use eframe::wgpu::*;

pub struct DensityFieldStage {
    pub pipeline: ComputePipeline,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
    texture_size: Extent3d,
}

impl DensityFieldStage {
    pub fn create(
        device: &Device,
        particles_buffer: &Buffer,
        params_buffer: &Buffer,
        spatial_lookup_buffer: &Buffer,
        start_indices_buffer: &Buffer,
        end_indices_buffer: &Buffer,
        density_texture: &Texture,
    ) -> Self {
        let texture_size = density_texture.size();

        let texture_view = density_texture.create_view(&TextureViewDescriptor {
            dimension: Some(TextureViewDimension::D3),
            ..Default::default()
        });

        let bind_group_layout = BindGroupLayoutBuilder::new(device)
            .buffer(0, ShaderStages::COMPUTE, true) // particles buffer
            .uniform(1, ShaderStages::COMPUTE) // params
            .buffer(2, ShaderStages::COMPUTE, true) // spatial_lookup
            .buffer(3, ShaderStages::COMPUTE, true) // start_indices
            .buffer(4, ShaderStages::COMPUTE, true) // end_indices
            .storage_texture(5, ShaderStages::COMPUTE, TextureFormat::R32Float)
            .build("Density Field Bind Group Layout");

        let pipeline = ComputePipelineBuilder::new(device)
            .bind_group_layout(&[&bind_group_layout])
            .shader(
                include_str!("./shaders/density_field.wgsl"),
                "Density Field Shader",
            )
            .entry_point("main")
            .build("Density Field Pipeline");

        let bind_group = BindGroupBuilder::new(device, &bind_group_layout)
            .buffer(0, particles_buffer)
            .buffer(1, params_buffer)
            .buffer(2, spatial_lookup_buffer)
            .buffer(3, start_indices_buffer)
            .buffer(4, end_indices_buffer)
            .texture(5, &texture_view)
            .build("Density Field Bind Group");

        DensityFieldStage {
            pipeline,
            bind_group_layout,
            bind_group,
            texture_size,
        }
    }

    pub fn execute(&self, compute_pass: &mut ComputePass) {
        // Dispatch with 8×8×8 workgroups to cover all samples
        let workgroups_x = self.texture_size.width.div_ceil(8);
        let workgroups_y = self.texture_size.height.div_ceil(8);
        let workgroups_z = self.texture_size.depth_or_array_layers.div_ceil(4);

        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);
        compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, workgroups_z);
    }
}
