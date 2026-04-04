use eframe::wgpu::{Buffer, Device, Queue, RenderPass, TextureFormat};

pub trait Renderable {
    fn new(rcc: RenderCC) -> Self;
    fn render(pass: RenderPass, rc: RenderContext);
}

pub struct RenderCC<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,

    pub camera_buf: &'a Buffer,

    pub texture_format: TextureFormat,
}

pub struct RenderContext<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,
}
