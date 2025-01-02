use cgmath::Angle;
use cgmath::InnerSpace;
use cgmath::Rotation;

use crate::data::transform::Transform;

const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2 - 0.0001;

pub struct OrbitalCameraController {
    target: cgmath::Point3<f32>,
    distance: f32,
    yaw: cgmath::Rad<f32>,
    pitch: cgmath::Rad<f32>,
    sensitivity: f32,
    is_left_mouse_pressed: bool,
    rotation_horizontal: f32,
    rotation_vertical: f32,
}

impl OrbitalCameraController {
    pub fn new(
        target: cgmath::Point3<f32>,
        distance: f32,
        yaw: cgmath::Rad<f32>,
        pitch: cgmath::Rad<f32>,
        sensitivity: f32,
    ) -> Self {
        Self {
            target,
            distance,
            yaw,
            pitch,
            sensitivity,
            is_left_mouse_pressed: false,
            rotation_horizontal: 0.0,
            rotation_vertical: 0.0,
        }
    }

    pub fn handle_mouse_input(
        &mut self,
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
    ) {
        if button == winit::event::MouseButton::Left {
            self.is_left_mouse_pressed = state == winit::event::ElementState::Pressed;
        }
    }

    pub fn handle_mouse_movement(&mut self, delta_x: f32, delta_y: f32) {
        if !self.is_left_mouse_pressed {
            return;
        }

        self.rotation_horizontal += delta_x;
        self.rotation_vertical += delta_y;
    }

    pub fn generate_updated_camera_transform(
        &mut self,
        delta_time: std::time::Duration,
    ) -> Option<Transform> {
        match self.rotation_vertical != 0.0 || self.rotation_horizontal != 0.0 {
            true => {
                self.apply_scaled_rotation(delta_time);
                self.rotation_horizontal = 0.0;
                self.rotation_vertical = 0.0;
                Some(self.calculate_camera_transform())
            }
            false => None,
        }
    }

    fn apply_scaled_rotation(&mut self, delta_time: std::time::Duration) {
        let delta_time = delta_time.as_secs_f32();

        self.yaw += cgmath::Rad(self.rotation_horizontal) * self.sensitivity * delta_time;
        self.pitch += cgmath::Rad(self.rotation_vertical) * self.sensitivity * delta_time;

        self.pitch = cgmath::Rad(self.pitch.0.clamp(-SAFE_FRAC_PI_2, SAFE_FRAC_PI_2));
    }

    pub fn calculate_camera_transform(&self) -> Transform {
        let view_direction = cgmath::Vector3::<f32>::new(
            self.yaw.sin() * self.pitch.cos(),
            -self.pitch.sin(),
            -(self.yaw.cos() * self.pitch.cos()),
        )
        .normalize();

        let translation = self.target + (self.distance * -view_direction);

        let rotation = cgmath::Quaternion::<f32>::between_vectors(
            -(cgmath::Vector3::unit_z()),
            view_direction,
        );

        Transform {
            translation: cgmath::Vector3 {
                x: translation.x,
                y: translation.y,
                z: translation.z,
            },
            rotation,
            ..Default::default()
        }
    }
}
