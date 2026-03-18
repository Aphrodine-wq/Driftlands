use bevy::prelude::*;
use rand::Rng;
use std::collections::HashSet;
use crate::hud::not_paused;
use crate::player::{Player, Health, Hunger, ActiveBuff, BuffType, ArmorSlots};
use crate::daynight::{DayNightCycle, DayPhase};
use crate::season::SeasonCycle;
use crate::inventory::{Inventory, ItemType};
use crate::world::chunk::Chunk;
use crate::world::generation::Biome;
use crate::world::{CHUNK_WORLD_SIZE};
use crate::npc::Invulnerable;
use crate::building::{Building, BuildingType, Door};
use crate::camera::{ScreenShake, HitStop};
use crate::death::DeathStats;
use crate::particles::SpawnParticlesEvent;
use crate::audio::SoundEvent;
use crate::hud::FloatingTextRequest;
use crate::gathering::spawn_dropped_item;
use crate::dungeon::cave_spider_random_drop;
use crate::enchanting::enemy_on_hit_effect;
use crate::status_effects::ApplyStatusEvent;
use crate::weather::WeatherSystem;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ResearchPointEvent>()
            .add_event::<PlayerHitEvent>()
            .insert_resource(DodgeCooldown::default())
            .insert_resource(ComboState::default())
            .insert_resource(WeaponSpecialState::default())
            .insert_resource(PlayerHitQueue::default())
            .add_systems(Update, (
                spawn_night_enemies,
                despawn_enemies_at_sunrise,
                enemy_ai,
                player_attack,
                enemy_attack_player,
                update_hit_flash,
            ).run_if(not_paused))
            .add_systems(Update, (
                boss_death_loot,
                update_enemy_health_bars,
                projectile_movement,
                projectile_hit,
                knockback_system,
                update_slash_arcs,
            ).run_if(not_paused))
            .add_systems(Update, (
                enemy_ranged_attacks,
                enemy_projectile_hit_player,
                update_ice_spike_aoe,
                update_burn_zones,
                update_dive_bombs,
                spawn_burn_zones_from_magma,
            ).run_if(not_paused))
            .add_systems(Update, (
                drain_player_hit_queue,
                apply_weapon_effects,
                dodge_roll_input,
                dodge_roll_tick,
                shield_block_input,
            ).run_if(not_paused))
            .add_systems(Update, (
                combo_tracker,
                weapon_specials,
            ).run_if(not_paused));
    }
}

// --- Events ---

/// Fired whenever the player earns research points.
#[derive(Event)]
pub struct ResearchPointEvent {
    pub amount: u32,
}

/// Fired when the player lands a melee hit on an enemy.
/// Used by enchanting/combo/weapon special systems without touching player_attack params.
#[derive(Event)]
pub struct PlayerHitEvent {
    pub target: Entity,
    pub weapon: ItemType,
    pub damage: f32,
    pub player_pos: Vec2,
    pub enemy_pos: Vec2,
}

// --- Components ---

#[derive(Component)]
pub struct Enemy {
    pub enemy_type: EnemyType,
    pub health: f32,
    pub max_health: f32,
    pub damage: f32,
    pub speed: f32,
    #[allow(dead_code)]
    pub aggro_range: f32,
    pub state: EnemyState,
    #[allow(dead_code)]
    pub patrol_target: Vec2,
    pub attack_cooldown: Timer,
    /// How far the enemy can detect the player (default 120, bosses 200).
    pub detection_range: f32,
    /// Cooldown timer for attack state (seconds remaining).
    pub attack_cooldown_timer: f32,
    /// Current patrol movement direction.
    pub patrol_direction: Vec2,
    /// Time remaining in current patrol direction (seconds).
    pub patrol_timer: f32,
    /// Timer used for the Alert pause before transitioning to Chase.
    pub alert_timer: f32,
    /// Distance from world origin at spawn time (US-032: used for HP scaling).
    #[allow(dead_code)]
    pub distance_from_origin: f32,
    /// Cooldown for ranged/ability attack (seconds remaining).
    pub ability_cooldown_timer: f32,
}

/// Marks an enemy as a boss and carries its name and loot table.
/// When the enemy's health drops to zero the `boss_death_loot` system
/// adds every entry in `loot_table` to the player's inventory before
/// the entity is despawned.
#[derive(Component)]
pub struct Boss {
    #[allow(dead_code)]
    pub name: String,
    pub loot_table: Vec<(ItemType, u32)>,
    pub has_roared: bool,
    /// Phase 2 at or below 50% HP: faster and hits harder (PRD 4.2).
    pub phase_2: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnemyType {
    // --- regular night/biome enemies ---
    ShadowCrawler,
    FeralWolf,
    NightBat,
    CaveSpider,
    FungalZombie,
    LavaElemental,
    IceWraith,
    BogLurker,
    SandScorpion,
    // --- elite enemies (expansion) ---
    AlphaWolf,
    VenomScorpion,
    FrostLich,
    MagmaGolem,
    // --- dungeon boss (US-007) ---
    #[allow(dead_code)]
    StoneGolem,
    // --- biome bosses (US-008) ---
    ForestGuardian,
    SwampBeast,
    DesertWyrm,
    FrostGiant,
    MagmaKing,
    FungalOverlord,
    CrystalSentinel,
    TidalSerpent,
    MountainTitan,
}

impl EnemyType {
    pub fn stats(&self) -> (f32, f32, f32, f32, Color, Vec2) {
        // (health, damage, speed, aggro_range, color, size)
        match self {
            EnemyType::ShadowCrawler => (30.0, 5.0, 80.0, 150.0, Color::srgb(0.4, 0.1, 0.5), Vec2::new(10.0, 10.0)),
            EnemyType::FeralWolf => (40.0, 8.0, 100.0, 180.0, Color::srgb(0.5, 0.5, 0.5), Vec2::new(12.0, 10.0)),
            EnemyType::NightBat => (18.0, 4.0, 130.0, 140.0, Color::srgb(0.2, 0.15, 0.25), Vec2::new(8.0, 8.0)),
            EnemyType::CaveSpider => (20.0, 4.0, 120.0, 120.0, Color::srgb(0.3, 0.2, 0.15), Vec2::new(8.0, 8.0)),
            EnemyType::FungalZombie => (50.0, 6.0, 40.0, 100.0, Color::srgb(0.3, 0.5, 0.2), Vec2::new(12.0, 14.0)),
            EnemyType::LavaElemental => (60.0, 12.0, 50.0, 130.0, Color::srgb(0.9, 0.3, 0.1), Vec2::new(14.0, 14.0)),
            EnemyType::IceWraith => (35.0, 7.0, 70.0, 160.0, Color::srgb(0.7, 0.85, 1.0), Vec2::new(10.0, 12.0)),
            EnemyType::BogLurker => (45.0, 6.0, 60.0, 100.0, Color::srgb(0.25, 0.4, 0.2), Vec2::new(12.0, 12.0)),
            EnemyType::SandScorpion => (30.0, 8.0, 90.0, 140.0, Color::srgb(0.7, 0.55, 0.3), Vec2::new(10.0, 8.0)),
            // Elite enemies
            EnemyType::AlphaWolf     => (80.0, 12.0, 110.0, 200.0, Color::srgb(0.35, 0.35, 0.4), Vec2::new(14.0, 12.0)),
            EnemyType::VenomScorpion => (60.0, 14.0, 95.0, 160.0, Color::srgb(0.5, 0.7, 0.2), Vec2::new(12.0, 10.0)),
            EnemyType::FrostLich     => (70.0, 10.0, 65.0, 180.0, Color::srgb(0.5, 0.6, 0.9), Vec2::new(12.0, 14.0)),
            EnemyType::MagmaGolem    => (120.0, 16.0, 30.0, 150.0, Color::srgb(0.8, 0.3, 0.05), Vec2::new(16.0, 16.0)),
            // Dungeon boss
            EnemyType::StoneGolem => (200.0, 15.0, 30.0, 200.0, Color::srgb(0.6, 0.6, 0.6), Vec2::new(20.0, 20.0)),
            // Biome bosses
            EnemyType::ForestGuardian  => (200.0, 12.0, 40.0, 200.0, Color::srgb(0.2, 0.6, 0.15), Vec2::new(20.0, 20.0)),
            EnemyType::SwampBeast      => (180.0, 14.0, 35.0, 200.0, Color::srgb(0.15, 0.35, 0.1), Vec2::new(22.0, 22.0)),
            EnemyType::DesertWyrm      => (250.0, 18.0, 45.0, 200.0, Color::srgb(0.8, 0.65, 0.3), Vec2::new(22.0, 20.0)),
            EnemyType::FrostGiant      => (280.0, 16.0, 25.0, 200.0, Color::srgb(0.6, 0.8, 1.0), Vec2::new(24.0, 24.0)),
            EnemyType::MagmaKing       => (300.0, 20.0, 20.0, 200.0, Color::srgb(0.9, 0.4, 0.1), Vec2::new(24.0, 24.0)),
            EnemyType::FungalOverlord  => (160.0, 10.0, 50.0, 200.0, Color::srgb(0.5, 0.2, 0.6), Vec2::new(18.0, 18.0)),
            EnemyType::CrystalSentinel => (220.0, 15.0, 30.0, 200.0, Color::srgb(0.6, 0.5, 0.8), Vec2::new(20.0, 22.0)),
            EnemyType::TidalSerpent   => (240.0, 16.0, 35.0, 200.0, Color::srgb(0.2, 0.5, 0.8), Vec2::new(22.0, 20.0)),
            EnemyType::MountainTitan  => (260.0, 17.0, 25.0, 200.0, Color::srgb(0.5, 0.45, 0.35), Vec2::new(24.0, 24.0)),
        }
    }

