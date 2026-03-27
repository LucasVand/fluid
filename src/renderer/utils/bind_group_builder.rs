use eframe::wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, Buffer,
    BufferBinding, BufferSize, Device,
};

pub struct BindGroupBuilder<'a> {
    device: &'a Device,
    layout: &'a BindGroupLayout,
    entries: Vec<BindGroupEntry<'a>>,
}
impl<'a> BindGroupBuilder<'a> {
    pub fn new(device: &'a Device, layout: &'a BindGroupLayout) -> Self {
        Self {
            device,
            layout,
            entries: Vec::new(),
        }
    }
    pub fn push(mut self, entry: BindGroupEntry<'a>) -> Self {
        self.entries.push(entry);

        self
    }

    pub fn buffer(self, binding: u32, buffer: &'a Buffer) -> Self {
        self.push(BindGroupEntry {
            binding: binding,
            resource: buffer.as_entire_binding(),
        })
    }
    pub fn buffer_chunked(self, binding: u32, size: u64, offset: u64, buffer: &'a Buffer) -> Self {
        self.push(BindGroupEntry {
            binding: binding,
            resource: BindingResource::Buffer(BufferBinding {
                buffer,
                offset: offset,
                size: Some(BufferSize::new(size).unwrap()),
            }),
        })
    }
    pub fn build(self, label: &'a str) -> BindGroup {
        if self.entries.is_empty() {
            panic!("BindGroupBuilder: no entries added. Call .buffer() or .buffer_chunked() to add bind group entries before build()");
        }

        self.device.create_bind_group(&BindGroupDescriptor {
            label: Some(label),
            layout: self.layout,
            entries: &self.entries,
        })
    }
}
