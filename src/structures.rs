use bevy::prelude::*;
use std::collections::HashSet;

use crate::hud::not_paused;
use crate::inventory::ItemType;
use crate::npc::{self, Invulnerable, Knowledge};
use crate::world::generation::{Biome, WorldGenerator};
use crate::world::chunk::Chunk;
use crate::world::{ChunkObject, WorldState, CHUNK_WORLD_SIZE};
use crate::combat::{Enemy, EnemyType, EnemyState};

pub struct StructuresPlugin;

impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnedStructures::default())
            .add_systems(Update, check_chunk_structures.run_if(not_paused));
    }
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StructureType {
    AbandonedVillage,
    MineShaft,
    TraderOutpost,
    Watchtower,
    FishingDock,
    WolfDen,
    SpiderNest,
    ScorpionBurrow,
}

/// Marker for any world structure entity.
#[derive(Component)]
pub struct WorldStructure {
    pub structure_type: StructureType,
}

/// Marker for supply crates that can be looted.
#[derive(Component)]
pub struct SupplyCrate {
    pub loot: Vec<(ItemType, u32)>,
    pub looted: bool,
}

/// Marker for ruin wall entities in structures.
#[derive(Component)]
pub struct RuinWall;

/// Marker for mine shaft entrance entities.
#[derive(Component)]
pub struct MineEntrance;

/// Marker for structure-based trader NPCs (permanent, non-despawning).
#[derive(Component)]
pub struct StructureTrader;

/// Marker for watchtower entities.
#[derive(Component)]
pub struct Watchtower;

/// Marker for fishing dock entities.
#[derive(Component)]
pub struct FishingDock;