    /// Wave 7C: Human-readable display name for death recap.
    pub fn display_name(&self) -> &'static str {
        match self {
            EnemyType::ShadowCrawler => "Shadow Crawler",
            EnemyType::FeralWolf => "Feral Wolf",
            EnemyType::NightBat => "Night Bat",
            EnemyType::CaveSpider => "Cave Spider",
            EnemyType::FungalZombie => "Fungal Zombie",
            EnemyType::LavaElemental => "Lava Elemental",
            EnemyType::IceWraith => "Ice Wraith",
            EnemyType::BogLurker => "Bog Lurker",
            EnemyType::SandScorpion => "Sand Scorpion",
            EnemyType::AlphaWolf => "Alpha Wolf",
            EnemyType::VenomScorpion => "Venom Scorpion",
            EnemyType::FrostLich => "Frost Lich",
            EnemyType::MagmaGolem => "Magma Golem",
            EnemyType::StoneGolem => "Stone Golem",
            EnemyType::ForestGuardian => "Forest Guardian",
            EnemyType::SwampBeast => "Swamp Beast",
            EnemyType::DesertWyrm => "Desert Wyrm",
            EnemyType::FrostGiant => "Frost Giant",
            EnemyType::MagmaKing => "Magma King",
            EnemyType::FungalOverlord => "Fungal Overlord",
            EnemyType::CrystalSentinel => "Crystal Sentinel",
            EnemyType::TidalSerpent => "Tidal Serpent",
            EnemyType::MountainTitan => "Mountain Titan",
        }
    }

    pub fn for_biome(biome: Biome) -> Self {
        match biome {
            Biome::Forest => EnemyType::FeralWolf,
            Biome::Coastal => EnemyType::ShadowCrawler,
            Biome::Swamp => EnemyType::BogLurker,
            Biome::Desert => EnemyType::SandScorpion,
            Biome::Tundra => EnemyType::IceWraith,
            Biome::Volcanic => EnemyType::LavaElemental,
            Biome::Fungal => EnemyType::FungalZombie,
            Biome::CrystalCave => EnemyType::CaveSpider,
            Biome::Mountain => EnemyType::FeralWolf,
        }
    }
}

