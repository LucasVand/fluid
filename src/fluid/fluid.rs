use eframe::{
    egui::Key,
    wgpu::{BufferUsages, RenderPass},
};
use glam::{Mat4, Vec3};

use crate::{
    adjustable::Adjuster,
    fluid::{
        fluid_params::FluidParams,
        model_context::FluidModelContext,
        particle::{GpuParticle, Particle},
        render::render::FluidRenderer,
        sim::fluid_sim::FluidSim,
    },
    renderer::{
        renderable::{RenderCC, RenderContext, Renderable},
        utils::{BufferBuilder, box3d::Box3d},
    },
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
                self.mcc.particles = Self::create_box(self.mcc.particles.len(), self.mcc.bounds);
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

            a.show(rc.ctx, &mut self.modifiers_open);
        }

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
        let size = 100.0;
        let bounds =
            Box3d::from_center(Vec3::new(0.0, 0.0, 0.0), Vec3::new(size * 2.0, size, size));

        let model_mat = Fluid::model_matrix(Vec3::ZERO, Vec3::ZERO, 0.1);

        let bytes: &[u8] = &bytemuck::cast_slice(&model_mat);

        let model_buf = BufferBuilder::new(rcc.device)
            .contents_slice(bytes)
            .usages(BufferUsages::UNIFORM | BufferUsages::COPY_SRC)
            .build("Model Buf");

        let particles: Vec<Particle> = Self::create_box(2_usize.pow(14), bounds);

        let gpu_particles: Vec<GpuParticle> = particles.iter().map(|p| p.into()).collect();

        let particles_buf = BufferBuilder::new(rcc.device)
            .contents_slice(&bytemuck::cast_slice(&gpu_particles))
            .usages(BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST)
            .build("Particles Buffer");

        let mcc = FluidModelContext {
            particles: particles,
            params: FluidParams {
                target_density: 0.16,
                pressure_multiplier: 1.0,
                near_pressure_multiplier: 10.0,
                smoothing_radius: 15.0,
                gravity: 250.0,
                damping: 0.7,
                time_step: 1.0 / 120.0,
                particle_size: 2.0,
                viscosity_strength: 0.8,
                color_multiplier: 0.008,
                color_offset: 0.63,
                bounds: bounds,
                is_running: false,
            },
            bounds: bounds,
            model_buf: model_buf,
            particles_buf: particles_buf,
        };

        let renderer = FluidRenderer::new(rcc, &mcc);

        let sim = FluidSim::new(&rcc, &mcc);

        Self {
            renderer: renderer,
            sim: sim,
            mcc: mcc,
            modifiers_open: false,
        }
    }
    pub fn model_matrix(pos: Vec3, rotation: Vec3, scale: f32) -> [[f32; 4]; 4] {
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
        return model.to_cols_array_2d();
    }
    pub fn create_box(size: usize, bounds: Box3d) -> Vec<Particle> {
        let mut particles = Vec::new();

        let cube_size = f32::cbrt(size as f32).ceil() as usize;

        let particle_dist = 3.0;
        let center_offset = (cube_size as f32 * particle_dist) / 2.0;
        let center = bounds.center() - Vec3::new(center_offset, center_offset, center_offset);

        for i in 0..cube_size {
            for j in 0..cube_size {
                for k in 0..cube_size {
                    if i * cube_size * cube_size + j * cube_size + k < size {
                        particles.push(Particle::new(
                            Vec3::new(
                                j as f32 * particle_dist,
                                i as f32 * particle_dist,
                                k as f32 * particle_dist,
                            ) + center,
                            Vec3::ZERO,
                        ));
                    }
                }
            }
        }
        return particles;
    }
}
