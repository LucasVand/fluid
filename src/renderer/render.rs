use eframe::{
    CreationContext,
    wgpu::{Device, Queue},
};

pub struct Render {
    queue: Queue,
    device: Device,
}

impl Render {
    pub fn new(cc: CreationContext<'_>) -> Self {
        let state = cc.wgpu_render_state.as_ref().unwrap();
        let device = state.device.clone();
        let queue = state.queue.clone();

        Self { queue, device }
    }
    let texture = d.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen color"),
            size: wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: state.target_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&Default::default());

        let depth_tex = d.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size: wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let depth = depth_tex.create_view(&Default::default());

}