/// Returns the boss enemy type for a given biome (US-008).
pub fn boss_for_biome(biome: Biome) -> EnemyType {
    match biome {
        Biome::Forest      => EnemyType::ForestGuardian,
        Biome::Swamp       => EnemyType::SwampBeast,
        Biome::Desert      => EnemyType::DesertWyrm,
        Biome::Tundra      => EnemyType::FrostGiant,
        Biome::Volcanic    => EnemyType::MagmaKing,
        Biome::Fungal      => EnemyType::FungalOverlord,
        Biome::CrystalCave => EnemyType::CrystalSentinel,
        Biome::Coastal     => EnemyType::TidalSerpent,
        Biome::Mountain    => EnemyType::MountainTitan,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnemyState {
    Idle,
    Patrol,
    Alert,
    Chase,
    Attack,
}

#[derive(Component)]
pub struct HitFlash {
    pub timer: Timer,
    pub original_color: Color,
}

#[derive(Component)]
pub struct EnemyHealthBar {
    pub parent_enemy: Entity,
    pub is_fill: bool,
}

#[derive(Component)]
pub struct PlayerAttackCooldown {
    pub timer: Timer,
}

#[derive(Component)]
pub struct Projectile {
    pub velocity: Vec2,
    pub damage: f32,
    pub lifetime: f32,
}

/// Marker component to distinguish enemy-fired projectiles from player arrows.
#[derive(Component)]
pub struct EnemyProjectile;

/// AoE marker spawned by FrostLich: after a delay, deals damage in a radius.
#[derive(Component)]
pub struct IceSpikeAoE {
    pub delay: f32,
    pub damage: f32,
    pub radius: f32,
    pub position: Vec2,
}

/// Burn zone left by LavaElemental magma projectiles.
#[derive(Component)]
pub struct BurnZone {
    pub damage_per_sec: f32,
    pub radius: f32,
    pub lifetime: f32,
}

/// NightBat dive-bomb telegraph marker.
#[derive(Component)]
pub struct DiveBomb {
    pub telegraph_timer: f32,
    pub damage: f32,
    pub target_pos: Vec2,
    pub recovery_timer: f32,
}

#[derive(Component)]
pub struct Knockback {
    pub direction: Vec2,
    pub timer: f32,
}

/// Visual slash arc spawned at the player on melee attack.
#[derive(Component)]
pub struct SlashArc {
    pub timer: f32,
    pub max_timer: f32,
}

// --- Dodge Roll ---

/// Player is dodging: invulnerable, 3x speed in facing direction.
#[derive(Component)]
pub struct Dodging {
    pub timer: f32,
    pub direction: Vec2,
}

/// Cooldown between dodge rolls.
#[derive(Resource)]
pub struct DodgeCooldown {
    pub timer: f32,
}

impl Default for DodgeCooldown {
    fn default() -> Self { Self { timer: 0.0 } }
}

// --- Shield Blocking ---

/// Player is actively blocking with a shield.
#[derive(Component)]
pub struct Blocking {
    pub start_time: f32,
}

// --- Combo System ---

/// Tracks combo hits for sequential melee attacks.
#[derive(Resource)]
pub struct ComboState {
    pub count: u32,
    pub timer: f32,
}

impl Default for ComboState {
    fn default() -> Self { Self { count: 0, timer: 0.0 } }
}

// --- Weapon Specials ---

/// Tracks per-weapon hit counters for special abilities.
#[derive(Resource, Default)]
pub struct WeaponSpecialState {
    pub flame_hits: u32,
    pub frost_hits: u32,
    pub venom_consecutive: u32,
    pub venom_last_target: Option<Entity>,
}

/// Queue for PlayerHitEvents generated by player_attack (avoids 17th param).
#[derive(Resource, Default)]
pub struct PlayerHitQueue {
    pub pending: Vec<PlayerHitEvent>,
}

// --- Loot ---

fn loot_for_enemy(enemy_type: EnemyType) -> (ItemType, u32) {
    match enemy_type {
        EnemyType::FeralWolf => (ItemType::Wood, 2),
        EnemyType::ShadowCrawler => (ItemType::PlantFiber, 2),
        EnemyType::NightBat => (ItemType::PlantFiber, 1),
        EnemyType::CaveSpider => (ItemType::CrystalShard, 1),
        EnemyType::FungalZombie => (ItemType::MushroomCap, 2),
        EnemyType::LavaElemental => (ItemType::Sulfur, 2),
        EnemyType::IceWraith => (ItemType::IceShard, 2),
        EnemyType::BogLurker => (ItemType::Reed, 2),
        EnemyType::SandScorpion => (ItemType::CactusFiber, 2),
        EnemyType::AlphaWolf => (ItemType::RareHerb, 2),
        EnemyType::VenomScorpion => (ItemType::Sulfur, 3),
        EnemyType::FrostLich => (ItemType::FrostGem, 1),
        EnemyType::MagmaGolem => (ItemType::ObsidianShard, 3),
        _ => (ItemType::Stone, 2),
    }
}

fn damage_text_style(damage: f32, reference_max: f32) -> (String, Color) {
    let ratio = if reference_max > 0.0 {
        (damage / reference_max).clamp(0.0, 1.0)
    } else {
        0.0
    };

    if ratio >= 0.6 {
        // Crit-like big hit
        (format!("-{:.0}!!", damage), Color::srgb(0.95, 0.8, 0.35))
    } else if ratio >= 0.3 {
        // Strong hit
        (format!("-{:.0}!", damage), Color::srgb(1.0, 0.5, 0.2))
    } else {
        // Normal chip damage
        (format!("-{:.0}", damage), Color::srgb(1.0, 0.3, 0.3))
    }
}

// --- Helpers ---

/// Returns the appropriate texture for an enemy type, with the tint color.
fn enemy_sprite(enemy_type: EnemyType, assets: &crate::assets::GameAssets) -> (Handle<Image>, Color) {
    let (_hp, _dmg, _spd, _aggro, color, _size) = enemy_type.stats();
    let texture = match enemy_type {
        EnemyType::FeralWolf => assets.enemy_wolf.clone(),
        EnemyType::CaveSpider => assets.enemy_spider.clone(),
        EnemyType::ShadowCrawler => assets.enemy_crawler.clone(),
        EnemyType::NightBat => assets.enemy_crawler.clone(),
        EnemyType::FungalZombie => assets.enemy_zombie.clone(),
        EnemyType::LavaElemental => assets.enemy_elemental.clone(),
        EnemyType::IceWraith => assets.enemy_wraith.clone(),
        EnemyType::BogLurker => assets.enemy_zombie.clone(),
        EnemyType::SandScorpion | EnemyType::VenomScorpion => assets.enemy_scorpion.clone(),
        EnemyType::AlphaWolf => assets.enemy_wolf.clone(),
        EnemyType::FrostLich => assets.enemy_wraith.clone(),
        EnemyType::MagmaGolem => assets.enemy_elemental.clone(),
        // All bosses use the boss texture with their stat color as tint
        EnemyType::ForestGuardian | EnemyType::SwampBeast | EnemyType::DesertWyrm
        | EnemyType::FrostGiant | EnemyType::MagmaKing | EnemyType::FungalOverlord
        | EnemyType::CrystalSentinel | EnemyType::TidalSerpent | EnemyType::MountainTitan
        | EnemyType::StoneGolem => assets.enemy_boss.clone(),
    };
    (texture, color)
}

// --- Systems ---

fn spawn_night_enemies(
    mut commands: Commands,
    cycle: Res<DayNightCycle>,
    season: Res<SeasonCycle>,
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<&Enemy>,
    chunk_query: Query<&Chunk>,
    assets: Res<crate::assets::GameAssets>,
    death_stats: Res<DeathStats>,
) {
    if cycle.phase_with_season(season.current) != DayPhase::Night {
        return;
    }

    // Wave 7C: Death difficulty scaling — each death increases spawn cap by 5%
    let death_multiplier = 1.0 + 0.05 * death_stats.death_count as f32;

    // US-032: Spawn cap scales with day count — 5 + (day_count / 5), capped at 20
    // Wave 7C: Further scaled by death count
    let base_cap = (5 + (cycle.day_count / 5) as usize).min(20);
    let spawn_cap = (base_cap as f32 * death_multiplier).ceil() as usize;
    if enemy_query.iter().count() >= spawn_cap {
        return;
    }

    // US-032: Night spawn rate increases each day — 1% base + 0.1% per day, capped at 3%
    let spawn_chance = (0.01 + 0.001 * cycle.day_count as f32).min(0.03);
    let mut rng = rand::thread_rng();
    if rng.gen::<f32>() > spawn_chance {
        return;
    }

    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    // Determine biome at player position
    let player_chunk_x = (player_tf.translation.x / CHUNK_WORLD_SIZE).floor() as i32;
    let player_chunk_y = (player_tf.translation.y / CHUNK_WORLD_SIZE).floor() as i32;
    let biome = chunk_query.iter()
        .find(|c| c.position.x == player_chunk_x && c.position.y == player_chunk_y)
        .map(|c| c.biome)
        .unwrap_or(Biome::Forest);

    // 5% elite spawn chance scaling with day count (+0.5% per day, capped at 15%)
    let elite_chance = (0.05 + 0.005 * cycle.day_count as f32).min(0.15);
    let enemy_type = if rng.gen::<f32>() < 0.15 {
        EnemyType::NightBat
    } else if rng.gen::<f32>() < elite_chance {
        // Spawn biome-appropriate elite
        match biome {
            Biome::Forest | Biome::Mountain => EnemyType::AlphaWolf,
            Biome::Desert | Biome::Coastal => EnemyType::VenomScorpion,
            Biome::Tundra | Biome::CrystalCave => EnemyType::FrostLich,
            Biome::Volcanic => EnemyType::MagmaGolem,
            Biome::Swamp => EnemyType::VenomScorpion,
            Biome::Fungal => EnemyType::FrostLich,
        }
    } else {
        EnemyType::for_biome(biome)
    };
    let (health, damage, speed, aggro_range, _color, size) = enemy_type.stats();
    let (texture, tint) = enemy_sprite(enemy_type, &assets);

    let angle = rng.gen::<f32>() * std::f32::consts::TAU;
    let dist = rng.gen_range(300.0..500.0);
    let spawn_pos = player_pos + Vec2::new(angle.cos(), angle.sin()) * dist;

    // US-032: Scale HP based on spawn distance from world origin (+10% per 500px beyond 500px)
    let distance_from_origin = spawn_pos.length();
    let hp_multiplier = if distance_from_origin > 500.0 {
        1.0 + 0.1 * ((distance_from_origin - 500.0) / 500.0).floor()
    } else {
        1.0
    };
    let scaled_health = health * hp_multiplier;

    let patrol_dir = Vec2::new(
        rng.gen_range(-1.0f32..1.0),
        rng.gen_range(-1.0f32..1.0),
    ).normalize_or_zero();

    commands.spawn((
        Enemy {
            enemy_type,
            health: scaled_health,
            max_health: scaled_health,
            damage,
            speed,
            aggro_range,
            state: EnemyState::Idle,
            patrol_target: spawn_pos,
            attack_cooldown: Timer::from_seconds(1.0, TimerMode::Once),
            detection_range: 120.0,
            attack_cooldown_timer: 0.0,
            patrol_direction: patrol_dir,
            patrol_timer: rng.gen_range(2.0..4.0),
            alert_timer: 0.0,
            distance_from_origin,
            ability_cooldown_timer: 0.0,
        },
        Sprite {
            image: texture,
            color: tint,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(spawn_pos.x, spawn_pos.y, 5.0),
    ));
}

fn despawn_enemies_at_sunrise(
    mut commands: Commands,
    cycle: Res<DayNightCycle>,
    season: Res<SeasonCycle>,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    if cycle.phase_with_season(season.current) != DayPhase::Sunrise {
        return;
    }

    for entity in enemy_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn enemy_ai(
    mut _commands: Commands,
    mut enemy_query: Query<(&mut Enemy, &mut Transform, Option<&mut Boss>), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
    time: Res<Time>,
    building_query: Query<(&Transform, &Building, Option<&Door>), (Without<Player>, Without<Enemy>)>,
    mut sound_events: EventWriter<SoundEvent>,
    weather: Res<WeatherSystem>,
) {
    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    let mut rng = rand::thread_rng();
    let dt = time.delta_secs();

    // Fog halves enemy detection range
    let fog_mult = if weather.current == crate::weather::Weather::Fog { 0.5 } else { 1.0 };

    for (mut enemy, mut tf, mut maybe_boss) in enemy_query.iter_mut() {
        let enemy_pos = tf.translation.truncate();
        let dist_to_player = enemy_pos.distance(player_pos);

        if enemy.attack_cooldown_timer > 0.0 {
            enemy.attack_cooldown_timer -= dt;
        }
        if enemy.ability_cooldown_timer > 0.0 {
            enemy.ability_cooldown_timer -= dt;
        }

        let phase_mult = maybe_boss.as_ref().map(|b| if b.phase_2 { 1.2 } else { 1.0 }).unwrap_or(1.0);
        let speed = enemy.speed * phase_mult;
        let effective_detection = enemy.detection_range * fog_mult;

        // State machine
        match enemy.state {
            EnemyState::Idle => {
                // Check if player is within detection range (fog halves it)
                if dist_to_player <= effective_detection {
                    enemy.state = EnemyState::Alert;
                    enemy.alert_timer = 0.5;
                    // Boss roar on first alert
                    if let Some(ref mut boss) = maybe_boss {
                        if !boss.has_roared {
                            boss.has_roared = true;
                            sound_events.send(SoundEvent::BossRoar);
                        }
                    }
                } else {
                    // Transition to patrol with a random direction
                    enemy.patrol_direction = Vec2::new(
                        rng.gen_range(-1.0f32..1.0),
                        rng.gen_range(-1.0f32..1.0),
                    ).normalize_or_zero();
                    enemy.patrol_timer = rng.gen_range(2.0..4.0);
                    enemy.state = EnemyState::Patrol;
                }
            }
            EnemyState::Patrol => {
                // Check if player is within detection range (fog halves it)
                if dist_to_player <= effective_detection {
                    enemy.state = EnemyState::Alert;
                    enemy.alert_timer = 0.5;
                    // Boss roar on first alert
                    if let Some(ref mut boss) = maybe_boss {
                        if !boss.has_roared {
                            boss.has_roared = true;
                            sound_events.send(SoundEvent::BossRoar);
                        }
                    }
                } else {
                    let move_delta = enemy.patrol_direction * speed * 0.5 * dt;
                    let new_x = tf.translation.x + move_delta.x;
                    let new_y = tf.translation.y + move_delta.y;
                    if !is_blocked_by_building_enemy(new_x, new_y, &building_query) {
                        tf.translation.x = new_x;
                        tf.translation.y = new_y;
                    }

                    // Decrement patrol timer; pick new direction when expired
                    enemy.patrol_timer -= dt;
                    if enemy.patrol_timer <= 0.0 {
                        enemy.patrol_direction = Vec2::new(
                            rng.gen_range(-1.0f32..1.0),
                            rng.gen_range(-1.0f32..1.0),
                        ).normalize_or_zero();
                        enemy.patrol_timer = rng.gen_range(2.0..4.0);
                    }
                }
            }
            EnemyState::Alert => {
                // Pause for alert_timer seconds, then transition to Chase
                enemy.alert_timer -= dt;
                if enemy.alert_timer <= 0.0 {
                    enemy.state = EnemyState::Chase;
                }
            }
            EnemyState::Chase => {
                if dist_to_player > effective_detection + 80.0 {
                    enemy.patrol_direction = Vec2::new(
                        rng.gen_range(-1.0f32..1.0),
                        rng.gen_range(-1.0f32..1.0),
                    ).normalize_or_zero();
                    enemy.patrol_timer = rng.gen_range(2.0..4.0);
                    enemy.state = EnemyState::Patrol;
                } else if dist_to_player <= 24.0 {
                    enemy.state = EnemyState::Attack;
                } else {
                    let dir = (player_pos - enemy_pos).normalize_or_zero();
                    let move_delta = dir * speed * dt;
                    let new_x = tf.translation.x + move_delta.x;
                    let new_y = tf.translation.y + move_delta.y;
                    if !is_blocked_by_building_enemy(new_x, new_y, &building_query) {
                        tf.translation.x = new_x;
                        tf.translation.y = new_y;
                    }
                }
            }
            EnemyState::Attack => {
                // Damage is applied by the separate `enemy_attack_player` system.
                // Set cooldown and transition back to Chase.
                if enemy.attack_cooldown_timer <= 0.0 {
                    enemy.attack_cooldown_timer = 1.0;
                }
                enemy.state = EnemyState::Chase;
            }
        }
    }
}

fn is_blocked_by_building_enemy(
    x: f32,
    y: f32,
    building_query: &Query<(&Transform, &Building, Option<&Door>), (Without<Player>, Without<Enemy>)>,
) -> bool {
    let half = 5.0;
    for (tf, building, door) in building_query.iter() {
        let blocks = match building.building_type {
            BuildingType::WoodWall | BuildingType::StoneWall | BuildingType::MetalWall | BuildingType::WoodFence => true,
            BuildingType::WoodDoor | BuildingType::StoneDoor | BuildingType::MetalDoor => {
                door.map(|d| !d.is_open).unwrap_or(true)
            }
            _ => false,
        };
        if !blocks {
            continue;
        }
        let bpos = tf.translation.truncate();
        let bsize = building.building_type.size();
        let half_w = bsize.x / 2.0;
        let half_h = bsize.y / 2.0;
        if x + half > bpos.x - half_w
            && x - half < bpos.x + half_w
            && y + half > bpos.y - half_h
            && y - half < bpos.y + half_h
        {
            return true;
        }
    }
    false
}

fn player_attack(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    building_state: Res<crate::building::BuildingState>,
    mut player_query: Query<(Entity, &Transform, Option<&ActiveBuff>), With<Player>>,
    mut enemy_query: Query<(Entity, &Transform, &mut Enemy, &mut Sprite, Option<&Boss>), (Without<Player>, Without<Invulnerable>)>,
    mut cooldown_query: Query<&mut PlayerAttackCooldown>,
    mut inventory: ResMut<Inventory>,
    mut rp_events: EventWriter<ResearchPointEvent>,
    mut screen_shake: ResMut<ScreenShake>,
    mut hit_stop: ResMut<HitStop>,
    mut death_stats: ResMut<DeathStats>,
    mut particle_events: EventWriter<SpawnParticlesEvent>,
    mut sound_events: EventWriter<SoundEvent>,
    mut floating_text_events: EventWriter<FloatingTextRequest>,
    mut hit_queue: ResMut<PlayerHitQueue>,
) {
    // Don't attack in build mode
    if building_state.active {
        return;
    }

    // Handle cooldown
    if let Ok(mut cd) = cooldown_query.get_single_mut() {
        cd.timer.tick(time.delta());
        if !cd.timer.finished() {
            return;
        }
    }

    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok((player_entity, player_tf, maybe_buff)) = player_query.get_single_mut() else { return };
    let player_pos = player_tf.translation.truncate();

    // Ranged attack with bow
    let is_bow = inventory.selected_item()
        .map(|s| s.item == ItemType::WoodBow)
        .unwrap_or(false);
    if is_bow {
        if !inventory.has_items(ItemType::Arrow, 1) {
            return;
        }
        // Find nearest enemy for aim direction, or shoot right
        let mut aim_dir = Vec2::X;
        let mut nearest_dist = f32::MAX;
        for (_, tf, _, _, _) in enemy_query.iter() {
            let dist = player_pos.distance(tf.translation.truncate());
            if dist < nearest_dist && dist <= 300.0 {
                nearest_dist = dist;
                aim_dir = (tf.translation.truncate() - player_pos).normalize_or_zero();
            }
        }
        if aim_dir == Vec2::ZERO {
            aim_dir = Vec2::X;
        }

        inventory.remove_items(ItemType::Arrow, 1);
        commands.spawn((
            Projectile {
                velocity: aim_dir * 400.0,
                damage: 8.0,
                lifetime: 2.0,
            },
            Sprite {
                color: Color::srgb(0.8, 0.7, 0.3),
                custom_size: Some(Vec2::new(4.0, 2.0)),
                ..default()
            },
            Transform::from_xyz(player_pos.x, player_pos.y, 8.0),
        ));

        // Set/reset cooldown
        if let Ok(mut cd) = cooldown_query.get_single_mut() {
            cd.timer.reset();
        } else {
            commands.entity(player_entity).insert(PlayerAttackCooldown {
                timer: Timer::from_seconds(0.5, TimerMode::Once),
            });
        }
        return;
    }

    // Calculate weapon damage from equipped item
    let base_damage = inventory.selected_item()
        .and_then(|slot| slot.item.weapon_damage())
        .unwrap_or(5.0); // Fist damage
    let strength_mult = maybe_buff
        .filter(|b| b.buff_type == BuffType::Strength)
        .map(|b| b.magnitude)
        .unwrap_or(1.0);
    let damage = base_damage * strength_mult;

    // Find nearest enemy within 40px
    let mut nearest: Option<(Entity, f32)> = None;
    for (entity, tf, _, _, _) in enemy_query.iter() {
        let dist = player_pos.distance(tf.translation.truncate());
        if dist <= 40.0 {
            if nearest.is_none() || dist < nearest.unwrap().1 {
                nearest = Some((entity, dist));
            }
        }
    }

    let Some((target_entity, _)) = nearest else { return };

    // Deal damage
    let mut killed = false;
    if let Ok((_, enemy_tf, mut enemy, mut sprite, maybe_boss)) = enemy_query.get_mut(target_entity) {
        enemy.health -= damage;

        // Flash white on hit (short for snappy feel)
        let original_color = sprite.color;
        sprite.color = Color::WHITE;
        commands.entity(target_entity).insert(HitFlash {
            timer: Timer::from_seconds(0.08, TimerMode::Once),
            original_color,
        });

        // Screen shake + hit-stop: scale by damage tier and boss status
        let is_boss = maybe_boss.is_some();
        let ratio = (damage / enemy.max_health).clamp(0.0, 1.0);
        if ratio >= 0.6 {
            screen_shake.timer = 0.16;
            screen_shake.intensity = if is_boss { 7.5 } else { 4.5 };
            hit_stop.timer = hit_stop.timer.max(0.07);
        } else if ratio >= 0.3 {
            screen_shake.timer = 0.12;
            screen_shake.intensity = if is_boss { 6.0 } else { 3.5 };
            hit_stop.timer = hit_stop.timer.max(0.045);
        } else {
            screen_shake.timer = 0.08;
            screen_shake.intensity = if is_boss { 4.0 } else { 2.0 };
        }

        // Knockback: push enemy away from player
        let knockback_dir = (enemy_tf.translation.truncate() - player_pos).normalize_or_zero();
        commands.entity(target_entity).insert(Knockback {
            direction: knockback_dir,
            timer: 0.1,
        });

        // Spawn hit particles (more for impact)
        let pos = enemy_tf.translation.truncate();
        particle_events.send(SpawnParticlesEvent {
            position: pos,
            color: Color::srgb(0.9, 0.15, 0.15),
            count: 6,
        });

        // Sound: hit
        sound_events.send(SoundEvent::Hit);

        // Slash arc visual at midpoint (slightly larger, clear opacity)
        let mid = (player_pos + enemy_tf.translation.truncate()) * 0.5;
        let angle = (enemy_tf.translation.truncate() - player_pos).to_angle();
        commands.spawn((
            SlashArc { timer: 0.14, max_timer: 0.14 },
            Sprite {
                custom_size: Some(Vec2::new(28.0, 28.0)),
                color: Color::srgba(1.0, 1.0, 0.95, 0.88),
                ..default()
            },
            Transform::from_xyz(mid.x, mid.y, 20.0)
                .with_rotation(Quat::from_rotation_z(angle)),
        ));

        // Fire PlayerHitEvent for enchanting/combo/weapon specials
        let weapon = inventory.selected_item().map(|s| s.item).unwrap_or(ItemType::Wood);
        hit_queue.pending.push(PlayerHitEvent {
            target: target_entity,
            weapon,
            damage,
            player_pos,
            enemy_pos: enemy_tf.translation.truncate(),
        });

        // US-028: Floating damage number at enemy position (tiered visuals)
        let (text, color) = damage_text_style(damage, enemy.max_health);
        floating_text_events.send(FloatingTextRequest {
            text: text.clone(),
            position: enemy_tf.translation.truncate(),
            color,
        });

        if enemy.health <= 0.0 {
            killed = true;
            // Impact burst on kill + brief freeze for emphasis
            let pos = enemy_tf.translation.truncate();
            particle_events.send(SpawnParticlesEvent {
                position: pos,
                color: Color::srgb(0.95, 0.3, 0.2),
                count: 8,
            });
            hit_stop.timer = hit_stop.timer.max(if maybe_boss.is_some() { 0.12 } else { 0.06 });
        }
    }

    if killed {
        // Get enemy type and position before despawning
        let (enemy_type, enemy_pos) = enemy_query.get(target_entity)
            .map(|(_, tf, e, _, _)| (e.enemy_type, tf.translation.truncate()))
            .unwrap_or((EnemyType::ShadowCrawler, player_pos));
        let (drop_item, drop_count) = loot_for_enemy(enemy_type);
        inventory.add_item(drop_item, drop_count);

        // US-036: Cave spiders have a 30% chance to drop a bonus item
        if enemy_type == EnemyType::CaveSpider {
            let mut rng = rand::thread_rng();
            if let Some((bonus_item, bonus_count)) = cave_spider_random_drop(&mut rng) {
                spawn_dropped_item(&mut commands, enemy_pos, bonus_item, bonus_count, &mut rng);
            }
        }

        // Award research points for a kill (+5 RP)
        rp_events.send(ResearchPointEvent { amount: 5 });

        // Track kill in death stats
        death_stats.total_kills += 1;

        // Sound: death
        sound_events.send(SoundEvent::Death);

        // Note: boss loot is handled by boss_death_loot; despawn happens there
        // for bosses. For regular enemies we despawn here.
        commands.entity(target_entity).despawn();
    }

    // Set/reset cooldown
    if cooldown_query.get_single_mut().is_ok() {
        if let Ok(mut cd) = cooldown_query.get_single_mut() {
            cd.timer.reset();
        }
    } else {
        commands.entity(player_entity).insert(PlayerAttackCooldown {
            timer: Timer::from_seconds(0.5, TimerMode::Once),
        });
    }
}

fn enemy_attack_player(
    time: Res<Time>,
    armor: Res<ArmorSlots>,
    mut enemy_query: Query<(&mut Enemy, &Transform, Option<&mut Boss>), Without<Player>>,
    mut player_query: Query<(&Transform, &mut Health, &mut Sprite, Option<&Dodging>, Option<&Blocking>, Option<&crate::death::RespawnInvulnerability>), With<Player>>,
    mut commands: Commands,
    player_entity_query: Query<Entity, With<Player>>,
    mut floating_text_events: EventWriter<FloatingTextRequest>,
    mut screen_shake: ResMut<ScreenShake>,
    mut status_events: EventWriter<ApplyStatusEvent>,
    mut death_stats: ResMut<DeathStats>,
) {
    let Ok((player_tf, mut health, mut sprite, maybe_dodging, maybe_blocking, maybe_invuln)) = player_query.get_single_mut() else { return };
    let player_pos = player_tf.translation.truncate();
    let total_armor = armor.total_armor();

    // 2B: Dodge invulnerability — skip all damage
    if maybe_dodging.is_some() {
        // Still need to tick enemy cooldowns
        for (mut enemy, _, _) in enemy_query.iter_mut() {
            enemy.attack_cooldown.tick(time.delta());
        }
        return;
    }

    // Wave 7C: Respawn invulnerability — skip all damage
    if maybe_invuln.is_some() {
        for (mut enemy, _, _) in enemy_query.iter_mut() {
            enemy.attack_cooldown.tick(time.delta());
        }
        return;
    }

    let mut took_damage = false;

    for (mut enemy, tf, mut maybe_boss) in enemy_query.iter_mut() {
        if enemy.state != EnemyState::Chase && enemy.state != EnemyState::Attack {
            continue;
        }

        enemy.attack_cooldown.tick(time.delta());

        let dist = player_pos.distance(tf.translation.truncate());
        if dist <= 24.0 && enemy.attack_cooldown.finished() {
            let damage_mult = maybe_boss.as_ref().map(|b| if b.phase_2 { 1.2 } else { 1.0 }).unwrap_or(1.0);
            let mut final_damage = ((enemy.damage * damage_mult) - total_armor as f32).max(1.0);

            // 2C: Shield blocking
            if let Some(block) = maybe_blocking {
                let block_duration = time.elapsed_secs() - block.start_time;
                if block_duration < 0.2 {
                    // Perfect block: 100% negation + knockback enemy
                    final_damage = 0.0;
                    let _kb_dir = (tf.translation.truncate() - player_pos).normalize_or_zero();
                    commands.entity(player_entity_query.get_single().unwrap_or(Entity::PLACEHOLDER)).remove::<Blocking>();
                    floating_text_events.send(FloatingTextRequest {
                        text: "PERFECT BLOCK!".to_string(),
                        position: player_pos + Vec2::new(0.0, 20.0),
                        color: Color::srgb(0.3, 0.8, 1.0),
                    });
                    // Stun enemy briefly
                    enemy.alert_timer = 1.0;
                    enemy.state = EnemyState::Alert;
                } else {
                    // Normal block: 80% damage reduction
                    final_damage *= 0.2;
                }
            }

            health.take_damage(final_damage);

            // Wave 7C: Track damage source for death recap
            if health.is_dead() {
                death_stats.last_damage_source = enemy.enemy_type.display_name().to_string();
            }

            if let Some(ref mut boss) = maybe_boss {
                if !boss.phase_2 && health.current <= health.max * 0.5 {
                    boss.phase_2 = true;
                }
            }
            enemy.attack_cooldown.reset();
            took_damage = true;

            // Apply enemy status effects on hit (Poison, Burn, Freeze, etc.)
            if let Some((effect, duration, chance)) = enemy_on_hit_effect(enemy.enemy_type) {
                if rand::thread_rng().gen::<f32>() < chance {
                    if let Ok(pe) = player_entity_query.get_single() {
                        status_events.send(ApplyStatusEvent {
                            target: pe,
                            effect,
                            duration,
                        });
                    }
                }
            }

            // US-028: Floating damage number at player position (tiered by HP)
            let (text, color) = damage_text_style(final_damage, health.max);
            floating_text_events.send(FloatingTextRequest {
                text: text.clone(),
                position: player_pos,
                color,
            });
        }
    }

    // Player hit reaction: screen shake + red flash
    if took_damage {
        screen_shake.timer = 0.12;
        screen_shake.intensity = 2.5;
        let original_color = sprite.color;
        sprite.color = Color::srgb(1.0, 0.2, 0.2);
        if let Ok(entity) = player_entity_query.get_single() {
            commands.entity(entity).insert(HitFlash {
                timer: Timer::from_seconds(0.08, TimerMode::Once),
                original_color,
            });
        }
    }
}

fn update_hit_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut HitFlash, &mut Sprite)>,
) {
    for (entity, mut flash, mut sprite) in query.iter_mut() {
        flash.timer.tick(time.delta());
        if flash.timer.finished() {
            sprite.color = flash.original_color;
            commands.entity(entity).remove::<HitFlash>();
        }
    }
}

/// When an enemy that has a `Boss` component reaches 0 health, add all
/// entries from its loot table to the player inventory then despawn it.
fn boss_death_loot(
    mut commands: Commands,
    boss_query: Query<(Entity, &Enemy, &Boss)>,
    mut inventory: ResMut<Inventory>,
    mut rp_events: EventWriter<ResearchPointEvent>,
    mut death_stats: ResMut<DeathStats>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    for (entity, enemy, boss) in boss_query.iter() {
        if enemy.health <= 0.0 {
            // Grant all loot
            for (item, count) in &boss.loot_table {
                inventory.add_item(*item, *count);
            }
            // Boss kill grants 20 research points
            rp_events.send(ResearchPointEvent { amount: 20 });
            // Track kill in death stats
            death_stats.total_kills += 1;
            // Sound: boss death
            sound_events.send(SoundEvent::Death);
            commands.entity(entity).despawn();
        }
    }
}

fn projectile_movement(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Projectile, &mut Transform)>,
    hit_stop: Res<HitStop>,
) {
    if hit_stop.timer > 0.0 {
        return;
    }
    for (entity, mut proj, mut tf) in query.iter_mut() {
        tf.translation.x += proj.velocity.x * time.delta_secs();
        tf.translation.y += proj.velocity.y * time.delta_secs();
        proj.lifetime -= time.delta_secs();
        if proj.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn projectile_hit(
    mut commands: Commands,
    proj_query: Query<(Entity, &Transform, &Projectile)>,
    mut enemy_query: Query<(Entity, &Transform, &mut Enemy, &mut Sprite, Option<&Boss>), Without<Invulnerable>>,
    mut inventory: ResMut<Inventory>,
    mut rp_events: EventWriter<ResearchPointEvent>,
    mut death_stats: ResMut<DeathStats>,
    mut screen_shake: ResMut<ScreenShake>,
    mut hit_stop: ResMut<HitStop>,
    mut particle_events: EventWriter<SpawnParticlesEvent>,
    mut sound_events: EventWriter<SoundEvent>,
    mut floating_text_events: EventWriter<FloatingTextRequest>,
) {
    for (proj_entity, proj_tf, proj) in proj_query.iter() {
        let proj_pos = proj_tf.translation.truncate();
        for (enemy_entity, enemy_tf, mut enemy, mut sprite, maybe_boss) in enemy_query.iter_mut() {
            let dist = proj_pos.distance(enemy_tf.translation.truncate());
            if dist <= 15.0 {
                enemy.health -= proj.damage;

                let pos = enemy_tf.translation.truncate();
                particle_events.send(SpawnParticlesEvent {
                    position: pos,
                    color: Color::srgb(0.9, 0.15, 0.15),
                    count: 6,
                });

                // Flash
                let original_color = sprite.color;
                sprite.color = Color::WHITE;
                commands.entity(enemy_entity).insert(HitFlash {
                    timer: Timer::from_seconds(0.08, TimerMode::Once),
                    original_color,
                });

                // Screen shake + hit-stop on projectile hit
                let is_boss = maybe_boss.is_some();
                let ratio = (proj.damage / enemy.max_health).clamp(0.0, 1.0);
                if ratio >= 0.6 {
                    screen_shake.timer = 0.16;
                    screen_shake.intensity = if is_boss { 7.0 } else { 4.0 };
                    hit_stop.timer = hit_stop.timer.max(0.07);
                } else if ratio >= 0.3 {
                    screen_shake.timer = 0.13;
                    screen_shake.intensity = if is_boss { 5.5 } else { 3.0 };
                    hit_stop.timer = hit_stop.timer.max(0.04);
                } else {
                    screen_shake.timer = 0.10;
                    screen_shake.intensity = if is_boss { 3.5 } else { 1.8 };
                }

                // Knockback from projectile direction
                let knockback_dir = proj.velocity.normalize_or_zero();
                commands.entity(enemy_entity).insert(Knockback {
                    direction: knockback_dir,
                    timer: 0.1,
                });

                // Sound: ranged hit
                sound_events.send(SoundEvent::Hit);

                // US-028: Floating damage number at enemy position (tiered)
                let (text, color) = damage_text_style(proj.damage, enemy.max_health);
                floating_text_events.send(FloatingTextRequest {
                    text: text.clone(),
                    position: enemy_tf.translation.truncate(),
                    color,
                });

                commands.entity(proj_entity).despawn();
                if enemy.health <= 0.0 {
                    particle_events.send(SpawnParticlesEvent {
                        position: pos,
                        color: Color::srgb(0.95, 0.3, 0.2),
                        count: 8,
                    });
                    hit_stop.timer = hit_stop.timer.max(if maybe_boss.is_some() { 0.12 } else { 0.06 });
                    let (drop_item, drop_count) = loot_for_enemy(enemy.enemy_type);
                    inventory.add_item(drop_item, drop_count);
                    // US-036: Cave spiders have a 30% chance to drop a bonus item
                    if enemy.enemy_type == EnemyType::CaveSpider {
                        let mut rng = rand::thread_rng();
                        let enemy_pos = enemy_tf.translation.truncate();
                        if let Some((bonus_item, bonus_count)) = cave_spider_random_drop(&mut rng) {
                            spawn_dropped_item(&mut commands, enemy_pos, bonus_item, bonus_count, &mut rng);
                        }
                    }
                    rp_events.send(ResearchPointEvent { amount: 5 });
                    // Track kill in death stats
                    death_stats.total_kills += 1;
                    // Sound: death
                    sound_events.send(SoundEvent::Death);
                    commands.entity(enemy_entity).despawn();
                }
                break;
            }
        }
    }
}

fn knockback_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Knockback, &mut Transform)>,
    hit_stop: Res<HitStop>,
) {
    if hit_stop.timer > 0.0 {
        return;
    }
    for (entity, mut kb, mut tf) in query.iter_mut() {
        let move_amount = kb.direction * 80.0 * time.delta_secs();
        tf.translation.x += move_amount.x;
        tf.translation.y += move_amount.y;
        kb.timer -= time.delta_secs();
        if kb.timer <= 0.0 {
            commands.entity(entity).remove::<Knockback>();
        }
    }
}

fn update_enemy_health_bars(
    mut commands: Commands,
    enemy_query: Query<(Entity, &Transform, &Enemy), Without<EnemyHealthBar>>,
    mut bar_query: Query<(Entity, &EnemyHealthBar, &mut Transform, &mut Sprite)>,
) {
    // Update existing bars: reposition to follow their parent enemy
    let mut seen_parents = HashSet::new();
    let mut bars_to_despawn = Vec::new();

    for (bar_entity, bar, mut bar_tf, mut bar_sprite) in bar_query.iter_mut() {
        // Find the parent enemy
        let mut found = false;
        for (enemy_entity, enemy_tf, enemy) in enemy_query.iter() {
            if enemy_entity == bar.parent_enemy {
                found = true;
                seen_parents.insert(enemy_entity);
                if enemy.health >= enemy.max_health || enemy.health <= 0.0 {
                    bars_to_despawn.push(bar_entity);
                } else {
                    let ratio = (enemy.health / enemy.max_health).clamp(0.0, 1.0);
                    if bar.is_fill {
                        let fill_width = 16.0 * ratio;
                        let fill_offset = (16.0 - fill_width) / 2.0;
                        bar_sprite.custom_size = Some(Vec2::new(fill_width, 2.0));
                        bar_tf.translation.x = enemy_tf.translation.x - fill_offset;
                        bar_tf.translation.y = enemy_tf.translation.y + 12.0;
                    } else {
                        bar_tf.translation.x = enemy_tf.translation.x;
                        bar_tf.translation.y = enemy_tf.translation.y + 12.0;
                    }
                }
                break;
            }
        }
        if !found {
            bars_to_despawn.push(bar_entity);
        }
    }

    for entity in bars_to_despawn {
        commands.entity(entity).despawn();
    }

    // Spawn bars for newly damaged enemies that don't have bars yet
    for (enemy_entity, enemy_tf, enemy) in enemy_query.iter() {
        if enemy.health >= enemy.max_health || enemy.health <= 0.0 || seen_parents.contains(&enemy_entity) {
            continue;
        }

        let ratio = (enemy.health / enemy.max_health).clamp(0.0, 1.0);
        let bar_y = enemy_tf.translation.y + 12.0;

        // Background (red)
        commands.spawn((
            EnemyHealthBar { parent_enemy: enemy_entity, is_fill: false },
            Sprite {
                color: Color::srgb(0.6, 0.1, 0.1),
                custom_size: Some(Vec2::new(16.0, 2.0)),
                ..default()
            },
            Transform::from_xyz(enemy_tf.translation.x, bar_y, 9.0),
        ));

        // Fill (green)
        let fill_width = 16.0 * ratio;
        let fill_offset = (16.0 - fill_width) / 2.0;
        commands.spawn((
            EnemyHealthBar { parent_enemy: enemy_entity, is_fill: true },
            Sprite {
                color: Color::srgb(0.1, 0.7, 0.1),
                custom_size: Some(Vec2::new(fill_width, 2.0)),
                ..default()
            },
            Transform::from_xyz(enemy_tf.translation.x - fill_offset, bar_y, 9.1),
        ));
    }
}

fn update_slash_arcs(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut SlashArc, &mut Sprite, &mut Transform)>,
) {
    for (entity, mut arc, mut sprite, mut tf) in query.iter_mut() {
        arc.timer -= time.delta_secs();
        if arc.timer <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        // Expand, rotate slightly, and fade
        let t = 1.0 - (arc.timer / arc.max_timer);
        let scale = 1.0 + t * 0.5;
        tf.scale = Vec3::splat(scale);
        tf.rotate_z(0.5 * time.delta_secs());
        let alpha = (1.0 - t).clamp(0.0, 1.0);
        let c = sprite.color.to_srgba();
        let boosted = c.red + 0.1 * t;
        sprite.color = Color::srgba(boosted.clamp(0.0, 1.0), c.green, c.blue, alpha * 0.9);
    }
}

