use cgmath::{InnerSpace, Zero};

#[derive(Copy, Clone)]
pub struct Transform {
    pub translation: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub scale: cgmath::Vector3<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: cgmath::Vector3::zero(),
            rotation: cgmath::Quaternion::zero(),
            scale: cgmath::Vector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        }
    }
}

impl From<Transform> for cgmath::Matrix4<f32> {
    fn from(value: Transform) -> Self {
        cgmath::Matrix4::from_translation(value.translation)
            * cgmath::Matrix4::from(value.rotation)
            * cgmath::Matrix4::from_nonuniform_scale(value.scale.x, value.scale.y, value.scale.z)
    }
}

impl From<cgmath::Matrix4<f32>> for Transform {
    fn from(value: cgmath::Matrix4<f32>) -> Self {
        let translation = cgmath::Vector3::<f32>::new(value.w.x, value.w.y, value.w.z);

        let mut rotation_matrix = cgmath::Matrix3::<f32>::new(
            value.x.x, value.x.y, value.x.z, value.y.x, value.y.y, value.y.z, value.z.x, value.z.y,
            value.z.z,
        );

        let scale_x = rotation_matrix.x.magnitude();
        let scale_y = rotation_matrix.y.magnitude();
        let scale_z = rotation_matrix.z.magnitude();
        let scale = cgmath::Vector3::<f32>::new(scale_x, scale_y, scale_z);

        rotation_matrix.x *= 1.0 / scale_x;
        rotation_matrix.y *= 1.0 / scale_y;
        rotation_matrix.z *= 1.0 / scale_z;

        let rotation = cgmath::Quaternion::from(rotation_matrix);

        Self {
            translation,
            rotation,
            scale,
        }
    }
}
