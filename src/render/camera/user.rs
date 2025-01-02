use crate::data::transform::Transform;
use crate::render::camera::Camera;

pub struct UserCamera {
    pub camera: std::rc::Rc<Camera>,
    pub transform: Transform,
}

impl UserCamera {
    pub fn create_view_matrix(&self) -> cgmath::Matrix4<f32> {
        Camera::create_view_matrix_from_transform(self.transform)
    }
}
