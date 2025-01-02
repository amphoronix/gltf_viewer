use anyhow::Result;

pub mod ktx2;

pub trait CubeMapLoader {
    fn face_dimensions(&self) -> (u32, u32);
    fn mip_level_count(&self) -> u32;
    fn load_positive_x_face(&self, mip_level: u32) -> Result<&[u8]>;
    fn load_negative_x_face(&self, mip_level: u32) -> Result<&[u8]>;
    fn load_positive_y_face(&self, mip_level: u32) -> Result<&[u8]>;
    fn load_negative_y_face(&self, mip_level: u32) -> Result<&[u8]>;
    fn load_positive_z_face(&self, mip_level: u32) -> Result<&[u8]>;
    fn load_negative_z_face(&self, mip_level: u32) -> Result<&[u8]>;
}
