use anyhow::Result;
use cgmath::Zero;

use crate::camera::OrbitalCameraController;
use crate::render::RenderSystem;

pub struct ViewSystem {
    pub window: std::sync::Arc<winit::window::Window>,
    pub render_system: RenderSystem,
    pub camera_controller: OrbitalCameraController,
}

impl ViewSystem {
    pub async fn from_window(window: winit::window::Window) -> Result<Self> {
        let window = std::sync::Arc::new(window);

        let mut render_system = RenderSystem::from_window(window.clone()).await?;

        let camera_controller = OrbitalCameraController::new(
            (0.0, 0.0, 0.0).into(),
            10.0,
            cgmath::Rad::<f32>::zero(),
            cgmath::Rad::<f32>::zero(),
            2.0,
        );

        render_system.set_user_camera_transform(camera_controller.calculate_camera_transform());

        Ok(Self {
            window,
            render_system,
            camera_controller,
        })
    }

    pub fn update_view(&mut self, delta_time: std::time::Duration) -> Result<()> {
        if let Some(transform) = self
            .camera_controller
            .generate_updated_camera_transform(delta_time)
        {
            self.render_system.set_user_camera_transform(transform);
        }

        self.render_system.render()?;

        Ok(())
    }
}
