use bevy::prelude::*;
use rand::Rng;
use crate::player::Player;
use crate::combat::{Enemy, EnemyType, EnemyState, Boss};
use crate::inventory::ItemType;
use crate::world::generation::{WorldGenerator, Biome};

pub struct DungeonPlugin;

impl Plugin for DungeonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DungeonRegistry>()
            .add_systems(Update, (
                check_dungeon_entrance,
                check_dungeon_exit,
            ));
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marks a world-surface dungeon entrance portal sprite.
#[derive(Component)]
pub struct DungeonEntrance {
    /// World-surface position so we can teleport the player back here.
    pub surface_pos: Vec2,
    /// Unique ID used to look up the matching DungeonInstance.
    pub id: u32,
    /// Biome this entrance was spawned in — used to select the boss type.
    pub biome: Biome,
}

/// Marks the exit portal spawned inside a dungeon.
#[derive(Component)]
pub struct DungeonExit {
    /// Surface position to return to.
    pub surface_pos: Vec2,
}

/// Marks floor/wall tiles that belong to a dungeon interior.
#[derive(Component)]
pub struct DungeonTile;

/// Marks an enemy that was spawned inside a dungeon so we can clean them up
/// when the player leaves.
#[derive(Component)]
pub struct DungeonEnemy;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Tracks the next available dungeon id and a small cooldown so the
/// entrance-trigger does not fire every frame.
#[derive(Resource)]
pub struct DungeonRegistry {
    pub next_id: u32,
    /// When Some, the player is inside the dungeon with this id and their
    /// surface return position is stored here.
    pub current_dungeon: Option<(u32, Vec2)>,
    /// Brief cooldown (in seconds) to prevent instant re-triggering after
    /// a teleport.
    pub cooldown: f32,
}

impl Default for DungeonRegistry {
    fn default() -> Self {
        Self {
            next_id: 0,
            current_dungeon: None,
            cooldown: 0.0,
        }
    }
}

