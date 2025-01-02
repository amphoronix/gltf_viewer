use anyhow::Result;

use crate::error::Error;
use crate::resource::cubemap::CubeMapLoader;

pub struct Ktx2CubeMapLoader<T: AsRef<[u8]>> {
    reader: ktx2::Reader<T>,
}

impl<T: AsRef<[u8]>> Ktx2CubeMapLoader<T> {
    pub fn from_reader(reader: ktx2::Reader<T>) -> Self {
        Self { reader }
    }

    fn load_face(&self, face_index: u32, mip_level: u32) -> Result<&[u8]> {
        let level_data = match self.reader.levels().nth(mip_level as usize) {
            Some(level_data) => level_data,
            None => return Err(
                Error::new(
                    format!("The given cubemap does not have a mip level that matches the specified index: {mip_level}")
                ).into()
            ),
        };

        let (width, height) = self.face_dimensions();
        let width = width / 2_u32.pow(mip_level);
        let height = height / 2_u32.pow(mip_level);

        let face_size = 4 * (std::mem::size_of::<half::f16>() as u32) * width * height;

        let range_begin = (face_size * face_index) as usize;
        let range_end = (face_size * (face_index + 1)) as usize;

        Ok(&level_data[range_begin..range_end])
    }
}

impl<T: AsRef<[u8]>> CubeMapLoader for Ktx2CubeMapLoader<T> {
    fn face_dimensions(&self) -> (u32, u32) {
        (
            self.reader.header().pixel_width,
            self.reader.header().pixel_height,
        )
    }

    fn mip_level_count(&self) -> u32 {
        self.reader.header().level_count
    }

    fn load_positive_x_face(&self, mip_level: u32) -> Result<&[u8]> {
        self.load_face(0, mip_level)
    }

    fn load_negative_x_face(&self, mip_level: u32) -> Result<&[u8]> {
        self.load_face(1, mip_level)
    }

    fn load_positive_y_face(&self, mip_level: u32) -> Result<&[u8]> {
        self.load_face(2, mip_level)
    }

    fn load_negative_y_face(&self, mip_level: u32) -> Result<&[u8]> {
        self.load_face(3, mip_level)
    }

    fn load_positive_z_face(&self, mip_level: u32) -> Result<&[u8]> {
        self.load_face(4, mip_level)
    }

    fn load_negative_z_face(&self, mip_level: u32) -> Result<&[u8]> {
        self.load_face(5, mip_level)
    }
}
