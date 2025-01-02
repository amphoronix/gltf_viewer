use anyhow::Result;

use crate::render::pipeline::RenderPipelineConfiguration;

pub struct ShaderModulePackage {
    pub vertex_shader_module: wgpu::ShaderModule,
    pub fragment_shader_module: wgpu::ShaderModule,
}

impl ShaderModulePackage {
    pub fn from_templates(
        vertex_template_name: &str,
        fragment_template_name: &str,
        name: &str,
        device: &wgpu::Device,
        tera: &tera::Tera,
        shader_template_config: Option<&ShaderTemplateConfiguration>,
    ) -> Result<Self> {
        let shader_template_context = match shader_template_config {
            Some(shader_template_config) => tera::Context::from_serialize(shader_template_config)?,
            None => tera::Context::new(),
        };

        let vertex_shader_source = ShaderModulePackage::render_shader(
            vertex_template_name,
            tera,
            &shader_template_context,
        )?;

        let fragment_shader_source = ShaderModulePackage::render_shader(
            fragment_template_name,
            tera,
            &shader_template_context,
        )?;

        if shader_template_config.is_some() {
            log::debug!(
                "Creating shader module package {name} from config: {:?}",
                shader_template_config
            );
        } else {
            log::debug!("Creating shader module package {name}");
        }

        Ok(ShaderModulePackage {
            vertex_shader_module: device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(&format!("{}_VERTEX_SHADER_MODULE", name)),
                source: wgpu::ShaderSource::Wgsl(vertex_shader_source.into()),
            }),
            fragment_shader_module: device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(&format!("{}_FRAGMENT_SHADER_MODULE", name)),
                source: wgpu::ShaderSource::Wgsl(fragment_shader_source.into()),
            }),
        })
    }

    fn render_shader(
        template_name: &str,
        tera: &tera::Tera,
        template_context: &tera::Context,
    ) -> Result<String> {
        match tera.render(template_name, template_context) {
            Ok(shader_source) => Ok(shader_source),
            Err(error) => Err(error.into()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Serialize)]
pub struct ShaderTemplateConfiguration {
    pub has_normal: bool,
    pub has_tangent: bool,
    pub has_tex_coord_0: bool,
    pub tex_coord_0_location: u32,
    pub has_tex_coord_1: bool,
    pub tex_coord_1_location: u32,
    pub has_color_0: bool,
    pub color_0_location: u32,
}

impl ShaderTemplateConfiguration {
    pub fn from_render_pipeline_config(config: &RenderPipelineConfiguration) -> Self {
        Self {
            has_normal: config.has_normal,
            has_tangent: config.has_tangent,
            has_tex_coord_0: config.has_tex_coord_0,
            tex_coord_0_location: config.get_tex_coord_0_location(),
            has_tex_coord_1: config.has_tex_coord_1,
            tex_coord_1_location: config.get_tex_coord_1_location(),
            has_color_0: config.has_color_0,
            color_0_location: config.get_color_0_location(),
        }
    }
}
