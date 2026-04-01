use std::mem;

use bytemuck::{Pod, Zeroable};
use eframe::wgpu::{
    BindGroup, BindGroupLayout, Buffer, BufferUsages, Device, PrimitiveTopology, Queue, RenderPass,
    ShaderStages, TextureFormat,
};
use glam::{Mat4, Vec3};

use crate::{
    fluid::axis_lines::AxisLines,
    fluid_sim::Particle,
    renderer::utils::{
        bind_group_builder::BindGroupBuilder, bind_group_layout_builder::BindGroupLayoutBuilder,
        box3d::Box3d, buffer_builder::BufferBuilder, generic_shared_buffer::SharedBuffer,
        icosphere::Icosphere, render_pipeline_builder::RenderPipelineBuilder,
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
    color_multiplier: f32,
    color_offset: f32,
    particle_size: f32,
}

impl RenderParams {
    pub fn new(color_multiplier: f32, color_offset: f32, particle_size: f32) -> RenderParams {
        RenderParams {
            color_multiplier,
            color_offset,
            particle_size,
        }
    }

    pub fn from_app(app: &crate::fluid_app::FluidApp) -> RenderParams {
        RenderParams {
            color_multiplier: app.color_muliplier,
            color_offset: app.color_offset,
            particle_size: app.particle_size,
        }
    }
}

pub struct FluidRenderer {
    particle_buffer: Buffer,
    particles_bind_group: BindGroup,
    particle_pipeline: eframe::wgpu::RenderPipeline,
    particle_count: u64,
    params_index: u64,
    model_index: u64,
    shared_uniform: SharedBuffer,
    wireframe: Wireframe,
    axis: AxisLines,
    sphere_shared_buffer: SharedBuffer,
    sphere_vertex_index: u64,
    sphere_index_index: u64,
    sphere_index_count: u32,
}

impl FluidRenderer {
    pub fn new(
        device: &Device,
        queue: &eframe::wgpu::Queue,
        particle_count: u64,
        texture_format: TextureFormat,
        bounds: Box3d,
        global_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let mut shared_uniform = SharedBuffer::with_usages(
            device,
            2_u64.pow(13),
            BufferUsages::STORAGE | BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let particle_size = mem::size_of::<GpuParticle>() as u64;
        let particle_buffer = BufferBuilder::new(device)
            .usages(BufferUsages::STORAGE | BufferUsages::COPY_DST)
            .size(particle_count * particle_size)
            .build("Particles Buffer");

        let render_params_size = mem::size_of::<RenderParams>() as u64;
        let params_index =
            shared_uniform.allocate_uniform_empty(render_params_size, "Render Params");

        let model_size = mem::size_of::<[[f32; 4]; 4]>() as u64;
        let model_index = shared_uniform.allocate_uniform_empty(model_size, "Model");

        let particles_bind_group_layout = BindGroupLayoutBuilder::new(device)
            .buffer(1, ShaderStages::VERTEX_FRAGMENT, true)
            .uniform(0, ShaderStages::VERTEX_FRAGMENT)
            .uniform(2, ShaderStages::VERTEX_FRAGMENT)
            .build("Particles Buffer Layout");

        let particle_pipeline = RenderPipelineBuilder::new(device)
            .shader(include_str!("../shaders/draw.wgsl"), "Draw Shader")
            .primitive(PrimitiveTopology::TriangleList)
            .bind_group_layout(&[&particles_bind_group_layout, global_bind_group_layout])
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
            .color_format(texture_format)
            .build("Particle Render Pipeline");

        let particles_bind_group = BindGroupBuilder::new(device, &particles_bind_group_layout)
            .buffer(1, &particle_buffer)
            .buffer_chunked(
                0,
                model_size,
                shared_uniform.get_offset(model_index),
                shared_uniform.get_buffer(),
            )
            .buffer_chunked(
                2,
                render_params_size,
                shared_uniform.get_offset(params_index),
                shared_uniform.get_buffer(),
            )
            .build("Particles Buffer Bind Group");

        let wireframe = Wireframe::new(
            device,
            queue,
            texture_format,
            bounds,
            global_bind_group_layout,
            &particles_bind_group_layout,
        );

        let axislines = AxisLines::new(
            device,
            queue,
            texture_format,
            20.0,
            global_bind_group_layout,
            &particles_bind_group_layout,
        );

        // Generate icosphere
        let sphere = Icosphere::new(2); // 2 subdivisions = smooth sphere

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
            axis: axislines,
            particle_buffer,
            particles_bind_group,
            particle_pipeline,
            particle_count,
            params_index,
            model_index,
            shared_uniform,
            wireframe,
            sphere_shared_buffer,
            sphere_vertex_index,
            sphere_index_index,
            sphere_index_count,
        }
    }

    pub fn update_particles(&self, queue: &Queue, particles: &[Particle], params: RenderParams) {
        let gpu_particles: Vec<GpuParticle> = particles
            .iter()
            .map(|p| GpuParticle {
                pos: p.pos,
                _pad: 0.0,
                vel: p.vel,
                _pad0: 0.0,
                is_boundry: p.is_boundary as u32,
                _pad1: [0.0; 3],
            })
            .collect();

        queue.write_buffer(
            &self.particle_buffer,
            0,
            bytemuck::cast_slice(&gpu_particles),
        );

        queue.write_buffer(
            self.shared_uniform.get_buffer(),
            self.shared_uniform.get_offset(self.params_index),
            bytemuck::bytes_of(&params),
        );

        self.update_model(queue, Self::model_matrix(Vec3::ZERO, Vec3::ZERO, 0.1));
    }
    pub fn model_matrix(pos: Vec3, rotation: Vec3, scale: f32) -> [[f32; 4]; 4] {
        // Cube position, rotation, and scale
        let position = pos;
        let rotation = rotation;
        let scale = Vec3::splat(scale);

        // Translation
        let translate = Mat4::from_translation(position);

        // Rotation (yaw, pitch, roll)
        let rotate = Mat4::from_rotation_y(rotation.y)
            * Mat4::from_rotation_x(rotation.x)
            * Mat4::from_rotation_z(rotation.z);

        // Scale
        let scale = Mat4::from_scale(scale);

        // Combine to get model matrix
        let model = translate * rotate * scale;
        return model.to_cols_array_2d();
    }

    pub fn update_model(&self, queue: &Queue, model_matrix: [[f32; 4]; 4]) {
        queue.write_buffer(
            self.shared_uniform.get_buffer(),
            self.shared_uniform.get_offset(self.model_index),
            bytemuck::bytes_of(&model_matrix),
        );
    }

    pub fn draw_particles<'a>(
        &'a self,
        pass: &mut RenderPass<'a>,
        globals_bind_group: &'a eframe::wgpu::BindGroup,
    ) {
        pass.set_pipeline(&self.particle_pipeline);
        pass.set_bind_group(0, &self.particles_bind_group, &[]);
        pass.set_bind_group(1, globals_bind_group, &[]);
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

        self.wireframe
            .draw(pass, globals_bind_group, &self.particles_bind_group);
        self.axis
            .draw(pass, globals_bind_group, &self.particles_bind_group);
    }

    pub fn update_bounds(&mut self, queue: &Queue, bounds: Box3d) {
        self.wireframe.update_bounds(queue, bounds);
    }
}
