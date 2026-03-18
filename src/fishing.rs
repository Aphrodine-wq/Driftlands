use bevy::prelude::*;
use rand::Rng;
use crate::hud::not_paused;
use crate::inventory::{Inventory, ItemType};
use crate::player::Player;
use crate::building::BuildingState;
use crate::world::chunk::Chunk;
use crate::world::tile::TileType;
use crate::world::{TILE_SIZE, CHUNK_WORLD_SIZE};
use crate::world::chunk::CHUNK_SIZE;
use crate::world::generation::Biome;

// ---------------------------------------------------------------------------
// Fish types
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FishType {
    Trout,
    Salmon,
    Catfish,
    Pufferfish,
    Eel,
    Crab,
}

const ALL_FISH: [FishType; 6] = [
    FishType::Trout,
    FishType::Salmon,
    FishType::Catfish,
    FishType::Pufferfish,
    FishType::Eel,
    FishType::Crab,
];

impl FishType {
    pub fn raw_item(self) -> ItemType {
        match self {
            FishType::Trout => ItemType::RawTrout,
            FishType::Salmon => ItemType::RawSalmon,
            FishType::Catfish => ItemType::RawCatfish,
            FishType::Pufferfish => ItemType::RawPufferfish,
            FishType::Eel => ItemType::RawEel,
            FishType::Crab => ItemType::RawCrab,
        }
    }

    pub fn catch_difficulty(self) -> f32 {
        match self {
            FishType::Trout => 0.3,
            FishType::Salmon => 0.4,
            FishType::Catfish => 0.5,
            FishType::Pufferfish => 0.7,
            FishType::Eel => 0.6,
            FishType::Crab => 0.5,
        }
    }

    pub fn rarity_weight(self) -> u32 {
        match self {
            FishType::Trout => 30,
            FishType::Salmon => 25,
            FishType::Catfish => 20,
            FishType::Pufferfish => 5,
            FishType::Eel => 10,
            FishType::Crab => 15,
        }
    }

    /// Biomes where this fish can be caught.
    pub fn biomes(self) -> &'static [Biome] {
        match self {
            FishType::Trout => &[
                Biome::Forest,
                Biome::Coastal,
                Biome::Swamp,
                Biome::Desert,
                Biome::Tundra,
                Biome::Volcanic,
                Biome::Fungal,
                Biome::CrystalCave,
                Biome::Mountain,
            ],
            FishType::Salmon => &[Biome::Forest, Biome::Tundra],
            FishType::Catfish => &[Biome::Coastal, Biome::Swamp],
            FishType::Pufferfish => &[Biome::Coastal],
            FishType::Eel => &[Biome::Swamp, Biome::Coastal],
            FishType::Crab => &[Biome::Coastal, Biome::Swamp],
        }
    }
}

// ---------------------------------------------------------------------------
// Fishing phases & state
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FishingPhase {
    Idle,
    Casting,
    Waiting,
    Hooked,
    Reeling,
    Caught,
}

#[derive(Resource)]
pub struct FishingState {
    pub phase: FishingPhase,
    pub cast_timer: f32,
    pub bite_timer: f32,
    pub hook_window: f32,
    pub reel_progress: f32,
    pub target_fish: Option<FishType>,
    /// 1 = FishingRod, 2 = SteelFishingRod
    pub rod_tier: u32,
}

