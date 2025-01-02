{% include "primitive/camera.wgsl" %}
{% include "primitive/data.wgsl" %}
{% include "primitive/material.wgsl" %}

@group(1) @binding(0)
var<uniform> transform: mat4x4<f32>;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_position = transform * vec4<f32>(in.position, 1.0);

    out.clip_position = camera.view_projection * world_position;
    out.world_position = world_position.xyz;

{% if has_normal %}
    out.normal = (transform * vec4<f32>(in.normal, 1.0)).xyz;

{% if has_tangent %}
    out.tangent = transform * in.tangent;
{% endif %}

{% endif %}

{% if has_tex_coord_0 %}
    out.tex_coord_0 = in.tex_coord_0;
{% endif %}

{% if has_tex_coord_1 %}
    out.tex_coord_1 = in.tex_coord_1;
{% endif %}

{% if has_color_0 %}
    out.color_0 = in.color_0;
{% endif %}

    return out;
}
