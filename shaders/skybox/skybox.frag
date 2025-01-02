{% include "skybox/data.wgsl" %}

@group(0) @binding(1)
var skybox_texture: texture_cube<f32>;
@group(0) @binding(2)
var skybox_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sampled_color = textureSample(
        skybox_texture,
        skybox_sampler,
        in.tex_coord,
    );

    var color = sampled_color.rgb;

    // HDR tonemapping
    color = color / (color + vec3<f32>(1.0));

    return vec4(color, 1.0);
}
