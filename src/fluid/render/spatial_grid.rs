use eframe::wgpu::{
    BindGroup, BufferUsages, PrimitiveTopology, RenderPass, ShaderStages, TextureFormat,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode,
};
use glam::Vec3;

use crate::{
    fluid::model_context::FluidModelContext,
    renderer::{
        renderable::RenderCC,
        utils::{
            bind_group_builder::BindGroupBuilder,
            bind_group_layout_builder::BindGroupLayoutBuilder, generic_shared_buffer::SharedBuffer,
            render_pipeline_builder::RenderPipelineBuilder,
        },
    },
};

pub struct SpatialGrid {
    vertex_index: u64,
    index_index: u64,
    index_count: u32,
    pipeline: eframe::wgpu::RenderPipeline,
    shared_buffer: SharedBuffer,
    bind_group: BindGroup,
}

impl SpatialGrid {
    pub fn new(rcc: &RenderCC, mcc: &FluidModelContext, cell_size: f32, radius: f32) -> Self {
        let device = rcc.device;
        let queue = rcc.queue;

        let mut shared_buffer = SharedBuffer::with_usages(
            device,
            2_u64.pow(18),
            BufferUsages::VERTEX | BufferUsages::INDEX | BufferUsages::COPY_DST,
        );

        // Generate grid cells within radius of origin
        let mut vertices = Vec::new();
        let mut indices: Vec<u16> = Vec::new();

        let grid_range = (radius / cell_size).ceil() as i32;

        for x in -grid_range..=grid_range {
            for y in -grid_range..=grid_range {
                for z in -grid_range..=grid_range {
                    let dist = ((x * x + y * y + z * z) as f32).sqrt() * cell_size;
                    if dist > radius {
                        continue;
                    }

                    let min = Vec3::new(
                        x as f32 * cell_size,
                        y as f32 * cell_size,
                        z as f32 * cell_size,
                    );
                    let max = min + Vec3::splat(cell_size);

                    let base_idx = vertices.len() as u16;

                    // Add cube vertices
                    vertices.push(min);
                    vertices.push(Vec3::new(max.x, min.y, min.z));
                    vertices.push(Vec3::new(max.x, max.y, min.z));
                    vertices.push(Vec3::new(min.x, max.y, min.z));
                    vertices.push(Vec3::new(min.x, min.y, max.z));
                    vertices.push(Vec3::new(max.x, min.y, max.z));
                    vertices.push(Vec3::new(max.x, max.y, max.z));
                    vertices.push(Vec3::new(min.x, max.y, max.z));

                    // Add cube edges (line list)
                    let edge_indices = vec![
                        0, 1, 1, 2, 2, 3, 3, 0, 4, 5, 5, 6, 6, 7, 7, 4, 0, 4, 1, 5, 2, 6, 3, 7,
                    ];

                    for idx in edge_indices {
                        indices.push(base_idx + idx);
                    }
                }
            }
        }

        let index_count = indices.len() as u32;

        let vertex_index = shared_buffer.allocate(
            queue,
            bytemuck::cast_slice(&vertices),
            "Spatial Grid Vertices",
        );
        let index_index = shared_buffer.allocate(
            queue,
            bytemuck::cast_slice(&indices),
            "Spatial Grid Indices",
        );

        let bgl = BindGroupLayoutBuilder::new(device)
            .uniform(0, ShaderStages::VERTEX_FRAGMENT)
            .uniform(1, ShaderStages::VERTEX_FRAGMENT)
            .build("Spatial Grid Bindgroup Layout");

        let bind_group = BindGroupBuilder::new(device, &bgl)
            .buffer_slice(0, rcc.camera_buf)
            .buffer(1, &mcc.model_buf)
            .build("Spatial Grid Bindgroup");

        let pipeline = RenderPipelineBuilder::new(device)
            .shader(
                include_str!("../../shaders/spatial_grid.wgsl"),
                "Spatial Grid Shader",
            )
            .primitive(PrimitiveTopology::LineList)
            .bind_group_layout(&[&bgl])
            .vertex_entry("vs_main")
            .vertex_buffers(vec![VertexBufferLayout {
                array_stride: 4 * 3,
                step_mode: VertexStepMode::Vertex,
                attributes: &[VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                }],
            }])
            .depth(TextureFormat::Depth32Float)
            .fragment_entry("fs_main")
            .color_format(rcc.texture_format)
            .build("Spatial Grid Pipeline");

        SpatialGrid {
            vertex_index,
            index_index,
            index_count,
            pipeline,
            shared_buffer,
            bind_group,
        }
    }

    pub fn render(&self, render_pass: &mut RenderPass) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.shared_buffer.get_slice(self.vertex_index));
        render_pass.set_index_buffer(
            self.shared_buffer.get_slice(self.index_index),
            eframe::wgpu::IndexFormat::Uint16,
        );
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}
