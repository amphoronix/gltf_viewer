struct MetallicRoughnessUniform {
    base_color_factor: vec4<f32>,
    metallic_factor: f32,
    roughness_factor: f32,
}

@group(2) @binding(0)
var<uniform> metallic_roughness: MetallicRoughnessUniform;

@group(2) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(2) @binding(2)
var base_color_sampler: sampler;

@group(2) @binding(3)
var metallic_roughness_texture: texture_2d<f32>;
@group(2) @binding(4)
var metallic_roughness_sampler: sampler;
