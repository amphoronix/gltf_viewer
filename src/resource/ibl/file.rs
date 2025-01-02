use anyhow::Result;

use crate::args::IblEnvironmentPaths;
use crate::resource::cubemap::ktx2::Ktx2CubeMapLoader;
use crate::resource::cubemap::CubeMapLoader;
use crate::resource::ibl::IblEnvironmentLoader;

pub struct FileSystemIblEnvironmentLoader {
    pub paths: IblEnvironmentPaths,
}

impl FileSystemIblEnvironmentLoader {
    fn get_cubemap_loader(&self, path: &String) -> Result<Ktx2CubeMapLoader<Vec<u8>>> {
        let ktx2_data = std::fs::read(path)?;
        let reader = ktx2::Reader::new(ktx2_data)?;

        Ok(Ktx2CubeMapLoader::from_reader(reader))
    }
}

impl IblEnvironmentLoader for FileSystemIblEnvironmentLoader {
    fn load_equirectangular_skybox(&self) -> Result<image::Rgba32FImage> {
        Ok(image::open(&self.paths.skybox)?.to_rgba32f())
    }

    fn get_diffuse_cubemap_loader(&self) -> Result<impl CubeMapLoader> {
        self.get_cubemap_loader(&self.paths.diffuse)
    }

    fn get_specular_cubemap_loader(&self) -> Result<impl CubeMapLoader> {
        self.get_cubemap_loader(&self.paths.specular)
    }

    fn load_ggx_lut(&self, path: &std::path::Path) -> Result<image::Rgba32FImage> {
        Ok(image::open(path)?.to_rgba32f())
    }
}
