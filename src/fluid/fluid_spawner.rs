use glam::Vec3;

use crate::{fluid::particle::Particle, renderer::utils::box3d::Box3d};

pub fn create_box(size: usize, bounds: Box3d) -> Vec<Particle> {
    let mut particles = Vec::new();

    let cube_size = f32::cbrt(size as f32).ceil() as usize;

    let particle_dist = 3.0;
    let center_offset = (cube_size as f32 * particle_dist) / 2.0;
    let center = bounds.center() - Vec3::new(center_offset * 2.0, center_offset, center_offset);

    for i in 0..cube_size {
        for j in 0..cube_size {
            for k in 0..cube_size {
                if i * cube_size * cube_size + j * cube_size + k < size {
                    particles.push(Particle::new(
                        Vec3::new(
                            j as f32 * particle_dist * 2.0,
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
