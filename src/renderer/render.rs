use std::mem;

use eframe::{
    CreationContext,
    egui::{Context, Key, TextureId, Ui},
    wgpu::{
        BindGroup, Color, Device, FilterMode, LoadOp, Operations, Queue, RenderPassColorAttachment,
        RenderPassDepthStencilAttachment, RenderPassDescriptor, ShaderStages, StoreOp, Texture,
        TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
    },
};
use glam::{Mat4, Vec3};

use crate::{
    fluid_sim::Particle,
    renderer::{
        camera::{Camera, GpuCamera},
        renderable::{RenderCC, RenderContext, Renderable},
        utils::{generic_shared_buffer::SharedBuffer, texture_builder::TextureBuilder},
    },
};

pub struct Render {
    pub renderables: Vec<Box<dyn Renderable>>,
    queue: Queue,
    device: Device,

    image_view: TextureView,
    depth_view: TextureView,
    depth_texture: Texture,
    image_texture: Texture,

    shared_uniform: SharedBuffer,
    camera_index: u64,

    pub texture_id: TextureId,
    texture_format: TextureFormat,

    camera: Camera,
}

impl Render {
    pub fn new(cc: &CreationContext<'_>) -> Self {
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

        let texture_id = state.renderer.write().register_native_texture(
            &device,
            &image_view,
            FilterMode::Linear,
        );

        let mut camera = Camera::new();
        camera.rotate_about(0.0, 0.0, Vec3::ZERO);

        let mut shared_uniform = SharedBuffer::new(&device, 2_u64.pow(13));
        let camera_index =
            shared_uniform.allocate_uniform(&queue, bytemuck::bytes_of(&camera.to_gpu()), "Camera");

        Self {
            texture_format: texture_format,
            queue,
            device,
            shared_uniform,
            camera_index,
            texture_id,
            depth_view,
            depth_texture,
            image_texture,
            image_view,
            camera,
            renderables: Vec::new(),
        }
    }
    fn get_creation_context(&self) -> RenderCC {
        RenderCC {
            device: &self.device,
            queue: &self.queue,
            camera_buf: self.shared_uniform.get_slice(self.camera_index),
            texture_format: self.texture_format,
        }
    }
    pub fn add_renderable<F: FnOnce(RenderCC) -> Box<dyn Renderable>>(&mut self, renderable: F) {
        self.renderables
            .push(renderable(self.get_creation_context()));
    }

    pub fn render(&mut self, dt: f32, ctx: &Context, ui: &mut Ui) {
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
                        store: StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let rc = RenderContext {
                device: &self.device,
                queue: &self.queue,
                ui: ui,
                ctx,
                dt: dt,
            };

            for renderable in self.renderables.iter_mut() {
                renderable.render(&mut pass, &rc);
            }
        }

        self.queue.submit(Some(encoder.finish()));
    }
    pub fn input(&mut self, ctx: &Context) {
        ctx.input(|i| {
            let h = &i.keys_down;
            if h.contains(&Key::W) {
                self.camera.move_towards(1.0, Vec3::ZERO);
            }
            // if h.contains(&Key::A) {
            //     self.camera.walk(-1.0, 0.0);
            // }
            if h.contains(&Key::S) {
                self.camera.move_towards(-1.0, Vec3::ZERO);
            }
            // if h.contains(&Key::D) {
            //     self.camera.walk(1.0, 0.0);
            // }

            if i.pointer.primary_down() {
                let delta = i.pointer.delta();
                let pitch = delta.y;
                let yaw = -delta.x;
                self.camera.rotate_about(yaw, pitch, Vec3::ZERO);
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

        self.shared_uniform.update(
            &self.queue,
            self.camera_index,
            bytemuck::bytes_of(&self.camera.to_gpu()),
        );
    }
}
