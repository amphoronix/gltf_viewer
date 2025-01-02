use anyhow::Result;

use crate::resource::gltf::asset::GltfAsset;

pub mod file;

pub trait GltfLoader {
    fn asset(&self) -> &impl GltfAsset;
    fn load_bytes_from_accessor(&mut self, accessor_id: usize) -> Result<&[u8]>;
    fn read_bytes_from_accessor(&self, accessor_id: usize) -> Result<&[u8]>;
    fn load_image(&mut self, image_id: usize) -> Result<image::RgbaImage>;
}
