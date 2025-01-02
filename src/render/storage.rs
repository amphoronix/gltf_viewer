use std::collections::HashMap;

use crate::render::image::Image;
use crate::render::material::Material;
use crate::render::mesh::Mesh;
use crate::render::node::RenderNode;
use crate::render::pipeline::{RenderPipeline, RenderPipelineConfiguration};
use crate::render::sampler::Sampler;
use crate::render::shader::{ShaderModulePackage, ShaderTemplateConfiguration};
use crate::render::texture::Texture2DPackage;

#[derive(Default)]
pub struct RenderSystemSceneStorage {
    pub node_registry: HashMap<usize, std::rc::Rc<RenderNode>>,
    pub mesh_registry: HashMap<usize, std::rc::Rc<Mesh>>,
    pub material_registry: HashMap<Option<usize>, std::rc::Rc<Material>>,
    pub texture_registry: HashMap<usize, std::rc::Rc<Texture2DPackage>>,
    pub image_registry: HashMap<usize, std::rc::Rc<Image>>,
    pub sampler_registry: HashMap<Option<usize>, std::rc::Rc<Sampler>>,
    pub render_pipeline_registry: HashMap<RenderPipelineConfiguration, std::rc::Rc<RenderPipeline>>,
    pub shader_module_package_registry:
        HashMap<ShaderTemplateConfiguration, std::rc::Rc<ShaderModulePackage>>,
    pub default_texture: Option<std::rc::Rc<Texture2DPackage>>,
}
