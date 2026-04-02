use eframe::{
    App, CreationContext,
    egui::{
        self, CentralPanel, Color32, CornerRadius, Image, Key, Pos2, Rect, Stroke, StrokeKind,
        Vec2, load::SizedTexture,
    },
    epaint::Hsva,
    wgpu::Device,
};
use glam::Vec3;

use crate::{
    adjustable::Adjuster,
    fluid::render::RenderParams,
    fluid_sim::FluidSim,
    renderer::{render::Render, utils::box3d::Box3d},
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
    pub particle_count: usize,
    pub device: Device,
}

impl FluidApp {
    pub fn new(cc: &CreationContext<'_>, initial_size: Rect) -> Self {
        let size = 50.0;
        let bounds = Box3d::from_center(Vec3::new(0.0, 0.0, 0.0), Vec3::new(size, size, size));
        let count = 2000;
        let device = cc.wgpu_render_state.as_ref().unwrap().device.clone();

        let sim = FluidSim::new(cc, count as usize, bounds);
        Self {
            particle_size: 2.0,
            render: Render::new(cc, sim.particles.len() as u64, bounds),
            sim: sim,
            modifiers_open: false,
            pos: None,
            color_muliplier: 0.005,
            color_offset: 0.63,
            radius: 130.0,
            strength: 120.0,
            particle_count: count,
            device: cc.wgpu_render_state.as_ref().unwrap().device.clone(),
        }
    }
    pub fn reset(&mut self) {
        let bounds = self.sim.bounds;

        self.sim.particles = FluidSim::create_box(2000, bounds);
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
        if ctx.input(|i| i.key_pressed(Key::D)) {
            self.sim.debug_pressure_stats();
        }

        self.render.input(ctx);

        // let t = self.create_texture(ctx);
        CentralPanel::default().show(ctx, |ui| {
            let dt = ctx.input(|i| i.unstable_dt);

            self.sim.update(1.0 / 120.0);
            self.sim.update(1.0 / 120.0);
            self.sim
                .update_boundary_density_multiplied(self.sim.boundary_density_multiplier);
            self.render
                .render(&self.sim.particles, RenderParams::from_app(self));

            if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
                if !self.sim.running {
                    self.sim.start();
                    self.sim.update(1.0 / 120.0);
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
            a.add_drag(
                &mut self.sim.boundary_density_multiplier,
                "Boundary Density Multiplier",
            );
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
