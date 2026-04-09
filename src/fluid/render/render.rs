use std::mem;

use bytemuck::{Pod, Zeroable};
use eframe::wgpu::{
    BindGroup, BufferUsages, PrimitiveTopology, Queue, RenderPass, ShaderStages, TextureFormat,
};
use glam::Vec3;

use crate::{
    fluid::{
        fluid_params::FluidParams, model_context::FluidModelContext, render::axis_lines::AxisLines,
    },
    renderer::{
        renderable::RenderCC,
        utils::{
            bind_group_builder::BindGroupBuilder,
            bind_group_layout_builder::BindGroupLayoutBuilder, generic_shared_buffer::SharedBuffer,
            icosphere::Icosphere, render_pipeline_builder::RenderPipelineBuilder,
        },
    },
};

use super::wireframe::Wireframe;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct GpuParticle {
    pos: Vec3,
    _pad: f32,
    vel: Vec3,
    _pad0: f32,
    is_boundry: u32,
    _pad1: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct RenderParams {
    pub color_multiplier: f32,
    pub color_offset: f32,
    pub particle_size: f32,
}
impl From<&FluidParams> for RenderParams {
    fn from(value: &FluidParams) -> Self {
        RenderParams {
            color_multiplier: value.color_multiplier,
            color_offset: value.color_offset,
            particle_size: value.particle_size,
        }
    }
}

pub struct FluidRenderer {
    particles_bind_group: BindGroup,
    particle_pipeline: eframe::wgpu::RenderPipeline,
    particle_count: u64,
    params_index: u64,
    shared_uniform: SharedBuffer,
    wireframe: Wireframe,
    axis: AxisLines,
    sphere_shared_buffer: SharedBuffer,
    sphere_vertex_index: u64,
    sphere_index_index: u64,
    sphere_index_count: u32,
    queue: Queue,
}

impl FluidRenderer {
    pub fn new(rcc: &RenderCC, mcc: &FluidModelContext) -> Self {
        let device = rcc.device;
        let queue = rcc.queue;
        let particle_count = mcc.particles.len() as u64;

        let mut shared_uniform = SharedBuffer::with_usages(
            device,
            2_u64.pow(13),
            BufferUsages::STORAGE | BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let render_params_size = mem::size_of::<RenderParams>() as u64;
        let render_params: RenderParams = (&mcc.params).into();
        let params_index = shared_uniform.allocate_uniform(
            queue,
            &bytemuck::bytes_of(&render_params),
            "Render Params",
        );

        let bgl = BindGroupLayoutBuilder::new(device)
            .buffer(1, ShaderStages::VERTEX_FRAGMENT, true)
            .uniform(0, ShaderStages::VERTEX_FRAGMENT)
            .uniform(2, ShaderStages::VERTEX_FRAGMENT)
            .uniform(3, ShaderStages::VERTEX_FRAGMENT)
            .build("Particles Buffer Layout");

        let particle_pipeline = RenderPipelineBuilder::new(device)
            .shader(include_str!("../../shaders/draw.wgsl"), "Draw Shader")
            .primitive(PrimitiveTopology::TriangleList)
            .bind_group_layout(&[&bgl])
            .vertex_entry("vs_main")
            .vertex_buffers(vec![eframe::wgpu::VertexBufferLayout {
                array_stride: mem::size_of::<crate::renderer::utils::icosphere::SphereVertex>()
                    as u64,
                step_mode: eframe::wgpu::VertexStepMode::Vertex,
                attributes: &[
                    eframe::wgpu::VertexAttribute {
                        format: eframe::wgpu::VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0,
                    },
                    eframe::wgpu::VertexAttribute {
                        format: eframe::wgpu::VertexFormat::Float32x3,
                        offset: 12,
                        shader_location: 1,
                    },
                ],
            }])
            .depth(TextureFormat::Depth32Float)
            .fragment_entry("fs_main")
            .color_format(rcc.texture_format)
            .build("Particle Render Pipeline");

        let particles_bind_group = BindGroupBuilder::new(device, &bgl)
            .buffer(1, &mcc.particles_buf)
            .buffer(0, &mcc.model_buf)
            .buffer_chunked(
                2,
                render_params_size,
                shared_uniform.get_offset(params_index),
                shared_uniform.get_buffer(),
            )
            .buffer_slice(3, rcc.camera_buf)
            .build("Particles Buffer Bind Group");

        let wireframe = Wireframe::new(rcc, mcc);

        let axislines = AxisLines::new(rcc, mcc, 15.0);

        // Generate icosphere
        let sphere = Icosphere::new(1); // 2 subdivisions = smooth sphere

        let mut sphere_shared_buffer = SharedBuffer::with_usages(
            device,
            2_u64.pow(16),
            BufferUsages::VERTEX | BufferUsages::INDEX | BufferUsages::COPY_DST,
        );

        let sphere_vertex_index = sphere_shared_buffer.allocate(
            queue,
            bytemuck::cast_slice(&sphere.vertices),
            "Sphere Vertices",
        );

        let sphere_index_index = sphere_shared_buffer.allocate(
            queue,
            bytemuck::cast_slice(&sphere.indices),
            "Sphere Indices",
        );

        let sphere_index_count = sphere.indices.len() as u32;

        FluidRenderer {
            queue: rcc.queue.clone(),
            axis: axislines,
            particles_bind_group,
            particle_pipeline,
            particle_count,
            params_index,
            shared_uniform,
            wireframe,
            sphere_shared_buffer,
            sphere_vertex_index,
            sphere_index_index,
            sphere_index_count,
        }
    }
    pub fn update_params(&self, params: &FluidParams) {
        let new: RenderParams = params.into();
        self.shared_uniform
            .update(&self.queue, self.params_index, bytemuck::bytes_of(&new));
    }

    pub fn draw_particles(&self, pass: &mut RenderPass) {
        pass.set_pipeline(&self.particle_pipeline);
        pass.set_bind_group(0, &self.particles_bind_group, &[]);

        pass.set_vertex_buffer(
            0,
            self.sphere_shared_buffer
                .get_slice(self.sphere_vertex_index),
        );
        pass.set_index_buffer(
            self.sphere_shared_buffer.get_slice(self.sphere_index_index),
            eframe::wgpu::IndexFormat::Uint32,
        );
        pass.draw_indexed(0..self.sphere_index_count, 0, 0..self.particle_count as u32);

        self.wireframe.draw(pass);
        self.axis.draw(pass);
    }
}
