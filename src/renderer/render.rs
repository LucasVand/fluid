use std::{f32::consts::PI, mem};

use bytemuck::{Pod, Zeroable};
use eframe::{
    CreationContext,
    egui::{Context, Key, TextureId, Vec2},
    wgpu::{
        BindGroup, Buffer, BufferUsages, Color, Device, FilterMode, LoadOp, Operations,
        PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
        RenderPassDescriptor, RenderPipeline, ShaderStages, StoreOp, Texture, TextureDimension,
        TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
    },
};
use glam::{Mat4, Vec3};

use crate::{
    fluid_app::FluidApp,
    fluid_sim::Particle,
    renderer::{
        camera::{Camera, GpuCamera},
        utils::{
            bind_group_builder::BindGroupBuilder,
            bind_group_layout_builder::BindGroupLayoutBuilder, buffer_builder::BufferBuilder,
            generic_shared_buffer::SharedBuffer, render_pipeline_builder::RenderPipelineBuilder,
            texture_builder::TextureBuilder,
        },
    },
};

pub struct Render {
    queue: Queue,
    device: Device,

    image_view: TextureView,
    depth_view: TextureView,
    depth_texture: Texture,
    image_texture: Texture,
    render_pipeline: RenderPipeline,

    particle_buffer: Buffer,

    shared_uniform: SharedBuffer,
    params_index: u64,
    camera_index: u64,
    model_index: u64,

    particles_bind_group: BindGroup,

    pub texture_id: TextureId,

    particle_count: u64,

    camera: Camera,
}

impl Render {
    pub fn new(cc: &CreationContext<'_>, particle_count: u64) -> Self {
        let state = cc.wgpu_render_state.as_ref().unwrap();
        let device = state.device.clone();
        let queue = state.queue.clone();

        let texture_format = state.target_format;

        let ppp = cc.egui_ctx.pixels_per_point();
        let texture_size: [u32; 2] = [(1200.0 * ppp) as u32, (700.0 * ppp) as u32];

        let image_texture = TextureBuilder::new(&device)
            .format(texture_format)
            .size(texture_size[0], texture_size[1], 1)
            .dimension(TextureDimension::D2)
            .usages(TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING)
            .build("Image Texture");

        let image_view = image_texture.create_view(&TextureViewDescriptor {
            label: Some("Image Texture View"),
            ..Default::default()
        });

        let depth_texture = TextureBuilder::new(&device)
            .format(TextureFormat::Depth32Float)
            .size(texture_size[0], texture_size[1], 1)
            .dimension(TextureDimension::D2)
            .usages(TextureUsages::RENDER_ATTACHMENT)
            .build("Image Depth Texture");

        let depth_view = depth_texture.create_view(&Default::default());

        let particles_bind_group_layout = BindGroupLayoutBuilder::new(&device)
            .buffer(1, ShaderStages::VERTEX_FRAGMENT, true)
            .uniform(0, ShaderStages::VERTEX_FRAGMENT)
            .uniform(2, ShaderStages::VERTEX_FRAGMENT)
            .uniform(3, ShaderStages::VERTEX_FRAGMENT)
            .build("Particles Buffer Layout");

        let render_pipeline = RenderPipelineBuilder::new(&device)
            .shader(include_str!("../shaders/draw.wgsl"), "Draw Shader")
            .primitive(PrimitiveTopology::TriangleStrip)
            .bind_group_layout(&[&particles_bind_group_layout])
            .vertex_entry("vs_main")
            .depth(TextureFormat::Depth32Float)
            .fragment_entry("fs_main")
            .color_format(texture_format)
            .build("Render Pipeline");

        let texture_id = state.renderer.write().register_native_texture(
            &device,
            &image_view,
            FilterMode::Linear,
        );

        let particle_size = mem::size_of::<GpuParticle>() as u64;
        let particle_buffer = BufferBuilder::new(&device)
            .usages(BufferUsages::STORAGE | BufferUsages::COPY_DST)
            .size(particle_count * particle_size)
            .build("Particles Buffer");

        let render_params_size = mem::size_of::<RenderParams>() as u64;
        let mut shared_uniform = SharedBuffer::new(&device, 2_u64.pow(13));

        let params_index =
            shared_uniform.allocate_uniform_empty(render_params_size, "Render Params");

        let camera_size = mem::size_of::<GpuCamera>() as u64;
        let camera_index = shared_uniform.allocate_uniform_empty(camera_size, "Camera");

        let model_size = mem::size_of::<[[f32; 4]; 4]>() as u64;
        let model_index = shared_uniform.allocate_uniform_empty(model_size, "Model");

        let particles_bind_group = BindGroupBuilder::new(&device, &particles_bind_group_layout)
            .buffer(1, &particle_buffer)
            .buffer_chunked(
                0,
                render_params_size,
                shared_uniform.get_offset(params_index),
                shared_uniform.get_buffer(),
            )
            .buffer_chunked(
                2,
                camera_size,
                shared_uniform.get_offset(camera_index),
                shared_uniform.get_buffer(),
            )
            .buffer_chunked(
                3,
                model_size,
                shared_uniform.get_offset(model_index),
                shared_uniform.get_buffer(),
            )
            .build("Particles Buffer Bind Group");

        Self {
            queue,
            device,
            shared_uniform,
            camera_index: camera_index,
            params_index: params_index,
            model_index: model_index,
            particles_bind_group,
            particle_count,
            particle_buffer,
            texture_id,
            depth_view,
            render_pipeline,
            depth_texture,
            image_texture,
            image_view,
            camera: Camera::new(),
        }
    }