// ---------------------------------------------------------------------------
// Wave 2: Combat expansion systems
// ---------------------------------------------------------------------------

/// Drains the PlayerHitQueue resource into PlayerHitEvent events.
fn drain_player_hit_queue(
    mut queue: ResMut<PlayerHitQueue>,
    mut events: EventWriter<PlayerHitEvent>,
) {
    for hit in queue.pending.drain(..) {
        events.send(hit);
    }
}

/// 2A: Applies enchanted weapon effects (burn, freeze, poison, lifesteal) on player melee hits.
fn apply_weapon_effects(
    mut events: EventReader<PlayerHitEvent>,
    mut status_events: EventWriter<ApplyStatusEvent>,
    mut player_health: Query<&mut Health, With<Player>>,
) {
    for ev in events.read() {
        // On-hit status effect
        if let Some((effect, duration)) = crate::enchanting::weapon_on_hit_effect(ev.weapon) {
            // FrostBlade has 30% chance
            if ev.weapon == ItemType::FrostBlade {
                if rand::thread_rng().gen::<f32>() < 0.3 {
                    status_events.send(ApplyStatusEvent {
                        target: ev.target,
                        effect,
                        duration,
                    });
                }
            } else {
                status_events.send(ApplyStatusEvent {
                    target: ev.target,
                    effect,
                    duration,
                });
            }
        }

        // Lifesteal
        let fraction = crate::enchanting::weapon_lifesteal_fraction(ev.weapon);
        if fraction > 0.0 {
            let heal_amount = ev.damage * fraction;
            if let Ok(mut health) = player_health.get_single_mut() {
                health.heal(heal_amount);
            }
        }
    }
}

