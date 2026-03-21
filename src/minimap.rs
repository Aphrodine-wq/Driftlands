use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::collections::{HashSet, HashMap};
use crate::player::{Player, PlayerFacing};
use crate::camera::GameCamera;
use crate::world::chunk::{Chunk, CHUNK_SIZE};
use crate::world::{TILE_SIZE, CHUNK_WORLD_SIZE};
use crate::dungeon::DungeonEntrance;
use crate::death::{SpawnPoint, DeathStats};
use crate::npc::Trader;

pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ExploredChunks::default())
            .insert_resource(MinimapState::default())
            .insert_resource(MinimapDirty::default())
            .add_systems(Startup, spawn_minimap)
            .add_systems(Update, (update_explored_chunks, mark_minimap_dirty, update_minimap, toggle_minimap));
    }
}

const MINIMAP_SIZE: usize = 120;
const MINIMAP_SCALE: f32 = 4.0; // Each minimap pixel = 4 world pixels

/// Frame-skip: only redraw every N frames when dirty (~6 redraws/sec at 60fps)
const MINIMAP_FRAME_SKIP: u32 = 10;

#[derive(Component)]
pub struct Minimap;

/// US-040: Controls minimap visibility and fullscreen map toggle.
#[derive(Resource)]
pub struct MinimapState {
    /// N key toggles corner minimap on/off.
    pub minimap_visible: bool,
    /// M key toggles fullscreen map overlay.
    pub fullscreen_open: bool,
}

impl Default for MinimapState {
    fn default() -> Self {
        Self {
            minimap_visible: true,
            fullscreen_open: false,
        }
    }
}

/// Tracks whether the minimap image needs to be regenerated.
/// Dirty when: player moves to new chunk, chunks load/unload, minimap toggled.
#[derive(Resource)]
pub struct MinimapDirty {
    pub dirty: bool,
    /// Frame counter for frame-skip logic.
    pub frame_counter: u32,
    /// Track the player's last chunk position to detect chunk transitions.
    pub last_player_chunk: IVec2,
    /// Track chunk count to detect loads/unloads.
    pub last_chunk_count: usize,
    /// Track visibility state to detect toggles.
    pub last_visible: bool,
    pub last_fullscreen: bool,
}

