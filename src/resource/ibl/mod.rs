use anyhow::Result;

use crate::resource::cubemap::CubeMapLoader;

pub mod file;

pub trait IblEnvironmentLoader {
    fn load_equirectangular_skybox(&self) -> Result<image::Rgba32FImage>;
    fn get_diffuse_cubemap_loader(&self) -> Result<impl CubeMapLoader>;
    fn get_specular_cubemap_loader(&self) -> Result<impl CubeMapLoader>;
    fn load_ggx_lut(&self, path: &std::path::Path) -> Result<image::Rgba32FImage>;
}
