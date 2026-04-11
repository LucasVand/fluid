use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3, Vec4};

pub struct Camera {
    pub position: Vec3, // 12
    pub forwards: Vec3, // 12
    pub right: Vec3,    // 12
    pub up: Vec3,       // 12
    yaw: f32,           // 4
    pitch: f32,         // 4
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuCamera {
    matrix: [[f32; 4]; 4], // 64
    position: Vec3,        // 12
    _pad: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuCameraNew {
    matrix: [[f32; 4]; 4],   // 64
    inv_proj: [[f32; 4]; 4], // 64
    inv_view: [[f32; 4]; 4], // 64
    position: Vec3,          // 12
    _pad: f32,
}

const MOVEMENT_AMOUNT: f32 = 0.7;
const ROTATE_AMOUNT: f32 = 0.2;
impl Camera {
    pub fn new() -> Self {
        let position = Vec3::new(-70.0, 0.0, -70.0);
        let yaw: f32 = 0.0;
        let pitch: f32 = 0.0;

        let forwards = Vec3::new(1.0, 0.0, 0.0);
        let right = Vec3::new(0.0, 0.0, 1.0);
        let up = Vec3::new(0.0, 1.0, 0.0);

        Camera {
            position,
            yaw,
            pitch,
            forwards,
            right,
            up,
        }
    }

    pub fn spin(&mut self, d_yaw: f32, d_pitch: f32) {
        self.yaw = self.yaw + d_yaw * ROTATE_AMOUNT;
        if self.yaw > 360.0 {
            self.yaw = self.yaw - 360.0;
        }
        if self.yaw < 0.0 {
            self.yaw = self.yaw + 360.0;
        }

        self.pitch = f32::min(89.0, f32::max(-89.0, self.pitch + d_pitch * ROTATE_AMOUNT));

        let c = f32::cos(f32::to_radians(self.yaw));
        let s = f32::sin(f32::to_radians(self.yaw));
        let c2 = f32::cos(f32::to_radians(self.pitch));
        let s2 = f32::sin(f32::to_radians(self.pitch));

        self.forwards.x = c * c2;
        self.forwards.z = s * c2;
        self.forwards.y = s2;

        self.up.x = 0.0;
        self.up.y = 1.0;
        self.up.z = 0.0;

        self.right = self.forwards.cross(self.up).normalize();

        self.up = self.right.cross(self.forwards).normalize();
    }
    pub fn rotate_about(&mut self, d_yaw: f32, d_pitch: f32, point: Vec3) {
        self.spin(-d_yaw, -d_pitch);

        let c = f32::cos(f32::to_radians(self.yaw));
        let s = f32::sin(f32::to_radians(self.yaw));
        let c2 = f32::cos(f32::to_radians(self.pitch));
        let s2 = f32::sin(f32::to_radians(self.pitch));

        let mut point_dir = point - self.position;

        let distance = point_dir.length();

        point_dir = point_dir.normalize();

        point_dir.x = c * c2;
        point_dir.z = s * c2;
        point_dir.y = s2;

        self.position = point;
        self.position -= point_dir * distance;
    }
    pub fn move_towards(&mut self, d: f32, point: Vec3) {
        let point_dir = (point - self.position).normalize();

        self.position += point_dir * d * MOVEMENT_AMOUNT;
    }

    pub fn walk(&mut self, d_right: f32, d_forwards: f32) {
        // i want to move with the angle too
        //
        // let y: f32 = self.position.y;
        self.position = self.position
            + self.right * d_right * MOVEMENT_AMOUNT
            + self.forwards * d_forwards * MOVEMENT_AMOUNT;
        // self.position.y = y;
    }

    pub fn get_projection(&self) -> Mat4 {
        let (proj, view) = self.get_proj_view();

        return proj * view;
    }
    pub fn get_proj_view(&self) -> (Mat4, Mat4) {
        let c0 = Vec4::new(self.right.x, self.up.x, -self.forwards.x, 0.0);
        let c1 = Vec4::new(self.right.y, self.up.y, -self.forwards.y, 0.0);
        let c2 = Vec4::new(self.right.z, self.up.z, -self.forwards.z, 0.0);

        let a: f32 = -self.right.dot(self.position);
        let b: f32 = -self.up.dot(self.position);
        let c: f32 = self.forwards.dot(self.position);
        let c3 = Vec4::new(a, b, c, 1.0);
        let view = Mat4::from_cols(c0, c1, c2, c3);

        let fov_y: f32 = f32::to_radians(80.0);
        let aspect = 1200.0 / 700.0;
        let z_near = 0.1;
        let z_far = 100.0;
        let projection = Mat4::perspective_rh_gl(fov_y, aspect, z_near, z_far);

        (projection, view)
    }
    pub fn to_gpu(&self) -> GpuCamera {
        GpuCamera {
            matrix: self.get_projection().to_cols_array_2d(),
            position: self.position,
            _pad: 0.0,
        }
    }
    pub fn to_gpu_new(&self) -> GpuCameraNew {
        let m = self.get_projection();
        let (proj, view) = self.get_proj_view();
        let inv_proj = proj.inverse();
        let inv_view = view.inverse();
        GpuCameraNew {
            matrix: m.to_cols_array_2d(),
            inv_proj: inv_proj.to_cols_array_2d(),
            inv_view: inv_view.to_cols_array_2d(),
            position: self.position,
            _pad: 0.0,
        }
    }
}
