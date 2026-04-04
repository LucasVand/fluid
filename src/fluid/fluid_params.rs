use crate::renderer::utils::box3d::Box3d;

// let mcc = FluidModelContext {
//      particles: Vec::new(),
//          target_density: 0.08,
//          pressure_multiplier: 1.0,
//          near_pressure_multiplier: 1.0,
//          smoothing_radius: 15.0,
//          gravity: 250.0,
//          damping: 0.7,
//          time_step: 1.0 / 120.0,
//          particle_size: 2.0,
//          viscosity_strength: 1.0,
//          _pad2: [0.0; 3],
//          bounds_min: bounds.min,
//          _pad0: 0.0,
//          bounds_max: bounds.max,
//          _pad1: 0.0,
//          color_multiplier: 0.08,
//          color_offset: 0.63,
//          particle_size: 2.0,
//      },
//      bounds: bounds,
//      model_buf: model_buf,
//  };
pub struct FluidParams {
    pub target_density: f32,
    pub pressure_multiplier: f32,
    pub near_pressure_multiplier: f32,
    pub smoothing_radius: f32,

    pub gravity: f32,
    pub damping: f32,
    pub time_step: f32,
    pub particle_size: f32,

    pub viscosity_strength: f32,
    pub bounds: Box3d,
    pub color_multiplier: f32,
    pub color_offset: f32,

    pub is_running: bool,
}
