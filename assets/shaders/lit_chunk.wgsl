#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct PointLightUniform {
    position: vec2<f32>,
    radius: f32,
    intensity: f32,
    _pad: f32,
    color: vec3<f32>,
    _pad2: f32,
}

struct LightingUniform {
    ambient_color: vec3<f32>,
    _pad0: f32,
    sun_direction: vec2<f32>,
    sun_intensity: f32,
    sun_color: vec3<f32>,
    _pad1: f32,
    point_lights: array<PointLightUniform, 4u>,
}

@group(2) @binding(0) var<uniform> lighting: LightingUniform;
@group(2) @binding(1) var<uniform> time: f32;
@group(2) @binding(2) var color_texture: texture_2d<f32>;
@group(2) @binding(3) var color_sampler: sampler;
@group(2) @binding(4) var normal_texture: texture_2d<f32>;
@group(2) @binding(5) var normal_sampler: sampler;

// Quantize to 5 bands for pixel-art lighting (0.2, 0.4, 0.6, 0.8, 1.0)
fn quantize_light(f: f32) -> f32 {
    let t = clamp(f, 0.0, 1.0);
    if (t < 0.2) { return 0.2; }
    if (t < 0.4) { return 0.4; }
    if (t < 0.6) { return 0.6; }
    if (t < 0.8) { return 0.8; }
    return 1.0;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let base = textureSample(color_texture, color_sampler, mesh.uv);
    var normal_uv = mesh.uv;
    let n_enc_first = textureSample(normal_texture, normal_sampler, mesh.uv);
    if (n_enc_first.a < 0.5) {
        normal_uv += vec2<f32>(sin(time) * 0.02, cos(time * 0.7) * 0.02);
    }
    let n_enc = textureSample(normal_texture, normal_sampler, normal_uv);
    let n = vec3<f32>(n_enc.r * 2.0 - 1.0, n_enc.g * 2.0 - 1.0, n_enc.b * 2.0 - 1.0);
    let n_len = length(n);
    let n_norm = select(vec3<f32>(0.0, 0.0, 1.0), n / n_len, n_len > 0.001);
    let sun_dir_3d = vec3<f32>(lighting.sun_direction.x, lighting.sun_direction.y, 0.5);
    let lambert = max(dot(n_norm, normalize(sun_dir_3d)), 0.0);
    let sun_term = lighting.sun_color * (lighting.sun_intensity * quantize_light(lambert));
    var point_term = vec3<f32>(0.0, 0.0, 0.0);
    let world_xy = mesh.world_position.xy;
    for (var i = 0u; i < 4u; i++) {
        let pl = lighting.point_lights[i];
        let d = distance(world_xy, pl.position);
        let falloff = 1.0 - smoothstep(0.0, pl.radius, d);
        point_term += pl.color * (pl.intensity * falloff);
    }
    let light_total = lighting.ambient_color + sun_term + point_term;
    let lit = base * vec4<f32>(light_total, 1.0);
    return lit;
}
