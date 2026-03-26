use bytemuck::Pod;
use eframe::wgpu::{BufferUsages, Device};

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
}
