use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;

use crate::error::Error;
use crate::resource::gltf::asset::file::FileSystemGltfAsset;
use crate::resource::gltf::asset::GltfAsset;
use crate::resource::gltf::loader::GltfLoader;

pub struct FileSystemGltfLoader<'a> {
    asset: &'a FileSystemGltfAsset,
    buffer_registry: HashMap<usize, Vec<u8>>,
    image_registry: HashMap<String, Vec<u8>>,
}

impl<'a> FileSystemGltfLoader<'a> {
    pub fn new(asset: &'a FileSystemGltfAsset) -> Self {
        Self {
            asset,
            buffer_registry: HashMap::new(),
            image_registry: HashMap::new(),
        }
    }

    fn get_buffer_read_info(&self, accessor_id: usize) -> Result<GltfBufferReadInfo> {
        let accessor = match self.asset.gltf().accessors().nth(accessor_id) {
            Some(accessor) => accessor,
            None => {
                return Err(
                    Error::new(format!("The given accessor ID is invalid: {accessor_id}")).into(),
                )
            }
        };

        let view = match accessor.view() {
            Some(view) => view,
            None => {
                return Err(Error::new(format!(
                    "The specified accessor has no buffer view: {accessor_id}"
                ))
                .into())
            }
        };

        let buffer = view.buffer();

        Ok(GltfBufferReadInfo {
            index: buffer.index(),
            uri: self.get_buffer_uri(&buffer)?,
            offset: view.offset() + accessor.offset(),
            length: accessor.count() * accessor.size(),
        })
    }

    fn get_buffer_uri(&self, buffer: &gltf::Buffer) -> Result<String> {
        match buffer.source() {
            gltf::buffer::Source::Uri(uri) => Ok(String::from(uri)),
            gltf::buffer::Source::Bin => {
                Err(Error::new(String::from("Loading inline buffers is not supported.")).into())
            }
        }
    }

    fn load_buffer_data(&mut self, buffer_id: usize, uri: &str) -> Result<()> {
        if self.buffer_registry.contains_key(&buffer_id) {
            return Ok(());
        }

        let buffer_path = Path::new(&self.asset.root).join(uri);
        let data = std::fs::read(buffer_path)?;

        self.buffer_registry.insert(buffer_id, data);

        Ok(())
    }

    fn read_buffer_data(&self, buffer_id: usize, offset: usize, length: usize) -> Result<&[u8]> {
        let data = match self.buffer_registry.get(&buffer_id) {
            Some(data) => data,
            None => {
                return Err(Error::new(format!(
                    "The given buffer ID is not associated with a loaded buffer: {buffer_id}"
                ))
                .into())
            }
        };

        Ok(&data[offset..offset + length])
    }

    fn load_image_data(&mut self, uri: &String) -> Result<()> {
        if self.image_registry.contains_key(uri) {
            return Ok(());
        }

        let image_path = Path::new(&self.asset.root).join(uri);
        let data = std::fs::read(image_path)?;

        self.image_registry.insert(uri.to_string(), data);

        Ok(())
    }

    fn read_image_data<'b>(&'b self, uri: &String) -> Result<&'b [u8]> {
        let data = match self.image_registry.get(uri) {
            Some(data) => data,
            None => {
                return Err(Error::new(format!(
                    "The given image URI is not associated with a loaded image: {uri}"
                ))
                .into())
            }
        };

        Ok(&data[..])
    }
}

impl<'a> GltfLoader for FileSystemGltfLoader<'a> {
    fn asset<'b>(&'b self) -> &'b impl GltfAsset {
        self.asset
    }

    fn load_bytes_from_accessor(&mut self, accessor_id: usize) -> Result<&[u8]> {
        let buffer_read_info = self.get_buffer_read_info(accessor_id)?;
        self.load_buffer_data(buffer_read_info.index, &buffer_read_info.uri)?;

        self.read_buffer_data(
            buffer_read_info.index,
            buffer_read_info.offset,
            buffer_read_info.length,
        )
    }

    fn read_bytes_from_accessor(&self, accessor_id: usize) -> Result<&[u8]> {
        let buffer_read_info = self.get_buffer_read_info(accessor_id)?;

        self.read_buffer_data(
            buffer_read_info.index,
            buffer_read_info.offset,
            buffer_read_info.length,
        )
    }

    fn load_image(&mut self, image_id: usize) -> Result<image::RgbaImage> {
        let image = match self.asset.gltf().images().nth(image_id) {
            Some(image) => image,
            None => {
                return Err(Error::new(format!("The given image ID is invalid: {image_id}")).into())
            }
        };

        let (data, mime_type) = match image.source() {
            gltf::image::Source::Uri { uri, mime_type } => {
                let uri = String::from(uri);
                self.load_image_data(&uri)?;
                (self.read_image_data(&uri)?, mime_type)
            }
            gltf::image::Source::View { view, mime_type } => {
                let buffer = view.buffer();

                let index = buffer.index();
                let uri = self.get_buffer_uri(&buffer)?;
                let offset = view.offset();
                let length = view.length();

                self.load_buffer_data(index, &uri)?;
                (
                    self.read_buffer_data(index, offset, length)?,
                    Some(mime_type),
                )
            }
        };

        let image_format = match mime_type {
            Some(mime_type) => match image::ImageFormat::from_mime_type(mime_type) {
                Some(image_format) => Some(image_format),
                None => {
                    return Err(Error::new(format!(
                        "The given MIME type is not supported: {mime_type}"
                    ))
                    .into())
                }
            },
            None => None,
        };

        let loaded_image = match image_format {
            Some(image_format) => image::load_from_memory_with_format(data, image_format)?,
            None => image::load_from_memory(data)?,
        };

        Ok(loaded_image.to_rgba8())
    }
}

struct GltfBufferReadInfo {
    index: usize,
    uri: String,
    offset: usize,
    length: usize,
}
