{% include "constants.wgsl" %}

fn distribution_ggx(
    N: vec3<f32>,
    H: vec3<f32>,
    roughness: f32,
) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let N_dot_H = max(
        dot(N, H),
        0.0,
    );
    let N_dot_H2 = N_dot_H * N_dot_H;

    let numerator = a2;

    var denominator = N_dot_H2 * (a2 - 1.0) + 1.0;
    denominator = PI * denominator * denominator;

    return numerator / denominator;
}

fn geometry_schlick_ggx(
    N_dot_V: f32,
    roughness: f32,
) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;

    let numerator = N_dot_V;
    let denominator = N_dot_V * (1.0 - k) + k;

    return numerator / denominator;
}

fn geometry_smith(
    N: vec3<f32>,
    V: vec3<f32>,
    L: vec3<f32>,
    roughness: f32,
) -> f32 {
    let N_dot_V = max(
        dot(N, V),
        0.0,
    );
    let N_dot_L = max(
        dot(N, L),
        0.0,
    );
    let ggx_2 = geometry_schlick_ggx(N_dot_V, roughness);
    let ggx_1 = geometry_schlick_ggx(N_dot_L, roughness);

    return ggx_1 * ggx_2;
}

fn fresnel_schlick(
    cos_theta: f32,
    F0: vec3<f32>,
) -> vec3<f32> {
    return F0 + (1.0 - F0) * pow(
        clamp(
            1.0 - cos_theta,
            0.0,
            1.0,
        ),
        5.0,
    );
}

fn fresnel_schlick_roughness(
    cos_theta: f32,
    F0: vec3<f32>,
    roughness: f32,
) -> vec3<f32> {
    return F0 + (
        max(
            vec3(1.0 - roughness),
            F0,
        ) - F0
    ) * pow(
        clamp(
            1.0 - cos_theta,
            0.0,
            1.0,
        ),
        5.0,
    );
}
