use std::mem;

use bytemuck::{Pod, Zeroable};
use eframe::wgpu::{
    BindGroup, BufferUsages, PrimitiveTopology, Queue, RenderPass, ShaderStages, TextureFormat,
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

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct ColoredVertex {
    pos: Vec3,
    color: Vec3,
}

pub struct AxisLines {
    line_vertex_index: u64,
    line_index_index: u64,
    line_index_count: u32,
    line_pipeline: eframe::wgpu::RenderPipeline,
    shared_buffer: SharedBuffer,
    bind_group: BindGroup,
}

impl AxisLines {
    pub fn new(rcc: &RenderCC, mcc: &FluidModelContext, length: f32) -> Self {
        let device = rcc.device;
        let queue = rcc.queue;

        let mut shared_buffer = SharedBuffer::with_usages(
            device,
            2_u64.pow(16),
            BufferUsages::VERTEX | BufferUsages::INDEX | BufferUsages::COPY_DST,
        );

        // Create axis lines: X=red, Y=green, Z=blue
        let axis_vertices = vec![
            ColoredVertex {
                pos: Vec3::ZERO,
                color: Vec3::new(1.0, 0.0, 0.0), // Origin red
            },
            ColoredVertex {
                pos: Vec3::new(length, 0.0, 0.0),
                color: Vec3::new(1.0, 0.0, 0.0), // X axis red
            },
            ColoredVertex {
                pos: Vec3::ZERO,
                color: Vec3::new(0.0, 1.0, 0.0), // Origin green
            },
            ColoredVertex {
                pos: Vec3::new(0.0, length, 0.0),
                color: Vec3::new(0.0, 1.0, 0.0), // Y axis green
            },
            ColoredVertex {
                pos: Vec3::ZERO,
                color: Vec3::new(0.0, 0.0, 1.0), // Origin blue
            },
            ColoredVertex {
                pos: Vec3::new(0.0, 0.0, length),
                color: Vec3::new(0.0, 0.0, 1.0), // Z axis blue
            },
        ];

        let line_indices: Vec<u16> = vec![0, 1, 2, 3, 4, 5];

        let line_vertex_index = shared_buffer.allocate(
            queue,
            bytemuck::cast_slice(&axis_vertices),
            "Axis Lines Vertices",
        );
        let line_index_index = shared_buffer.allocate(
            queue,
            bytemuck::cast_slice(&line_indices),
            "Axis Lines Indices",
        );

        let bgl = BindGroupLayoutBuilder::new(device)
            .uniform(0, ShaderStages::VERTEX_FRAGMENT)
            .uniform(1, ShaderStages::VERTEX_FRAGMENT)
            .build("Wireframe Bindgroup Layout");

        let bind_group = BindGroupBuilder::new(device, &bgl)
            .buffer_slice(0, rcc.camera_buf)
            .buffer(1, &mcc.model_buf)
            .build("Wireframe Bindgroup");

        let line_pipeline = RenderPipelineBuilder::new(device)
            .shader(
                include_str!("../../shaders/axis_lines.wgsl"),
                "Axis Lines Shader",
            )
            .primitive(PrimitiveTopology::LineList)
            .bind_group_layout(&[&bgl])
            .vertex_entry("vs_main")
            .vertex_buffers(vec![VertexBufferLayout {
                array_stride: mem::size_of::<ColoredVertex>() as u64,
                step_mode: VertexStepMode::Vertex,
                attributes: &[
                    VertexAttribute {
                        format: VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0,
                    },
                    VertexAttribute {
                        format: VertexFormat::Float32x3,
                        offset: 12,
                        shader_location: 1,
                    },
                ],
            }])
            .depth(TextureFormat::Depth32Float)
            .fragment_entry("fs_main")
            .color_format(rcc.texture_format)
            .build("Axis Lines Pipeline");

        AxisLines {
            bind_group: bind_group,
            line_vertex_index,
            line_index_index,
            line_index_count: line_indices.len() as u32,
            line_pipeline,
            shared_buffer,
        }
    }

    pub fn draw(&self, pass: &mut RenderPass) {
        pass.set_pipeline(&self.line_pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);

        pass.set_vertex_buffer(0, self.shared_buffer.get_slice(self.line_vertex_index));
        pass.set_index_buffer(
            self.shared_buffer.get_slice(self.line_index_index),
            eframe::wgpu::IndexFormat::Uint16,
        );

        pass.draw_indexed(0..self.line_index_count, 0, 0..1);
    }

    pub fn update_length(&mut self, queue: &Queue, length: f32) {
        let axis_vertices = vec![
            ColoredVertex {
                pos: Vec3::ZERO,
                color: Vec3::new(1.0, 0.0, 0.0),
            },
            ColoredVertex {
                pos: Vec3::new(length, 0.0, 0.0),
                color: Vec3::new(1.0, 0.0, 0.0),
            },
            ColoredVertex {
                pos: Vec3::ZERO,
                color: Vec3::new(0.0, 1.0, 0.0),
            },
            ColoredVertex {
                pos: Vec3::new(0.0, length, 0.0),
                color: Vec3::new(0.0, 1.0, 0.0),
            },
            ColoredVertex {
                pos: Vec3::ZERO,
                color: Vec3::new(0.0, 0.0, 1.0),
            },
            ColoredVertex {
                pos: Vec3::new(0.0, 0.0, length),
                color: Vec3::new(0.0, 0.0, 1.0),
            },
        ];

        self.shared_buffer.update(
            queue,
            self.line_vertex_index,
            bytemuck::cast_slice(&axis_vertices),
        );
    }
}
