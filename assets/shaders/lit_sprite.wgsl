#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct LitSpriteMaterial {
    color: vec4<f32>,
};

@group(2) @binding(0) var<uniform> material: LitSpriteMaterial;
@group(2) @binding(1) var color_texture: texture_2d<f32>;
@group(2) @binding(2) var color_sampler: sampler;
@group(2) @binding(3) var normal_texture: texture_2d<f32>;
@group(2) @binding(4) var normal_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    return material.color * textureSample(color_texture, color_sampler, mesh.uv);
}
