use anyhow::Result;

use crate::error::Error;

pub mod file;

pub trait GltfAsset {
    fn gltf(&self) -> &gltf::Gltf;

    fn get_scene(&self, scene_id: usize) -> Result<gltf::Scene> {
        match self.gltf().scenes().nth(scene_id) {
            Some(scene) => Ok(scene),
            None => {
                Err(Error::new(format!("No scene exists with the given ID: {scene_id}")).into())
            }
        }
    }
}
