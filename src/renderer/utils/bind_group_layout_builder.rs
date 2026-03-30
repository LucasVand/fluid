use eframe::wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BufferBindingType, Device, ShaderStages, TextureSampleType, TextureViewDimension,
    SamplerBindingType,
};

pub struct BindGroupLayoutBuilder<'a> {
    entries: Vec<BindGroupLayoutEntry>,
    device: &'a Device,
}
impl<'a> BindGroupLayoutBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self {
            device,
            entries: Vec::new(),
        }
    }
    fn push(mut self, entry: BindGroupLayoutEntry) -> Self {
        self.entries.push(entry);

        self
    }
    pub fn uniform(self, binding: u32, visibility: ShaderStages) -> Self {
        self.push(BindGroupLayoutEntry {
            binding: binding,
            visibility,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        })
    }
    pub fn uniform_dyn(self, binding: u32, visibility: ShaderStages) -> Self {
        self.push(BindGroupLayoutEntry {
            binding: binding,
            visibility,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: true,
                min_binding_size: None,
            },
            count: None,
        })
    }
    pub fn buffer(self, binding: u32, visibility: ShaderStages, read_only: bool) -> Self {
        self.push(BindGroupLayoutEntry {
            binding,
            visibility,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage {
                    read_only: read_only,
                },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        })
    }
    pub fn texture(
        self,
        binding: u32,
        visibility: ShaderStages,
        dimension: TextureViewDimension,
        sample_type: TextureSampleType,
    ) -> Self {
        self.push(BindGroupLayoutEntry {
            binding,
            visibility,
            ty: BindingType::Texture {
                sample_type,
                view_dimension: dimension,
                multisampled: false,
            },
            count: None,
        })
    }
    pub fn sampler(self, binding: u32, visibility: ShaderStages) -> Self {
        self.push(BindGroupLayoutEntry {
            binding,
            visibility,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
        })
    }
    pub fn sampler_comparison(self, binding: u32, visibility: ShaderStages) -> Self {
        self.push(BindGroupLayoutEntry {
            binding,
            visibility,
            ty: BindingType::Sampler(SamplerBindingType::Comparison),
            count: None,
        })
    }
    pub fn build(self, label: &'a str) -> BindGroupLayout {
        if self.entries.is_empty() {
            panic!("BindGroupLayoutBuilder: no entries added. Call .uniform(), .uniform_dyn(), .buffer(), .texture(), .sampler(), or .sampler_comparison() to add bind group layout entries before build()");
        }

        self.device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some(label),
                entries: &self.entries,
            })
    }
}