/// 2B: Dodge roll input — Space bar triggers dodge.
fn dodge_roll_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut dodge_cd: ResMut<DodgeCooldown>,
    player_query: Query<(Entity, Option<&Dodging>), With<Player>>,
) {
    let Ok((player_entity, maybe_dodging)) = player_query.get_single() else { return };
    if maybe_dodging.is_some() { return; }
    if dodge_cd.timer > 0.0 { return; }
    if !keyboard.just_pressed(KeyCode::Space) { return; }

    // Determine dodge direction from WASD
    let mut dir = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) { dir.y += 1.0; }
    if keyboard.pressed(KeyCode::KeyS) { dir.y -= 1.0; }
    if keyboard.pressed(KeyCode::KeyA) { dir.x -= 1.0; }
    if keyboard.pressed(KeyCode::KeyD) { dir.x += 1.0; }
    if dir == Vec2::ZERO { dir = Vec2::X; } // Default right
    dir = dir.normalize_or_zero();

    commands.entity(player_entity).insert(Dodging {
        timer: 0.3,
        direction: dir,
    });
    dodge_cd.timer = 1.5;
}

/// 2B: Dodge roll tick — move fast, then remove Dodging component.
fn dodge_roll_tick(
    mut commands: Commands,
    time: Res<Time>,
    mut dodge_cd: ResMut<DodgeCooldown>,
    mut player_query: Query<(Entity, &mut Dodging, &mut Transform, &mut Sprite), With<Player>>,
) {
    let dt = time.delta_secs();
    dodge_cd.timer = (dodge_cd.timer - dt).max(0.0);

    for (entity, mut dodging, mut tf, mut sprite) in player_query.iter_mut() {
        dodging.timer -= dt;

        // Move at 3x speed in dodge direction
        let speed = 150.0 * 3.0;
        tf.translation.x += dodging.direction.x * speed * dt;
        tf.translation.y += dodging.direction.y * speed * dt;

        // Squish effect
        let t = dodging.timer / 0.3;
        tf.scale = Vec3::new(1.0 + 0.3 * (1.0 - t), 1.0 - 0.2 * (1.0 - t), 1.0);

        // Brief afterimage tint
        let c = sprite.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, 0.5 + 0.5 * t);

        if dodging.timer <= 0.0 {
            tf.scale = Vec3::ONE;
            sprite.color = Color::srgba(c.red, c.green, c.blue, 1.0);
            commands.entity(entity).remove::<Dodging>();
        }
    }
}

