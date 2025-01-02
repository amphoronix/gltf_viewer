use crate::data::transform::Transform;
use crate::render::camera::CameraInstance;
use crate::render::mesh::MeshInstance;

pub struct RenderNode {
    pub id: usize,
    #[allow(dead_code)]
    pub local_transform: Transform,
    #[allow(dead_code)]
    pub children: Vec<std::rc::Rc<RenderNode>>,
    pub mesh: Option<MeshInstance>,
    pub camera: Option<CameraInstance>,
}

impl RenderNode {
    pub fn new(
        id: usize,
        local_transform: Transform,
        children: Vec<std::rc::Rc<RenderNode>>,
        mesh: Option<MeshInstance>,
        camera: Option<CameraInstance>,
    ) -> Self {
        Self {
            id,
            local_transform,
            children,
            mesh,
            camera,
        }
    }
}
