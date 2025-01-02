use crate::render::cubemap::CubeMap;
use crate::render::lut::GgxLut;
use crate::render::skybox::Skybox;

pub struct IblEnvironment {
    pub skybox: Skybox,
    pub diffuse_cubemap: CubeMap,
    pub specular_cubemap: CubeMap,
    pub ggx_lut: GgxLut,
}
