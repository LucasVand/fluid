use glam::Vec3;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuParticle {
    pub position: Vec3,
    pub _pad0: f32,

    pub predicted_position: Vec3,
    pub _pad1: f32,

    pub velocity: Vec3,
    pub _pad2: f32,

    pub density: f32,
    pub near_density: f32,
    pub is_boundry: u32,
    pub _pad3: [f32; 1],
}

impl From<&Particle> for GpuParticle {
    fn from(p: &Particle) -> Self {
        GpuParticle {
            position: p.pos,
            predicted_position: p.predicted,
            velocity: p.vel,
            density: p.density.0,
            near_density: p.density.1,
            is_boundry: if p.is_boundary { 1 } else { 0 },
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
            _pad3: [0.0; 1],
        }
    }
}

impl From<&GpuParticle> for Particle {
    fn from(gp: &GpuParticle) -> Self {
        let mut p = Particle::new(gp.position, gp.velocity);
        p.predicted = gp.predicted_position;
        p.density = (gp.density, gp.near_density);
        p.is_boundary = gp.is_boundry != 0;
        p
    }
}

#[derive(Debug)]
pub struct Particle {
    pub pos: Vec3,
    pub vel: Vec3,
    pub density: (f32, f32),
    pub predicted: Vec3,
    pub is_boundary: bool,
}
impl Particle {
    pub fn new(pos: Vec3, vel: Vec3) -> Self {
        Particle {
            pos: pos,
            vel: vel,
            density: (0.0, 0.0),
            predicted: pos,
            is_boundary: false,
        }
    }
}
