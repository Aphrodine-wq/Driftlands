//! Custom 2D materials for lit terrain chunks and sprites.
//! Used with Mesh2d + MeshMaterial2d; lighting is applied in the chunk/sprite shaders.

use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{AlphaMode2d, Material2d, Material2dPlugin};

use crate::lighting::LightingUniform;

pub struct LitMaterialsPlugin;

impl Plugin for LitMaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            Material2dPlugin::<LitChunkMaterial>::default(),
            Material2dPlugin::<LitSpriteMaterial>::default(),
        ))
        .add_systems(Startup, spawn_lit_quad_mesh)
        .add_systems(Update, update_chunk_lighting);
    }
}

/// Copies current LightingSettings and time into all LitChunkMaterial assets each frame.
fn update_chunk_lighting(
    lighting: Res<crate::lighting::LightingSettings>,
    time: Res<Time>,
    mut materials: ResMut<Assets<LitChunkMaterial>>,
) {
    let u = LightingUniform::from_settings(&lighting);
    let t = time.elapsed_secs();
    for (_, mat) in materials.iter_mut() {
        mat.lighting = u.clone();
        mat.time = t;
    }
}

fn spawn_lit_quad_mesh(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let quad = meshes.add(Rectangle::default());
    commands.insert_resource(LitQuadMesh { quad });
}

/// Shared unit quad mesh for lit chunks and sprites (scale via Transform).
#[derive(Resource)]
pub struct LitQuadMesh {
    pub quad: Handle<Mesh>,
}

/// Material for terrain chunks. Uses normal map and global lighting for directional + ambient.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct LitChunkMaterial {
    #[uniform(0)]
    pub lighting: LightingUniform,
    #[uniform(1)]
    pub time: f32,
    #[texture(2)]
    #[sampler(3)]
    pub color_texture: Handle<Image>,
    #[texture(4)]
    #[sampler(5)]
    pub normal_texture: Handle<Image>,
}

impl Material2d for LitChunkMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/lit_chunk.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Opaque
    }
}

/// Material for world objects and characters. Supports tint (e.g. hit flash, health).
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct LitSpriteMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    #[texture(1)]
    #[sampler(2)]
    pub color_texture: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub normal_texture: Handle<Image>,
}

impl Material2d for LitSpriteMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/lit_sprite.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