/// 2C: Shield block input — hold RMB with shield equipped.
fn shield_block_input(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    inventory: Res<Inventory>,
    armor: Res<ArmorSlots>,
    building_state: Res<crate::building::BuildingState>,
    player_query: Query<(Entity, Option<&Blocking>), With<Player>>,
) {
    let Ok((player_entity, maybe_blocking)) = player_query.get_single() else { return };

    if building_state.active { return; }

    let has_shield = armor.shield.is_some();
    // Don't block if holding a fishing rod or pet item
    let holding_non_combat = inventory.selected_item().map(|s| {
        matches!(s.item,
            ItemType::FishingRod | ItemType::SteelFishingRod |
            ItemType::PetCollar | ItemType::PetFood
        )
    }).unwrap_or(false);

    if has_shield && !holding_non_combat && mouse.pressed(MouseButton::Right) {
        if maybe_blocking.is_none() {
            commands.entity(player_entity).insert(Blocking {
                start_time: time.elapsed_secs(),
            });
        }
    } else if maybe_blocking.is_some() {
        commands.entity(player_entity).remove::<Blocking>();
    }
}

/// 2D: Combo tracking — sequential melee hits within 1.5s build combo multiplier.
fn combo_tracker(
    time: Res<Time>,
    mut combo: ResMut<ComboState>,
    mut events: EventReader<PlayerHitEvent>,
    mut floating_text_events: EventWriter<FloatingTextRequest>,
) {
    let dt = time.delta_secs();

    // Decay combo timer
    if combo.count > 0 {
        combo.timer -= dt;
        if combo.timer <= 0.0 {
            combo.count = 0;
        }
    }

    for ev in events.read() {
        combo.count += 1;
        combo.timer = 1.5;

        if combo.count >= 3 {
            floating_text_events.send(FloatingTextRequest {
                text: format!("x{} COMBO!", combo.count),
                position: ev.enemy_pos + Vec2::new(0.0, 15.0),
                color: Color::srgb(1.0, 0.85, 0.2),
            });
            // Reset after 3-hit combo
            combo.count = 0;
            combo.timer = 0.0;
        } else if combo.count == 2 {
            floating_text_events.send(FloatingTextRequest {
                text: "x2!".to_string(),
                position: ev.enemy_pos + Vec2::new(0.0, 15.0),
                color: Color::srgb(0.9, 0.75, 0.3),
            });
        }
    }
}

