{% include "equirectangular/data.wgsl" %}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let x = -1.0 + f32((vertex_index & 1) << 2);
    let y = -1.0 + f32((vertex_index & 2) << 1);

    var out: VertexOutput;

    out.clip_position = vec4(x, y, 0.0, 1.0);
    out.face_coords = vec2(-x, -y);

    return out;
}
