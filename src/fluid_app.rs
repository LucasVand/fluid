use eframe::{
    App, CreationContext,
    egui::{CentralPanel, Image, Pos2, Rect, load::SizedTexture},
    wgpu::Device,
};

use crate::{fluid::fluid::Fluid, renderer::render::Render};

pub struct FluidApp {
    pub render: Render,
    pub modifiers_open: bool,
    pub pos: Option<Pos2>,
    pub device: Device,
}

impl FluidApp {
    pub fn new(cc: &CreationContext<'_>, _inital_size: Rect) -> Self {
        let device = cc.wgpu_render_state.as_ref().unwrap().device.clone();

        let mut r = Render::new(cc);
        r.add_renderable(|rcc| Box::new(Fluid::new(&rcc)));

        Self {
            render: r,
            modifiers_open: false,
            pos: None,
            device: device,
        }
    }
    pub fn reset(&mut self) {
        // let bounds = self.sim.bounds;

        // self.sim.particles = FluidSim::create_box(2000, bounds);
    }
}
impl App for FluidApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.render.input(ctx);

        CentralPanel::default().show(ctx, |ui| {
            let dt = ctx.input(|i| i.unstable_dt);

            self.render.render(dt, ctx, ui);

            let rect = ui.max_rect();
            ui.add(Image::from_texture(SizedTexture {
                id: self.render.texture_id,
                size: rect.size(),
            }));
        });

        ctx.request_repaint();
    }
}
