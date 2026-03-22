use std::time::Duration;

use eframe::{
    App, CreationContext,
    egui::{
        self, Align, Align2, CentralPanel, Color32, ColorImage, Context, CornerRadius, Image, Key,
        Pos2, Rect, Stroke, StrokeKind, TextureHandle, TextureOptions, Vec2, load::SizedTexture,
    },
};

use crate::{adjustable::Adjuster, fluid_sim::FluidSim, spatial_map::SpatialMap};

pub struct FluidApp {
    pub sim: FluidSim,
    pub modifiers_open: bool,
    pub pos: Option<Pos2>,
}

impl FluidApp {
    pub fn new(cc: &CreationContext<'_>, initial_size: Rect) -> Self {
        Self {
            sim: FluidSim::new(3000, initial_size),
            modifiers_open: false,
            pos: None,
        }
    }
}
impl App for FluidApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        if ctx.input(|i| i.pointer.any_down()) {
            self.pos = ctx.pointer_hover_pos();
        }

        // let t = self.create_texture(ctx);
        CentralPanel::default().show(ctx, |ui| {
            let dt = ctx.input(|i| i.unstable_dt);
            let rect = ui.max_rect();

            self.sim.set_bounds(ui.max_rect());
            self.sim.update(dt);
            // self.sim.update(dt / 3.0);
            // self.sim.update(dt / 3.0);

            // ui.add(Image::from_texture(&t).fit_to_exact_size(rect.size()));

            let painter = ui.painter();
            painter.rect_stroke(
                self.sim.bounds,
                CornerRadius::ZERO,
                Stroke::new(0.5, Color32::WHITE),
                StrokeKind::Outside,
            );

            for particle in self.sim.particles.iter() {
                painter.circle_filled(
                    particle.pos.to_pos2(),
                    self.sim.particle_size,
                    Color32::WHITE,
                );
            }

            let size = self.sim.spatial_map.size();
            let p = &self.sim.spatial_map;

            for j in 0..size.y as usize {
                for i in 0..size.x as usize {
                    let min = self.sim.bounds.min
                        + Vec2::new(i as f32 * p.cell_size, j as f32 * p.cell_size);
                    let rect = Rect::from_min_size(min, Vec2::new(p.cell_size, p.cell_size));
                    let mut color = Color32::GRAY;
                    if let Some(point) = self.pos {
                        let key = p.coords_to_key((i, j));
                        let point_key = p.pos_to_key(point.to_vec2());
                        if key == point_key {
                            color = Color32::BLUE;
                        }
                    }
                    painter.rect_stroke(
                        rect,
                        CornerRadius::ZERO,
                        Stroke::new(1.0, color),
                        StrokeKind::Middle,
                    );
                    // painter.text(
                    //     rect.center(),
                    //     Align2::CENTER_CENTER,
                    //     p.coords_to_key((i, j)),
                    //     egui::FontId::new(20.0, egui::FontFamily::Monospace),
                    //     Color32::GREEN,
                    // );
                    // for i in p.get(rect.center().to_vec2()) {
                    //     let particle = &self.sim.particles[i];
                    //     painter.text(
                    //         particle.pos.to_pos2(),
                    //         Align2::CENTER_CENTER,
                    //         p.pos_to_key(particle.pos),
                    //         egui::FontId::new(20.0, egui::FontFamily::Monospace),
                    //         Color32::GREEN,
                    //     );
                    // }
                }
            }

            if let Some(point) = self.pos {
                painter.circle_stroke(
                    point,
                    self.sim.smoothing_radius,
                    Stroke::new(2.0, Color32::LIGHT_BLUE),
                );
                let indexes = p.get_around(point.to_vec2());
                for index in indexes {
                    let particle = &self.sim.particles[index];
                    painter.circle_filled(
                        particle.pos.to_pos2(),
                        self.sim.particle_size,
                        Color32::RED,
                    );
                }
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
            a.add_float(&mut self.sim.target_density, 0.0..=0.01, "Target Density");
            a.add_drag(&mut self.sim.pressure_multiplier, "Pressure Multiplier");

            a.show(ctx, &mut self.modifiers_open);
        }

        // ctx.request_repaint_after(Duration::from_millis(500));
        ctx.request_repaint();
    }
}
impl FluidApp {
    fn create_texture(&self, ctx: &Context) -> TextureHandle {
        let size = 100;
        let pixels = vec![Color32::BLACK; size * size]
            .iter()
            .enumerate()
            .map(|(index, _c)| {
                let x = index % size;
                let y = index / size;

                let normal = Vec2::new(x as f32 / size as f32, y as f32 / size as f32);
                let pos = normal * self.sim.bounds.size() + self.sim.bounds.min.to_vec2();

                // let c = (f32::cos((pos.x * 0.020 - 3.0 + f32::sin(pos.y * 0.02))) + 1.0) * 0.5;
                // return Color32::from_gray((c * 255.0) as u8);

                let den = self.sim.calculate_density(pos);
                // return Color32::from_rgb(0, 0, (den * 255.0) as u8);
                let pressure = self.sim.convert_density_to_pressure(den)
                    / self.sim.pressure_multiplier
                    * 2550.0;
                if pressure < 0.0 {
                    return Color32::from_rgb(0, 0, pressure.abs() as u8);
                } else {
                    return Color32::from_rgb(pressure.abs() as u8, 0, 0);
                }
            })
            .collect();

        let image = egui::ColorImage::new([size, size], pixels);
        return ctx.load_texture("Density Map", image, TextureOptions::LINEAR);
    }
}
