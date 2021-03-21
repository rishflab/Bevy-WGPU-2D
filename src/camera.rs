use std::f32;

use glam::{Mat4, Vec3, Vec4};

use crate::app::WINDOW_SIZE;
use crate::renderer::gpu_primitives::CameraUniform;
use crate::renderer::sprite::PIXELS_PER_METRE;

pub const SPRITE_SCALING_FACTOR: u8 = 2;

pub struct ActiveCamera;

pub trait Camera {
    fn generate_matrix(&self) -> CameraUniform;
}

#[derive(Clone, Copy)]
pub struct ParallaxCamera {
    pub eye: glam::Vec3,
    pub look_dir: glam::Vec3,
    pub fov_y: f32,
    pub near: f32,
    pub far: f32,
}

impl ParallaxCamera {
    pub fn new(eye: glam::Vec3, look_to: glam::Vec3, fov_y: f32, near: f32, far: f32) -> Self {
        ParallaxCamera {
            eye,
            look_dir: look_to,
            fov_y,
            near,
            far,
        }
    }
    pub fn generate_ortho(&self) -> glam::Mat4 {
        let h =
            (WINDOW_SIZE.height as f32) / (SPRITE_SCALING_FACTOR as f32 * PIXELS_PER_METRE as f32);
        let w =
            (WINDOW_SIZE.width as f32) / (SPRITE_SCALING_FACTOR as f32 * PIXELS_PER_METRE as f32);

        let mx_ortho =
            glam::Mat4::orthographic_lh(-w / 2.0, w / 2.0, -h / 2.0, h / 2.0, self.near, self.far);

        let mx_view = look_to_lh(self.eye, self.look_dir, Vec3::unit_y());

        mx_ortho * mx_view
    }

    pub fn generate_perspective(&self) -> glam::Mat4 {
        let mx_perspective = glam::Mat4::perspective_lh(
            self.fov_y,
            WINDOW_SIZE.width as f32 / WINDOW_SIZE.height as f32,
            self.near,
            self.far,
        );

        let mx_view = look_to_lh(self.eye, self.look_dir, Vec3::unit_y());

        mx_perspective * mx_view
    }
}

impl Camera for ParallaxCamera {
    fn generate_matrix(&self) -> CameraUniform {
        let ortho = *self.generate_ortho().as_ref();
        let persp = *self.generate_perspective().as_ref();
        CameraUniform { ortho, persp }
    }
}

fn look_to_lh(eye: Vec3, dir: Vec3, up: Vec3) -> Mat4 {
    let f = dir.normalize();
    let s = up.cross(f).normalize();
    let u = f.cross(s);
    let (fx, fy, fz) = f.into();
    let (sx, sy, sz) = s.into();
    let (ux, uy, uz) = u.into();
    Mat4::from_cols(
        Vec4::new(sx, ux, fx, 0.0),
        Vec4::new(sy, uy, fy, 0.0),
        Vec4::new(sz, uz, fz, 0.0),
        Vec4::new(-s.dot(eye), -u.dot(eye), -f.dot(eye), 1.0),
    )
}