impl DungeonRegistry {
    pub fn allocate_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Y-offset base for dungeon interiors (each dungeon is spaced 2000 units apart
/// so their geometry never overlaps).
const DUNGEON_BASE_Y: f32 = -10_000.0;
const DUNGEON_SPACING: f32 = 2_000.0;

/// Tile size used when building dungeon rooms (same as surface tile size).
const DTILE: f32 = 16.0;

/// Trigger radius: if the player comes within this distance of an entrance the
/// teleport fires.
const ENTRANCE_TRIGGER_RADIUS: f32 = 16.0;

/// Exit trigger radius.
const EXIT_TRIGGER_RADIUS: f32 = 16.0;

// ---------------------------------------------------------------------------
// Public helper – called from world/mod.rs
// ---------------------------------------------------------------------------

/// Deterministically decide whether a dungeon entrance should be placed at this
/// world tile position (for Mountain / Volcanic / CrystalCave biomes).
/// Returns `true` with ~1% probability.
pub fn should_spawn_entrance(world_tile_x: i32, world_tile_y: i32, seed: u32) -> bool {
    // Use a different seed offset so this doesn't collide with object hashes.
    let hash = WorldGenerator::position_hash(world_tile_x, world_tile_y, seed.wrapping_add(9999));
    (hash % 100) == 0 // exactly 1 %
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Every frame: tick the cooldown, then check if the player is near any entrance.
fn check_dungeon_entrance(
    mut commands: Commands,
    time: Res<Time>,
    mut registry: ResMut<DungeonRegistry>,
    mut player_query: Query<&mut Transform, With<Player>>,
    entrance_query: Query<(&DungeonEntrance, &Transform), Without<Player>>,
) {
    // Tick cooldown.
    if registry.cooldown > 0.0 {
        registry.cooldown -= time.delta_secs();
        return;
    }

    // Only trigger from the surface (not already inside a dungeon).
    if registry.current_dungeon.is_some() {
        return;
    }

    let Ok(mut player_tf) = player_query.get_single_mut() else { return };
    let player_pos = player_tf.translation.truncate();

    for (entrance, entrance_tf) in entrance_query.iter() {
        let entrance_pos = entrance_tf.translation.truncate();
        if player_pos.distance(entrance_pos) <= ENTRANCE_TRIGGER_RADIUS {
            let surface_pos = player_pos;
            let dungeon_id = entrance.id;
            let biome = entrance.biome;

            // Compute the dungeon's anchor position in world space.
            let dungeon_anchor = dungeon_world_pos(dungeon_id);

            // Teleport player to the dungeon's entrance point.
            let player_dungeon_pos = dungeon_anchor + Vec2::new(0.0, 32.0);
            player_tf.translation.x = player_dungeon_pos.x;
            player_tf.translation.y = player_dungeon_pos.y;
            // Keep Z.

            // Record that we are now inside this dungeon.
            registry.current_dungeon = Some((dungeon_id, surface_pos));
            registry.cooldown = 1.0;

            // Generate the dungeon interior.
            generate_dungeon(&mut commands, dungeon_id, dungeon_anchor, surface_pos, biome);

            break;
        }
    }
}

/// Every frame: while inside a dungeon, check if the player is near the exit.
fn check_dungeon_exit(
    mut commands: Commands,
    time: Res<Time>,
    mut registry: ResMut<DungeonRegistry>,
    mut player_query: Query<&mut Transform, With<Player>>,
    exit_query: Query<(Entity, &DungeonExit, &Transform), Without<Player>>,
    dungeon_tiles_query: Query<Entity, With<DungeonTile>>,
    dungeon_enemies_query: Query<Entity, With<DungeonEnemy>>,
) {
    // Tick cooldown.
    if registry.cooldown > 0.0 {
        registry.cooldown -= time.delta_secs();
        return;
    }

    let Some((_dungeon_id, surface_pos)) = registry.current_dungeon else { return };

    let Ok(mut player_tf) = player_query.get_single_mut() else { return };
    let player_pos = player_tf.translation.truncate();

    for (exit_entity, exit, exit_tf) in exit_query.iter() {
        let exit_pos = exit_tf.translation.truncate();
        if player_pos.distance(exit_pos) <= EXIT_TRIGGER_RADIUS {
            // Teleport player back to surface.
            player_tf.translation.x = surface_pos.x;
            player_tf.translation.y = surface_pos.y;

            registry.current_dungeon = None;
            registry.cooldown = 1.0;

            // Despawn the dungeon exit.
            commands.entity(exit_entity).despawn();

            // Despawn all dungeon tiles and enemies.
            for entity in dungeon_tiles_query.iter() {
                commands.entity(entity).despawn();
            }
            for entity in dungeon_enemies_query.iter() {
                commands.entity(entity).despawn();
            }

            break;
        }
    }
}

// ---------------------------------------------------------------------------
// Dungeon generation
// ---------------------------------------------------------------------------

/// Returns the world-space anchor (bottom-left of the first room) for a dungeon.
fn dungeon_world_pos(dungeon_id: u32) -> Vec2 {
    Vec2::new(0.0, DUNGEON_BASE_Y - dungeon_id as f32 * DUNGEON_SPACING)
}

/// Generates 3–5 rectangular rooms connected in a line, spawns CaveSpiders
/// in all but the last room, spawns a biome boss in the last room, and
/// places an exit portal adjacent to the boss.
fn generate_dungeon(
    commands: &mut Commands,
    dungeon_id: u32,
    anchor: Vec2,
    surface_pos: Vec2,
    biome: Biome,
) {
    let mut rng = rand::thread_rng();

    let num_rooms: usize = rng.gen_range(3..=5);

    // Room layout: place them left-to-right with small gaps.
    let room_width_tiles: usize = rng.gen_range(6..=10);
    let room_height_tiles: usize = rng.gen_range(5..=8);
    let gap_tiles: usize = 2;

    let step_x = (room_width_tiles + gap_tiles) as f32 * DTILE;

    let mut room_centers: Vec<Vec2> = Vec::new();

    for room_idx in 0..num_rooms {
        let room_origin = Vec2::new(
            anchor.x + room_idx as f32 * step_x,
            anchor.y,
        );

        // Spawn floor tiles for this room.
        for ty in 0..room_height_tiles {
            for tx in 0..room_width_tiles {
                let tile_pos = Vec2::new(
                    room_origin.x + tx as f32 * DTILE + DTILE / 2.0,
                    room_origin.y + ty as f32 * DTILE + DTILE / 2.0,
                );
                spawn_floor_tile(commands, tile_pos);
            }
        }

        // Spawn wall tiles around the perimeter.
        spawn_room_walls(commands, room_origin, room_width_tiles, room_height_tiles);

        // Record center of this room.
        let center = Vec2::new(
            room_origin.x + room_width_tiles as f32 * DTILE / 2.0,
            room_origin.y + room_height_tiles as f32 * DTILE / 2.0,
        );
        room_centers.push(center);
    }

    // Spawn CaveSpiders (3–8) spread across all rooms except the last one.
    let num_spiders: usize = rng.gen_range(3..=8);
    let non_boss_rooms = if num_rooms > 1 { num_rooms - 1 } else { 1 };
    for i in 0..num_spiders {
        let room_idx = i % non_boss_rooms;
        let center = room_centers[room_idx];
        let offset = Vec2::new(
            rng.gen_range(-24.0..24.0),
            rng.gen_range(-16.0..16.0),
        );
        let spawn_pos = center + offset;
        spawn_cave_spider(commands, spawn_pos);
    }

    // Spawn the boss in the last room (US-007 / US-008).
    let boss_center = *room_centers.last().unwrap_or(&anchor);
    spawn_dungeon_boss(commands, boss_center, biome);

    // Spawn exit portal offset slightly from the boss so the player can reach it
    // after defeating the boss.
    let exit_pos = boss_center + Vec2::new(0.0, -40.0);
    spawn_dungeon_exit(commands, exit_pos, surface_pos);

    // Spawn a visual label sprite for the dungeon entrance point so the
    // player can see where they entered (placed at dungeon anchor + player
    // drop-in offset).
    let entry_marker = anchor + Vec2::new(0.0, 32.0);
    commands.spawn((
        DungeonTile,
        Sprite {
            color: Color::srgba(0.6, 0.4, 0.8, 0.5),
            custom_size: Some(Vec2::new(DTILE, DTILE)),
            ..default()
        },
        Transform::from_xyz(entry_marker.x, entry_marker.y, 3.0),
    ));
}

fn spawn_floor_tile(commands: &mut Commands, pos: Vec2) {
    commands.spawn((
        DungeonTile,
        Sprite {
            color: Color::srgb(0.28, 0.25, 0.22),
            custom_size: Some(Vec2::new(DTILE, DTILE)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 0.5),
    ));
}

fn spawn_room_walls(
    commands: &mut Commands,
    room_origin: Vec2,
    width: usize,
    height: usize,
) {
    // Spawn one tile outside the perimeter on all four sides as "wall" tiles.
    let wall_color = Color::srgb(0.18, 0.16, 0.14);
    let wall_size = Vec2::new(DTILE, DTILE);

    // Top and bottom rows (including corners, one tile outside).
    for tx in -1i32..=(width as i32) {
        for &ty_edge in &[-1i32, height as i32] {
            let pos = Vec2::new(
                room_origin.x + tx as f32 * DTILE + DTILE / 2.0,
                room_origin.y + ty_edge as f32 * DTILE + DTILE / 2.0,
            );
            commands.spawn((
                DungeonTile,
                Sprite {
                    color: wall_color,
                    custom_size: Some(wall_size),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y, 0.5),
            ));
        }
    }

    // Left and right columns (excluding already-spawned corner rows).
    for ty in 0..height {
        for &tx_edge in &[-1i32, width as i32] {
            let pos = Vec2::new(
                room_origin.x + tx_edge as f32 * DTILE + DTILE / 2.0,
                room_origin.y + ty as f32 * DTILE + DTILE / 2.0,
            );
            commands.spawn((
                DungeonTile,
                Sprite {
                    color: wall_color,
                    custom_size: Some(wall_size),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y, 0.5),
            ));
        }
    }
}

fn spawn_cave_spider(commands: &mut Commands, pos: Vec2) {
    let enemy_type = EnemyType::CaveSpider;
    let (health, damage, speed, aggro_range, color, size) = enemy_type.stats();

    commands.spawn((
        DungeonEnemy,
        Enemy {
            enemy_type,
            health,
            max_health: health,
            damage,
            speed,
            aggro_range,
            state: EnemyState::Idle,
            patrol_target: pos,
            attack_cooldown: Timer::from_seconds(1.0, TimerMode::Once),
        },
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 5.0),
    ));
}

/// Spawns the dungeon boss in the last room.
///
/// Uses `boss_for_biome` (US-008) to pick the right enemy type and builds a
/// matching loot table.  Falls back to `StoneGolem` with the generic loot
/// table when the biome has no dedicated boss variant.
fn spawn_dungeon_boss(commands: &mut Commands, pos: Vec2, biome: Biome) {
    use crate::combat::boss_for_biome;

    let boss_type = boss_for_biome(biome);
    let (health, damage, speed, aggro_range, color, size) = boss_type.stats();

    // Build a biome-specific loot table.  Every boss drops an AncientCore,
    // a Gemstone, and a Blueprint, plus their unique biome drop.
    let loot_table: Vec<(ItemType, u32)> = match boss_type {
        EnemyType::ForestGuardian  => vec![
            (ItemType::AncientCore, 1),
            (ItemType::Gemstone, 1),
            (ItemType::Blueprint, 1),
            (ItemType::GuardianHeart, 1),
        ],
        EnemyType::SwampBeast      => vec![
            (ItemType::AncientCore, 1),
            (ItemType::Gemstone, 1),
            (ItemType::Blueprint, 1),
            (ItemType::SwampEssence, 1),
        ],
        EnemyType::DesertWyrm      => vec![
            (ItemType::AncientCore, 1),
            (ItemType::Gemstone, 1),
            (ItemType::Blueprint, 1),
            (ItemType::WyrmScale, 1),
        ],
        EnemyType::FrostGiant      => vec![
            (ItemType::AncientCore, 1),
            (ItemType::Gemstone, 1),
            (ItemType::Blueprint, 1),
            (ItemType::FrostGem, 1),
        ],
        EnemyType::MagmaKing       => vec![
            (ItemType::AncientCore, 1),
            (ItemType::Gemstone, 1),
            (ItemType::Blueprint, 1),
            (ItemType::MagmaCore, 1),
        ],
        EnemyType::FungalOverlord  => vec![
            (ItemType::AncientCore, 1),
            (ItemType::Gemstone, 1),
            (ItemType::Blueprint, 1),
            (ItemType::FungalSporeEssence, 1),
        ],
        EnemyType::CrystalSentinel => vec![
            (ItemType::AncientCore, 1),
            (ItemType::Gemstone, 1),
            (ItemType::Blueprint, 1),
            (ItemType::CrystalHeart, 1),
        ],
        // Generic StoneGolem (Coastal / Mountain fallback)
        _ => vec![
            (ItemType::AncientCore, 1),
            (ItemType::Gemstone, 1),
            (ItemType::Blueprint, 1),
        ],
    };

    let boss_name = format!("{:?}", boss_type);

    commands.spawn((
        DungeonEnemy,
        Enemy {
            enemy_type: boss_type,
            health,
            max_health: health,
            damage,
            speed,
            aggro_range,
            state: EnemyState::Idle,
            patrol_target: pos,
            attack_cooldown: Timer::from_seconds(1.5, TimerMode::Once),
        },
        Boss {
            name: boss_name,
            loot_table,
        },
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 5.0),
    ));
}

fn spawn_dungeon_exit(commands: &mut Commands, pos: Vec2, surface_pos: Vec2) {
    // Pulsing green portal marker.
    commands.spawn((
        DungeonExit { surface_pos },
        Sprite {
            color: Color::srgb(0.2, 0.85, 0.4),
            custom_size: Some(Vec2::new(DTILE, DTILE)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 3.0),
    ));
}

// ---------------------------------------------------------------------------
// Public helper used by world/mod.rs to spawn a DungeonEntrance entity
// ---------------------------------------------------------------------------

/// Spawns a `DungeonEntrance` marker at the given world-space position and
/// assigns it a fresh ID from the registry.
pub fn spawn_entrance(
    commands: &mut Commands,
    registry: &mut DungeonRegistry,
    world_x: f32,
    world_y: f32,
    chunk_pos: IVec2,
) {
    // Default biome for spawned entrances — world/mod.rs can pass the real
    // biome if it wants biome-specific bosses. We default to Mountain so the
    // StoneGolem boss is used when no biome information is available.
    spawn_entrance_with_biome(commands, registry, world_x, world_y, chunk_pos, Biome::Mountain);
}

/// Spawns a `DungeonEntrance` with an explicit biome so the correct boss is
/// selected when the player enters.
pub fn spawn_entrance_with_biome(
    commands: &mut Commands,
    registry: &mut DungeonRegistry,
    world_x: f32,
    world_y: f32,
    chunk_pos: IVec2,
    biome: Biome,
) {
    let id = registry.allocate_id();
    let surface_pos = Vec2::new(world_x, world_y);

    commands.spawn((
        DungeonEntrance { surface_pos, id, biome },
        // Use ChunkObject-equivalent tagging by storing chunk_pos inside the
        // entrance so the world's chunk-unload system can clean it up.
        crate::world::ChunkObject { chunk_pos },
        Sprite {
            color: Color::srgb(0.5, 0.15, 0.6),
            custom_size: Some(Vec2::new(14.0, 14.0)),
            ..default()
        },
        Transform::from_xyz(world_x, world_y, 2.5),
    ));
}
