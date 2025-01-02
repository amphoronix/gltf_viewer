@group(0) @binding(1)
var ibl_diffuse_cubemap_texture: texture_cube<f32>;
@group(0) @binding(2)
var ibl_diffuse_cubemap_sampler: sampler;

@group(0) @binding(3)
var ibl_specular_cubemap_texture: texture_cube<f32>;
@group(0) @binding(4)
var ibl_specular_cubemap_sampler: sampler;

@group(0) @binding(5)
var ibl_ggx_lut: texture_2d<f32>;
@group(0) @binding(6)
var ibl_ggx_lut_sampler: sampler;
