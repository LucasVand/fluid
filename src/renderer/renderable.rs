use eframe::{
    egui::{Context, Ui},
    wgpu::{BufferSlice, Device, Queue, RenderPass, TextureFormat},
};

pub trait Renderable {
    fn render(&mut self, pass: &mut RenderPass, rc: &RenderContext);
}

pub struct RenderCC<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,

    pub camera_buf: BufferSlice<'a>,

    pub texture_format: TextureFormat,
}

pub struct RenderContext<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub ctx: &'a Context,
    pub ui: &'a mut Ui,
    pub dt: f32,
}
