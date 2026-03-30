use eframe::{
    App, CreationContext,
    egui::{
        self, CentralPanel, Color32, CornerRadius, Image, Key, Pos2, Rect, Stroke, StrokeKind,
        Vec2, load::SizedTexture,
    },
    epaint::Hsva,
};
use glam::Vec3;

use crate::{
    adjustable::Adjuster,
    fluid_sim::FluidSim,
    renderer::{
        render::{Render, RenderParams},
        utils::box3d::Box3d,
    },
};

pub struct FluidApp {
    pub render: Render,
    pub sim: FluidSim,
    pub modifiers_open: bool,
    pub pos: Option<Pos2>,
    pub color_muliplier: f32,
    pub color_offset: f32,
    pub particle_size: f32,
    pub radius: f32,
    pub strength: f32,
}

impl FluidApp {
    pub fn new(cc: &CreationContext<'_>, initial_size: Rect) -> Self {
        let size = 100.0;
        Self {
            particle_size: 2.0,
            render: Render::new(cc, 4000),
            sim: FluidSim::new(
                10,
                Box3d::from_center(Vec3::new(0.0, 0.0, 0.0), Vec3::new(size, size, size)),
            ),
            modifiers_open: false,
            pos: None,
            color_muliplier: 0.005,
            color_offset: 0.63,
            radius: 130.0,
            strength: 120.0,
        }
    }
    pub fn reset(&mut self) {
        let bounds = self.sim.bounds;
        let count = self.sim.particles.len();
        self.sim = FluidSim::new(count, bounds);
    }
}
impl App for FluidApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        // if ctx.input(|i| i.pointer.primary_down()) {
        //     self.sim.apply_force(
        //         ctx.pointer_hover_pos().unwrap().to_vec2(),
        //         self.radius,
        //         self.strength,
        //     );
        //     self.pos = ctx.pointer_hover_pos();
        // } else if ctx.input(|i| i.pointer.secondary_clicked()) {
        //     self.pos = ctx.pointer_hover_pos();
        //     println!(
        //         "Den: {:?}",
        //         self.sim
        //             .calculate_density(ctx.pointer_hover_pos().unwrap().to_vec2())
        //     );
        // } else {
        //     self.pos = None;
        // }

        if ctx.input(|i| i.key_pressed(Key::Space)) {
            self.sim.toggle_running();
        }
        if ctx.input(|i| i.key_pressed(Key::R)) {
            self.reset();
        }

        self.render.input(ctx);

        // let t = self.create_texture(ctx);
        CentralPanel::default().show(ctx, |ui| {
            let dt = ctx.input(|i| i.unstable_dt);

            self.sim.update(dt);
            self.render
                .render(&self.sim.particles, RenderParams::new(self));

            if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
                if !self.sim.running {
                    self.sim.start();
                    self.sim.update(1.0 / 60.0);
                    self.sim.stop();
                }
            }

            let rect = ui.max_rect();
            ui.add(Image::from_texture(SizedTexture {
                id: self.render.texture_id,
                size: rect.size(),
            }));
        });

        {
            if ctx.input(|i| i.key_pressed(Key::M)) {
                self.modifiers_open = !self.modifiers_open;
            }

            let mut a: Adjuster<'_> = Adjuster::new();
            a.add_float(
                &mut self.sim.smoothing_radius,
                0.0..=200.0,
                "Smoothing Radius",
            );
            a.add_float(&mut self.particle_size, 0.0..=50.0, "Particle Size");
            a.add_float(&mut self.sim.gradient_step, 0.0..=0.01, "Gradient Step");
            a.add_drag(&mut self.sim.target_density, "Target Density");
            a.add_drag(&mut self.sim.pressure_multiplier, "Pressure Multiplier");
            a.add_drag(
                &mut self.sim.near_pressure_multiplier,
                "Near Pressure Multiplier",
            );
            a.add_drag(&mut self.sim.viscosity_strength, "Viscosity Strength");
            a.add_drag(&mut self.color_muliplier, "Color Multiplier");
            a.add_float(&mut self.color_offset, 0.0..=1.0, "Color Offset");
            a.add_drag(&mut self.radius, "Force Radius");
            a.add_drag(&mut self.strength, "Force Strength");
            a.add_drag(&mut self.sim.gravity, "Gravity Strength");

            a.show(ctx, &mut self.modifiers_open);
        }

        // ctx.request_repaint_after(Duration::from_millis(500));
        ctx.request_repaint();
    }
}
