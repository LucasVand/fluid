use bytemuck::Pod;
use eframe::wgpu::{self, Buffer, BufferDescriptor, BufferUsages, Device, util::DeviceExt};

pub struct BufferBuilder<'a> {
    device: &'a Device,

    content: Option<&'a [u8]>,
    size: Option<u64>,
    usages: Option<BufferUsages>,
}

impl<'a> BufferBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self {
            device,
            content: None,
            size: None,
            usages: None,
        }
    }
    pub fn contents<T: Pod>(mut self, contents: &'a T) -> Self {
        self.content = Some(bytemuck::bytes_of(contents));
        self
    }
    pub fn contents_slice(mut self, contents: &'a [u8]) -> Self {
        self.content = Some(contents);
        self
    }
    pub fn size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }
    pub fn usages(mut self, usages: BufferUsages) -> Self {
        self.usages = Some(usages);
        self
    }

    pub fn build(self, label: &'a str) -> Buffer {
        let usages = self.usages.expect(
            "BufferBuilder: buffer usages not set. Call .usages(BufferUsages) before build()",
        );

        match self.content {
            Some(content) => self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(label),
                    contents: content,
                    usage: usages,
                }),
            None => {
                let size = self.size.expect(
                    "BufferBuilder: neither content nor size set. Call either .contents(data) or .size(bytes) before build()",
                );
                self.device.create_buffer(&BufferDescriptor {
                    label: Some(label),
                    size,
                    usage: usages,
                    mapped_at_creation: false,
                })
            }
        }
    }
}
