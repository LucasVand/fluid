use eframe::{
    App, CreationContext,
    egui::{self, CentralPanel, Color32, CornerRadius, Key, Pos2, Rect, Stroke, StrokeKind, Vec2},
    epaint::Hsva,
};

use crate::{adjustable::Adjuster, fluid_sim::FluidSim};

pub struct FluidApp {
    pub sim: FluidSim,
    pub modifiers_open: bool,
    pub pos: Option<Pos2>,
    pub color_muliplier: f32,
    pub color_offset: f32,
    pub radius: f32,
    pub strength: f32,
}

impl FluidApp {
    pub fn new(cc: &CreationContext<'_>, initial_size: Rect) -> Self {
        Self {
            sim: FluidSim::new(4000, initial_size),
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
        if ctx.input(|i| i.pointer.primary_down()) {
            self.sim.apply_force(
                ctx.pointer_hover_pos().unwrap().to_vec2(),
                self.radius,
                self.strength,
            );
            self.pos = ctx.pointer_hover_pos();
        } else if ctx.input(|i| i.pointer.secondary_clicked()) {
            self.pos = ctx.pointer_hover_pos();
            println!(
                "Den: {:?}",
                self.sim
                    .calculate_density(ctx.pointer_hover_pos().unwrap().to_vec2())
            );
        } else {
            self.pos = None;
        }

        if ctx.input(|i| i.key_pressed(Key::Space)) {
            self.sim.toggle_running();
        }
        if ctx.input(|i| i.key_pressed(Key::R)) {
            self.reset();
        }

        // let t = self.create_texture(ctx);
        CentralPanel::default().show(ctx, |ui| {
            let dt = ctx.input(|i| i.unstable_dt);
            let rect = ui.max_rect();

            self.sim
                .set_bounds(ui.max_rect().translate(Vec2::new(10.0, 10.0)) * 0.9);
            self.sim.update(dt);
            if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
                if !self.sim.running {
                    self.sim.start();
                    self.sim.update(1.0 / 60.0);
                    self.sim.stop();
                }
            }
            // self.scene.sim.update(dt / 3.0);
            // self.scene.sim.update(dt / 3.0);

            let painter = ui.painter();
            painter.rect_stroke(
                self.sim.bounds,
                CornerRadius::ZERO,
                Stroke::new(0.5, Color32::WHITE),
                StrokeKind::Outside,
            );

            for particle in self.sim.particles.iter() {
                let vel = -particle.vel.length() * self.color_muliplier + self.color_offset;
                let color = Hsva::new(vel.max(0.0), 0.7, 0.8, 1.0);
                painter.circle_filled(particle.pos.to_pos2(), self.sim.particle_size, color);
            }

            // let size = (ui.max_rect().size() / self.scene.sim.spatial_map.cell_size).ceil();
            // let p = &self.scene.sim.spatial_map;
            //
            // for j in 0..size.y as usize {
            //     for i in 0..size.x as usize {
            //         let min = Vec2::new(i as f32 * p.cell_size, j as f32 * p.cell_size).to_pos2();
            //         let rect = Rect::from_min_size(min, Vec2::new(p.cell_size, p.cell_size));
            //         let mut color = Color32::GRAY;
            //         if let Some(point) = self.pos {
            //             let key = p.coords_to_key((i as isize, j as isize));
            //             let point_key = p.pos_to_key(point.to_vec2());
            //             if key == point_key {
            //                 color = Color32::BLUE;
            //             }
            //         }
            //         painter.rect_stroke(
            //             rect,
            //             CornerRadius::ZERO,
            //             Stroke::new(1.0, color),
            //             StrokeKind::Middle,
            //         );
            //         // painter.text(
            //         //     rect.center(),
            //         //     Align2::CENTER_CENTER,
            //         //     p.coords_to_key((i, j)),
            //         //     egui::FontId::new(20.0, egui::FontFamily::Monospace),
            //         //     Color32::GREEN,
            //         // );
            //         // for i in p.get(rect.center().to_vec2()) {
            //         //     let particle = &self.scene.sim.particles[i];
            //         //     painter.text(
            //         //         particle.pos.to_pos2(),
            //         //         Align2::CENTER_CENTER,
            //         //         p.pos_to_key(particle.pos),
            //         //         egui::FontId::new(20.0, egui::FontFamily::Monospace),
            //         //         Color32::GREEN,
            //         //     );
            //         // }
            //     }
            // }
            //
            if let Some(point) = self.pos {
                painter.circle_stroke(point, self.radius, Stroke::new(1.0, Color32::LIGHT_BLUE));
            }
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
            a.add_float(&mut self.sim.particle_size, 0.0..=50.0, "Particle Size");
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
