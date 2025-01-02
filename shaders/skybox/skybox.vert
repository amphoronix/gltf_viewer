{% include "skybox/data.wgsl" %}

@group(0) @binding(0)
var<uniform> view_projection: mat4x4<f32>;

@vertex
fn vs_main(@location(0) position: vec3<f32>) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = (view_projection * vec4(position, 1.0)).xyww;
    out.tex_coord = position;

    return out;
}
