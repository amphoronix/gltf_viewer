pub mod projection;
pub mod user;

use cgmath::Rotation;

use crate::data::projection::PerspectiveProjection;
use crate::data::transform::Transform;
use crate::render::camera::projection::PerspectiveCameraProjection;

pub struct Camera {
    pub projection: PerspectiveCameraProjection,
}

impl Camera {
    pub fn create_projection_matrix(&self, aspect_ratio: f32) -> PerspectiveProjection {
        let aspect_ratio = match self.projection.aspect_ratio {
            Some(aspect_ratio) => aspect_ratio,
            None => aspect_ratio,
        };

        PerspectiveProjection {
            aspect_ratio,
            fovy: self.projection.fovy,
            znear: self.projection.znear,
            zfar: self.projection.zfar,
        }
    }

    pub fn create_view_matrix_from_transform(transform: Transform) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::look_to_rh(
            cgmath::Point3 {
                x: transform.translation.x,
                y: transform.translation.y,
                z: transform.translation.z,
            },
            transform.rotation.rotate_vector(-cgmath::Vector3::unit_z()),
            cgmath::Vector3::unit_y(),
        )
    }

    pub fn create_view_matrix_from_transform_matrix(
        transform_matrix: cgmath::Matrix4<f32>,
    ) -> cgmath::Matrix4<f32> {
        Camera::create_view_matrix_from_transform(transform_matrix.into())
    }
}

pub struct CameraInstance {
    pub camera: std::rc::Rc<Camera>,
    pub global_transform_matrix: cgmath::Matrix4<f32>,
}

impl CameraInstance {
    pub fn create_view_matrix(&self) -> cgmath::Matrix4<f32> {
        Camera::create_view_matrix_from_transform_matrix(self.global_transform_matrix)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub position: [f32; 3],
    _padding: u32,
    pub view_projection_matrix: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new(
        position: cgmath::Point3<f32>,
        view_projection_matrix: cgmath::Matrix4<f32>,
    ) -> Self {
        Self {
            position: position.into(),
            _padding: 0,
            view_projection_matrix: view_projection_matrix.into(),
        }
    }
}
