use std::mem;

use bytemuck::{Pod, Zeroable};
use eframe::wgpu::{
    BindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, BufferUsages,
    PrimitiveTopology, Queue, RenderPass, ShaderStages, TextureFormat, TextureViewDescriptor,
    TextureViewDimension,
};
use glam::Vec3;

use crate::{
    fluid::{
        fluid_params::FluidParams,
        model_context::FluidModelContext,
        render::{axis_lines::AxisLines, wireframe::Wireframe},
    },
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
pub struct RenderParams {
    pub color_multiplier: f32,
    pub color_offset: f32,
    pub particle_size: f32,
    _pad: f32,
    pub bound_min: Vec3,
    _pad1: f32,
    pub bound_max: Vec3,
    _pad2: f32,
    pub scattering: Vec3,
    _pad3: f32,
    pub density_multiplier: f32,
    _pad4: [f32; 3],
}
impl From<&FluidParams> for RenderParams {
    fn from(value: &FluidParams) -> Self {
        RenderParams {
            color_multiplier: value.color_multiplier,
            color_offset: value.color_offset,
            particle_size: value.particle_size,
            _pad: 0.0,
            bound_min: value.bounds.min,
            _pad1: 0.0,
            bound_max: value.bounds.max,
            _pad2: 0.0,
            scattering: Vec3::new(
                value.red_scattering,
                value.blue_scattering,
                value.green_scattering,
            ),
            _pad3: 0.0,
            density_multiplier: value.render_density_multiplier,
            _pad4: [0.0; 3],
        }
    }
}

pub struct FluidRenderer {
    particles_bind_group: BindGroup,
    particle_pipeline: eframe::wgpu::RenderPipeline,
    shared_uniform: SharedBuffer,
    wireframe: Wireframe,
    axis: AxisLines,
    queue: Queue,
    params_index: u64,
}

impl FluidRenderer {
    pub fn new(rcc: &RenderCC, mcc: &FluidModelContext) -> Self {
        let device = rcc.device;
        let queue = rcc.queue;

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
            .uniform(0, ShaderStages::VERTEX_FRAGMENT)
            .uniform(1, ShaderStages::VERTEX_FRAGMENT)
            .uniform(2, ShaderStages::VERTEX_FRAGMENT)
            .storage_texture_read(3, ShaderStages::VERTEX_FRAGMENT, TextureFormat::R32Float)
            .build("Particles Buffer Layout");

        let particle_pipeline = RenderPipelineBuilder::new(device)
            .shader(include_str!("./shaders/ray_march.wgsl"), "Draw Shader")
            .primitive(PrimitiveTopology::TriangleList)
            .bind_group_layout(&[&bgl])
            .vertex_entry("vs_main")
            .depth(TextureFormat::Depth32Float)
            .fragment_entry("fs_main")
            .color_format(rcc.texture_format)
            // .blend_state(BlendState {
            //     color: BlendComponent {
            //         src_factor: BlendFactor::SrcAlpha,
            //         dst_factor: BlendFactor::OneMinusSrcAlpha,
            //         operation: BlendOperation::Add,
            //     },
            //     alpha: BlendComponent::REPLACE,
            // })
            .build("Ray March Render Pipeline");

        let texture_view = mcc.density_map.create_view(&TextureViewDescriptor {
            dimension: Some(TextureViewDimension::D3),
            ..Default::default()
        });

        let particles_bind_group = BindGroupBuilder::new(device, &bgl)
            .buffer(0, &mcc.model_buf)
            .buffer_chunked(
                1,
                render_params_size,
                shared_uniform.get_offset(params_index),
                shared_uniform.get_buffer(),
            )
            .buffer_slice(2, rcc.camera_buf)
            .texture(3, &texture_view)
            .build("Particles Buffer Bind Group");

        let wireframe = Wireframe::new(rcc, mcc);

        let axislines = AxisLines::new(rcc, mcc, 15.0);

        FluidRenderer {
            params_index,
            queue: rcc.queue.clone(),
            axis: axislines,
            particles_bind_group,
            particle_pipeline,
            shared_uniform,
            wireframe,
        }
    }
    pub fn update_params(&self, params: &FluidParams) {
        let new: RenderParams = params.into();
        self.shared_uniform
            .update(&self.queue, self.params_index, bytemuck::bytes_of(&new));
    }

    pub fn draw_particles(&self, pass: &mut RenderPass) {
        // self.wireframe.draw(pass);
        // self.axis.draw(pass);

        pass.set_pipeline(&self.particle_pipeline);
        pass.set_bind_group(0, &self.particles_bind_group, &[]);
        pass.draw(0..6, 0..1);
    }
}
