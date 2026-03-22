use core::panic;
use std::{cmp::max, f32::consts::PI, mem};

use eframe::egui::{Rect, Vec2, debug_text::print};
use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};

use crate::spatial_map::SpatialMap;

pub struct FluidSim {
    pub particles: Vec<Particle>,
    pub spatial_map: SpatialMap,
    pub bounds: Rect,
    pub particle_size: f32,
    pub smoothing_radius: f32,
    pub mass: f32,
    pub gradient_step: f32,
    pub target_density: f32,
    pub pressure_multiplier: f32,
}

#[derive(Debug)]
pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub property: f32,
    pub density: f32,
    pub predicted: Vec2,
}
impl Particle {
    pub fn new(pos: Vec2, vel: Vec2) -> Self {
        Particle {
            pos: pos,
            vel: vel,
            property: 0.0,
            density: 0.0,
            predicted: pos,
        }
    }
}

impl FluidSim {
    const DAMPING: f32 = 0.7;
    const GRAVITY: Vec2 = Vec2::new(0.0, 0.0);
    pub fn new(size: usize, bounds: Rect) -> FluidSim {
        let mut parts = Self::create_box(size, bounds);
        for p in parts.iter_mut() {
            p.property = (f32::cos(p.pos.x * 0.020 - 3.0 + f32::sin(p.pos.y * 0.02)) + 1.0) * 0.5;
        }
        let smoothing_radius = 25.0;
        let mut s = Self {
            spatial_map: SpatialMap::new(bounds, smoothing_radius, parts.len()),
            particles: parts,
            bounds,
            particle_size: 3.0,
            smoothing_radius: smoothing_radius,
            mass: 1.0,
            gradient_step: 0.001,
            target_density: 0.0005,
            pressure_multiplier: 0.5,
        };
        s.update_spatial_map();
        s.update_densities();

        return s;
    }
    pub fn create_random(size: usize, bounds: Rect) -> Vec<Particle> {
        fastrand::seed(10);
        let mut particles = Vec::new();
        for _ in 0..size {
            let x = fastrand::f32() * bounds.size().x;
            let y = fastrand::f32() * bounds.size().y;
            particles.push(Particle::new(
                Vec2::new(x, y) + bounds.min.to_vec2(),
                Vec2::ZERO,
            ));
        }
        return particles;
    }
    pub fn create_box(size: usize, bounds: Rect) -> Vec<Particle> {
        let mut particles = Vec::new();

        let rect_size = f32::sqrt(size as f32).ceil() as usize;

        let particle_dist = 10.0;
        let center_offset = (rect_size as f32 * particle_dist) / 2.0;
        let center = bounds.center().to_vec2() - Vec2::new(center_offset, center_offset);

        for i in 0..rect_size {
            for j in 0..rect_size {
                if i * rect_size + j < size {
                    particles.push(Particle::new(
                        Vec2::new(j as f32 * particle_dist, i as f32 * particle_dist) + center,
                        Vec2::ZERO,
                    ));
                }
            }
        }
        return particles;
    }
    pub fn update_densities(&mut self) {
        let den: Vec<f32> = self
            .particles
            .par_iter()
            .map(|p| {
                return self.calculate_density(p.predicted);
            })
            .collect();

        den.into_iter().enumerate().for_each(|(index, d)| {
            self.particles[index].density = d;
        });
    }
    pub fn update_spatial_map(&mut self) {
        self.spatial_map
            .update_params(self.bounds, self.smoothing_radius);

        for (index, part) in self.particles.iter().enumerate() {
            self.spatial_map.insert(index, part.predicted);
        }
        self.spatial_map.finalize();
    }
    pub fn update(&mut self, delta_time: f32) {
        for part in self.particles.iter_mut() {
            part.vel += Self::GRAVITY * delta_time;

            part.predicted = part.pos + part.vel * 1.0 / 60.0;
        }

        self.update_spatial_map();

        self.update_densities();

        // pressure forces
        let forces: Vec<Vec2> = self
            .particles
            .par_iter()
            .enumerate()
            .map(|(index, _p)| {
                return self.calculate_pressure_force(index);
            })
            .collect();

        for (index, force) in forces.into_iter().enumerate() {
            let den = self.particles[index].density;

            self.particles[index].vel += force / den;
        }

        let mut parts = mem::take(&mut self.particles);
        for part in parts.iter_mut() {
            part.pos += part.vel * delta_time;
            self.collide_all_sides(part);
        }
        self.particles = parts;
    }
    fn smoothing_kernal(radius: f32, dist: f32) -> f32 {
        if dist >= radius {
            return 0.0;
        }
        let volume = PI * radius.powi(4) / 6.0;

        return (radius - dist) * (radius - dist) / volume;
    }
    fn smoothing_kernal_derivative(radius: f32, dist: f32) -> f32 {
        if dist >= radius {
            return 0.0;
        }

        let scale = 12.0 / (PI * radius.powi(4));
        return (dist - radius) * scale;
    }
    pub fn calculate_density(&self, sample: Vec2) -> f32 {
        let mut density: f32 = 0.00001;

        let sample_clamped = self.bounds.clamp(sample.to_pos2()).to_vec2();
        let possible = self.spatial_map.get_around(sample_clamped);
        let mut out_dist = Vec::new();
        for i in possible {
            let part = &self.particles[i];
            let pos = part.predicted;

            if part.predicted == sample_clamped {
                continue;
            }
            let dst = (pos - sample_clamped).length();
            out_dist.push(dst);
            let influence = Self::smoothing_kernal(self.smoothing_radius, dst);

            density += self.mass * influence;
        }
        if density == 0.00001 {
            println!("bad density: {}", sample_clamped);
            println!(
                "Out of bounds: {}",
                self.bounds.contains(sample_clamped.to_pos2())
            );
            println!("Out Dists: {:?}", out_dist);
        }

        return density;
    }

