use crate::render::texture::Texture2DPackage;

pub struct Material {
    base_color_factor: [f32; 4],
    #[allow(dead_code)]
    base_color_texture: std::rc::Rc<Texture2DPackage>,
    metallic_factor: f32,
    roughness_factor: f32,
    #[allow(dead_code)]
    metallic_roughness_texture: std::rc::Rc<Texture2DPackage>,
    pub gpu_metallic_roughness_uniform_buffer: wgpu::Buffer,
    pub gpu_bind_group: wgpu::BindGroup,
}

impl Material {
    pub fn new(
        base_color_factor: [f32; 4],
        base_color_texture: std::rc::Rc<Texture2DPackage>,
        metallic_factor: f32,
        roughness_factor: f32,
        metallic_roughness_texture: std::rc::Rc<Texture2DPackage>,
        gpu_metallic_roughness_uniform_buffer: wgpu::Buffer,
        gpu_bind_group: wgpu::BindGroup,
        queue: &wgpu::Queue,
    ) -> Self {
        let object = Self {
            base_color_factor,
            base_color_texture,
            metallic_factor,
            roughness_factor,
            metallic_roughness_texture,
            gpu_metallic_roughness_uniform_buffer,
            gpu_bind_group,
        };
        object.initialize_uniform_buffer(queue);

        object
    }

    fn initialize_uniform_buffer(&self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.gpu_metallic_roughness_uniform_buffer,
            0,
            bytemuck::cast_slice(&[MetallicRoughnessUniform::new(
                self.base_color_factor,
                self.metallic_factor,
                self.roughness_factor,
            )]),
        );
        queue.submit([]);
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MetallicRoughnessUniform {
    base_color_factor: [f32; 4],
    metallic_factor: f32,
    roughness_factor: f32,
    _padding: u64,
}

impl MetallicRoughnessUniform {
    pub fn new(base_color_factor: [f32; 4], metallic_factor: f32, roughness_factor: f32) -> Self {
        Self {
            base_color_factor,
            metallic_factor,
            roughness_factor,
            _padding: 0,
        }
    }
}