/// Marker for enemy camp loot piles (bone pile, egg sac, etc.).
#[derive(Component)]
pub struct CampLoot {
    pub loot: Vec<(ItemType, u32)>,
    pub looted: bool,
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Tracks which chunks have already been checked for structure spawning.
#[derive(Resource, Default)]
pub struct SpawnedStructures {
    pub processed: HashSet<IVec2>,
}

// ---------------------------------------------------------------------------
// Deterministic loot generation
// ---------------------------------------------------------------------------

/// Generate loot for a supply crate based on a deterministic hash value.
fn generate_crate_loot(hash: u32) -> Vec<(ItemType, u32)> {
    let roll = hash % 6;
    match roll {
        0 => vec![(ItemType::Wood, 10), (ItemType::Stone, 5)],
        1 => vec![(ItemType::IronOre, 3), (ItemType::Coal, 2)],
        2 => vec![(ItemType::Arrow, 15), (ItemType::Torch, 3)],
        3 => vec![(ItemType::HealthPotion, 2)],
        4 => vec![(ItemType::PlantFiber, 8), (ItemType::Rope, 3)],
        _ => vec![(ItemType::Flint, 5), (ItemType::Berry, 8)],
    }
}

// ---------------------------------------------------------------------------
// Structure check system
// ---------------------------------------------------------------------------

fn check_chunk_structures(
    mut commands: Commands,
    world_state: Res<WorldState>,
    chunk_query: Query<&Chunk>,
    mut spawned: ResMut<SpawnedStructures>,
) {
    // Iterate all loaded chunks and process any that haven't been handled yet.
    for chunk in chunk_query.iter() {
        let chunk_pos = chunk.position;

        if spawned.processed.contains(&chunk_pos) {
            continue;
        }

        spawned.processed.insert(chunk_pos);

        let seed = world_state.seed;
        let biome = chunk.biome;

        // Use a structure-specific seed offset to avoid collisions with object placement.
        let struct_hash = WorldGenerator::position_hash(chunk_pos.x, chunk_pos.y, seed.wrapping_add(9999));

        // Check each structure type using modular arithmetic for the desired frequency.
        // Abandoned Village: Forest/Mountain, 1 in 20 chunks
        if matches!(biome, Biome::Forest | Biome::Mountain)
            && (struct_hash % 20) == 0
        {
            spawn_abandoned_village(&mut commands, chunk_pos, seed, biome);
        }
        // Mine Shaft: Mountain/CrystalCave, 1 in 30 chunks
        else if matches!(biome, Biome::Mountain | Biome::CrystalCave)
            && (WorldGenerator::position_hash(chunk_pos.x, chunk_pos.y, seed.wrapping_add(10001)) % 30) == 0
        {
            spawn_mine_shaft(&mut commands, chunk_pos, seed);
        }
        // Trader Outpost: any biome, 1 in 50 chunks
        else if (WorldGenerator::position_hash(chunk_pos.x, chunk_pos.y, seed.wrapping_add(10002)) % 50) == 0
        {
            spawn_trader_outpost(&mut commands, chunk_pos, seed);
        }
        // Watchtower: any biome, 1 in 40 chunks
        else if (WorldGenerator::position_hash(chunk_pos.x, chunk_pos.y, seed.wrapping_add(10003)) % 40) == 0
        {
            spawn_watchtower(&mut commands, chunk_pos, seed);
        }
        // Fishing Dock: Coastal/Swamp, 1 in 25 chunks
        else if matches!(biome, Biome::Coastal | Biome::Swamp)
            && (WorldGenerator::position_hash(chunk_pos.x, chunk_pos.y, seed.wrapping_add(10004)) % 25) == 0
        {
            spawn_fishing_dock(&mut commands, chunk_pos, seed);
        }

        // --- Enemy Camps (Wave 5B) --- separate hash checks at different seed offsets
        // Wolf Den: Forest/Mountain, 1 in 15 chunks
        if matches!(biome, Biome::Forest | Biome::Mountain)
            && (WorldGenerator::position_hash(chunk_pos.x, chunk_pos.y, seed.wrapping_add(11000)) % 15) == 0
        {
            spawn_wolf_den(&mut commands, chunk_pos, seed);
        }
        // Spider Nest: CrystalCave, 1 in 15 chunks
        if matches!(biome, Biome::CrystalCave)
            && (WorldGenerator::position_hash(chunk_pos.x, chunk_pos.y, seed.wrapping_add(11001)) % 15) == 0
        {
            spawn_spider_nest(&mut commands, chunk_pos, seed);
        }
        // Scorpion Burrow: Desert, 1 in 15 chunks
        if matches!(biome, Biome::Desert)
            && (WorldGenerator::position_hash(chunk_pos.x, chunk_pos.y, seed.wrapping_add(11002)) % 15) == 0
        {
            spawn_scorpion_burrow(&mut commands, chunk_pos, seed);
        }
    }
}

// ---------------------------------------------------------------------------
// Structure spawners
// ---------------------------------------------------------------------------

/// Calculate the world-space center of a chunk.
fn chunk_center(chunk_pos: IVec2) -> Vec2 {
    Vec2::new(
        chunk_pos.x as f32 * CHUNK_WORLD_SIZE + CHUNK_WORLD_SIZE / 2.0,
        chunk_pos.y as f32 * CHUNK_WORLD_SIZE + CHUNK_WORLD_SIZE / 2.0,
    )
}

/// Deterministic offset within a chunk for variety.
fn structure_offset(chunk_pos: IVec2, seed: u32) -> Vec2 {
    let hx = WorldGenerator::position_hash(chunk_pos.x, chunk_pos.y, seed.wrapping_add(20000));
    let hy = WorldGenerator::position_hash(chunk_pos.x, chunk_pos.y, seed.wrapping_add(20001));
    // Map to [-64, 64] range within the chunk (chunk is 256 wide, stay inside)
    let ox = ((hx % 128) as f32) - 64.0;
    let oy = ((hy % 128) as f32) - 64.0;
    Vec2::new(ox, oy)
}

fn spawn_abandoned_village(commands: &mut Commands, chunk_pos: IVec2, seed: u32, biome: Biome) {
    let center = chunk_center(chunk_pos) + structure_offset(chunk_pos, seed);

    // 4 RuinWall entities in a rough square
    let wall_offsets = [
        Vec2::new(-16.0, 16.0),
        Vec2::new(16.0, 16.0),
        Vec2::new(-16.0, -16.0),
        Vec2::new(16.0, -16.0),
    ];

    for (i, offset) in wall_offsets.iter().enumerate() {
        let pos = center + *offset;
        commands.spawn((
            WorldStructure { structure_type: StructureType::AbandonedVillage },
            RuinWall,
            ChunkObject { chunk_pos },
            Sprite {
                color: Color::srgb(0.40, 0.38, 0.35),
                custom_size: Some(Vec2::new(16.0, 20.0)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 5.0),
        ));

        // First 2 walls also get a supply crate nearby
        if i < 2 {
            let crate_pos = pos + Vec2::new(8.0, -8.0);
            let loot_hash = WorldGenerator::position_hash(
                chunk_pos.x.wrapping_add(i as i32),
                chunk_pos.y,
                seed.wrapping_add(30000),
            );
            let loot = generate_crate_loot(loot_hash);
            commands.spawn((
                WorldStructure { structure_type: StructureType::AbandonedVillage },
                SupplyCrate { loot, looted: false },
                ChunkObject { chunk_pos },
                Sprite {
                    color: Color::srgb(0.55, 0.40, 0.20),
                    custom_size: Some(Vec2::new(10.0, 8.0)),
                    ..default()
                },
                Transform::from_xyz(crate_pos.x, crate_pos.y, 5.0),
            ));
        }
    }

    // Spawn a Quest Giver NPC in the village center
    npc::spawn_quest_giver(commands, center.x, center.y, chunk_pos);

    // Forest villages also get a Farmer NPC
    if biome == Biome::Forest {
        let farmer_pos = center + Vec2::new(0.0, -20.0);
        npc::spawn_farmer(commands, farmer_pos.x, farmer_pos.y, chunk_pos);
    }
}

fn spawn_mine_shaft(commands: &mut Commands, chunk_pos: IVec2, seed: u32) {
    let center = chunk_center(chunk_pos) + structure_offset(chunk_pos, seed);

    // Mine entrance marker
    commands.spawn((
        WorldStructure { structure_type: StructureType::MineShaft },
        MineEntrance,
        ChunkObject { chunk_pos },
        Sprite {
            color: Color::srgb(0.30, 0.25, 0.20),
            custom_size: Some(Vec2::new(18.0, 14.0)),
            ..default()
        },
        Transform::from_xyz(center.x, center.y, 5.0),
    ));

    // Ore reward chest next to entrance
    let chest_pos = center + Vec2::new(12.0, 0.0);
    let loot_hash = WorldGenerator::position_hash(chunk_pos.x, chunk_pos.y, seed.wrapping_add(31000));
    let loot = match loot_hash % 3 {
        0 => vec![(ItemType::IronOre, 8), (ItemType::Coal, 4)],
        1 => vec![(ItemType::CrystalShard, 3), (ItemType::IronOre, 5)],
        _ => vec![(ItemType::IronOre, 6), (ItemType::Gemstone, 1)],
    };
    commands.spawn((
        WorldStructure { structure_type: StructureType::MineShaft },
        SupplyCrate { loot, looted: false },
        ChunkObject { chunk_pos },
        Sprite {
            color: Color::srgb(0.55, 0.40, 0.20),
            custom_size: Some(Vec2::new(10.0, 8.0)),
            ..default()
        },
        Transform::from_xyz(chest_pos.x, chest_pos.y, 5.0),
    ));

    // Blacksmith NPC near the mine entrance
    let bs_pos = center + Vec2::new(-14.0, 0.0);
    npc::spawn_blacksmith(commands, bs_pos.x, bs_pos.y, chunk_pos);
}

fn spawn_trader_outpost(commands: &mut Commands, chunk_pos: IVec2, seed: u32) {
    let center = chunk_center(chunk_pos) + structure_offset(chunk_pos, seed);

    // Permanent trader NPC
    commands.spawn((
        WorldStructure { structure_type: StructureType::TraderOutpost },
        StructureTrader,
        Invulnerable,
        Knowledge::default(),
        ChunkObject { chunk_pos },
        Sprite {
            color: Color::srgb(0.1, 0.75, 0.2),
            custom_size: Some(Vec2::new(12.0, 12.0)),
            ..default()
        },
        Transform::from_xyz(center.x, center.y, 6.0),
    ));

    // Workbench marker nearby
    let bench_pos = center + Vec2::new(-14.0, 0.0);
    commands.spawn((
        WorldStructure { structure_type: StructureType::TraderOutpost },
        ChunkObject { chunk_pos },
        Sprite {
            color: Color::srgb(0.50, 0.35, 0.15),
            custom_size: Some(Vec2::new(14.0, 10.0)),
            ..default()
        },
        Transform::from_xyz(bench_pos.x, bench_pos.y, 5.0),
    ));
}

fn spawn_watchtower(commands: &mut Commands, chunk_pos: IVec2, seed: u32) {
    let center = chunk_center(chunk_pos) + structure_offset(chunk_pos, seed);

    commands.spawn((
        WorldStructure { structure_type: StructureType::Watchtower },
        Watchtower,
        ChunkObject { chunk_pos },
        Sprite {
            color: Color::srgb(0.45, 0.42, 0.38),
            custom_size: Some(Vec2::new(14.0, 28.0)),
            ..default()
        },
        Transform::from_xyz(center.x, center.y, 5.0),
    ));
}

fn spawn_fishing_dock(commands: &mut Commands, chunk_pos: IVec2, seed: u32) {
    let center = chunk_center(chunk_pos) + structure_offset(chunk_pos, seed);

    commands.spawn((
        WorldStructure { structure_type: StructureType::FishingDock },
        FishingDock,
        ChunkObject { chunk_pos },
        Sprite {
            color: Color::srgb(0.45, 0.35, 0.22),
            custom_size: Some(Vec2::new(20.0, 12.0)),
            ..default()
        },
        Transform::from_xyz(center.x, center.y, 5.0),
    ));
}

// ---------------------------------------------------------------------------
// Wave 5B: Enemy Camp spawners
// ---------------------------------------------------------------------------

/// Helper: spawn a camp enemy at a world position with the given type.
fn spawn_camp_enemy(commands: &mut Commands, pos: Vec2, enemy_type: EnemyType, chunk_pos: IVec2) {
    let (health, damage, speed, aggro_range, color, size) = enemy_type.stats();
    let distance_from_origin = pos.length();
    let hp_mult = if distance_from_origin > 500.0 {
        1.0 + 0.1 * ((distance_from_origin - 500.0) / 500.0).floor()
    } else {
        1.0
    };
    let scaled_hp = health * hp_mult;

    commands.spawn((
        Enemy {
            enemy_type,
            health: scaled_hp,
            max_health: scaled_hp,
            damage,
            speed,
            aggro_range,
            state: EnemyState::Idle,
            patrol_target: pos,
            attack_cooldown: bevy::time::Timer::from_seconds(1.0, TimerMode::Once),
            detection_range: 120.0,
            attack_cooldown_timer: 0.0,
            patrol_direction: Vec2::new(1.0, 0.0),
            patrol_timer: 3.0,
            alert_timer: 0.0,
            distance_from_origin,
            ability_cooldown_timer: 0.0,
        },
        ChunkObject { chunk_pos },
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 5.0),
    ));
}

/// Wolf Den: 3-4 wolves + bone pile loot in Forest/Mountain biomes.
fn spawn_wolf_den(commands: &mut Commands, chunk_pos: IVec2, seed: u32) {
    let center = chunk_center(chunk_pos) + structure_offset(chunk_pos, seed);

    // Den structure marker
    commands.spawn((
        WorldStructure { structure_type: StructureType::WolfDen },
        ChunkObject { chunk_pos },
        Sprite {
            color: Color::srgb(0.35, 0.30, 0.25),
            custom_size: Some(Vec2::new(18.0, 14.0)),
            ..default()
        },
        Transform::from_xyz(center.x, center.y, 5.0),
    ));

    // Determine wolf count (3-4) from hash
    let count_hash = WorldGenerator::position_hash(chunk_pos.x, chunk_pos.y, seed.wrapping_add(12000));
    let wolf_count = 3 + (count_hash % 2) as usize;

    let wolf_offsets = [
        Vec2::new(-18.0, 12.0),
        Vec2::new(16.0, 10.0),
        Vec2::new(-10.0, -16.0),
        Vec2::new(20.0, -12.0),
    ];

    for i in 0..wolf_count {
        let pos = center + wolf_offsets[i];
        spawn_camp_enemy(commands, pos, EnemyType::FeralWolf, chunk_pos);
    }

    // Bone pile loot
    let loot_pos = center + Vec2::new(0.0, -20.0);
    commands.spawn((
        WorldStructure { structure_type: StructureType::WolfDen },
        CampLoot {
            loot: vec![(ItemType::TitanBone, 1), (ItemType::PlantFiber, 4)],
            looted: false,
        },
        ChunkObject { chunk_pos },
        Sprite {
            color: Color::srgb(0.75, 0.7, 0.6),
            custom_size: Some(Vec2::new(10.0, 8.0)),
            ..default()
        },
        Transform::from_xyz(loot_pos.x, loot_pos.y, 5.0),
    ));
}

/// Spider Nest: 5-6 spiders + egg sac entities in CrystalCave biome.
fn spawn_spider_nest(commands: &mut Commands, chunk_pos: IVec2, seed: u32) {
    let center = chunk_center(chunk_pos) + structure_offset(chunk_pos, seed);

    // Nest structure marker
    commands.spawn((
        WorldStructure { structure_type: StructureType::SpiderNest },
        ChunkObject { chunk_pos },
        Sprite {
            color: Color::srgb(0.25, 0.20, 0.15),
            custom_size: Some(Vec2::new(22.0, 18.0)),
            ..default()
        },
        Transform::from_xyz(center.x, center.y, 5.0),
    ));

    // Determine spider count (5-6) from hash
    let count_hash = WorldGenerator::position_hash(chunk_pos.x, chunk_pos.y, seed.wrapping_add(12001));
    let spider_count = 5 + (count_hash % 2) as usize;

    let spider_offsets = [
        Vec2::new(-20.0, 14.0),
        Vec2::new(18.0, 12.0),
        Vec2::new(-14.0, -16.0),
        Vec2::new(22.0, -10.0),
        Vec2::new(0.0, 20.0),
        Vec2::new(-6.0, -22.0),
    ];

    for i in 0..spider_count {
        let pos = center + spider_offsets[i];
        spawn_camp_enemy(commands, pos, EnemyType::CaveSpider, chunk_pos);
    }

    // Egg sac loot entities (2)
    let egg_offsets = [Vec2::new(-8.0, -24.0), Vec2::new(10.0, -22.0)];
    for (i, offset) in egg_offsets.iter().enumerate() {
        let pos = center + *offset;
        let loot_hash = WorldGenerator::position_hash(
            chunk_pos.x.wrapping_add(i as i32),
            chunk_pos.y,
            seed.wrapping_add(12010),
        );
        let loot = match loot_hash % 3 {
            0 => vec![(ItemType::CrystalShard, 2), (ItemType::PlantFiber, 3)],
            1 => vec![(ItemType::CrystalShard, 3)],
            _ => vec![(ItemType::PlantFiber, 4), (ItemType::Rope, 1)],
        };
        commands.spawn((
            WorldStructure { structure_type: StructureType::SpiderNest },
            CampLoot { loot, looted: false },
            ChunkObject { chunk_pos },
            Sprite {
                color: Color::srgb(0.6, 0.55, 0.45),
                custom_size: Some(Vec2::new(8.0, 8.0)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 5.0),
        ));
    }
}

/// Scorpion Burrow: 2-3 scorpions + VenomScorpion in Desert biome.
fn spawn_scorpion_burrow(commands: &mut Commands, chunk_pos: IVec2, seed: u32) {
    let center = chunk_center(chunk_pos) + structure_offset(chunk_pos, seed);

    // Burrow structure marker
    commands.spawn((
        WorldStructure { structure_type: StructureType::ScorpionBurrow },
        ChunkObject { chunk_pos },
        Sprite {
            color: Color::srgb(0.6, 0.5, 0.3),
            custom_size: Some(Vec2::new(16.0, 12.0)),
            ..default()
        },
        Transform::from_xyz(center.x, center.y, 5.0),
    ));

    // Determine regular scorpion count (2-3) from hash
    let count_hash = WorldGenerator::position_hash(chunk_pos.x, chunk_pos.y, seed.wrapping_add(12002));
    let scorp_count = 2 + (count_hash % 2) as usize;

    let scorp_offsets = [
        Vec2::new(-16.0, 10.0),
        Vec2::new(14.0, 8.0),
        Vec2::new(0.0, -18.0),
    ];

    for i in 0..scorp_count {
        let pos = center + scorp_offsets[i];
        spawn_camp_enemy(commands, pos, EnemyType::SandScorpion, chunk_pos);
    }

    // Elite VenomScorpion guarding the burrow
    let elite_pos = center + Vec2::new(0.0, 14.0);
    spawn_camp_enemy(commands, elite_pos, EnemyType::VenomScorpion, chunk_pos);
}
