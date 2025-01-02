{% include "primitive/camera.wgsl" %}
{% include "primitive/data.wgsl" %}
{% include "primitive/ibl.wgsl" %}
{% include "primitive/material.wgsl" %}
{% include "primitive/pbr.wgsl" %}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let albedo = (
        metallic_roughness.base_color_factor * textureSample(
            base_color_texture,
            base_color_sampler,
            in.tex_coord_0,
        )
    ).rgb;

    let metallic_roughness_texture = textureSample(
        metallic_roughness_texture,
        metallic_roughness_sampler,
        in.tex_coord_0,
    );

    let roughness = metallic_roughness.roughness_factor * metallic_roughness_texture[1];
    let metallic = metallic_roughness.metallic_factor * metallic_roughness_texture[2];

    let N = normalize(in.normal);
    let V = normalize(camera.position - in.world_position);
    let R = reflect(-V, N);

    var F0 = vec3<f32>(0.04);
    F0 = mix(F0, albedo, metallic);

    let F = fresnel_schlick_roughness(
        max(
            dot(N, V),
            0.0,
        ),
        F0,
        roughness,
    );

    let kS = F;
    var kD = vec3<f32>(1.0) - kS;
    kD *= 1.0 - metallic;

    // IBL Diffuse
    let irradiance = textureSample(
        ibl_diffuse_cubemap_texture,
        ibl_diffuse_cubemap_sampler,
        N,
    ).rgb;
    let diffuse = irradiance * albedo;

    // IBL Specular
    let ibl_specular_cubemap_mip_level_count = textureNumLevels(ibl_specular_cubemap_texture);
    let prefiltered_color = textureSampleLevel(
        ibl_specular_cubemap_texture,
        ibl_specular_cubemap_sampler,
        R,
        roughness * f32(ibl_specular_cubemap_mip_level_count),
    ).rgb;
    let brdf = textureSample(
        ibl_ggx_lut,
        ibl_ggx_lut_sampler,
        vec2(
            max(
                dot(N, V),
                0.0,
            ),
            roughness,
        ),
    ).rg;
    let specular = prefiltered_color * (F * brdf.x + brdf.y);

    let ambient = kD * diffuse + specular;

    // Only ambient light is present within the scene
    var color = ambient;

    // HDR tonemapping
    color = color / (color + vec3<f32>(1.0));

    return vec4<f32>(color, 1.0);
}
