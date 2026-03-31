use eframe::wgpu::{BindGroup, Device, Queue, RenderPass};

pub trait Renderable {
    fn setup(device: &Device, queue: &Queue) -> Self;
    fn render(pass: RenderPass, globals_bind_group: BindGroup);
}
