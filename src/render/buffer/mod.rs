pub mod allocator;

pub struct VertexBuffer {
    pub gpu_buffer: wgpu::Buffer,
    pub segments: Vec<VertexBufferSegmentDescriptor>,
}

pub struct VertexBufferSegmentDescriptor {
    pub type_: gltf::Semantic,
    pub offset: usize,
    pub length: usize,
}

pub struct IndexBuffer {
    pub gpu_buffer: wgpu::Buffer,
    pub type_: wgpu::IndexFormat,
}
