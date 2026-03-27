use bytemuck::Pod;
use eframe::wgpu::{BufferUsages, Buffer, Device};

pub struct BufferBuilder<'a> {
    device: &'a Device,

    content: Option<&'a [u8]>,
    usages: Option<BufferUsages>,
}

impl<'a> BufferBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self {
            device,
            content: None,
            usages: None,
        }
    }
    pub fn contents<T: Pod>(mut self, contents: &'a T) -> Self {
        self.content = Some(bytemuck::bytes_of(contents));
        self
    }
    pub fn usages(mut self, usages: BufferUsages) -> Self {
        self.usages = Some(usages);
        self
    }

    pub fn build(self, label: &'a str) -> Buffer {
        let content = self
            .content
            .expect("BufferBuilder: buffer content not set. Call .contents(data) before build()");
        let usages = self
            .usages
            .expect("BufferBuilder: buffer usages not set. Call .usages(BufferUsages) before build()");

        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: content,
                usage: usages,
            })
    }
}
