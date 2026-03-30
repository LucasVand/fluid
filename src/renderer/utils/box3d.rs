use glam::Vec3;

#[derive(Copy, Clone)]
pub struct Box3d {
    pub min: Vec3,
    pub max: Vec3,
}

impl Box3d {
    /// Create a box from min and max corners
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self {
            min: min.min(max),
            max: max.max(min),
        }
    }

    /// Create a box from center point and half-extents
    pub fn from_center(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    /// Create a box from position and size (position is min corner)
    pub fn from_pos_size(pos: Vec3, size: Vec3) -> Self {
        Self {
            min: pos,
            max: pos + size,
        }
    }

    /// Get the center point of the box
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Get the size (extents) of the box
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// Get the half-extents of the box
    pub fn half_extents(&self) -> Vec3 {
        self.size() * 0.5
    }

    /// Check if a point is inside the box
    pub fn contains(&self, point: Vec3) -> bool {
        point.cmpge(self.min).all() && point.cmple(self.max).all()
    }

    /// Check if this box intersects with another box
    pub fn intersects(&self, other: &Box3d) -> bool {
        self.min.cmple(other.max).all() && self.max.cmpge(other.min).all()
    }

    /// Expand the box to contain a point
    pub fn expand_to_contain(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    /// Expand the box to contain another box
    pub fn expand_to_contain_box(&mut self, other: &Box3d) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }

    /// Translate the box by an offset
    pub fn translate(&self, offset: Vec3) -> Self {
        Self {
            min: self.min + offset,
            max: self.max + offset,
        }
    }

    /// Scale the box around its center
    pub fn scale(&self, scale: f32) -> Self {
        let center = self.center();
        let half_extents = self.half_extents() * scale;
        Self::from_center(center, half_extents)
    }

    /// Get the closest point on the box to a given point
    pub fn closest_point(&self, point: Vec3) -> Vec3 {
        point.clamp(self.min, self.max)
    }

    /// Calculate distance from a point to the box (0 if inside)
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        let closest = self.closest_point(point);
        (point - closest).length()
    }
}
