use bevy::prelude::*;
use rand::Rng;
use crate::hud::not_paused;
use crate::player::Player;
use crate::combat::{Enemy, EnemyType, EnemyState, Boss, spawn_health_bar_children};
use crate::inventory::ItemType;
use crate::world::generation::{WorldGenerator, Biome};
use crate::gathering::spawn_dropped_item;
use crate::audio::SoundEvent;

pub struct DungeonPlugin;

impl Plugin for DungeonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DungeonRegistry>()
            .add_systems(Update, (
                check_dungeon_entrance,
                check_dungeon_exit,
                dungeon_chest_interaction,
            ).run_if(not_paused));
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marks a world-surface dungeon entrance portal sprite.
#[derive(Component)]
pub struct DungeonEntrance {
    /// World-surface position so we can teleport the player back here.
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub surface_pos: Vec2,
}

/// Marks floor/wall tiles that belong to a dungeon interior.
#[derive(Component)]
pub struct DungeonTile;

/// Marks an enemy that was spawned inside a dungeon so we can clean them up
/// when the player leaves.
#[derive(Component)]
pub struct DungeonEnemy;

/// A loot chest spawned inside a dungeon room. Interactable with E key.
/// Drops random items from the dungeon loot table, then despawns.
#[derive(Component)]
pub struct DungeonChest {
    pub opened: bool,
}

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
    dungeon_chests_query: Query<Entity, With<DungeonChest>>,
) {
    // Tick cooldown.
    if registry.cooldown > 0.0 {
        registry.cooldown -= time.delta_secs();
        return;
    }

    let Some((_dungeon_id, surface_pos)) = registry.current_dungeon else { return };

    let Ok(mut player_tf) = player_query.get_single_mut() else { return };
    let player_pos = player_tf.translation.truncate();

    for (exit_entity, _exit, exit_tf) in exit_query.iter() {
        let exit_pos = exit_tf.translation.truncate();
        if player_pos.distance(exit_pos) <= EXIT_TRIGGER_RADIUS {
            // Teleport player back to surface.
            player_tf.translation.x = surface_pos.x;
            player_tf.translation.y = surface_pos.y;

            registry.current_dungeon = None;
            registry.cooldown = 1.0;

            // Despawn the dungeon exit.
            commands.entity(exit_entity).despawn();

            // Despawn all dungeon tiles, enemies, and chests.
            for entity in dungeon_tiles_query.iter() {
                commands.entity(entity).despawn();
            }
            for entity in dungeon_enemies_query.iter() {
                commands.entity(entity).despawn();
            }
            for entity in dungeon_chests_query.iter() {
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
    _dungeon_id: u32,
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

    let (floor_color, _wall_color) = dungeon_colors(biome);
    let mut room_centers: Vec<Vec2> = Vec::new();

    for room_idx in 0..num_rooms {
        let room_origin = Vec2::new(
            anchor.x + room_idx as f32 * step_x,
            anchor.y,
        );

        // Spawn floor tiles for this room (biome-tinted).
        for ty in 0..room_height_tiles {
            for tx in 0..room_width_tiles {
                let tile_pos = Vec2::new(
                    room_origin.x + tx as f32 * DTILE + DTILE / 2.0,
                    room_origin.y + ty as f32 * DTILE + DTILE / 2.0,
                );
                spawn_floor_tile_colored(commands, tile_pos, floor_color);
            }
        }

        // Determine doorway openings for this room.
        let has_left_doorway = room_idx > 0;
        let has_right_doorway = room_idx < num_rooms - 1;

        // Spawn wall tiles around the perimeter (with doorway gaps).
        let (_, wall_color) = dungeon_colors(biome);
        spawn_room_walls(commands, room_origin, room_width_tiles, room_height_tiles, (has_left_doorway, has_right_doorway), wall_color);

        // Spawn a corridor connecting this room to the previous one.
        if room_idx > 0 {
            let prev_room_origin = Vec2::new(
                anchor.x + (room_idx - 1) as f32 * step_x,
                anchor.y,
            );
            spawn_corridor(commands, prev_room_origin, room_width_tiles, room_height_tiles);
        }

        // Record center of this room.
        let center = Vec2::new(
            room_origin.x + room_width_tiles as f32 * DTILE / 2.0,
            room_origin.y + room_height_tiles as f32 * DTILE / 2.0,
        );
        room_centers.push(center);
    }

    // Spawn biome-appropriate dungeon enemies (3–8) in all rooms except the last.
    let dungeon_enemy_type = dungeon_enemy_for_biome(biome);
    let num_enemies: usize = rng.gen_range(3..=8);
    let non_boss_rooms = if num_rooms > 1 { num_rooms - 1 } else { 1 };
    for i in 0..num_enemies {
        let room_idx = i % non_boss_rooms;
        let center = room_centers[room_idx];
        let offset = Vec2::new(
            rng.gen_range(-24.0..24.0),
            rng.gen_range(-16.0..16.0),
        );
        let spawn_pos = center + offset;
        spawn_dungeon_enemy(commands, spawn_pos, dungeon_enemy_type);
    }

    // US-036: Spawn loot chests in non-boss rooms (60% chance per room).
    for room_idx in 0..non_boss_rooms {
        if rng.gen::<f32>() < 0.6 {
            let center = room_centers[room_idx];
            let chest_offset = Vec2::new(
                rng.gen_range(-16.0..16.0),
                rng.gen_range(-12.0..12.0),
            );
            let chest_pos = center + chest_offset;
            spawn_dungeon_chest(commands, chest_pos);
        }
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

/// Returns biome-specific floor and wall colors for dungeon generation.
fn dungeon_colors(biome: Biome) -> (Color, Color) {
    match biome {
        Biome::Tundra => (
            Color::srgb(0.3, 0.32, 0.38),   // icy floor
            Color::srgb(0.2, 0.22, 0.3),     // blue-tinted walls
        ),
        Biome::Volcanic => (
            Color::srgb(0.32, 0.2, 0.15),    // scorched floor
            Color::srgb(0.3, 0.12, 0.08),    // red-tinted walls
        ),
        Biome::Fungal => (
            Color::srgb(0.22, 0.28, 0.2),    // mossy floor
            Color::srgb(0.15, 0.22, 0.12),   // green-tinted walls
        ),
        Biome::Swamp => (
            Color::srgb(0.2, 0.25, 0.18),    // murky floor
            Color::srgb(0.12, 0.18, 0.1),    // dark green walls
        ),
        _ => (
            Color::srgb(0.28, 0.25, 0.22),   // default stone floor
            Color::srgb(0.18, 0.16, 0.14),   // default walls
        ),
    }
}

/// Returns the enemy type to spawn in dungeon rooms for a given biome.
fn dungeon_enemy_for_biome(biome: Biome) -> EnemyType {
    match biome {
        Biome::Tundra => EnemyType::IceWraith,
        Biome::Volcanic => EnemyType::LavaElemental,
        Biome::Fungal => EnemyType::FungalZombie,
        Biome::Swamp => EnemyType::BogLurker,
        _ => EnemyType::CaveSpider,
    }
}

fn spawn_floor_tile(commands: &mut Commands, pos: Vec2) {
    spawn_floor_tile_colored(commands, pos, Color::srgb(0.28, 0.25, 0.22));
}

fn spawn_floor_tile_colored(commands: &mut Commands, pos: Vec2, color: Color) {
    commands.spawn((
        DungeonTile,
        Sprite {
            color,
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
    doorways: (bool, bool),
    wall_color: Color,
) {
    // Spawn one tile outside the perimeter on all four sides as "wall" tiles.
    let wall_size = Vec2::new(DTILE, DTILE);

    // Compute the 2-tile doorway opening indices (vertically centered).
    let door_lo = height / 2 - 1;
    let door_hi = height / 2;

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
        let is_doorway_row = ty == door_lo || ty == door_hi;

        for &tx_edge in &[-1i32, width as i32] {
            // Skip wall tile if this is a doorway opening.
            let is_left_wall = tx_edge == -1;
            let is_right_wall = tx_edge == width as i32;

            if is_doorway_row {
                if is_left_wall && doorways.0 {
                    continue;
                }
                if is_right_wall && doorways.1 {
                    continue;
                }
            }

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

/// Fills the gap between two adjacent rooms with floor tiles so the player can
/// walk through the doorway opening.  The corridor is 2 tiles wide (matching
/// the doorway height) and covers the wall positions that were skipped by
/// the doorway openings.
fn spawn_corridor(
    commands: &mut Commands,
    left_room_origin: Vec2,
    room_width: usize,
    room_height: usize,
) {
    // The doorway opening indices (vertically centered, matching spawn_room_walls).
    let door_lo = room_height / 2 - 1;
    let door_hi = room_height / 2;

    // The two X positions in the gap:
    //   - room_width      : where the left room's right wall was removed
    //   - room_width + 1  : where the right room's left wall was removed
    for &tx in &[room_width as i32, room_width as i32 + 1] {
        for &ty in &[door_lo, door_hi] {
            let pos = Vec2::new(
                left_room_origin.x + tx as f32 * DTILE + DTILE / 2.0,
                left_room_origin.y + ty as f32 * DTILE + DTILE / 2.0,
            );
            spawn_floor_tile(commands, pos);
        }
    }
}

fn spawn_dungeon_enemy(commands: &mut Commands, pos: Vec2, enemy_type: EnemyType) {
    let (health, damage, speed, aggro_range, color, size) = enemy_type.stats();

    let mut rng = rand::thread_rng();
    let patrol_dir = Vec2::new(
        rng.gen_range(-1.0f32..1.0),
        rng.gen_range(-1.0f32..1.0),
    ).normalize_or_zero();

    let entity = commands.spawn((
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
            detection_range: 120.0,
            attack_cooldown_timer: 0.0,
            patrol_direction: patrol_dir,
            patrol_timer: rng.gen_range(2.0..4.0),
            alert_timer: 0.0,
            distance_from_origin: pos.length(),
            ability_cooldown_timer: 0.0,
        },
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 5.0),
    )).id();
    spawn_health_bar_children(commands, entity, size.y);
}

/// Spawns the dungeon boss in the last room.
///
/// Uses `boss_for_biome` (US-008) to pick the right enemy type and builds a
/// matching loot table.  Falls back to `StoneGolem` with the generic loot
/// table when the biome has no dedicated boss variant.
fn spawn_dungeon_boss(commands: &mut Commands, pos: Vec2, biome: Biome) {
    use crate::combat::{boss_for_biome, BossAbility, BossAbilityType};

    let boss_type = boss_for_biome(biome);
    let (base_health, damage, speed, aggro_range, color, size) = boss_type.stats();

    // US-032: Boss HP scales with distance from world origin (+20% per 1000px)
    let distance_from_origin = pos.length();
    let boss_hp_multiplier = 1.0 + 0.2 * (distance_from_origin / 1000.0).floor();
    let health = base_health * boss_hp_multiplier;

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
        EnemyType::TidalSerpent => vec![
            (ItemType::AncientCore, 1),
            (ItemType::Gemstone, 1),
            (ItemType::Blueprint, 1),
            (ItemType::CoralEssence, 1),
        ],
        EnemyType::MountainTitan => vec![
            (ItemType::AncientCore, 1),
            (ItemType::Gemstone, 1),
            (ItemType::Blueprint, 1),
            (ItemType::TitanBone, 1),
        ],
        // Generic StoneGolem fallback
        _ => vec![
            (ItemType::AncientCore, 1),
            (ItemType::Gemstone, 1),
            (ItemType::Blueprint, 1),
        ],
    };

    let boss_name = format!("{:?}", boss_type);

    let mut rng = rand::thread_rng();
    let patrol_dir = Vec2::new(
        rng.gen_range(-1.0f32..1.0),
        rng.gen_range(-1.0f32..1.0),
    ).normalize_or_zero();

    let mut entity_cmds = commands.spawn((
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
            detection_range: 200.0,
            attack_cooldown_timer: 0.0,
            patrol_direction: patrol_dir,
            patrol_timer: rng.gen_range(2.0..4.0),
            alert_timer: 0.0,
            distance_from_origin,
            ability_cooldown_timer: 0.0,
        },
        Boss {
            name: boss_name,
            loot_table,
            has_roared: false,
            phase_2: false,
        },
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 5.0),
    ));

    // Attach unique boss ability if this boss type has one
    if let Some(ability_type) = BossAbilityType::for_enemy(boss_type) {
        entity_cmds.insert(BossAbility::new(ability_type));
    }

    let entity = entity_cmds.id();
    spawn_health_bar_children(commands, entity, size.y);
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

/// Spawns a gold-colored loot chest entity inside a dungeon room.
fn spawn_dungeon_chest(commands: &mut Commands, pos: Vec2) {
    commands.spawn((
        DungeonChest { opened: false },
        DungeonTile, // so it's cleaned up on exit
        Sprite {
            color: Color::srgb(0.85, 0.75, 0.2), // gold
            custom_size: Some(Vec2::new(10.0, 10.0)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 4.0),
    ));
}

/// Returns a random set of 2-4 items from the dungeon loot table.
fn dungeon_loot_table(rng: &mut impl Rng) -> Vec<(ItemType, u32)> {
    let possible_items: Vec<(ItemType, u32, u32)> = vec![
        // (item, count, weight) — higher weight = more common
        (ItemType::IronOre, 2, 20),
        (ItemType::CrystalShard, 2, 20),
        (ItemType::HealthPotion, 1, 15),
        (ItemType::Arrow, 5, 20),
        (ItemType::Blueprint, 1, 10),       // rare 10%
        (ItemType::JournalPage, 1, 15),     // rare 15%
    ];

    let num_items = rng.gen_range(2..=4);
    let mut result = Vec::new();
    let total_weight: u32 = possible_items.iter().map(|i| i.2).sum();

    for _ in 0..num_items {
        let roll = rng.gen_range(0..total_weight);
        let mut cumulative = 0;
        for &(item, count, weight) in &possible_items {
            cumulative += weight;
            if roll < cumulative {
                result.push((item, count));
                break;
            }
        }
    }

    result
}

/// Returns a random drop for a cave spider death (30% chance).
/// Drops one of: CaveSlime, SpiderSilk, or Berry.
pub fn cave_spider_random_drop(rng: &mut impl Rng) -> Option<(ItemType, u32)> {
    if rng.gen::<f32>() >= 0.3 {
        return None;
    }
    let drops = [
        (ItemType::CaveSlime, 1),
        (ItemType::SpiderSilk, 1),
        (ItemType::Berry, 2),
    ];
    Some(drops[rng.gen_range(0..drops.len())])
}

/// System: E key near a dungeon chest opens it and drops loot.
fn dungeon_chest_interaction(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    game_settings: Res<crate::settings::GameSettings>,
    player_query: Query<&Transform, With<Player>>,
    mut chest_query: Query<(Entity, &mut DungeonChest, &Transform), Without<Player>>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    if !keyboard.just_pressed(game_settings.keybinds.interact) {
        return;
    }

    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    // Find nearest unopened chest within 32px
    let mut nearest: Option<(Entity, f32)> = None;
    for (entity, chest, tf) in chest_query.iter() {
        if chest.opened {
            continue;
        }
        let dist = player_pos.distance(tf.translation.truncate());
        if dist <= 32.0 {
            if nearest.is_none() || dist < nearest.unwrap().1 {
                nearest = Some((entity, dist));
            }
        }
    }

    let Some((target, _)) = nearest else { return };

    if let Ok((entity, mut chest, tf)) = chest_query.get_mut(target) {
        chest.opened = true;
        let chest_pos = tf.translation.truncate();

        // Generate loot and spawn as dropped items
        let mut rng = rand::thread_rng();
        let loot = dungeon_loot_table(&mut rng);
        for (item, count) in loot {
            spawn_dropped_item(&mut commands, chest_pos, item, count, &mut rng);
        }

        sound_events.send(SoundEvent::MenuOpen);

        // Despawn the chest
        commands.entity(entity).despawn();
    }
}

// ---------------------------------------------------------------------------
// Public helper used by world/mod.rs to spawn a DungeonEntrance entity
// ---------------------------------------------------------------------------

/// Spawns a `DungeonEntrance` marker at the given world-space position and
/// assigns it a fresh ID from the registry.
#[allow(dead_code)]
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
            color: Color::srgb(0.6, 0.2, 0.7),
            custom_size: Some(Vec2::new(18.0, 18.0)),
            ..default()
        },
        Transform::from_xyz(world_x, world_y, 2.5),
    ));
}
