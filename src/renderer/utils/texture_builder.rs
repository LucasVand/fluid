use eframe::wgpu::{self, Extent3d, Texture, TextureDimension, TextureFormat};
use eframe::wgpu::{Device, TextureUsages};

pub struct TextureBuilder<'a> {
    device: &'a Device,
    usages: Option<TextureUsages>,
    format: Option<TextureFormat>,
    view_formats: &'a [TextureFormat],
    mip_level_count: u32,
    sample_count: u32,
    dimension: Option<TextureDimension>,
    extent: Option<Extent3d>,
}

impl<'a> TextureBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self {
            device,
            usages: None,
            format: None,
            view_formats: &[],
            sample_count: 1,
            mip_level_count: 1,
            dimension: None,
            extent: None,
        }
    }

    pub fn usages(mut self, usages: TextureUsages) -> Self {
        self.usages = Some(usages);

        self
    }
    pub fn format(mut self, format: TextureFormat) -> Self {
        self.format = Some(format);

        self
    }
    pub fn view_formats(mut self, view_formats: &'a [TextureFormat]) -> Self {
        self.view_formats = view_formats;

        self
    }
    pub fn sample_count(mut self, sample_count: u32) -> Self {
        self.sample_count = sample_count;
        self
    }
    pub fn mip_level_count(mut self, mip_level_count: u32) -> Self {
        self.mip_level_count = mip_level_count;
        self
    }
    pub fn dimension(mut self, dimension: TextureDimension) -> Self {
        self.dimension = Some(dimension);
        self
    }
    pub fn size(mut self, width: u32, height: u32, depth_or_array_layers: u32) -> Self {
        self.extent = Some(Extent3d {
            width,
            height,
            depth_or_array_layers,
        });
        self
    }
    pub fn build(self, label: &'a str) -> Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: self.extent.unwrap(),
            mip_level_count: self.mip_level_count,
            sample_count: self.sample_count,
            dimension: self.dimension.unwrap(),
            format: self.format.unwrap(),
            usage: self.usages.unwrap(),
            view_formats: self.view_formats,
        })
    }
}
