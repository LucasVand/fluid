use std::{f32::consts::PI, mem};

use glam::Vec3;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::{renderer::utils::box3d::Box3d, spatial_map::SpatialMap};

pub struct FluidSim {
    pub particles: Vec<Particle>,
    pub spatial_map: SpatialMap,
    pub bounds: Box3d,
    pub particle_size: f32,
    pub smoothing_radius: f32,
    pub mass: f32,
    pub gradient_step: f32,
    pub target_density: f32,
    pub pressure_multiplier: f32,
    pub near_pressure_multiplier: f32,
    pub running: bool,
    pub gravity: f32,
    pub viscosity_strength: f32,
}

#[derive(Debug)]
pub struct Particle {
    pub pos: Vec3,
    pub vel: Vec3,
    pub property: f32,
    pub density: (f32, f32),
    pub predicted: Vec3,
}
impl Particle {
    pub fn new(pos: Vec3, vel: Vec3) -> Self {
        Particle {
            pos: pos,
            vel: vel,
            property: 0.0,
            density: (0.0, 0.0),
            predicted: pos,
        }
    }
}

impl FluidSim {
    const DAMPING: f32 = 0.7;
    pub fn new(size: usize, bounds: Box3d) -> FluidSim {
        let mut parts = Self::create_box(size, bounds);
        for p in parts.iter_mut() {
            p.property = (f32::cos(p.pos.x * 0.020 - 3.0 + f32::sin(p.pos.y * 0.02)) + 1.0) * 0.5;
        }
        let smoothing_radius = 40.0;
        let mut s = Self {
            gravity: 250.0,
            spatial_map: SpatialMap::new(smoothing_radius, parts.len()),
            particles: parts,
            bounds,
            particle_size: 2.0,
            smoothing_radius: smoothing_radius,
            mass: 1.0,
            gradient_step: 0.001,
            target_density: 0.014,
            pressure_multiplier: 1000.0,
            near_pressure_multiplier: 0.01,
            running: false,
            viscosity_strength: 100.0,
        };
        s.update_spatial_map();
        s.update_densities();

        return s;
    }
    pub fn toggle_running(&mut self) {
        self.running = !self.running;
    }
    pub fn stop(&mut self) {
        self.running = false;
    }
    pub fn start(&mut self) {
        self.running = true;
    }
    pub fn apply_force(&mut self, pos: Vec3, radius: f32, strength: f32) {
        for p in self.particles.iter_mut() {
            let offset = pos - p.pos;
            let dst_sq = offset.length_squared();
            if dst_sq < radius * radius {
                let dst = dst_sq.sqrt();
                let dir = if dst < f32::EPSILON {
                    Vec3::ZERO
                } else {
                    offset / dst
                };

                let center_t = dst / radius;
                let force = (dir * strength - p.vel) * center_t;

                p.vel += force / self.mass;
            }
        }
    }
    pub fn create_random(size: usize, bounds: Box3d) -> Vec<Particle> {
        fastrand::seed(10);
        let mut particles = Vec::new();
        for _ in 0..size {
            let x = fastrand::f32() * bounds.size().x;
            let y = fastrand::f32() * bounds.size().y;
            particles.push(Particle::new(Vec3::new(x, y, 0.0) + bounds.min, Vec3::ZERO));
        }
        return particles;
    }
    pub fn create_box(size: usize, bounds: Box3d) -> Vec<Particle> {
        let mut particles = Vec::new();

        let cube_size = f32::cbrt(size as f32).ceil() as usize;

        let particle_dist = 5.0;
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

    pub fn create_box_2d(size: usize, bounds: Box3d) -> Vec<Particle> {
        let mut particles = Vec::new();

        let grid_size = f32::sqrt(size as f32).ceil() as usize;

        let particle_dist = 5.0;
        let center_offset = (grid_size as f32 * particle_dist) / 2.0;
        let center = bounds.center() - Vec3::new(center_offset, center_offset, 0.0);

        for i in 0..grid_size {
            for j in 0..grid_size {
                if i * grid_size + j < size {
                    particles.push(Particle::new(
                        Vec3::new(j as f32 * particle_dist, i as f32 * particle_dist, 0.0) + center,
                        Vec3::ZERO,
                    ));
                }
            }
        }
        return particles;
    }
    pub fn update_densities(&mut self) {
        let den: Vec<(f32, f32)> = self
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
        self.spatial_map.update_params(self.smoothing_radius);

        for (index, part) in self.particles.iter().enumerate() {
            self.spatial_map.insert(index, part.predicted);
        }
        self.spatial_map.finalize();
    }
    pub fn update(&mut self, delta_time: f32) {
        if !self.running {
            return;
        }
        for part in self.particles.iter_mut() {
            part.vel += Vec3::new(0.0, -1.0, 0.0) * self.gravity * delta_time;

            part.predicted = part.pos + part.vel * 1.0 / 60.0;
        }

        self.update_spatial_map();

        self.update_densities();

        // pressure forces
        let forces: Vec<Vec3> = self
            .particles
            .par_iter()
            .enumerate()
            .map(|(index, _p)| {
                return self.calculate_pressure_force(index);
            })
            .collect();

        for (index, force) in forces.into_iter().enumerate() {
            let den = self.particles[index].density;

            self.particles[index].vel += (force / den.0) * delta_time;
        }

        // viscosity force
        let visc_forces: Vec<Vec3> = self
            .particles
            .par_iter()
            .enumerate()
            .map(|(index, _p)| {
                return self.calculate_viscosity_force(index);
            })
            .collect();

        for (index, force) in visc_forces.into_iter().enumerate() {
            self.particles[index].vel += force * delta_time;
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
    fn viscosity_smoothing_kernal(radius: f32, dist: f32) -> f32 {
        if dist >= radius {
            return 0.0;
        }
        let volume = PI * radius.powi(8) / 4.0;
        let value = f32::max(0.0, radius * radius - dist * dist);
        return value.powi(3) / volume;
    }
    fn near_density_smoothing_kernal(radius: f32, dist: f32) -> f32 {
        if dist >= radius {
            return 0.0;
        }

        let volume = PI * radius.powi(8) / 4.0;

        let v: f32 = radius - dist;
        return v * v * v;
    }
    fn near_density_smoothing_kernal_derivative(radius: f32, dist: f32) -> f32 {
        if dist >= radius {
            return 0.0;
        }

        let scale = 12.0 / (PI * radius.powi(4));
        let v: f32 = radius - dist;
        return -v * v;
    }
    pub fn calculate_density(&self, sample: Vec3) -> (f32, f32) {
        let mut density: f32 = 0.00001;
        let mut near_density: f32 = 0.00001;

        let possible = self.spatial_map.get_around(sample);
        let mut out_dist = Vec::new();
        for i in possible {
            let part = &self.particles[i];
            let pos = part.predicted;

            if part.predicted == sample {
                continue;
            }
            let dst = (pos - sample).length();
            out_dist.push(dst);
            let influence = Self::smoothing_kernal(self.smoothing_radius, dst);
            let near_influence = Self::near_density_smoothing_kernal(self.smoothing_radius, dst);

            density += self.mass * influence;
            near_density += self.mass * near_influence;
        }
        if density == 0.00001 {
            println!("bad density: {}", sample);
            println!("Out Dists: {:?}", out_dist);
        }

        return (density, near_density);
    }

    pub fn calculate_pressure_force(&self, particle_index: usize) -> Vec3 {
        let mut pressure_force = Vec3::ZERO;
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
            let slope_near =
                Self::near_density_smoothing_kernal_derivative(self.smoothing_radius, dst);
            let density = p.density;

            let shared_pressure =
                self.calculate_shared_pressure(density, self.particles[particle_index].density);
            pressure_force += shared_pressure.0 * dir * slope * self.mass / density.0;
            pressure_force += shared_pressure.1 * dir * slope_near * self.mass / density.1;
        }

        return pressure_force;
    }
    fn calculate_shared_pressure(&self, density1: (f32, f32), density2: (f32, f32)) -> (f32, f32) {
        let p_a = self.convert_density_to_pressure(density1.0, density1.1);
        let p_b = self.convert_density_to_pressure(density2.0, density2.1);

        return ((p_a.0 + p_b.0) / 2.0, (p_a.1 + p_b.1) / 2.0);
    }
    pub fn convert_density_to_pressure(&self, density: f32, near_density: f32) -> (f32, f32) {
        let density_error = density - self.target_density;
        let pressure = density_error * self.pressure_multiplier * 170.0;
        let near_pressure = near_density * self.near_pressure_multiplier;
        return (pressure, near_pressure);
    }
    fn calculate_viscosity_force(&self, particle_index: usize) -> Vec3 {
        let mut vis_force: Vec3 = Vec3::ZERO;
        let i_pos: Vec3 = self.particles[particle_index].predicted;
        let i_vel: Vec3 = self.particles[particle_index].vel;
        for n_index in self.spatial_map.get_around(i_pos) {
            let particle = &self.particles[n_index];

            let dst = (i_pos - particle.predicted).length();
            if dst == 0.0 {
                continue;
            }
            let influence = Self::viscosity_smoothing_kernal(self.smoothing_radius, dst);

            vis_force += (particle.vel - i_vel) * influence;
        }
        return vis_force * self.viscosity_strength;
    }

    pub fn collide_all_sides(&self, particle: &mut Particle) {
        let half_size = self.particle_size / 2.0;

        // Check X axis
        if particle.pos.x - half_size < self.bounds.min.x {
            particle.pos.x = self.bounds.min.x + half_size;
            particle.vel.x *= -1.0 * Self::DAMPING;
        }
        if particle.pos.x + half_size > self.bounds.max.x {
            particle.pos.x = self.bounds.max.x - half_size;
            particle.vel.x *= -1.0 * Self::DAMPING;
        }

        // Check Y axis
        if particle.pos.y - half_size < self.bounds.min.y {
            particle.pos.y = self.bounds.min.y + half_size;
            particle.vel.y *= -1.0 * Self::DAMPING;
        }
        if particle.pos.y + half_size > self.bounds.max.y {
            particle.pos.y = self.bounds.max.y - half_size;
            particle.vel.y *= -1.0 * Self::DAMPING;
        }

        // Check Z axis
        if particle.pos.z - half_size < self.bounds.min.z {
            particle.pos.z = self.bounds.min.z + half_size;
            particle.vel.z *= -1.0 * Self::DAMPING;
        }
        if particle.pos.z + half_size > self.bounds.max.z {
            particle.pos.z = self.bounds.max.z - half_size;
            particle.vel.z *= -1.0 * Self::DAMPING;
        }
    }
    pub fn set_bounds(&mut self, rect: Box3d) {
        self.bounds = rect;
    }
}
