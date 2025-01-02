use anyhow::Result;

use crate::render::buffer::{VertexBuffer, VertexBufferSegmentDescriptor};
use crate::resource::gltf::loader::GltfLoader;

pub struct VertexBufferAllocator {
    label: String,
    segments: Vec<VertexBufferSegmentAllocationDescriptor>,
}

impl VertexBufferAllocator {
    pub fn new(label: String) -> Self {
        Self {
            label,
            segments: vec![],
        }
    }

    pub fn add_segment(
        &mut self,
        semantic: gltf::Semantic,
        data_source: VertexBufferSegmentDataSource,
    ) {
        self.segments.push(VertexBufferSegmentAllocationDescriptor {
            semantic,
            data_source,
        });
    }

    pub fn finish(
        self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        gltf_loader: &mut impl GltfLoader,
    ) -> Result<VertexBuffer> {
        let size = self
            .segments
            .iter()
            .map(|segment| match &segment.data_source {
                VertexBufferSegmentDataSource::Accessor { length, .. } => *length,
                VertexBufferSegmentDataSource::Raw { data } => data.len() as u64,
            })
            .sum();

        let gpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&self.label),
            size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut offset: usize = 0;
        let mut segment_descriptors = Vec::<VertexBufferSegmentDescriptor>::new();
        for segment in self.segments.iter() {
            let data = match &segment.data_source {
                VertexBufferSegmentDataSource::Accessor { id, .. } => {
                    gltf_loader.load_bytes_from_accessor(*id)?
                }
                VertexBufferSegmentDataSource::Raw { data } => data,
            };

            queue.write_buffer(&gpu_buffer, offset as u64, data);

            segment_descriptors.push(VertexBufferSegmentDescriptor {
                type_: segment.semantic.clone(),
                offset,
                length: data.len(),
            });

            offset += data.len();
        }
        queue.submit([]);

        Ok(VertexBuffer {
            gpu_buffer,
            segments: segment_descriptors,
        })
    }
}

struct VertexBufferSegmentAllocationDescriptor {
    pub semantic: gltf::Semantic,
    pub data_source: VertexBufferSegmentDataSource,
}

pub enum VertexBufferSegmentDataSource {
    Accessor { id: usize, length: u64 },
    Raw { data: Vec<u8> },
}
