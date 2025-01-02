struct VertexInput {
    @location(0) position: vec3<f32>,

{% if has_normal %}
    @location(1) normal: vec3<f32>,

{% if has_tangent %}
    @location(2) tangent: vec4<f32>,
{% endif %}

{% endif %}

{% if has_tex_coord_0 %}
    @location({{ tex_coord_0_location }}) tex_coord_0: vec2<f32>,
{% endif %}

{% if has_tex_coord_1 %}
    @location({{ tex_coord_1_location }}) tex_coord_1: vec2<f32>,
{% endif %}

{% if has_color_0 %}
    @location({{ color_0_location }}) color_0: vec4<f32>,
{% endif %}
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,

{% if has_normal %}
    @location(1) normal: vec3<f32>,

{% if has_tangent %}
    @location(2) tangent: vec4<f32>,
{% endif %}

{% endif %}

{% if has_tex_coord_0 %}
    @location({{ tex_coord_0_location }}) tex_coord_0: vec2<f32>,
{% endif %}

{% if has_tex_coord_1 %}
    @location({{ tex_coord_1_location }}) tex_coord_1: vec2<f32>,
{% endif %}

{% if has_color_0 %}
    @location({{ color_0_location }}) color_0: vec4<f32>,
{% endif %}
}
