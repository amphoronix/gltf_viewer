use crate::render::primitive::Primitive;

pub struct Mesh {
    pub primitives: Vec<std::rc::Rc<Primitive>>,
}

pub struct MeshInstance {
    pub mesh: std::rc::Rc<Mesh>,
    #[allow(dead_code)]
    pub gpu_transform_uniform_buffer: wgpu::Buffer,
    pub gpu_transform_bind_group: wgpu::BindGroup,
}

impl MeshInstance {
    pub fn from_device(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        name: &str,
        mesh: std::rc::Rc<Mesh>,
        transform_matrix: cgmath::Matrix4<f32>,
        transform_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let gpu_transform_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("{name}_TRANSFORM_UNIFORM_BUFFER")),
            size: std::mem::size_of::<[[f32; 4]; 4]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let transform_data: [[f32; 4]; 4] = transform_matrix.into();

        queue.write_buffer(
            &gpu_transform_uniform_buffer,
            0,
            bytemuck::cast_slice(&[transform_data]),
        );

        let gpu_transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{name}_TRANSFORM_BIND_GROUP")),
            layout: transform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: gpu_transform_uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            mesh,
            gpu_transform_uniform_buffer,
            gpu_transform_bind_group,
        }
    }
}
