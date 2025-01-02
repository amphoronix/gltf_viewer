pub struct PerspectiveCameraProjection {
    pub aspect_ratio: Option<f32>,
    pub fovy: cgmath::Rad<f32>,
    pub znear: f32,
    pub zfar: f32,
}