    pub fn render(&self, particles: &Vec<Particle>, params: RenderParams) {
        self.write_particles(particles, params);

        let mut encoder = self.device.create_command_encoder(&Default::default());

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &self.image_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.render_pipeline);
            pass.set_bind_group(0, &self.particles_bind_group, &[]);
            pass.draw(0..4, 0..self.particle_count as u32);
        }
        self.queue.submit(Some(encoder.finish()));
    }
    fn write_particles(&self, particles: &Vec<Particle>, params: RenderParams) {
        let gpu_particles: Vec<GpuParticle> = particles
            .iter()
            .map(|p| GpuParticle {
                pos: p.pos,
                vel: p.vel,
            })
            .collect();

        self.queue.write_buffer(
            &self.particle_buffer,
            0,
            bytemuck::cast_slice(&gpu_particles),
        );

        self.shared_uniform
            .update(&self.queue, self.params_index, bytemuck::bytes_of(&params));

        self.shared_uniform.update(
            &self.queue,
            self.camera_index,
            bytemuck::bytes_of(&self.camera.to_gpu()),
        );
        self.shared_uniform.update(
            &self.queue,
            self.model_index,
            bytemuck::bytes_of(&Self::model_matrix(
                Vec3::ZERO,
                Vec3::new(0.0, 0.0, 0.0),
                0.1,
            )),
        );
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
    pub fn input(&mut self, ctx: &Context) {
        ctx.input(|i| {
            let h = &i.keys_down;
            if h.contains(&Key::W) {
                self.camera.walk(0.0, 1.0);
            }
            if h.contains(&Key::A) {
                self.camera.walk(-1.0, 0.0);
            }
            if h.contains(&Key::S) {
                self.camera.walk(0.0, -1.0);
            }
            if h.contains(&Key::D) {
                self.camera.walk(1.0, 0.0);
            }

            if i.pointer.primary_down() {
                let delta = i.pointer.delta();
                let pitch = delta.y;
                let yaw = -delta.x;
                self.camera.spin(yaw, pitch);
            }
        });
        if ctx.input(|i| i.key_pressed(Key::I)) {
            println!(
                "Camera pos: {:?}, forwards: {:?}, right: {:?}, up: {:?}",
                self.camera.position, self.camera.forwards, self.camera.right, self.camera.up
            );
            println!(
                "FR: {:?} FU: {:?}, RU: {:?}",
                self.camera.forwards.dot(self.camera.right),
                self.camera.forwards.dot(self.camera.up),
                self.camera.right.dot(self.camera.up)
            )
        }
    }
}
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct GpuParticle {
    pos: Vec3,
    vel: Vec3,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct RenderParams {
    color_multiplier: f32,
    color_offset: f32,
    particle_size: f32,
}
impl RenderParams {
    pub fn new(app: &FluidApp) -> RenderParams {
        RenderParams {
            color_multiplier: app.color_muliplier,
            color_offset: app.color_offset,
            particle_size: app.particle_size,
        }
    }
}
