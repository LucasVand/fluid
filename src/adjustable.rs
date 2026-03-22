use std::{mem, ops::RangeInclusive};

use eframe::egui::{self, CentralPanel, Context, DragValue, Key, Pos2, Slider, Vec2, ViewportId};

pub struct Adjuster<'a> {
    pub float_values: Vec<(String, &'a mut f32, RangeInclusive<f32>)>,
    pub drag_values: Vec<(String, &'a mut f32)>,
}

impl<'a> Adjuster<'a> {
    pub fn new() -> Self {
        Self {
            float_values: Vec::new(),
            drag_values: Vec::new(),
        }
    }
    pub fn add_float(
        &mut self,
        value: &'a mut f32,
        range: RangeInclusive<f32>,
        label: impl AsRef<str>,
    ) {
        self.float_values
            .push((label.as_ref().to_string(), value, range));
    }
    pub fn add_drag(&mut self, value: &'a mut f32, label: impl AsRef<str>) {
        self.drag_values.push((label.as_ref().to_string(), value));
    }
    pub fn show(&mut self, ctx: &Context, shown: &mut bool) {
        if !*shown {
            return;
        }

        let builder = egui::ViewportBuilder::default()
            .with_title("Modifiers")
            .with_position(Pos2::new(0.0, 0.0))
            .with_inner_size(Vec2::new(200.0, 300.0));

        let id = ViewportId::from_hash_of("Modifiers");
        ctx.show_viewport_immediate(id, builder, |ctx, _class| {
            let values = mem::take(&mut self.float_values);
            let drag_values = mem::take(&mut self.drag_values);
            if ctx.input(|i| i.key_pressed(Key::M)) {
                *shown = !*shown;
            }

            CentralPanel::default().show(ctx, |ui| {
                if ui.input(|i| i.viewport().close_requested()) {
                    *shown = false;
                }

                for (s, v, r) in values.into_iter() {
                    ui.label(s);
                    ui.add(Slider::new(v, r));
                }
                for (s, v) in drag_values.into_iter() {
                    ui.label(s);
                    ui.add(DragValue::new(v));
                }
            });
        });
    }
}
