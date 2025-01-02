pub struct Image {
    data: image::RgbaImage,
}

impl Image {
    pub fn from_rgba_image(rgba_image: image::RgbaImage) -> Self {
        Self { data: rgba_image }
    }

    pub fn data(&self) -> &image::RgbaImage {
        &self.data
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.data.dimensions()
    }
}
