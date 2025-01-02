#[derive(Copy, Clone)]
pub struct PerspectiveProjection {
    pub aspect_ratio: f32,
    pub fovy: cgmath::Rad<f32>,
    pub znear: f32,
    pub zfar: f32,
}

impl From<PerspectiveProjection> for cgmath::Matrix4<f32> {
    fn from(value: PerspectiveProjection) -> Self {
        OPENGL_TO_WGPU_MATRIX
            * cgmath::perspective(value.fovy, value.aspect_ratio, value.znear, value.zfar)
    }
}

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.0, 0.0, 0.0, 1.0,
);
