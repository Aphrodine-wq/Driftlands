use bevy::prelude::*;

use crate::lit_materials::{LitChunkMaterial, LitSpriteMaterial};
use crate::spatial::SpatialGrid;
use crate::world::{ChunkObject, WorldObject, WorldState};
use crate::hud::not_paused;

#[derive(Resource, Default)]
pub struct DebugPerfTiming {
    pub chunk_manage_ms: f32,
    pub spatial_update_ms: f32,
    /// Time spent (in ms) stitching/creating animation atlases the last time atlas building ran successfully.
    pub atlas_build_ms: f32,
    /// Total number of atlases built successfully since the app started.
    pub atlases_built_this_session: u32,
    /// Total number of sprites upgraded from frame-swapping to atlas rendering since the app started.
    pub atlas_upgrades: u64,
}

#[derive(Resource)]
pub struct DebugPerfOverlayState {
    pub enabled: bool,
    pub root: Option<Entity>,
}

impl Default for DebugPerfOverlayState {
    fn default() -> Self {
        Self { enabled: false, root: None }
    }
}

#[derive(Component)]
struct DebugPerfRoot;

#[derive(Component)]
struct DebugPerfText;

pub struct DebugPerfPlugin;

impl Plugin for DebugPerfPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugPerfTiming>()
            .init_resource::<DebugPerfOverlayState>()
            .add_systems(Startup, spawn_debug_perf_overlay)
            .add_systems(
                Update,
                (
                    toggle_debug_perf_overlay.run_if(not_paused),
                    update_debug_perf_overlay.run_if(not_paused),
                ),
            );
    }
}

fn spawn_debug_perf_overlay(mut commands: Commands) {
    // A lightweight, top-left panel. Updated every frame only while enabled.
    let root = commands
        .spawn((
            DebugPerfRoot,
            Visibility::Hidden,
            Text::new(""),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                left: Val::Px(8.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::FlexStart,
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.02, 0.8)),
            GlobalZIndex(999),
        ))
        .with_children(|parent| {
            parent.spawn((
                DebugPerfText,
                TextSpan::default(),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.9, 0.75)),
            ));
        })
        .id();

    commands.insert_resource(DebugPerfOverlayState { enabled: false, root: Some(root) });
}

fn toggle_debug_perf_overlay(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DebugPerfOverlayState>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        state.enabled = !state.enabled;
    }
}

fn update_debug_perf_overlay(
    state: Res<DebugPerfOverlayState>,
    timing: Res<DebugPerfTiming>,
    world_state: Res<WorldState>,
    sprite_materials: Res<Assets<LitSpriteMaterial>>,
    chunk_materials: Res<Assets<LitChunkMaterial>>,
    world_objects: Query<(), With<WorldObject>>,
    chunk_objects: Query<(), With<ChunkObject>>,
    grid: Res<SpatialGrid>,
    mut root_q: Query<&mut Visibility, With<DebugPerfRoot>>,
    mut text_q: Query<&mut TextSpan, With<DebugPerfText>>,
) {
    let Ok(mut root_vis) = root_q.get_single_mut() else { return };
    *root_vis = if state.enabled { Visibility::Visible } else { Visibility::Hidden };

    if !state.enabled {
        return;
    }

    let chunk_count = world_state.loaded_chunks.len();
    let world_object_count = world_objects.iter().count();
    let chunk_object_count = chunk_objects.iter().count();

    // `Assets::len()` is the total number of material assets currently held.
    let sprite_material_count = sprite_materials.len();
    let chunk_material_count = chunk_materials.len();

    // SpatialGrid contains additional HashMaps; avoid expensive full stats here.
    let grid_summary = grid.cell_size;

    let lines = [
        format!("Chunks: {}", chunk_count),
        format!("WorldObjects: {}", world_object_count),
        format!("ChunkObjects: {}", chunk_object_count),
        format!("LitSpriteMaterials: {}", sprite_material_count),
        format!("LitChunkMaterials: {}", chunk_material_count),
        format!("Chunk manage: {:.2} ms", timing.chunk_manage_ms),
        format!("Spatial update: {:.2} ms", timing.spatial_update_ms),
        format!("Atlas build: {:.2} ms", timing.atlas_build_ms),
        format!("Atlases built: {}", timing.atlases_built_this_session),
        format!("Atlas upgrades: {}", timing.atlas_upgrades),
        format!("Spatial cell size: {:.1}", grid_summary),
        "Toggle: F3".to_string(),
    ];

    let Ok(mut span) = text_q.get_single_mut() else { return };
    **span = lines.join("\n");
}