impl Default for MinimapDirty {
    fn default() -> Self {
        Self {
            dirty: true, // Force initial draw
            frame_counter: 0,
            last_player_chunk: IVec2::new(i32::MIN, i32::MIN),
            last_chunk_count: 0,
            last_visible: true,
            last_fullscreen: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct ExploredChunks {
    pub chunks: HashSet<IVec2>,
}

fn update_explored_chunks(
    player_query: Query<&Transform, With<Player>>,
    mut explored: ResMut<ExploredChunks>,
) {
    let Ok(player_tf) = player_query.get_single() else { return };
    let player_chunk_x = (player_tf.translation.x / CHUNK_WORLD_SIZE).floor() as i32;
    let player_chunk_y = (player_tf.translation.y / CHUNK_WORLD_SIZE).floor() as i32;

    // Mark chunks within radius 2 as explored
    for dy in -2..=2 {
        for dx in -2..=2 {
            explored.chunks.insert(IVec2::new(player_chunk_x + dx, player_chunk_y + dy));
        }
    }
}

/// Detect conditions that require a minimap redraw and set the dirty flag.
fn mark_minimap_dirty(
    player_query: Query<&Transform, With<Player>>,
    chunk_query: Query<&Chunk>,
    minimap_state: Res<MinimapState>,
    mut dirty: ResMut<MinimapDirty>,
) {
    let Ok(player_tf) = player_query.get_single() else { return };

    let player_chunk = IVec2::new(
        (player_tf.translation.x / CHUNK_WORLD_SIZE).floor() as i32,
        (player_tf.translation.y / CHUNK_WORLD_SIZE).floor() as i32,
    );
    let chunk_count = chunk_query.iter().count();

    // Player moved to a new chunk
    if player_chunk != dirty.last_player_chunk {
        dirty.dirty = true;
        dirty.last_player_chunk = player_chunk;
    }

    // Chunks loaded or unloaded
    if chunk_count != dirty.last_chunk_count {
        dirty.dirty = true;
        dirty.last_chunk_count = chunk_count;
    }

    // Minimap visibility toggled
    if minimap_state.minimap_visible != dirty.last_visible
        || minimap_state.fullscreen_open != dirty.last_fullscreen
    {
        dirty.dirty = true;
        dirty.last_visible = minimap_state.minimap_visible;
        dirty.last_fullscreen = minimap_state.fullscreen_open;
    }
}

fn spawn_minimap(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let image = create_minimap_image();
    let image_handle = images.add(image);

    commands.spawn((
        Minimap,
        Sprite {
            image: image_handle,
            custom_size: Some(Vec2::new(MINIMAP_SIZE as f32, MINIMAP_SIZE as f32)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 100.0),
    ));
}

fn create_minimap_image() -> Image {
    let size = Extent3d {
        width: MINIMAP_SIZE as u32,
        height: MINIMAP_SIZE as u32,
        depth_or_array_layers: 1,
    };
    let data = vec![30u8; MINIMAP_SIZE * MINIMAP_SIZE * 4]; // Dark background
    Image::new(
        size,
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

fn update_minimap(
    player_query: Query<(&Transform, &PlayerFacing), With<Player>>,
    camera_query: Query<&Transform, (With<GameCamera>, Without<Player>, Without<Minimap>)>,
    camera_proj_query: Query<&OrthographicProjection, With<GameCamera>>,
    chunk_query: Query<&Chunk>,
    mut minimap_query: Query<(&Minimap, &Sprite, &mut Transform), (Without<Player>, Without<GameCamera>)>,
    mut images: ResMut<Assets<Image>>,
    explored: Res<ExploredChunks>,
    minimap_state: Res<MinimapState>,
    dungeon_query: Query<&Transform, (With<DungeonEntrance>, Without<Player>, Without<GameCamera>, Without<Minimap>)>,
    trader_query: Query<&Transform, (With<Trader>, Without<Player>, Without<GameCamera>, Without<Minimap>, Without<DungeonEntrance>)>,
    spawn_point: Res<SpawnPoint>,
    death_stats: Res<DeathStats>,
    mut dirty: ResMut<MinimapDirty>,
) {
    let Ok((player_tf, player_facing)) = player_query.get_single() else { return };
    let Ok(cam_tf) = camera_query.get_single() else { return };
    let Ok((_, sprite, mut minimap_tf)) = minimap_query.get_single_mut() else { return };

    // Position minimap in top-right of screen, accounting for camera zoom
    let Ok(cam_proj) = camera_proj_query.get_single() else { return };
    let zoom = cam_proj.scale;
    // Offset in screen-space pixels, scaled by zoom so it stays in the corner
    minimap_tf.translation.x = cam_tf.translation.x + 520.0 * zoom;
    minimap_tf.translation.y = cam_tf.translation.y + 280.0 * zoom;
    // Scale minimap inversely with zoom so it stays the same screen size
    let minimap_display_scale = zoom * 1.3; // slightly larger than 1:1

    // US-040: Hide minimap if toggled off (always update transform, even when skipping redraw)
    let state = minimap_state.into_inner();
    if !state.minimap_visible && !state.fullscreen_open {
        minimap_tf.scale = Vec3::ZERO;
        return; // Hidden — no point redrawing
    } else if state.fullscreen_open {
        // Fullscreen map: scale up and center on camera
        minimap_tf.translation.x = cam_tf.translation.x;
        minimap_tf.translation.y = cam_tf.translation.y;
        minimap_tf.scale = Vec3::splat(3.0 * zoom);
    } else {
        minimap_tf.scale = Vec3::splat(minimap_display_scale);
    }

    // --- Dirty + frame-skip gate ---
    // Only redraw if dirty, and even then only every MINIMAP_FRAME_SKIP frames.
    if !dirty.dirty {
        return;
    }
    dirty.frame_counter += 1;
    if dirty.frame_counter < MINIMAP_FRAME_SKIP {
        return;
    }
    dirty.frame_counter = 0;
    dirty.dirty = false;

    // --- Full redraw from here ---

    // Update the minimap image
    let image_handle = &sprite.image;
    let Some(image) = images.get_mut(image_handle) else { return };

    let player_world_x = player_tf.translation.x;
    let player_world_y = player_tf.translation.y;

    // Clear image
    for pixel in image.data.chunks_exact_mut(4) {
        pixel[0] = 20;
        pixel[1] = 20;
        pixel[2] = 30;
        pixel[3] = 255;
    }

    // Build a HashMap of loaded chunks for O(1) lookup instead of iterating all chunks per pixel
    let chunk_map: HashMap<IVec2, &Chunk> = chunk_query
        .iter()
        .map(|chunk| (chunk.position, chunk))
        .collect();

    // For each pixel in the minimap, determine the world tile and get its color
    let half = MINIMAP_SIZE as f32 / 2.0;

    for my in 0..MINIMAP_SIZE {
        for mx in 0..MINIMAP_SIZE {
            // Map minimap pixel to world position
            let world_x = player_world_x + (mx as f32 - half) * MINIMAP_SCALE;
            let world_y = player_world_y + (half - my as f32) * MINIMAP_SCALE; // Flip Y

            // Find which chunk and tile this corresponds to
            let chunk_x = (world_x / CHUNK_WORLD_SIZE).floor() as i32;
            let chunk_y = (world_y / CHUNK_WORLD_SIZE).floor() as i32;

            // Check if this chunk has been explored
            let chunk_explored = explored.chunks.contains(&IVec2::new(chunk_x, chunk_y));
            if !chunk_explored {
                // Leave as dark background (already cleared)
                continue;
            }

            let tile_x = ((world_x / TILE_SIZE).floor() as i32 - chunk_x * CHUNK_SIZE as i32).clamp(0, CHUNK_SIZE as i32 - 1) as usize;
            let tile_y = ((world_y / TILE_SIZE).floor() as i32 - chunk_y * CHUNK_SIZE as i32).clamp(0, CHUNK_SIZE as i32 - 1) as usize;

            // O(1) chunk lookup via HashMap
            if let Some(chunk) = chunk_map.get(&IVec2::new(chunk_x, chunk_y)) {
                let tile = chunk.get_tile(tile_x, tile_y);
                let color = tile.color();
                let idx = (my * MINIMAP_SIZE + mx) * 4;
                if idx + 3 < image.data.len() {
                    image.data[idx] = color[0];
                    image.data[idx + 1] = color[1];
                    image.data[idx + 2] = color[2];
                    image.data[idx + 3] = 255;
                }
            }
        }
    }

    // Helper: set a pixel safely with bounds checking
    let size = MINIMAP_SIZE;
    let set_pixel = |data: &mut Vec<u8>, x: i32, y: i32, color: [u8; 4]| {
        if x >= 0 && x < size as i32 && y >= 0 && y < size as i32 {
            let idx = (y as usize * size + x as usize) * 4;
            if idx + 3 < data.len() {
                data[idx] = color[0];
                data[idx + 1] = color[1];
                data[idx + 2] = color[2];
                data[idx + 3] = color[3];
            }
        }
    };

    // Helper: convert world position to minimap pixel coordinates
    let world_to_minimap = |world_x: f32, world_y: f32| -> (i32, i32) {
        let mx = ((world_x - player_world_x) / MINIMAP_SCALE + half).floor() as i32;
        let my = ((player_world_y - world_y) / MINIMAP_SCALE + half).floor() as i32; // Flip Y
        (mx, my)
    };

    // --- POI markers ---

    // Dungeon entrances: purple dots (only if in explored chunks)
    for entrance_tf in dungeon_query.iter() {
        let wx = entrance_tf.translation.x;
        let wy = entrance_tf.translation.y;
        let chunk_x = (wx / CHUNK_WORLD_SIZE).floor() as i32;
        let chunk_y = (wy / CHUNK_WORLD_SIZE).floor() as i32;
        if explored.chunks.contains(&IVec2::new(chunk_x, chunk_y)) {
            let (mx, my) = world_to_minimap(wx, wy);
            let purple = [160, 80, 200, 255];
            for dy in -1..=1_i32 {
                for dx in -1..=1_i32 {
                    set_pixel(&mut image.data, mx + dx, my + dy, purple);
                }
            }
        }
    }

    // Spawn point: green dot
    let (sx, sy) = world_to_minimap(spawn_point.position.x, spawn_point.position.y);
    let green = [80, 200, 80, 255];
    for dy in -1..=1_i32 {
        for dx in -1..=1_i32 {
            set_pixel(&mut image.data, sx + dx, sy + dy, green);
        }
    }

    // Traders: yellow dots
    for trader_tf in trader_query.iter() {
        let (tx, ty) = world_to_minimap(trader_tf.translation.x, trader_tf.translation.y);
        let yellow = [255, 220, 50, 255];
        for dy in -1..=1_i32 {
            for dx in -1..=1_i32 {
                set_pixel(&mut image.data, tx + dx, ty + dy, yellow);
            }
        }
    }

    // Wave 7C: Gravestone marker (red dot) — shows death position on minimap
    if let Some(grave_pos) = death_stats.gravestone_pos {
        let (gx, gy) = world_to_minimap(grave_pos.x, grave_pos.y);
        let red = [220, 50, 50, 255];
        for dy in -1..=1_i32 {
            for dx in -1..=1_i32 {
                set_pixel(&mut image.data, gx + dx, gy + dy, red);
            }
        }
    }

    // --- Player directional arrow (white, at center) ---
    let cx = (MINIMAP_SIZE / 2) as i32;
    let cy = (MINIMAP_SIZE / 2) as i32;
    let white = [255, 255, 255, 255];

    // Center pixel always drawn
    set_pixel(&mut image.data, cx, cy, white);

    match player_facing {
        PlayerFacing::Right => {
            set_pixel(&mut image.data, cx + 1, cy, white);
            set_pixel(&mut image.data, cx, cy - 1, white);
            set_pixel(&mut image.data, cx, cy + 1, white);
        }
        PlayerFacing::Left => {
            set_pixel(&mut image.data, cx - 1, cy, white);
            set_pixel(&mut image.data, cx, cy - 1, white);
            set_pixel(&mut image.data, cx, cy + 1, white);
        }
        PlayerFacing::Up => {
            set_pixel(&mut image.data, cx, cy - 1, white);
            set_pixel(&mut image.data, cx - 1, cy, white);
            set_pixel(&mut image.data, cx + 1, cy, white);
        }
        PlayerFacing::Down => {
            set_pixel(&mut image.data, cx, cy + 1, white);
            set_pixel(&mut image.data, cx - 1, cy, white);
            set_pixel(&mut image.data, cx + 1, cy, white);
        }
    }

    // --- 2px border (dark gray) ---
    let border_color = [40, 40, 40, 255];
    for i in 0..size as i32 {
        set_pixel(&mut image.data, 0, i, border_color);
        set_pixel(&mut image.data, 1, i, border_color);
        set_pixel(&mut image.data, size as i32 - 2, i, border_color);
        set_pixel(&mut image.data, size as i32 - 1, i, border_color);
        set_pixel(&mut image.data, i, 0, border_color);
        set_pixel(&mut image.data, i, 1, border_color);
        set_pixel(&mut image.data, i, size as i32 - 2, border_color);
        set_pixel(&mut image.data, i, size as i32 - 1, border_color);
    }
}

// US-040: Toggle minimap visibility and fullscreen map
fn toggle_minimap(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<MinimapState>,
) {
    if keyboard.just_pressed(KeyCode::KeyN) {
        state.minimap_visible = !state.minimap_visible;
    }
    if keyboard.just_pressed(KeyCode::KeyM) {
        state.fullscreen_open = !state.fullscreen_open;
    }
}
