use eframe::wgpu::RenderPass;
use glam::Vec3;

use crate::{
    fluid::render::FluidRenderer,
    renderer::{
        renderable::{RenderCC, RenderContext, Renderable},
        utils::box3d::Box3d,
    },
};

pub struct Fluid {
    renderer: FluidRenderer,
}

impl Renderable for Fluid {
    fn new(rcc: RenderCC) -> Self {
        let size = 50.0;
        let bounds = Box3d::from_center(Vec3::new(0.0, 0.0, 0.0), Vec3::new(size, size, size));
        let count = 2000;

        let renderer = FluidRenderer::new(rcc, count, bounds);

        let sim = 

        Self { renderer: renderer }
    }

    fn render(pass: RenderPass, rc: RenderContext) {
        todo!()
    }
}
