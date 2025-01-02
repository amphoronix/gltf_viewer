struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) face_coords: vec2<f32>,
}
