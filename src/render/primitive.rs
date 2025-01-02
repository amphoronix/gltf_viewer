use crate::render::buffer::{IndexBuffer, VertexBuffer};
use crate::render::material::Material;
use crate::render::pipeline::RenderPipeline;

pub struct Primitive {
    pub vertex_buffer: VertexBuffer,
    pub index_buffer: Option<IndexBuffer>,
    pub material: std::rc::Rc<Material>,
    pub count: usize,
    pub render_pipeline: std::rc::Rc<RenderPipeline>,
}
