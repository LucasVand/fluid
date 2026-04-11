use eframe::{egui::Key, wgpu::RenderPass};
use glam::{Mat4, Vec3};

use crate::{
    adjustable::Adjuster,
    fluid::{
        fluid_spawner::create_box, model_context::FluidModelContext,
        render::ray_march_render::FluidRenderer, sim::fluid_sim::FluidSim,
    },
    renderer::renderable::{RenderCC, RenderContext, Renderable},
};

pub struct Fluid {
    renderer: FluidRenderer,
    sim: FluidSim,
    mcc: FluidModelContext,
    modifiers_open: bool,
}

impl Renderable for Fluid {
    fn render(&mut self, pass: &mut RenderPass, rc: &RenderContext) {
        rc.ctx.input(|i| {
            if i.key_pressed(Key::ArrowRight) {
                self.sim.update_params(&self.mcc.params);
                self.sim.update(rc, &mut self.mcc);
            }
            if i.key_pressed(Key::Space) {
                self.mcc.params.is_running = !self.mcc.params.is_running;
            }
            if i.key_pressed(Key::R) {
                self.mcc.particles = create_box(self.mcc.particles.len(), self.mcc.bounds, 5.0);
                self.sim.upload_particles(&self.mcc.particles);
            }
            if i.key_pressed(Key::M) {
                self.modifiers_open = !self.modifiers_open;
            }
        });
        {
            let mut a: Adjuster<'_> = Adjuster::new();
            a.add_float(
                &mut self.mcc.params.smoothing_radius,
                0.0..=200.0,
                "Smoothing Radius",
            );
            a.add_float(
                &mut self.mcc.params.particle_size,
                0.0..=50.0,
                "Particle Size",
            );

            a.add_drag(&mut self.mcc.params.target_density, "Target Density");
            a.add_drag(
                &mut self.mcc.params.pressure_multiplier,
                "Pressure Multiplier",
            );
            a.add_drag(
                &mut self.mcc.params.near_pressure_multiplier,
                "Near Pressure Multiplier",
            );
            a.add_drag(
                &mut self.mcc.params.viscosity_strength,
                "Viscosity Strength",
            );
            // a.add_drag(
            //     &mut self.mcc.params.boundary_density_multiplier,
            //     "Boundary Density Multiplier",
            // );
            a.add_drag(&mut self.mcc.params.color_multiplier, "Color Multiplier");
            a.add_float(&mut self.mcc.params.color_offset, 0.0..=1.0, "Color Offset");
            // a.add_drag(&mut self.radius, "Force Radius");
            // a.add_drag(&mut self.strength, "Force Strength");
            a.add_drag(&mut self.mcc.params.gravity, "Gravity Strength");
            a.add_drag(
                &mut self.mcc.params.render_density_multiplier,
                "Render Multiplier",
            );

            a.add_float(
                &mut self.mcc.params.red_scattering,
                0.0..=1.0,
                "Red Scattering",
            );

            a.add_float(
                &mut self.mcc.params.blue_scattering,
                0.0..=1.0,
                "Blue Scattering",
            );
            a.add_float(
                &mut self.mcc.params.green_scattering,
                0.0..=1.0,
                "Green Scattering",
            );

            a.show(rc.ctx, &mut self.modifiers_open);
        }

        self.mcc.params.time_step = rc.dt;
        self.sim.update_params(&self.mcc.params);
        if self.mcc.params.is_running {
            self.sim.update(rc, &mut self.mcc);
        }

        self.renderer.update_params(&self.mcc.params);
        self.renderer.draw_particles(pass);
    }
}

impl Fluid {
    pub fn new(rcc: &RenderCC) -> Self {
        let mcc = FluidModelContext::new(rcc);

        let renderer = FluidRenderer::new(rcc, &mcc);

        let sim = FluidSim::new(&rcc, &mcc);

        Self {
            renderer: renderer,
            sim: sim,
            mcc: mcc,
            modifiers_open: false,
        }
    }
    pub fn model_matrix(pos: Vec3, rotation: Vec3, scale: f32) -> Mat4 {
        // position, rotation, and scale
        let position = pos;
        let rotation = rotation;
        let scale = Vec3::splat(scale);

        // Translation
        let translate = Mat4::from_translation(position);

        // Rotation (yaw, pitch, roll)
        let rotate = Mat4::from_rotation_y(rotation.y)
            * Mat4::from_rotation_x(rotation.x)
            * Mat4::from_rotation_z(rotation.z);

        // Scale
        let scale = Mat4::from_scale(scale);

        // Combine to get model matrix
        let model = translate * rotate * scale;
        return model;
    }
}