    pub fn calculate_pressure_force(&self, particle_index: usize) -> Vec2 {
        let mut pressure_force = Vec2::ZERO;
        let sample = self.particles[particle_index].predicted;
        let possible = self.spatial_map.get_around(sample);

        for i in possible {
            let p = &self.particles[i];
            let dst = (p.predicted - sample).length();

            if dst == 0.0 {
                continue;
            }
            let dir = (p.predicted - sample) / dst;
            let slope = Self::smoothing_kernal_derivative(self.smoothing_radius, dst);
            let density = p.density;

            if density == 0.0 {
                println!("den zero, {:?}", p);
                panic!("");
            }
            let shared_pressure =
                self.calculate_shared_pressure(density, self.particles[particle_index].density);
            pressure_force += shared_pressure * dir * slope * self.mass / density;
        }

        return pressure_force;
    }
    fn calculate_shared_pressure(&self, density1: f32, density2: f32) -> f32 {
        let p_a = self.convert_density_to_pressure(density1);
        let p_b = self.convert_density_to_pressure(density2);

        return (p_a + p_b) / 2.0;
    }
    pub fn convert_density_to_pressure(&self, density: f32) -> f32 {
        let density_error = density - self.target_density;
        let pressure = density_error * self.pressure_multiplier * 170.0;
        return pressure;
    }

    pub fn collide_all_sides(&self, particle: &mut Particle) {
        let half_size = self.particle_size / 2.0;

        // Check X axis independently
        let left_diff = (particle.pos.x - half_size) - self.bounds.min.x;
        if left_diff < 0.0 {
            particle.pos.x = self.bounds.min.x + half_size;
            particle.vel.x *= -1.0 * Self::DAMPING;
        }

        let right_diff = self.bounds.max.x - (particle.pos.x + half_size);
        if right_diff < 0.0 {
            particle.pos.x = self.bounds.max.x - half_size;
            particle.vel.x *= -1.0 * Self::DAMPING;
        }

        // Check Y axis independently
        let top_diff = (particle.pos.y - half_size) - self.bounds.min.y;
        if top_diff < 0.0 {
            particle.pos.y = self.bounds.min.y + half_size;
            particle.vel.y *= -1.0 * Self::DAMPING;
        }

        let bottom_diff = self.bounds.max.y - (particle.pos.y + half_size);
        if bottom_diff < 0.0 {
            particle.pos.y = self.bounds.max.y - half_size;
            particle.vel.y *= -1.0 * Self::DAMPING;
        }
    }
    pub fn set_bounds(&mut self, rect: Rect) {
        self.bounds = rect;
    }
}
