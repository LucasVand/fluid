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
}
