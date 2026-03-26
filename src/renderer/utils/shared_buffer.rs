use std::{fmt::write, mem, ops::Range, slice::Iter};

use eframe::wgpu::{
    BindGroup, BindGroupLayout, Buffer, BufferDescriptor, BufferUsages, Device, Queue,
    ShaderStages, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode,
};

use crate::scenes::{
    shapes::shape::Shape,
    utils::{
        bind_group_builder::BindGroupBuilder, bind_group_layout_builder::BindGroupLayoutBuilder,
    },
};

pub struct SharedBuffer {
    pub index_buf: Buffer,
    pub vertex_buf: Buffer,
    pub model_uniform: Buffer,
    // u16 index
    index_index: u64,
    // float index
    vertex_index: u64,
    // byte index
    model_index: u64,
    object_data: Vec<(Range<u64>, Range<u64>, Range<u64>)>,

    pub uniform_bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
}

impl SharedBuffer {
    pub fn new(device: &Device, size: u64) -> SharedBuffer {
        let i_buf = device.create_buffer(&BufferDescriptor {
            label: Some("Shared Buffer Index"),
            size,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let v_buf = device.create_buffer(&BufferDescriptor {
            label: Some("Shared Buffer Vertex"),
            size,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let m_buf = device.create_buffer(&BufferDescriptor {
            label: Some("Shared Buffer Model Uniform"),
            size,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bg_layout = BindGroupLayoutBuilder::new(device)
            .uniform_dyn(0, ShaderStages::VERTEX_FRAGMENT)
            .build("Shared Buffer Uniform Group Layout");
        let bg = BindGroupBuilder::new(device, &bg_layout)
            .buffer_chunked(0, 256, 0, &m_buf)
            .build("Shared Buffer Uniform Group");

        Self {
            bind_group_layout: bg_layout,
            uniform_bind_group: bg,
            index_buf: i_buf,
            vertex_buf: v_buf,
            model_uniform: m_buf,
            index_index: 0,
            vertex_index: 0,
            model_index: 0,
            object_data: Vec::new(),
        }
    }
    pub fn push(&mut self, q: &Queue, vertex: &[f32], index: &[u16], model: &[[f32; 4]; 4]) {
        let i_start = self.index_index;
        let v_start = self.vertex_index;
        let m_start = self.model_index;

        q.write_buffer(
            &self.vertex_buf,
            self.vertex_index,
            bytemuck::cast_slice(vertex),
        );
        q.write_buffer(
            &self.index_buf,
            self.index_index,
            bytemuck::cast_slice(index),
        );
        if model.len() >= 256 {
            panic!("Model Binding is too large");
        }

        q.write_buffer(
            &self.model_uniform,
            self.model_index,
            bytemuck::cast_slice(model),
        );
        self.vertex_index += vertex.len() as u64 * 4;
        self.index_index += index.len() as u64 * 2;
        self.model_index += 256 as u64;

        self.object_data.push((
            v_start..self.vertex_index,
            i_start..self.index_index,
            m_start..self.model_index,
        ));
    }

    pub fn write_index(&mut self, q: &Queue, index: u64, model: [[f32; 4]; 4]) {
        q.write_buffer(
            &self.model_uniform,
            index * 256,
            bytemuck::cast_slice(&model),
        );
    }
    pub fn push_shape(&mut self, q: &Queue, s: &Shape) {
        self.push(
            q,
            &s.verticies,
            &s.indicies,
            &s.model_matrix().to_cols_array_2d(),
        );
    }
    pub fn layout(&self) -> VertexBufferLayout<'_> {
        VertexBufferLayout {
            array_stride: 24,
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
        }
    }
    pub fn iter(&self) -> Iter<'_, (Range<u64>, Range<u64>, Range<u64>)> {
        self.object_data.iter()
    }
}
