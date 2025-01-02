use std::path::Path;

use anyhow::Result;

use crate::resource::gltf::asset::GltfAsset;

pub struct FileSystemGltfAsset {
    gltf: gltf::Gltf,
    pub root: String,
}

impl FileSystemGltfAsset {
    pub fn from_path(gltf_path: &Path) -> Result<Self> {
        let absolute_path = gltf_path.canonicalize()?;

        if !absolute_path.is_file() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("The given path is not a file: {}", gltf_path.display()),
            )
            .into());
        }

        let root_path = match gltf_path.parent() {
            Some(root) => root,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "Unable to find the parent directory of the given path: {}",
                        gltf_path.display()
                    ),
                )
                .into())
            }
        };

        let root = match root_path.to_str() {
            Some(root) => String::from(root),
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("The given path is not valid UTF-8: {}", gltf_path.display()),
                )
                .into())
            }
        };

        let parsed_gltf = gltf::Gltf::open(gltf_path)?;

        Ok(Self {
            gltf: parsed_gltf,
            root,
        })
    }
}

impl GltfAsset for FileSystemGltfAsset {
    fn gltf(&self) -> &gltf::Gltf {
        &self.gltf
    }
}