/// 2E: Weapon specials — track hit counters, trigger special effects.
fn weapon_specials(
    mut events: EventReader<PlayerHitEvent>,
    mut specials: ResMut<WeaponSpecialState>,
    mut enemy_query: Query<(&mut Enemy, &Transform)>,
    mut status_events: EventWriter<ApplyStatusEvent>,
    mut player_health: Query<(&mut Health, &mut Hunger), (With<Player>, Without<Enemy>)>,
    mut floating_text_events: EventWriter<FloatingTextRequest>,
    mut particle_events: EventWriter<SpawnParticlesEvent>,
) {
    for ev in events.read() {
        match ev.weapon {
            ItemType::FlameBlade => {
                specials.flame_hits += 1;
                if specials.flame_hits >= 5 {
                    specials.flame_hits = 0;
                    // Fire burst AoE: damage all enemies within 40px
                    let burst_dmg = ev.damage * 0.5;
                    for (mut enemy, etf) in enemy_query.iter_mut() {
                        if etf.translation.truncate().distance(ev.enemy_pos) < 40.0 {
                            enemy.health -= burst_dmg;
                        }
                    }
                    particle_events.send(SpawnParticlesEvent {
                        position: ev.enemy_pos,
                        color: Color::srgb(1.0, 0.4, 0.1),
                        count: 12,
                    });
                    floating_text_events.send(FloatingTextRequest {
                        text: "FIRE BURST!".to_string(),
                        position: ev.enemy_pos + Vec2::new(0.0, 20.0),
                        color: Color::srgb(1.0, 0.5, 0.1),
                    });
                }
            }
            ItemType::FrostBlade => {
                specials.frost_hits += 1;
                if specials.frost_hits >= 3 {
                    specials.frost_hits = 0;
                    // Shatter-kill enemies below 15% HP
                    if let Ok((mut enemy, _)) = enemy_query.get_mut(ev.target) {
                        if enemy.health > 0.0 && enemy.health / enemy.max_health < 0.15 {
                            enemy.health = 0.0;
                            floating_text_events.send(FloatingTextRequest {
                                text: "SHATTER!".to_string(),
                                position: ev.enemy_pos + Vec2::new(0.0, 20.0),
                                color: Color::srgb(0.5, 0.8, 1.0),
                            });
                        }
                    }
                }
            }
            ItemType::VenomBlade => {
                if specials.venom_last_target == Some(ev.target) {
                    specials.venom_consecutive += 1;
                } else {
                    specials.venom_consecutive = 1;
                    specials.venom_last_target = Some(ev.target);
                }
                // Damage ramps +10% per consecutive hit
                let bonus = ev.damage * 0.1 * specials.venom_consecutive as f32;
                if bonus > 0.0 {
                    if let Ok((mut enemy, _)) = enemy_query.get_mut(ev.target) {
                        enemy.health -= bonus;
                    }
                }
            }
            ItemType::LifestealBlade => {
                // Check if kill — restore 15 hunger + 10 HP
                if let Ok((enemy, _)) = enemy_query.get(ev.target) {
                    if enemy.health <= 0.0 {
                        if let Ok((mut health, mut hunger)) = player_health.get_single_mut() {
                            health.heal(10.0);
                            hunger.current = (hunger.current + 15.0).min(hunger.max);
                            floating_text_events.send(FloatingTextRequest {
                                text: "LIFE DRAIN!".to_string(),
                                position: ev.player_pos + Vec2::new(0.0, 15.0),
                                color: Color::srgb(0.3, 1.0, 0.3),
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Wave 5A: Enemy Ranged Attacks
// ---------------------------------------------------------------------------

/// Ticks ability cooldowns and fires ranged attacks for specific enemy types.
fn enemy_ranged_attacks(
    mut commands: Commands,
    mut enemy_query: Query<(&mut Enemy, &Transform), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
) {
    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    for (mut enemy, tf) in enemy_query.iter_mut() {
        // Only fire when in Chase state
        if enemy.state != EnemyState::Chase {
            continue;
        }

        let enemy_pos = tf.translation.truncate();
        let dist = enemy_pos.distance(player_pos);

        // Tick ability cooldown (already ticked in enemy_ai too, but if timer was
        // just set to a ranged-specific value we honour it here).
        // We only act when cooldown <= 0 for the relevant type.

        match enemy.enemy_type {
            // IceWraith: frost bolt every 4s, range 40-160
            EnemyType::IceWraith => {
                if enemy.ability_cooldown_timer <= 0.0 && dist >= 40.0 && dist <= 160.0 {
                    let dir = (player_pos - enemy_pos).normalize_or_zero();
                    commands.spawn((
                        Projectile {
                            velocity: dir * 220.0,
                            damage: 7.0,
                            lifetime: 1.5,
                        },
                        EnemyProjectile,
                        Sprite {
                            color: Color::srgba(0.6, 0.85, 1.0, 0.9),
                            custom_size: Some(Vec2::new(5.0, 5.0)),
                            ..default()
                        },
                        Transform::from_xyz(enemy_pos.x, enemy_pos.y, 8.0),
                    ));
                    enemy.ability_cooldown_timer = 4.0;
                }
            }
            // FrostLich: ice spike AoE at player position every 6s, range 50-180
            EnemyType::FrostLich => {
                if enemy.ability_cooldown_timer <= 0.0 && dist >= 50.0 && dist <= 180.0 {
                    // Spawn telegraph marker at player position
                    commands.spawn((
                        IceSpikeAoE {
                            delay: 1.0,
                            damage: 12.0,
                            radius: 20.0,
                            position: player_pos,
                        },
                        Sprite {
                            color: Color::srgba(0.5, 0.6, 0.9, 0.4),
                            custom_size: Some(Vec2::new(40.0, 40.0)),
                            ..default()
                        },
                        Transform::from_xyz(player_pos.x, player_pos.y, 7.0),
                    ));
                    enemy.ability_cooldown_timer = 6.0;
                }
            }
            // LavaElemental: magma projectile every 5s, range 50-130, arcing, leaves burn zone
            EnemyType::LavaElemental => {
                if enemy.ability_cooldown_timer <= 0.0 && dist >= 50.0 && dist <= 130.0 {
                    let dir = (player_pos - enemy_pos).normalize_or_zero();
                    commands.spawn((
                        Projectile {
                            velocity: dir * 200.0,
                            damage: 10.0,
                            lifetime: 1.3,
                        },
                        EnemyProjectile,
                        Sprite {
                            color: Color::srgb(0.95, 0.35, 0.1),
                            custom_size: Some(Vec2::new(7.0, 7.0)),
                            ..default()
                        },
                        Transform::from_xyz(enemy_pos.x, enemy_pos.y, 8.0),
                    ));
                    enemy.ability_cooldown_timer = 5.0;
                }
            }
            // NightBat: dive-bomb attack every 3s, range 30-140
            EnemyType::NightBat => {
                if enemy.ability_cooldown_timer <= 0.0 && dist >= 30.0 && dist <= 140.0 {
                    // Spawn a telegraph marker at the target position
                    commands.spawn((
                        DiveBomb {
                            telegraph_timer: 0.5,
                            damage: 10.0,
                            target_pos: player_pos,
                            recovery_timer: 1.5,
                        },
                        Sprite {
                            color: Color::srgba(0.3, 0.1, 0.4, 0.5),
                            custom_size: Some(Vec2::new(16.0, 16.0)),
                            ..default()
                        },
                        Transform::from_xyz(player_pos.x, player_pos.y, 7.0),
                    ));
                    enemy.ability_cooldown_timer = 3.0;
                }
            }
            // SandScorpion: venom spit (moved from inline enemy_ai)
            EnemyType::SandScorpion => {
                if enemy.ability_cooldown_timer <= 0.0 && dist >= 50.0 && dist <= 120.0 {
                    let dir = (player_pos - enemy_pos).normalize_or_zero();
                    commands.spawn((
                        Projectile {
                            velocity: dir * 280.0,
                            damage: 5.0,
                            lifetime: 1.2,
                        },
                        EnemyProjectile,
                        Sprite {
                            color: Color::srgb(0.7, 0.55, 0.2),
                            custom_size: Some(Vec2::new(6.0, 6.0)),
                            ..default()
                        },
                        Transform::from_xyz(enemy_pos.x, enemy_pos.y, 8.0),
                    ));
                    enemy.ability_cooldown_timer = 2.5;
                }
            }
            _ => {}
        }
    }
}

/// Checks enemy projectiles (marked with EnemyProjectile) hitting the player.
fn enemy_projectile_hit_player(
    mut commands: Commands,
    proj_query: Query<(Entity, &Transform, &Projectile), With<EnemyProjectile>>,
    mut player_query: Query<(&Transform, &mut Health, Option<&crate::death::RespawnInvulnerability>), With<Player>>,
    mut floating_text_events: EventWriter<FloatingTextRequest>,
    mut status_events: EventWriter<ApplyStatusEvent>,
    mut screen_shake: ResMut<ScreenShake>,
    player_entity_query: Query<Entity, With<Player>>,
    mut death_stats: ResMut<DeathStats>,
) {
    let Ok((player_tf, mut health, maybe_invuln)) = player_query.get_single_mut() else { return };
    // Wave 7C: Skip all projectile damage if player has respawn invulnerability
    if maybe_invuln.is_some() { return; }
    let Ok(player_entity) = player_entity_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    for (proj_entity, proj_tf, proj) in proj_query.iter() {
        let proj_pos = proj_tf.translation.truncate();
        let dist = proj_pos.distance(player_pos);
        if dist <= 12.0 {
            health.current = (health.current - proj.damage).max(0.0);

            // Wave 7C: Track projectile damage source for death recap
            if health.is_dead() {
                death_stats.last_damage_source = "Enemy Projectile".to_string();
            }

            floating_text_events.send(FloatingTextRequest {
                text: format!("-{:.0}", proj.damage),
                position: player_pos + Vec2::new(0.0, 12.0),
                color: Color::srgb(1.0, 0.3, 0.3),
            });

            screen_shake.timer = 0.1;
            screen_shake.intensity = 2.0;

            // IceWraith frost bolt applies Freeze (detect by projectile color heuristic:
            // frost bolts are bluish). We check damage == 7 as an identifier.
            if (proj.damage - 7.0).abs() < 0.1 {
                status_events.send(ApplyStatusEvent {
                    target: player_entity,
                    effect: crate::status_effects::StatusEffectType::Freeze,
                    duration: 2.0,
                });
            }

            commands.entity(proj_entity).despawn();
        }
    }
}

/// Tick IceSpikeAoE markers: after delay expires, deal damage in radius then despawn.
fn update_ice_spike_aoe(
    mut commands: Commands,
    time: Res<Time>,
    mut aoe_query: Query<(Entity, &mut IceSpikeAoE, &mut Sprite)>,
    mut player_query: Query<(&Transform, &mut Health), With<Player>>,
    mut floating_text_events: EventWriter<FloatingTextRequest>,
    mut screen_shake: ResMut<ScreenShake>,
) {
    let dt = time.delta_secs();
    let Ok((player_tf, mut health)) = player_query.get_single_mut() else { return };
    let player_pos = player_tf.translation.truncate();

    for (entity, mut aoe, mut sprite) in aoe_query.iter_mut() {
        aoe.delay -= dt;

        // Fade in the telegraph
        let alpha = (1.0 - aoe.delay.max(0.0)).clamp(0.3, 0.9);
        sprite.color = Color::srgba(0.5, 0.6, 0.9, alpha);

        if aoe.delay <= 0.0 {
            // Check if player is within radius
            let dist = player_pos.distance(aoe.position);
            if dist <= aoe.radius {
                health.current = (health.current - aoe.damage).max(0.0);
                floating_text_events.send(FloatingTextRequest {
                    text: format!("-{:.0} ICE!", aoe.damage),
                    position: player_pos + Vec2::new(0.0, 12.0),
                    color: Color::srgb(0.5, 0.7, 1.0),
                });
                screen_shake.timer = 0.12;
                screen_shake.intensity = 3.0;
            }
            commands.entity(entity).despawn();
        }
    }
}

/// Burn zones left by LavaElemental magma projectiles. Spawned when a magma
/// EnemyProjectile despawns (lifetime expires) near where it was.
/// For simplicity, magma projectiles spawn a burn zone on hit via the
/// enemy_projectile_hit_player system, or when their lifetime expires.
/// This system just ticks existing burn zones and damages the player.
fn update_burn_zones(
    mut commands: Commands,
    time: Res<Time>,
    mut zone_query: Query<(Entity, &mut BurnZone, &Transform)>,
    mut player_query: Query<(&Transform, &mut Health), (With<Player>, Without<BurnZone>)>,
    mut floating_text_events: EventWriter<FloatingTextRequest>,
) {
    let dt = time.delta_secs();
    let Ok((player_tf, mut health)) = player_query.get_single_mut() else { return };
    let player_pos = player_tf.translation.truncate();

    for (entity, mut zone, zone_tf) in zone_query.iter_mut() {
        zone.lifetime -= dt;
        if zone.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        let dist = player_pos.distance(zone_tf.translation.truncate());
        if dist <= zone.radius {
            let damage = zone.damage_per_sec * dt;
            health.current = (health.current - damage).max(0.0);
            // Throttle floating text to ~once per second
            if (zone.lifetime * 4.0).fract() < dt * 4.0 {
                floating_text_events.send(FloatingTextRequest {
                    text: format!("-{:.0} BURN", damage.ceil()),
                    position: player_pos + Vec2::new(0.0, 12.0),
                    color: Color::srgb(1.0, 0.4, 0.1),
                });
            }
        }
    }
}

/// Tick NightBat dive-bomb attacks: telegraph, then damage, then despawn.
fn update_dive_bombs(
    mut commands: Commands,
    time: Res<Time>,
    mut bomb_query: Query<(Entity, &mut DiveBomb, &mut Sprite)>,
    mut player_query: Query<(&Transform, &mut Health), With<Player>>,
    mut floating_text_events: EventWriter<FloatingTextRequest>,
    mut screen_shake: ResMut<ScreenShake>,
) {
    let dt = time.delta_secs();
    let Ok((player_tf, mut health)) = player_query.get_single_mut() else { return };
    let player_pos = player_tf.translation.truncate();

    for (entity, mut bomb, mut sprite) in bomb_query.iter_mut() {
        if bomb.telegraph_timer > 0.0 {
            bomb.telegraph_timer -= dt;
            // Pulse the telegraph
            let alpha = 0.3 + 0.4 * (bomb.telegraph_timer * 8.0).sin().abs();
            sprite.color = Color::srgba(0.3, 0.1, 0.4, alpha);
        } else if bomb.recovery_timer > 0.0 {
            // Strike happened at the transition point
            if bomb.recovery_timer > 1.4 {
                // First frame after telegraph: apply damage
                let dist = player_pos.distance(bomb.target_pos);
                if dist <= 16.0 {
                    health.current = (health.current - bomb.damage).max(0.0);
                    floating_text_events.send(FloatingTextRequest {
                        text: format!("-{:.0} DIVE!", bomb.damage),
                        position: player_pos + Vec2::new(0.0, 12.0),
                        color: Color::srgb(0.6, 0.2, 0.8),
                    });
                    screen_shake.timer = 0.15;
                    screen_shake.intensity = 4.0;
                }
                // Flash white briefly
                sprite.color = Color::srgba(1.0, 1.0, 1.0, 0.8);
            }
            bomb.recovery_timer -= dt;
            if bomb.recovery_timer <= 0.0 {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Spawns a burn zone when a magma (LavaElemental) enemy projectile expires.
/// This hooks into projectile_movement — we detect magma projectiles by their
/// damage value (10.0) and spawn a burn zone at their position before despawn.
/// NOTE: This is done by checking expiring EnemyProjectile in projectile_movement.
/// We add a separate small system that watches for near-expiring magma projectiles.
fn spawn_burn_zones_from_magma(
    mut commands: Commands,
    proj_query: Query<(Entity, &Transform, &Projectile), With<EnemyProjectile>>,
) {
    for (_entity, tf, proj) in proj_query.iter() {
        // Magma projectiles have damage == 10.0 and are about to expire
        if (proj.damage - 10.0).abs() < 0.1 && proj.lifetime <= 0.05 && proj.lifetime > 0.0 {
            let pos = tf.translation.truncate();
            commands.spawn((
                BurnZone {
                    damage_per_sec: 4.0,
                    radius: 14.0,
                    lifetime: 3.0,
                },
                Sprite {
                    color: Color::srgba(0.9, 0.3, 0.05, 0.5),
                    custom_size: Some(Vec2::new(28.0, 28.0)),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y, 4.0),
            ));
        }
    }
}
