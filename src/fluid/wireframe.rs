use std::mem;

use eframe::wgpu::{
    BindGroup, BindGroupLayout, BufferUsages, Device, PrimitiveTopology, Queue, RenderPass,
    ShaderStages, TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode,
};
use glam::Vec3;

use crate::renderer::utils::{
    bind_group_builder::BindGroupBuilder, bind_group_layout_builder::BindGroupLayoutBuilder,
    box3d::Box3d, generic_shared_buffer::SharedBuffer,
    render_pipeline_builder::RenderPipelineBuilder,
};

pub struct Wireframe {
    line_vertex_index: u64,
    line_index_index: u64,
    line_index_count: u32,
    line_pipeline: eframe::wgpu::RenderPipeline,
    shared_buffer: SharedBuffer,
}

impl Wireframe {
    pub fn new(
        device: &Device,
        queue: &eframe::wgpu::Queue,
        texture_format: TextureFormat,
        bounds: Box3d,
        globals_bind_group: &BindGroupLayout,
        model_bind_group: &BindGroupLayout,
    ) -> Self {
        let mut shared_buffer = SharedBuffer::with_usages(
            device,
            2_u64.pow(16),
            BufferUsages::VERTEX | BufferUsages::INDEX | BufferUsages::COPY_DST,
        );

        let min = bounds.min;
        let max = bounds.max;

        let cube_vertices = vec![
            min,                            // 0
            Vec3::new(max.x, min.y, min.z), // 1
            Vec3::new(max.x, max.y, min.z), // 2
            Vec3::new(min.x, max.y, min.z), // 3
            Vec3::new(min.x, min.y, max.z), // 4
            Vec3::new(max.x, min.y, max.z), // 5
            Vec3::new(max.x, max.y, max.z), // 6
            Vec3::new(min.x, max.y, max.z), // 7
        ];

        let line_indices: Vec<u16> = vec![
            0, 1, 1, 2, 2, 3, 3, 0, 4, 5, 5, 6, 6, 7, 7, 4, 0, 4, 1, 5, 2, 6, 3, 7,
        ];

        let line_vertex_index = shared_buffer.allocate(
            queue,
            bytemuck::cast_slice(&cube_vertices),
            "Wireframe Vertices",
        );
        let line_index_index = shared_buffer.allocate(
            queue,
            bytemuck::cast_slice(&line_indices),
            "Wireframe Indices",
        );

        let line_pipeline = RenderPipelineBuilder::new(device)
            .shader(
                include_str!("../shaders/wireframe.wgsl"),
                "Wireframe Shader",
            )
            .primitive(PrimitiveTopology::LineList)
            .bind_group_layout(&[model_bind_group, globals_bind_group])
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
            .color_format(texture_format)
            .build("Line Pipeline");

        Wireframe {
            line_vertex_index,
            line_index_index,
            line_index_count: line_indices.len() as u32,
            line_pipeline,
            shared_buffer,
        }
    }

    pub fn draw<'a>(
        &self,
        pass: &mut RenderPass<'a>,
        globals_bind_group: &BindGroup,
        model_bind_group: &BindGroup,
    ) {
        pass.set_pipeline(&self.line_pipeline);
        pass.set_bind_group(0, model_bind_group, &[]);
        pass.set_bind_group(1, globals_bind_group, &[]);
        pass.set_vertex_buffer(0, self.shared_buffer.get_slice(self.line_vertex_index));
        pass.set_index_buffer(
            self.shared_buffer.get_slice(self.line_index_index),
            eframe::wgpu::IndexFormat::Uint16,
        );
        pass.draw_indexed(0..self.line_index_count, 0, 0..1);
    }

    pub fn update_bounds(&mut self, queue: &Queue, bounds: Box3d) {
        let min = bounds.min;
        let max = bounds.max;

        let cube_vertices = vec![
            min,                            // 0
            Vec3::new(max.x, min.y, min.z), // 1
            Vec3::new(max.x, max.y, min.z), // 2
            Vec3::new(min.x, max.y, min.z), // 3
            Vec3::new(min.x, min.y, max.z), // 4
            Vec3::new(max.x, min.y, max.z), // 5
            Vec3::new(max.x, max.y, max.z), // 6
            Vec3::new(min.x, max.y, max.z), // 7
        ];

        self.shared_buffer.update(
            queue,
            self.line_vertex_index,
            bytemuck::cast_slice(&cube_vertices),
        );
    }
}
