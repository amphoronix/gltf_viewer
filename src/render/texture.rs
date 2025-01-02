use crate::render::sampler::Sampler;

pub struct Texture2DPackage {
    #[allow(dead_code)]
    pub gpu_texture: wgpu::Texture,
    pub gpu_texture_view: wgpu::TextureView,
    pub sampler: std::rc::Rc<Sampler>,
}

pub struct DepthTexture2DPackage {
    #[allow(dead_code)]
    pub gpu_texture: wgpu::Texture,
    pub gpu_texture_view: wgpu::TextureView,
}