impl Default for FishingState {
    fn default() -> Self {
        Self {
            phase: FishingPhase::Idle,
            cast_timer: 0.0,
            bite_timer: 0.0,
            hook_window: 0.0,
            reel_progress: 0.0,
            target_fish: None,
            rod_tier: 1,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a world position to the chunk coordinate and local tile indices.
fn world_to_chunk_tile(pos: Vec3) -> (IVec2, usize, usize) {
    let chunk_x = (pos.x / CHUNK_WORLD_SIZE).floor() as i32;
    let chunk_y = (pos.y / CHUNK_WORLD_SIZE).floor() as i32;
    let local_x = ((pos.x - chunk_x as f32 * CHUNK_WORLD_SIZE) / TILE_SIZE) as usize;
    let local_y = ((pos.y - chunk_y as f32 * CHUNK_WORLD_SIZE) / TILE_SIZE) as usize;
    (IVec2::new(chunk_x, chunk_y), local_x.min(CHUNK_SIZE - 1), local_y.min(CHUNK_SIZE - 1))
}

/// Check whether any of the four cardinal-adjacent tiles around a world
/// position is a `TileType::Water` or `TileType::DeepWater`.
fn is_near_water(pos: Vec3, chunks: &Query<&Chunk>) -> bool {
    let offsets: [(f32, f32); 4] = [
        (TILE_SIZE, 0.0),
        (-TILE_SIZE, 0.0),
        (0.0, TILE_SIZE),
        (0.0, -TILE_SIZE),
    ];

    for (dx, dy) in offsets {
        let sample = Vec3::new(pos.x + dx, pos.y + dy, pos.z);
        let (chunk_coord, tx, ty) = world_to_chunk_tile(sample);
        for chunk in chunks.iter() {
            if chunk.position == chunk_coord {
                let tile = chunk.get_tile(tx, ty);
                if tile == TileType::Water || tile == TileType::DeepWater {
                    return true;
                }
            }
        }
    }
    false
}

/// Look up the biome of the chunk the player is standing in.
fn biome_at_player(pos: Vec3, chunks: &Query<&Chunk>) -> Option<Biome> {
    let (chunk_coord, _, _) = world_to_chunk_tile(pos);
    for chunk in chunks.iter() {
        if chunk.position == chunk_coord {
            return Some(chunk.biome);
        }
    }
    None
}

/// Pick a random fish from the biome-weighted pool.
fn pick_fish(biome: Biome) -> Option<FishType> {
    let pool: Vec<FishType> = ALL_FISH
        .iter()
        .copied()
        .filter(|f| f.biomes().contains(&biome))
        .collect();

    if pool.is_empty() {
        return None;
    }

    let total_weight: u32 = pool.iter().map(|f| f.rarity_weight()).sum();
    let mut rng = rand::thread_rng();
    let mut roll = rng.gen_range(0..total_weight);

    for fish in &pool {
        let w = fish.rarity_weight();
        if roll < w {
            return Some(*fish);
        }
        roll -= w;
    }

    // Fallback (should not be reached)
    pool.last().copied()
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Right-click with a fishing rod while standing near water to begin casting.
fn start_fishing(
    mouse: Res<ButtonInput<MouseButton>>,
    inventory: Res<Inventory>,
    building_state: Res<BuildingState>,
    mut fishing: ResMut<FishingState>,
    player_q: Query<&Transform, With<Player>>,
    chunks: Query<&Chunk>,
) {
    if building_state.active {
        return;
    }
    if fishing.phase != FishingPhase::Idle {
        return;
    }
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    let rod_tier = match inventory.selected_item().map(|s| s.item) {
        Some(ItemType::FishingRod) => 1u32,
        Some(ItemType::SteelFishingRod) => 2u32,
        _ => return,
    };

    let Ok(player_tf) = player_q.get_single() else { return };

    if !is_near_water(player_tf.translation, &chunks) {
        return;
    }

    fishing.phase = FishingPhase::Casting;
    fishing.cast_timer = 0.5;
    fishing.rod_tier = rod_tier;
    fishing.target_fish = None;
    fishing.reel_progress = 0.0;
}

/// Count down the cast animation timer, then transition to Waiting.
fn fishing_cast_timer(
    time: Res<Time>,
    mut fishing: ResMut<FishingState>,
) {
    if fishing.phase != FishingPhase::Casting {
        return;
    }

    fishing.cast_timer -= time.delta_secs();

    if fishing.cast_timer <= 0.0 {
        let mut rng = rand::thread_rng();
        fishing.bite_timer = rng.gen_range(2.0..8.0);
        fishing.phase = FishingPhase::Waiting;
    }
}

/// Count down the bite timer, pick a fish, and transition to Hooked.
fn fishing_bite(
    time: Res<Time>,
    mut fishing: ResMut<FishingState>,
    player_q: Query<&Transform, With<Player>>,
    chunks: Query<&Chunk>,
) {
    if fishing.phase != FishingPhase::Waiting {
        return;
    }

    fishing.bite_timer -= time.delta_secs();

    if fishing.bite_timer <= 0.0 {
        let Ok(player_tf) = player_q.get_single() else {
            fishing.phase = FishingPhase::Idle;
            return;
        };

        let biome = biome_at_player(player_tf.translation, &chunks)
            .unwrap_or(Biome::Forest);

        fishing.target_fish = pick_fish(biome);

        if fishing.target_fish.is_some() {
            fishing.phase = FishingPhase::Hooked;
            fishing.hook_window = 2.0;
        } else {
            // No fish available in this biome — reset
            fishing.phase = FishingPhase::Idle;
        }
    }
}

/// Hooked window — press E to start reeling or the fish escapes.
fn fishing_hook_window(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut fishing: ResMut<FishingState>,
) {
    if fishing.phase != FishingPhase::Hooked {
        return;
    }

    fishing.hook_window -= time.delta_secs();

    if keyboard.just_pressed(KeyCode::KeyE) {
        fishing.reel_progress = 0.0;
        fishing.phase = FishingPhase::Reeling;
        return;
    }

    if fishing.hook_window <= 0.0 {
        // Fish got away
        fishing.phase = FishingPhase::Idle;
        fishing.target_fish = None;
    }
}

/// While reeling, hold left-click to fill the reel bar. Fish difficulty
/// passively drains progress. Reach 1.0 to catch.
fn fishing_reel(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut fishing: ResMut<FishingState>,
) {
    if fishing.phase != FishingPhase::Reeling {
        return;
    }

    let dt = time.delta_secs();
    let difficulty = fishing
        .target_fish
        .map_or(0.3, |f| f.catch_difficulty());

    // Fish pulls back
    fishing.reel_progress -= difficulty * 0.3 * dt;

    // Player reels in
    if mouse.pressed(MouseButton::Left) {
        let reel_rate = 0.5 + 0.2 * fishing.rod_tier as f32;
        fishing.reel_progress += reel_rate * dt;
    }

    fishing.reel_progress = fishing.reel_progress.clamp(0.0, 1.0);

    if fishing.reel_progress >= 1.0 {
        fishing.phase = FishingPhase::Caught;
    }
}

/// Award the caught fish to the player's inventory and consume rod durability.
fn fishing_catch(
    mut fishing: ResMut<FishingState>,
    mut inventory: ResMut<Inventory>,
) {
    if fishing.phase != FishingPhase::Caught {
        return;
    }

    if let Some(fish) = fishing.target_fish {
        inventory.add_item(fish.raw_item(), 1);
    }

    inventory.use_selected_tool();

    // Reset
    fishing.phase = FishingPhase::Idle;
    fishing.target_fish = None;
    fishing.reel_progress = 0.0;
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct FishingPlugin;

impl Plugin for FishingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FishingState::default())
            .add_systems(Update, (
                start_fishing,
                fishing_cast_timer,
                fishing_bite,
                fishing_hook_window,
                fishing_reel,
                fishing_catch,
            ).run_if(not_paused));
    }
}
