{% include "constants.wgsl" %}
{% include "equirectangular/data.wgsl" %}

@group(0) @binding(0)
var equirectangular_texture: texture_2d<f32>;
@group(0) @binding(1)
var equirectangular_sampler: sampler;

struct FaceDirectionUVMapping {
    direction: vec3<f32>,
    u_mapping: vec3<f32>,
    v_mapping: vec3<f32>,
}

@group(1) @binding(0)
var<uniform> face_direction_uv_mapping: FaceDirectionUVMapping;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let face_coords = in.face_coords;
    let scan = face_coords_to_xyz(face_coords);
    let direction = normalize(scan);
    let uv = direction_to_uv(direction);

    return textureSample(
        equirectangular_texture,
        equirectangular_sampler,
        uv,
    );
}

fn face_coords_to_xyz(face_coords: vec2<f32>) -> vec3<f32> {
    return (face_direction_uv_mapping.direction +
        (face_direction_uv_mapping.u_mapping * face_coords.x) +
        (face_direction_uv_mapping.v_mapping * face_coords.y)
    );
}

fn direction_to_uv(direction: vec3<f32>) -> vec2<f32> {
    var u: f32 = 0;

    if (direction.x == 0.0) {
        u = (direction.z * PI) / 2.0;
    } else {
        u = atan2(direction.z, direction.x);
    }

    u = 0.5 * ((u / PI) + 1.0);
    let v = (asin(direction.y) / PI) + 0.5;

    return vec2(u, v);
}
