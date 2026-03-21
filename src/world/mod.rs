pub mod chunk;
pub mod generation;
pub mod tile;

use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::asset::AssetId;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::sprite::MeshMaterial2d;
use chunk::{Chunk, CHUNK_SIZE};
use generation::{WorldGenerator, Biome};
use tile::TileType;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;
use std::time::Instant;
use crate::dungeon::{DungeonRegistry, should_spawn_entrance};
use crate::hud::not_paused;
use crate::lit_materials::{LitChunkMaterial, LitQuadMesh, LitSpriteMaterial};
use crate::npc;
use crate::player::Player;
use crate::season::{Season, SeasonCycle};

pub const TILE_SIZE: f32 = 16.0;
pub const CHUNK_WORLD_SIZE: f32 = TILE_SIZE * CHUNK_SIZE as f32;
pub const RENDER_DISTANCE: i32 = 7;
/// Objects only spawn within this distance for performance (terrain renders further out)
pub const OBJECT_RENDER_DISTANCE: i32 = 4;

pub struct WorldPlugin;

/// Marker component for interaction hint text entities.
#[derive(Component)]
pub struct InteractionHint;

/// Chunk tile data loaded from save; used when spawning chunks so modified terrain persists.
#[derive(Resource, Default)]
pub struct LoadedChunkCache(pub HashMap<IVec2, Vec<Vec<TileType>>>);

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
struct SpriteMaterialKey {
    color: [u32; 4],
    color_texture: AssetId<Image>,
    normal_texture: AssetId<Image>,
}

impl SpriteMaterialKey {
    fn new(color: LinearRgba, color_texture: &Handle<Image>, normal_texture: &Handle<Image>) -> Self {
        Self {
            color: [
                color.red.to_bits(),
                color.green.to_bits(),
                color.blue.to_bits(),
                color.alpha.to_bits(),
            ],
            color_texture: color_texture.id(),
            normal_texture: normal_texture.id(),
        }
    }
}

/// Cache for `LitSpriteMaterial` so multiple world objects can share the same
/// GPU material instance instead of allocating one per spawn.
#[derive(Resource, Default)]
struct LitSpriteMaterialCache {
    by_key: HashMap<SpriteMaterialKey, Handle<LitSpriteMaterial>>,
}

/// Cache for per-chunk generated textures (both color + normal).
///
/// These `Image` assets are expensive to generate via `create_chunk_image` and
/// `create_chunk_normal_image`, so we keep the results and reuse the
/// `Handle<Image>`s when chunks are respawned (e.g. during texture readiness
/// refresh).
#[derive(Resource, Default)]
struct ChunkImageCache {
    for_season: Option<Season>,
    by_pos: HashMap<IVec2, (Handle<Image>, Handle<Image>)>,
}

/// Tracks whether tile PNG textures have finished async loading.
/// On first detection, forces a chunk respawn so textures appear.
#[derive(Resource, Default)]
struct TileTexturesReady(bool);

/// Marker component for the full-screen loading overlay.
#[derive(Component)]
struct LoadingOverlay;

/// Pending chunk generation results from the background worker (PRD 4.2 async chunk gen).
#[derive(Resource)]
pub struct ChunkGenAsync {
    pub request_tx: mpsc::Sender<(u32, IVec2)>,
    pub results: Arc<Mutex<VecDeque<(IVec2, Chunk)>>>,
    pub requested: HashSet<IVec2>,
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        let seed = rand::random::<u32>();
        let (request_tx, request_rx) = mpsc::channel();
        let results = Arc::new(Mutex::new(VecDeque::new()));
        let results_clone = Arc::clone(&results);
        thread::spawn(move || {
            while let Ok((s, pos)) = request_rx.recv() {
                let chunk = WorldGenerator::new(s).generate_chunk(pos);
                if let Ok(mut q) = results_clone.lock() {
                    q.push_back((pos, chunk));
                }
            }
        });
        app.insert_resource(WorldState::new(seed))
            .insert_resource(LitSpriteMaterialCache::default())
            .insert_resource(ChunkImageCache::default())
            .insert_resource(LoadedChunkCache::default())
            .insert_resource(TileTexturesReady::default())
            .insert_resource(ChunkGenAsync {
                request_tx,
                results,
                requested: HashSet::new(),
            })
            .add_systems(Startup, spawn_loading_overlay)
            .add_systems(PostStartup, spawn_initial_chunks)
            .add_systems(Update, refresh_chunks_on_texture_load)
            .add_systems(Update, manage_chunks)
            .add_systems(Update, show_interaction_hints.run_if(not_paused));
    }
}

#[derive(Resource)]
pub struct WorldState {
    pub generator: WorldGenerator,
    pub loaded_chunks: HashSet<IVec2>,
    pub seed: u32,
}

impl WorldState {
    pub fn new(seed: u32) -> Self {
        Self {
            generator: WorldGenerator::new(seed),
            loaded_chunks: HashSet::new(),
            seed,
        }
    }
}

#[derive(Component)]
pub struct WorldObject {
    pub object_type: WorldObjectType,
    pub health: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WorldObjectType {
    OakTree,
    PineTree,
    Rock,
    Bush,
    // Phase 3
    Cactus,
    IceCrystal,
    Mushroom,
    GiantMushroom,
    ReedClump,
    SulfurDeposit,
    CrystalNode,
    AlpineFlower,
    IronVein,
    CoalDeposit,
    // Phase 4
    AncientRuin,
    // Phase 5 — Ruins & campsites
    SupplyCrate,
    RuinWall,
    // US-035 — Biome-exclusive objects
    BerryBush,
    SandstoneRock,
    OasisPalm,
    FrozenOreDeposit,
    IceFormation,
    SulfurVent,
    ObsidianNode,
    GlowingSpore,
    BioLuminescentGel,
    CrystalCluster,
    EchoStone,
    Driftwood,
    ShellDeposit,
    SeaweedPatch,
    // POIs (PRD 4.5)
    AncientMachinery,
    Geyser,
    /// Forest meadow flowers (grass layer)
    Wildflower,
    /// Fallen log in forest (gather for wood)
    FallenLog,
}

impl WorldObjectType {
    pub fn max_health(&self) -> f32 {
        match self {
            WorldObjectType::OakTree => 100.0,
            WorldObjectType::PineTree => 80.0,
            WorldObjectType::Rock => 120.0,
            WorldObjectType::Bush => 30.0,
            WorldObjectType::Cactus => 60.0,
            WorldObjectType::IceCrystal => 80.0,
            WorldObjectType::Mushroom => 20.0,
            WorldObjectType::GiantMushroom => 90.0,
            WorldObjectType::ReedClump => 25.0,
            WorldObjectType::SulfurDeposit => 100.0,
            WorldObjectType::CrystalNode => 110.0,
            WorldObjectType::AlpineFlower => 15.0,
            WorldObjectType::IronVein => 150.0,
            WorldObjectType::CoalDeposit => 130.0,
            WorldObjectType::AncientRuin => 200.0,
            WorldObjectType::SupplyCrate => 40.0,
            WorldObjectType::RuinWall => 200.0,
            WorldObjectType::BerryBush => 25.0,
            WorldObjectType::SandstoneRock => 90.0,
            WorldObjectType::OasisPalm => 70.0,
            WorldObjectType::FrozenOreDeposit => 140.0,
            WorldObjectType::IceFormation => 100.0,
            WorldObjectType::SulfurVent => 80.0,
            WorldObjectType::ObsidianNode => 130.0,
            WorldObjectType::GlowingSpore => 15.0,
            WorldObjectType::BioLuminescentGel => 35.0,
            WorldObjectType::CrystalCluster => 120.0,
            WorldObjectType::EchoStone => 100.0,
            WorldObjectType::Driftwood => 40.0,
            WorldObjectType::ShellDeposit => 20.0,
            WorldObjectType::SeaweedPatch => 15.0,
            WorldObjectType::AncientMachinery => 180.0,
            WorldObjectType::Geyser => 80.0,
            WorldObjectType::Wildflower => 10.0,
            WorldObjectType::FallenLog => 40.0,
        }
    }

    pub fn min_tool_tier(&self) -> u32 {
        match self {
            WorldObjectType::IronVein | WorldObjectType::CrystalNode
            | WorldObjectType::ObsidianNode | WorldObjectType::FrozenOreDeposit => 2,
            WorldObjectType::AncientRuin | WorldObjectType::AncientMachinery => 3,
            _ => 0,
        }
    }

    pub fn default_color(&self) -> Color {
        match self {
            WorldObjectType::OakTree => Color::srgb(0.15, 0.45, 0.12),
            WorldObjectType::PineTree => Color::srgb(0.10, 0.35, 0.15),
            WorldObjectType::Rock => Color::srgb(0.45, 0.45, 0.48),
            WorldObjectType::Bush => Color::srgb(0.20, 0.50, 0.15),
            WorldObjectType::Cactus => Color::srgb(0.30, 0.55, 0.20),
            WorldObjectType::IceCrystal => Color::srgb(0.55, 0.70, 0.85),
            WorldObjectType::Mushroom => Color::srgb(0.60, 0.35, 0.25),
            WorldObjectType::GiantMushroom => Color::srgb(0.55, 0.25, 0.50),
            WorldObjectType::ReedClump => Color::srgb(0.40, 0.55, 0.25),
            WorldObjectType::SulfurDeposit => Color::srgb(0.75, 0.70, 0.20),
            WorldObjectType::CrystalNode => Color::srgb(0.50, 0.40, 0.70),
            WorldObjectType::AlpineFlower => Color::srgb(0.70, 0.50, 0.65),
            WorldObjectType::IronVein => Color::srgb(0.50, 0.40, 0.35),
            WorldObjectType::CoalDeposit => Color::srgb(0.20, 0.20, 0.22),
            WorldObjectType::AncientRuin => Color::srgb(0.45, 0.40, 0.50),
            WorldObjectType::SupplyCrate => Color::srgb(0.55, 0.40, 0.20),
            WorldObjectType::RuinWall => Color::srgb(0.40, 0.38, 0.35),
            WorldObjectType::BerryBush => Color::srgb(0.25, 0.45, 0.20),
            WorldObjectType::SandstoneRock => Color::srgb(0.65, 0.55, 0.40),
            WorldObjectType::OasisPalm => Color::srgb(0.30, 0.50, 0.15),
            WorldObjectType::FrozenOreDeposit => Color::srgb(0.50, 0.55, 0.65),
            WorldObjectType::IceFormation => Color::srgb(0.60, 0.70, 0.80),
            WorldObjectType::SulfurVent => Color::srgb(0.80, 0.70, 0.15),
            WorldObjectType::ObsidianNode => Color::srgb(0.15, 0.10, 0.20),
            WorldObjectType::GlowingSpore => Color::srgb(0.40, 0.70, 0.50),
            WorldObjectType::BioLuminescentGel => Color::srgb(0.30, 0.60, 0.70),
            WorldObjectType::CrystalCluster => Color::srgb(0.55, 0.45, 0.70),
            WorldObjectType::EchoStone => Color::srgb(0.50, 0.50, 0.60),
            WorldObjectType::Driftwood => Color::srgb(0.45, 0.35, 0.25),
            WorldObjectType::ShellDeposit => Color::srgb(0.70, 0.65, 0.55),
            WorldObjectType::SeaweedPatch => Color::srgb(0.20, 0.45, 0.30),
            WorldObjectType::AncientMachinery => Color::srgb(0.35, 0.38, 0.42),
            WorldObjectType::Geyser => Color::srgb(0.75, 0.80, 0.85),
            WorldObjectType::Wildflower => Color::srgb(0.95, 0.75, 0.85),
            WorldObjectType::FallenLog => Color::srgb(0.45, 0.32, 0.22),
        }
    }

    pub fn size(&self) -> Vec2 {
        match self {
            WorldObjectType::OakTree => Vec2::new(18.0, 28.0),
            WorldObjectType::PineTree => Vec2::new(14.0, 30.0),
            WorldObjectType::Rock => Vec2::new(13.0, 11.0),
            WorldObjectType::Bush => Vec2::new(13.0, 11.0),
            WorldObjectType::Cactus => Vec2::new(10.0, 22.0),
            WorldObjectType::IceCrystal => Vec2::new(12.0, 16.0),
            WorldObjectType::Mushroom => Vec2::new(10.0, 10.0),
            WorldObjectType::GiantMushroom => Vec2::new(18.0, 28.0),
            WorldObjectType::ReedClump => Vec2::new(12.0, 16.0),
            WorldObjectType::SulfurDeposit => Vec2::new(13.0, 10.0),
            WorldObjectType::CrystalNode => Vec2::new(14.0, 16.0),
            WorldObjectType::AlpineFlower => Vec2::new(8.0, 8.0),
            WorldObjectType::IronVein => Vec2::new(14.0, 11.0),
            WorldObjectType::CoalDeposit => Vec2::new(13.0, 11.0),
            WorldObjectType::AncientRuin => Vec2::new(18.0, 18.0),
            WorldObjectType::SupplyCrate => Vec2::new(12.0, 10.0),
            WorldObjectType::RuinWall => Vec2::new(18.0, 22.0),
            WorldObjectType::BerryBush => Vec2::new(12.0, 11.0),
            WorldObjectType::SandstoneRock => Vec2::new(14.0, 12.0),
            WorldObjectType::OasisPalm => Vec2::new(14.0, 26.0),
            WorldObjectType::FrozenOreDeposit => Vec2::new(13.0, 11.0),
            WorldObjectType::IceFormation => Vec2::new(14.0, 18.0),
            WorldObjectType::SulfurVent => Vec2::new(12.0, 14.0),
            WorldObjectType::ObsidianNode => Vec2::new(13.0, 11.0),
            WorldObjectType::GlowingSpore => Vec2::new(8.0, 10.0),
            WorldObjectType::BioLuminescentGel => Vec2::new(10.0, 8.0),
            WorldObjectType::CrystalCluster => Vec2::new(16.0, 18.0),
            WorldObjectType::EchoStone => Vec2::new(12.0, 14.0),
            WorldObjectType::Driftwood => Vec2::new(16.0, 8.0),
            WorldObjectType::ShellDeposit => Vec2::new(10.0, 8.0),
            WorldObjectType::SeaweedPatch => Vec2::new(12.0, 10.0),
            WorldObjectType::AncientMachinery => Vec2::new(16.0, 14.0),
            WorldObjectType::Geyser => Vec2::new(14.0, 16.0),
            WorldObjectType::Wildflower => Vec2::new(8.0, 8.0),
            WorldObjectType::FallenLog => Vec2::new(16.0, 10.0),
        }
    }
}

#[derive(Component)]
pub struct ChunkObject {
    pub chunk_pos: IVec2,
}

fn spawn_initial_chunks(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut chunk_materials: ResMut<Assets<LitChunkMaterial>>,
    mut sprite_materials: ResMut<Assets<LitSpriteMaterial>>,
    mut sprite_material_cache: ResMut<LitSpriteMaterialCache>,
    mut chunk_image_cache: ResMut<ChunkImageCache>,
    assets: Res<crate::assets::GameAssets>,
    quad_mesh: Res<LitQuadMesh>,
    mut world_state: ResMut<WorldState>,
    mut dungeon_registry: ResMut<DungeonRegistry>,
    mut loaded_chunk_cache: ResMut<LoadedChunkCache>,
    season_cycle: Res<SeasonCycle>,
) {
    let player_chunk = IVec2::ZERO; // Player starts at origin
    for cy in -RENDER_DISTANCE..=RENDER_DISTANCE {
        for cx in -RENDER_DISTANCE..=RENDER_DISTANCE {
            let chunk_pos = IVec2::new(cx, cy);
            spawn_chunk(
                &mut commands,
                &mut images,
                &mut chunk_materials,
                &mut sprite_materials,
                &mut sprite_material_cache,
                &mut chunk_image_cache,
                &assets,
                &quad_mesh,
                &mut world_state,
                &mut dungeon_registry,
                &mut loaded_chunk_cache,
                &season_cycle,
                chunk_pos,
                player_chunk,
            );
        }
    }
}

/// Spawns chunk entities from pre-computed chunk data (used by sync path and async result drain).
/// `player_chunk` is used for distance-based object culling: objects only spawn within
/// `OBJECT_RENDER_DISTANCE` to keep performance tight at the wider `RENDER_DISTANCE`.
fn spawn_chunk_from_data(
    commands: &mut Commands,
    images: &mut ResMut<Assets<Image>>,
    chunk_materials: &mut ResMut<Assets<LitChunkMaterial>>,
    sprite_materials: &mut ResMut<Assets<LitSpriteMaterial>>,
    sprite_material_cache: &mut LitSpriteMaterialCache,
    chunk_image_cache: &mut ChunkImageCache,
    assets: &Res<crate::assets::GameAssets>,
    quad_mesh: &Res<LitQuadMesh>,
    dungeon_registry: &mut ResMut<DungeonRegistry>,
    chunk_pos: IVec2,
    chunk: Chunk,
    seed: u32,
    generator: &WorldGenerator,
    season_cycle: &SeasonCycle,
    player_chunk: IVec2,
) {
    if chunk_image_cache.for_season != Some(season_cycle.current) {
        chunk_image_cache.for_season = Some(season_cycle.current);
        chunk_image_cache.by_pos.clear();
    }

    let (image_handle, normal_handle) = if let Some((color_handle, normal_handle)) =
        chunk_image_cache.by_pos.get(&chunk_pos)
    {
        (color_handle.clone(), normal_handle.clone())
    } else {
        let normal_image = create_chunk_normal_image(&chunk);
        let image = create_chunk_image(&chunk, generator, season_cycle.current, assets, images);
        let image_handle = images.add(image);
        let normal_handle = images.add(normal_image);
        chunk_image_cache
            .by_pos
            .insert(chunk_pos, (image_handle.clone(), normal_handle.clone()));
        (image_handle, normal_handle)
    };
    let chunk_material_handle = chunk_materials.add(LitChunkMaterial {
        lighting: crate::lighting::LightingUniform::default(),
        time: 0.0,
        color_texture: image_handle,
        normal_texture: normal_handle,
    });

    let world_x = chunk_pos.x as f32 * CHUNK_WORLD_SIZE + CHUNK_WORLD_SIZE / 2.0;
    let world_y = chunk_pos.y as f32 * CHUNK_WORLD_SIZE + CHUNK_WORLD_SIZE / 2.0;

    // Only spawn world objects for chunks within OBJECT_RENDER_DISTANCE for performance
    let dx = (chunk_pos.x - player_chunk.x).abs();
    let dy = (chunk_pos.y - player_chunk.y).abs();
    if dx <= OBJECT_RENDER_DISTANCE && dy <= OBJECT_RENDER_DISTANCE {
        spawn_chunk_objects(
            commands,
            assets,
            chunk_pos,
            &chunk,
            seed,
            generator,
            season_cycle,
            dungeon_registry,
            sprite_materials,
            sprite_material_cache,
            quad_mesh,
        );
    }

    commands.spawn((
        chunk,
        Mesh2d(quad_mesh.quad.clone()),
        MeshMaterial2d(chunk_material_handle),
        Transform::from_xyz(world_x, world_y, 0.0)
            .with_scale(Vec3::new(CHUNK_WORLD_SIZE, CHUNK_WORLD_SIZE, 1.0)),
    ));
}

fn spawn_chunk(
    commands: &mut Commands,
    images: &mut ResMut<Assets<Image>>,
    chunk_materials: &mut ResMut<Assets<LitChunkMaterial>>,
    sprite_materials: &mut ResMut<Assets<LitSpriteMaterial>>,
    sprite_material_cache: &mut LitSpriteMaterialCache,
    chunk_image_cache: &mut ChunkImageCache,
    assets: &Res<crate::assets::GameAssets>,
    quad_mesh: &Res<LitQuadMesh>,
    world_state: &mut ResMut<WorldState>,
    dungeon_registry: &mut ResMut<DungeonRegistry>,
    loaded_chunk_cache: &mut ResMut<LoadedChunkCache>,
    season_cycle: &SeasonCycle,
    chunk_pos: IVec2,
    player_chunk: IVec2,
) {
    if world_state.loaded_chunks.contains(&chunk_pos) {
        return;
    }

    let chunk = if let Some(tiles) = loaded_chunk_cache.0.remove(&chunk_pos) {
        let center_x = (chunk_pos.x * CHUNK_SIZE as i32 + CHUNK_SIZE as i32 / 2) as f64;
        let center_y = (chunk_pos.y * CHUNK_SIZE as i32 + CHUNK_SIZE as i32 / 2) as f64;
        let biome = world_state.generator.biome_at(center_x, center_y);
        Chunk::from_tiles(chunk_pos, &tiles, biome)
    } else {
        world_state.generator.generate_chunk(chunk_pos)
    };
    spawn_chunk_from_data(
        commands,
        images,
        chunk_materials,
        sprite_materials,
        sprite_material_cache,
        chunk_image_cache,
        assets,
        quad_mesh,
        dungeon_registry,
        chunk_pos,
        chunk,
        world_state.seed,
        &world_state.generator,
        &season_cycle,
        player_chunk,
    );
    world_state.loaded_chunks.insert(chunk_pos);
}

/// Returns a tree color variant based on a hash value (3 green shades).
fn tree_color_variant(hash: u32) -> Color {
    match hash % 3 {
        0 => Color::srgb(0.18, 0.42, 0.12), // darker
        1 => Color::srgb(0.22, 0.52, 0.16), // normal
        _ => Color::srgb(0.28, 0.62, 0.20), // lighter
    }
}

/// Tree color with seasonal tint applied.
fn tree_color_seasonal(variant_hash: u32, season: Season) -> Color {
    let base = tree_color_variant(variant_hash).to_srgba();
    let tint = season.tree_color().to_srgba();
    Color::srgba(
        base.red * tint.red,
        base.green * tint.green,
        base.blue * tint.blue,
        1.0,
    )
}

/// Returns a rock size variant based on a hash value (2 sizes).
fn rock_size_variant(hash: u32) -> Vec2 {
    match hash % 2 {
        0 => Vec2::new(10.0, 9.0),  // small
        _ => Vec2::new(13.0, 11.0), // normal
    }
}

/// Pebble size for ground decoration.
fn pebble_size() -> Vec2 {
    Vec2::new(6.0, 5.0)
}

/// True if this tile has at least one neighboring tile that is water.
fn has_water_neighbor(chunk: &Chunk, x: usize, y: usize) -> bool {
    let is_water = |tx: i32, ty: i32| -> bool {
        if tx < 0 || ty < 0 || tx >= CHUNK_SIZE as i32 || ty >= CHUNK_SIZE as i32 {
            return false;
        }
        let t = chunk.get_tile(tx as usize, ty as usize);
        matches!(t, TileType::Water | TileType::DeepWater)
    };
    let x = x as i32;
    let y = y as i32;
    is_water(x - 1, y) || is_water(x + 1, y) || is_water(x, y - 1) || is_water(x, y + 1)
}

fn spawn_chunk_objects(
    commands: &mut Commands,
    assets: &Res<crate::assets::GameAssets>,
    chunk_pos: IVec2,
    chunk: &Chunk,
    seed: u32,
    generator: &WorldGenerator,
    season_cycle: &SeasonCycle,
    dungeon_registry: &mut ResMut<DungeonRegistry>,
    materials: &mut ResMut<Assets<LitSpriteMaterial>>,
    sprite_material_cache: &mut LitSpriteMaterialCache,
    quad_mesh: &Res<LitQuadMesh>,
) {
    let biome = chunk.biome;

    // Track whether we have already placed one entrance in this chunk so we
    // don't cluster multiple portals.
    let mut entrance_placed_this_chunk = false;
    // Template ruins: one per chunk in eligible biomes; reserve tiles so nothing else spawns on them.
    let mut ruin_placed_this_chunk = false;
    let mut ruin_tiles: HashSet<(i32, i32)> = HashSet::new();
    let mut camp_placed_this_chunk = false;
    let mut poi_tiles: HashSet<(i32, i32)> = HashSet::new();

    for y in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let tile = chunk.get_tile(x, y);
            if !tile.is_walkable() {
                continue;
            }

            let world_tile_x = chunk_pos.x * CHUNK_SIZE as i32 + x as i32;
            let world_tile_y = chunk_pos.y * CHUNK_SIZE as i32 + y as i32;

            if ruin_tiles.contains(&(world_tile_x, world_tile_y)) || poi_tiles.contains(&(world_tile_x, world_tile_y)) {
                continue;
            }

            let wx = world_tile_x as f32 * TILE_SIZE + TILE_SIZE / 2.0;
            let wy = world_tile_y as f32 * TILE_SIZE + TILE_SIZE / 2.0;

            let hash = WorldGenerator::position_hash(world_tile_x, world_tile_y, seed);
            let hash2 = WorldGenerator::position_hash(world_tile_x, world_tile_y, seed.wrapping_add(50));
            // Variant hash for color/size variety (uses a different offset)
            let variant_hash = WorldGenerator::position_hash(world_tile_x, world_tile_y, seed.wrapping_add(123));
            let density_roll = hash % 100;
            let variant_roll = hash2 % 100;

            // --- Template ruin (5-tile cross: 4 RuinWall + 1 AncientRuin at center) ---
            if !ruin_placed_this_chunk
                && matches!(biome, Biome::Forest | Biome::Mountain | Biome::Desert)
                && density_roll == 97
            {
                for (dx, dy) in [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)] {
                    ruin_tiles.insert((world_tile_x + dx, world_tile_y + dy));
                }
                spawn_ruin_template(commands, assets, materials, sprite_material_cache, quad_mesh, wx, wy, chunk_pos);
                ruin_placed_this_chunk = true;
                continue;
            }

            // --- Abandoned camp (3-tile cluster: 2 SupplyCrate + 1 Rock) ---
            if !camp_placed_this_chunk && biome == Biome::Forest && density_roll == 96 {
                for (dx, dy) in [(0, 0), (1, 0), (0, 1)] {
                    poi_tiles.insert((world_tile_x + dx, world_tile_y + dy));
                }
                spawn_abandoned_camp(commands, assets, materials, sprite_material_cache, quad_mesh, wx, wy, chunk_pos);
                camp_placed_this_chunk = true;
                continue;
            }

            // --- Ancient machinery (Mountain/Desert) ---
            if matches!(biome, Biome::Mountain | Biome::Desert) && density_roll == 98 {
                spawn_world_object(
                    commands,
                    assets,
                    materials,
                    sprite_material_cache,
                    quad_mesh,
                    WorldObjectType::AncientMachinery,
                    wx,
                    wy,
                    chunk_pos,
                );
                continue;
            }

            // --- Geyser (Mountain natural formation) ---
            if biome == Biome::Mountain && density_roll == 94 {
                spawn_world_object(
                    commands,
                    assets,
                    materials,
                    sprite_material_cache,
                    quad_mesh,
                    WorldObjectType::Geyser,
                    wx,
                    wy,
                    chunk_pos,
                );
                continue;
            }

            // --- Dungeon entrance for all biomes ---
            if !entrance_placed_this_chunk
                && should_spawn_entrance(world_tile_x, world_tile_y, seed)
            {
                crate::dungeon::spawn_entrance_with_biome(commands, dungeon_registry, wx, wy, chunk_pos, biome);
                entrance_placed_this_chunk = true;
                // Skip placing a world object on the same tile.
                continue;
            }

            // --- Hermit NPC spawning (~0.2% chance in eligible biomes) ---
            if matches!(biome, Biome::Forest | Biome::Swamp | Biome::Mountain | Biome::Desert | Biome::Tundra) {
                let hermit_hash = WorldGenerator::position_hash(world_tile_x, world_tile_y, seed.wrapping_add(7777));
                if hermit_hash % 500 == 0 {
                    npc::spawn_hermit(commands, assets, wx, wy, chunk_pos);
                    continue;
                }
            }

            match biome {
                Biome::Forest => {
                    // Grove-based tree clustering: dense woods vs meadows (8–24% by area)
                    let grove = generator.grove_density(world_tile_x as f64, world_tile_y as f64);
                    let tree_threshold = (8.0 + grove * 16.0) as u32;
                    if density_roll < tree_threshold {
                        let obj = if variant_roll < 50 { WorldObjectType::OakTree } else { WorldObjectType::PineTree };
                        let color = tree_color_seasonal(variant_hash, season_cycle.current);
                        spawn_world_object_with_overrides(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            obj,
                            wx,
                            wy,
                            chunk_pos,
                            Some(color),
                            None,
                        );
                    } else if density_roll < 18 {
                        // Bush layer (dense undergrowth)
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::Bush,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 23 {
                        // Berry bushes (biome-exclusive, more plentiful)
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::BerryBush,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 27 {
                        // Mushrooms on forest floor
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::Mushroom,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 30 {
                        let size = rock_size_variant(variant_hash);
                        spawn_world_object_with_overrides(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::Rock,
                            wx,
                            wy,
                            chunk_pos,
                            None,
                            Some(size),
                        );
                    } else if has_water_neighbor(chunk, x, y) && density_roll < 8 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::ReedClump,
                            wx,
                            wy,
                            chunk_pos,
                        );
                        continue;
                    } else if density_roll >= 30 && density_roll < 38
                        && matches!(tile, TileType::Grass | TileType::DarkGrass)
                    {
                        // Wildflowers in meadows (grass tiles only)
                        let flower_color = match variant_hash % 4 {
                            0 => Color::srgb(0.95, 0.75, 0.85),
                            1 => Color::srgb(0.9, 0.9, 0.95),
                            2 => Color::srgb(0.85, 0.95, 0.75),
                            _ => Color::srgb(0.95, 0.85, 0.6),
                        };
                        spawn_world_object_with_overrides(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::Wildflower,
                            wx,
                            wy,
                            chunk_pos,
                            Some(flower_color),
                            None,
                        );
                    } else if density_roll >= 94 && density_roll < 96 {
                        let log_color = WorldObjectType::FallenLog.default_color();
                        spawn_world_object_with_overrides(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::FallenLog,
                            wx,
                            wy,
                            chunk_pos,
                            Some(log_color),
                            None,
                        );
                    } else if density_roll >= 88 && density_roll < 93 {
                        spawn_world_object_with_overrides(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::Rock,
                            wx,
                            wy,
                            chunk_pos,
                            None,
                            Some(pebble_size()),
                        );
                    } else if density_roll == 99 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::SupplyCrate,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    }
                }
                Biome::Coastal => {
                    // Reeds at water edges
                    if has_water_neighbor(chunk, x, y) && density_roll < 10 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::ReedClump,
                            wx,
                            wy,
                            chunk_pos,
                        );
                        continue;
                    }
                    // Driftwood, shell deposits, and seaweed — beachcomber paradise
                    if density_roll < 6 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::Driftwood,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 12 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::ShellDeposit,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 18 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::SeaweedPatch,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 22 {
                        let size = rock_size_variant(variant_hash);
                        spawn_world_object_with_overrides(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::Rock,
                            wx,
                            wy,
                            chunk_pos,
                            None,
                            Some(size),
                        );
                    } else if density_roll < 24 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::Bush,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    }
                }
                Biome::Swamp => {
                    if density_roll < 10 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::ReedClump,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 16 {
                        let color = Color::srgb(0.7, 0.2, 0.3);
                        spawn_world_object_with_overrides(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::Bush,
                            wx,
                            wy,
                            chunk_pos,
                            Some(color),
                            None,
                        );
                    } else if density_roll < 20 {
                        let color = tree_color_seasonal(variant_hash, season_cycle.current);
                        spawn_world_object_with_overrides(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::OakTree,
                            wx,
                            wy,
                            chunk_pos,
                            Some(color),
                            None,
                        );
                    } else if density_roll < 24 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::Mushroom,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 27 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::GlowingSpore,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    }
                }
                Biome::Desert => {
                    // Cacti, sandstone rocks, and occasional oasis palms
                    if density_roll < 6 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::Cactus,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 16 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::SandstoneRock,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 19 {
                        let size = rock_size_variant(variant_hash);
                        spawn_world_object_with_overrides(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::Rock,
                            wx,
                            wy,
                            chunk_pos,
                            None,
                            Some(size),
                        );
                    } else if density_roll >= 96 && density_roll < 98 {
                        // Occasional oasis palms
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::OasisPalm,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll == 99 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::RuinWall,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    }
                }
                Biome::Tundra => {
                    // Hardy pines, ice formations, frozen ore deposits, snow-covered rocks
                    if density_roll < 4 {
                        // Sparse pine trees (seasonal tint)
                        let base = Color::srgb(0.15, 0.3, 0.2).to_srgba();
                        let tint = season_cycle.current.tree_color().to_srgba();
                        let color = Color::srgba(base.red * tint.red, base.green * tint.green, base.blue * tint.blue, 1.0);
                        spawn_world_object_with_overrides(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::PineTree,
                            wx,
                            wy,
                            chunk_pos,
                            Some(color),
                            None,
                        );
                    } else if density_roll < 10 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::IceFormation,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 14 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::IceCrystal,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 19 {
                        let size = rock_size_variant(variant_hash);
                        spawn_world_object_with_overrides(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::Rock,
                            wx,
                            wy,
                            chunk_pos,
                            None,
                            Some(size),
                        );
                    } else if density_roll < 22 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::FrozenOreDeposit,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    }
                }
                Biome::Volcanic => {
                    // Obsidian nodes, sulfur vents, dramatic hellscape
                    if density_roll < 2 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::AncientRuin,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 8 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::ObsidianNode,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 14 {
                        // Sulfur vents with bright yellow color
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::SulfurVent,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 19 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::SulfurDeposit,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 23 {
                        let size = rock_size_variant(variant_hash);
                        spawn_world_object_with_overrides(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::Rock,
                            wx,
                            wy,
                            chunk_pos,
                            None,
                            Some(size),
                        );
                    } else if density_roll < 27 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::CoalDeposit,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    }
                }
                Biome::Fungal => {
                    // Dense alien fungal forest — towering mushrooms, spores, luminescent gel
                    if density_roll < 8 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::GiantMushroom,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 16 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::Mushroom,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 22 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::GlowingSpore,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 27 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::BioLuminescentGel,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    }
                }
                Biome::CrystalCave => {
                    // Crystal clusters, gemstone nodes, echo stones — magical underground
                    if density_roll < 8 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::CrystalCluster,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 14 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::CrystalNode,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 19 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::EchoStone,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 23 {
                        let size = rock_size_variant(variant_hash);
                        spawn_world_object_with_overrides(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::Rock,
                            wx,
                            wy,
                            chunk_pos,
                            None,
                            Some(size),
                        );
                    } else if density_roll < 26 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::IronVein,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    }
                }
                Biome::Mountain => {
                    if density_roll < 2 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::AncientRuin,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 10 {
                        let size = rock_size_variant(variant_hash);
                        spawn_world_object_with_overrides(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::Rock,
                            wx,
                            wy,
                            chunk_pos,
                            None,
                            Some(size),
                        );
                    } else if density_roll < 15 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::AlpineFlower,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 20 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::IronVein,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll < 24 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::CoalDeposit,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    } else if density_roll == 99 {
                        spawn_world_object(
                            commands,
                            assets,
                            materials,
                            sprite_material_cache,
                            quad_mesh,
                            WorldObjectType::RuinWall,
                            wx,
                            wy,
                            chunk_pos,
                        );
                    }
                }
            }
        }
    }
}

/// Spawns an abandoned camp POI: 2 SupplyCrates + 1 Rock in a small cluster.
fn spawn_abandoned_camp(
    commands: &mut Commands,
    assets: &Res<crate::assets::GameAssets>,
    materials: &mut ResMut<Assets<LitSpriteMaterial>>,
    sprite_material_cache: &mut LitSpriteMaterialCache,
    quad_mesh: &Res<LitQuadMesh>,
    anchor_x: f32,
    anchor_y: f32,
    chunk_pos: IVec2,
) {
    spawn_world_object(
        commands,
        assets,
        materials,
        sprite_material_cache,
        quad_mesh,
        WorldObjectType::SupplyCrate,
        anchor_x,
        anchor_y,
        chunk_pos,
    );
    spawn_world_object(
        commands,
        assets,
        materials,
        sprite_material_cache,
        quad_mesh,
        WorldObjectType::SupplyCrate,
        anchor_x + TILE_SIZE,
        anchor_y,
        chunk_pos,
    );
    spawn_world_object(
        commands,
        assets,
        materials,
        sprite_material_cache,
        quad_mesh,
        WorldObjectType::Rock,
        anchor_x,
        anchor_y + TILE_SIZE,
        chunk_pos,
    );
}

/// Spawns a 5-tile ruin template: 4 RuinWalls in a cross and 1 AncientRuin at center.
/// Used for procedural ruins (PRD 4.5) with journal/blueprint drops from the center object.
fn spawn_ruin_template(
    commands: &mut Commands,
    assets: &Res<crate::assets::GameAssets>,
    materials: &mut ResMut<Assets<LitSpriteMaterial>>,
    sprite_material_cache: &mut LitSpriteMaterialCache,
    quad_mesh: &Res<LitQuadMesh>,
    center_x: f32,
    center_y: f32,
    chunk_pos: IVec2,
) {
    let offsets = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    for (dx, dy) in offsets {
        let wx = center_x + dx as f32 * TILE_SIZE;
        let wy = center_y + dy as f32 * TILE_SIZE;
        spawn_world_object(
            commands,
            assets,
            materials,
            sprite_material_cache,
            quad_mesh,
            WorldObjectType::RuinWall,
            wx,
            wy,
            chunk_pos,
        );
    }
    spawn_world_object(
        commands,
        assets,
        materials,
        sprite_material_cache,
        quad_mesh,
        WorldObjectType::AncientRuin,
        center_x,
        center_y,
        chunk_pos,
    );
}

fn spawn_world_object(
    commands: &mut Commands,
    assets: &Res<crate::assets::GameAssets>,
    materials: &mut ResMut<Assets<LitSpriteMaterial>>,
    sprite_material_cache: &mut LitSpriteMaterialCache,
    quad_mesh: &Res<LitQuadMesh>,
    obj_type: WorldObjectType,
    x: f32,
    y: f32,
    chunk_pos: IVec2,
) {
    spawn_world_object_with_overrides(
        commands,
        assets,
        materials,
        sprite_material_cache,
        quad_mesh,
        obj_type,
        x,
        y,
        chunk_pos,
        None,
        None,
    );
}

fn spawn_world_object_with_overrides(
    commands: &mut Commands,
    assets: &Res<crate::assets::GameAssets>,
    materials: &mut ResMut<Assets<LitSpriteMaterial>>,
    sprite_material_cache: &mut LitSpriteMaterialCache,
    quad_mesh: &Res<LitQuadMesh>,
    obj_type: WorldObjectType,
    x: f32,
    y: f32,
    chunk_pos: IVec2,
    color_override: Option<Color>,
    size_override: Option<Vec2>,
) {
    let texture = match obj_type {
        WorldObjectType::OakTree => Some(assets.oak_tree.clone()),
        WorldObjectType::PineTree => Some(assets.pine_tree.clone()),
        WorldObjectType::Rock => Some(assets.rock.clone()),
        WorldObjectType::SandstoneRock => Some(assets.sandstone_rock.clone()),
        WorldObjectType::FallenLog => Some(assets.fallen_log.clone()),
        WorldObjectType::Bush => Some(assets.bush.clone()),
        WorldObjectType::BerryBush => Some(assets.berry_bush.clone()),
        WorldObjectType::Cactus => Some(assets.cactus.clone()),
        WorldObjectType::Mushroom => Some(assets.mushroom.clone()),
        WorldObjectType::GlowingSpore => Some(assets.glowing_spore_obj.clone()),
        WorldObjectType::Wildflower => Some(assets.wildflower.clone()),
        WorldObjectType::GiantMushroom => Some(assets.giant_mushroom.clone()),
        WorldObjectType::CrystalNode => Some(assets.crystal_node.clone()),
        WorldObjectType::CrystalCluster => Some(assets.crystal_cluster.clone()),
        WorldObjectType::IceCrystal => Some(assets.ice_crystal_obj.clone()),
        WorldObjectType::IceFormation => Some(assets.ice_formation.clone()),
        WorldObjectType::EchoStone => Some(assets.echo_stone_obj.clone()),
        WorldObjectType::IronVein => Some(assets.iron_vein.clone()),
        WorldObjectType::CoalDeposit => Some(assets.coal_deposit.clone()),
        WorldObjectType::FrozenOreDeposit => Some(assets.frozen_ore_deposit.clone()),
        WorldObjectType::SulfurDeposit => Some(assets.sulfur_deposit.clone()),
        WorldObjectType::ObsidianNode => Some(assets.obsidian_node.clone()),
        WorldObjectType::AncientMachinery => Some(assets.ancient_machinery.clone()),
        WorldObjectType::SupplyCrate => Some(assets.supply_crate.clone()),
        WorldObjectType::Geyser => Some(assets.geyser.clone()),
        WorldObjectType::AlpineFlower => Some(assets.alpine_flower.clone()),
        WorldObjectType::BioLuminescentGel => Some(assets.bio_luminescent_gel.clone()),
        WorldObjectType::Driftwood => Some(assets.driftwood.clone()),
        WorldObjectType::ShellDeposit => Some(assets.shell_deposit.clone()),
        WorldObjectType::SeaweedPatch => Some(assets.seaweed_patch.clone()),
        WorldObjectType::SulfurVent => Some(assets.sulfur_vent.clone()),
        WorldObjectType::OasisPalm => Some(assets.oasis_palm.clone()),
        WorldObjectType::AncientRuin => Some(assets.ancient_ruin_obj.clone()),
        WorldObjectType::RuinWall => Some(assets.ruins_pillar.clone()),
        WorldObjectType::ReedClump => Some(assets.reed_clump.clone()),
    };

    let color = color_override.unwrap_or_else(|| {
        if texture.is_some() {
            Color::WHITE
        } else {
            obj_type.default_color()
        }
    });
    let size = size_override.unwrap_or_else(|| obj_type.size());

    let (color_texture, normal_texture) = if let Some(tex) = &texture {
        let normal = match obj_type {
            WorldObjectType::OakTree | WorldObjectType::PineTree => assets.flat_normal_32.clone(),
            _ => assets.flat_normal_16.clone(),
        };
        (tex.clone(), normal)
    } else {
        (assets.white_pixel.clone(), assets.flat_normal_16.clone())
    };

    let color_linear = LinearRgba::from(color);
    let material_key =
        SpriteMaterialKey::new(color_linear, &color_texture, &normal_texture);

    let material_handle = if let Some(handle) = sprite_material_cache.by_key.get(&material_key) {
        handle.clone()
    } else {
        let handle = materials.add(LitSpriteMaterial {
            color: color_linear,
            color_texture,
            normal_texture,
        });
        sprite_material_cache.by_key.insert(material_key, handle.clone());
        handle
    };

    commands.spawn((
        WorldObject {
            object_type: obj_type,
            health: obj_type.max_health(),
        },
        ChunkObject { chunk_pos },
        Mesh2d(quad_mesh.quad.clone()),
        MeshMaterial2d(material_handle),
        Transform::from_xyz(x, y, 2.0).with_scale(Vec3::new(size.x, size.y, 1.0)),
    ));
}

/// Deterministic pixel hash for chunk texture noise (consistent across load/unload).
fn pixel_noise(world_tile_x: i32, world_tile_y: i32, sub_x: usize, sub_y: usize) -> f32 {
    let mut h = 2166136261u32;
    h = h.wrapping_mul(16777619) ^ (world_tile_x as u32);
    h = h.wrapping_mul(16777619) ^ (world_tile_y as u32);
    h = h.wrapping_mul(16777619) ^ (sub_x as u32);
    h = h.wrapping_mul(16777619) ^ (sub_y as u32);
    h ^= h >> 13;
    h = h.wrapping_mul(1274126177);
    h ^= h >> 16;
    // Map to -0.5..0.5
    (h as f32 / u32::MAX as f32) - 0.5
}

/// Maps a tile type + biome to the corresponding tile texture handle from GameAssets.
/// Returns a reference to avoid cloning Handle per tile.
fn tile_texture_for<'a>(tile: TileType, biome: Biome, assets: &'a crate::assets::GameAssets) -> Option<&'a Handle<Image>> {
    match (biome, tile) {
        // Forest
        (Biome::Forest, TileType::Grass | TileType::DarkGrass) => Some(&assets.forest_grass),
        (Biome::Forest, TileType::Dirt) => Some(&assets.dirt),
        // Coastal
        (Biome::Coastal, TileType::Sand) => Some(&assets.coastal_sand),
        (Biome::Coastal, TileType::Water) => Some(&assets.coastal_water),
        (Biome::Coastal, TileType::DeepWater) => None, // darker flat color for depth contrast
        (Biome::Coastal, TileType::Grass | TileType::DarkGrass) => Some(&assets.coastal_sand),
        // Desert
        (Biome::Desert, TileType::Sand) => Some(&assets.desert_sand),
        (Biome::Desert, TileType::Dirt) => Some(&assets.desert_cracked),
        (Biome::Desert, TileType::Grass | TileType::DarkGrass) => Some(&assets.desert_sand),
        // Tundra
        (Biome::Tundra, TileType::Snow) => Some(&assets.tundra_snow),
        (Biome::Tundra, TileType::Ice) => Some(&assets.tundra_ice),
        (Biome::Tundra, TileType::Grass | TileType::DarkGrass) => Some(&assets.tundra_snow),
        // Volcanic
        (Biome::Volcanic, TileType::Stone | TileType::Obsidian) => Some(&assets.volcanic_basalt),
        (Biome::Volcanic, TileType::Dirt) => Some(&assets.volcanic_ash),
        (Biome::Volcanic, TileType::Lava) => Some(&assets.volcanic_ash),
        (Biome::Volcanic, TileType::Grass | TileType::DarkGrass) => Some(&assets.volcanic_ash),
        // Swamp
        (Biome::Swamp, TileType::Mud) => Some(&assets.swamp_mud),
        (Biome::Swamp, TileType::Water) => Some(&assets.swamp_water),
        (Biome::Swamp, TileType::DeepWater) => None,
        (Biome::Swamp, TileType::Grass | TileType::DarkGrass) => Some(&assets.swamp_mud),
        // Fungal
        (Biome::Fungal, TileType::MushroomGround) => Some(&assets.fungal_mycelium),
        (Biome::Fungal, TileType::Grass | TileType::DarkGrass) => Some(&assets.fungal_mycelium),
        // CrystalCave
        (Biome::CrystalCave, TileType::CrystalFloor) => Some(&assets.crystal_ground),
        (Biome::CrystalCave, TileType::Stone) => Some(&assets.crystal_ground),
        (Biome::CrystalCave, TileType::Grass | TileType::DarkGrass) => Some(&assets.crystal_ground),
        // Mountain
        (Biome::Mountain, TileType::Stone | TileType::MountainStone) => Some(&assets.mountain_stone),
        (Biome::Mountain, TileType::Dirt) => Some(&assets.mountain_gravel),
        (Biome::Mountain, TileType::Grass | TileType::DarkGrass) => Some(&assets.mountain_gravel),
        // Generic water fallback (DeepWater uses flat color for visual depth)
        (_, TileType::Water) => Some(&assets.water),
        (_, TileType::DeepWater) => None,
        // Generic stone fallback
        (_, TileType::Stone) => Some(&assets.stone),
        // No texture match — use flat color fallback
        _ => None,
    }
}

fn create_chunk_image(
    chunk: &Chunk,
    generator: &WorldGenerator,
    season: Season,
    assets: &crate::assets::GameAssets,
    image_assets: &Assets<Image>,
) -> Image {
    let tile_res = 8; // 8x8 pixels per tile for a textured look
    let res = CHUNK_SIZE * tile_res;
    let size = Extent3d {
        width: res as u32,
        height: res as u32,
        depth_or_array_layers: 1,
    };

    let mut data = vec![0u8; res * res * 4];
    let grass_tint = season.grass_color().to_srgba();
    let (grass_r, grass_g, grass_b) = (grass_tint.red, grass_tint.green, grass_tint.blue);

    for y in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let tile = chunk.get_tile(x, y);
            let world_tile_x = chunk.position.x * CHUNK_SIZE as i32 + x as i32;
            let world_tile_y = chunk.position.y * CHUNK_SIZE as i32 + y as i32;
            let world_x_f64 = world_tile_x as f64;
            let world_y_f64 = world_tile_y as f64;
            let biome = generator.biome_at(world_x_f64, world_y_f64);

            // Biome transition blending: sample neighbors at ±8 tiles
            let blend_offset = 8.0;
            let neighbor_biomes = [
                generator.biome_at(world_x_f64 + blend_offset, world_y_f64),
                generator.biome_at(world_x_f64 - blend_offset, world_y_f64),
                generator.biome_at(world_x_f64, world_y_f64 + blend_offset),
                generator.biome_at(world_x_f64, world_y_f64 - blend_offset),
            ];
            let diff_count = neighbor_biomes.iter().filter(|&&b| b != biome).count();
            let (blend_factor, neighbor_color) = if diff_count > 0 {
                let nb = neighbor_biomes.iter().find(|&&b| b != biome).copied().unwrap_or(biome);
                // Reduced blend strength (0.08 per neighbor vs old 0.15) so textured
                // tiles tint subtly at edges instead of washing out to flat color.
                (0.08 * diff_count as f32, tile.biome_color(nb))
            } else {
                (0.0, [0u8; 4])
            };

            // Try to sample from the tile PNG texture
            let tex_img: Option<&Image> = tile_texture_for(tile, biome, assets)
                .and_then(|h| image_assets.get(h));

            if let Some(tex) = tex_img {
                let tex_w = tex.size().x as i64;
                let tex_h = tex.size().y as i64;
                if tex_w > 0 && tex_h > 0 && tex.data.len() >= (tex_w * tex_h * 4) as usize {
                    for ty in 0..tile_res {
                        for tx in 0..tile_res {
                            let img_y = res - 1 - (y * tile_res + ty);
                            let img_x = x * tile_res + tx;
                            let index = (img_y * res + img_x) * 4;

                            let src_x = (world_tile_x as i64 * tile_res as i64 + tx as i64).rem_euclid(tex_w) as usize;
                            let src_y = (world_tile_y as i64 * tile_res as i64 + ty as i64).rem_euclid(tex_h) as usize;
                            let src_idx = (src_y * tex_w as usize + src_x) * 4;

                            let mut r = tex.data[src_idx] as f32;
                            let mut g = tex.data[src_idx + 1] as f32;
                            let mut b = tex.data[src_idx + 2] as f32;

                            // Seasonal tint for grass and dirt tiles
                            if matches!(tile, TileType::Grass | TileType::DarkGrass | TileType::Dirt) {
                                r *= grass_r;
                                g *= grass_g;
                                b *= grass_b;
                            }

                            // Biome edge blend toward neighbor palette
                            if blend_factor > 0.0 {
                                r = (1.0 - blend_factor) * r + blend_factor * neighbor_color[0] as f32;
                                g = (1.0 - blend_factor) * g + blend_factor * neighbor_color[1] as f32;
                                b = (1.0 - blend_factor) * b + blend_factor * neighbor_color[2] as f32;
                            }

                            data[index] = r.clamp(0.0, 255.0) as u8;
                            data[index + 1] = g.clamp(0.0, 255.0) as u8;
                            data[index + 2] = b.clamp(0.0, 255.0) as u8;
                            data[index + 3] = 255;
                        }
                    }
                    continue; // textured tile done, skip flat-color fallback
                }
            }

            // Fallback: flat color with procedural detail (for unmapped tiles or unloaded textures)
            let base_color = tile.biome_color(biome);
            let base_color = if blend_factor > 0.0 {
                [
                    ((1.0 - blend_factor) * base_color[0] as f32 + blend_factor * neighbor_color[0] as f32) as u8,
                    ((1.0 - blend_factor) * base_color[1] as f32 + blend_factor * neighbor_color[1] as f32) as u8,
                    ((1.0 - blend_factor) * base_color[2] as f32 + blend_factor * neighbor_color[2] as f32) as u8,
                    255,
                ]
            } else {
                base_color
            };

            let tile_variant = pixel_noise(world_tile_x, world_tile_y, 7, 7);
            let variant_offset = (tile_variant * 8.0) as i32;

            for ty in 0..tile_res {
                for tx in 0..tile_res {
                    let img_y = res - 1 - (y * tile_res + ty);
                    let img_x = x * tile_res + tx;
                    let index = (img_y * res + img_x) * 4;

                    let noise = pixel_noise(world_tile_x, world_tile_y, tx, ty) * 15.0;
                    let mut r = base_color[0] as f32 + noise + variant_offset as f32;
                    let mut g = base_color[1] as f32 + noise + variant_offset as f32 * 0.9;
                    let mut b = base_color[2] as f32 + noise + variant_offset as f32 * 0.7;

                    if matches!(tile, TileType::Grass | TileType::DarkGrass | TileType::Dirt) {
                        r *= grass_r;
                        g *= grass_g;
                        b *= grass_b;
                    }

                    let detail = pixel_noise(world_tile_x, world_tile_y, tx.wrapping_add(10), ty.wrapping_add(10));
                    let blade = pixel_noise(world_tile_x, world_tile_y, tx.wrapping_add(20), ty);
                    match tile {
                        TileType::Grass | TileType::DarkGrass => {
                            if blade > 0.25 && blade < 0.32 {
                                r = (r * 1.06).clamp(0.0, 255.0);
                                g = (g * 1.08).clamp(0.0, 255.0);
                                b = (b * 1.02).clamp(0.0, 255.0);
                            } else if blade < -0.28 && blade > -0.35 {
                                r = (r * 0.88).clamp(0.0, 255.0);
                                g = (g * 0.90).clamp(0.0, 255.0);
                                b = (b * 0.88).clamp(0.0, 255.0);
                            }
                            if detail > 0.28 && detail < 0.42 {
                                r = (r * 0.85).clamp(0.0, 255.0);
                                g = (g * 0.9).clamp(0.0, 255.0);
                                b = (b * 0.85).clamp(0.0, 255.0);
                            } else if detail > 0.65 {
                                r = (r * 1.08).clamp(0.0, 255.0);
                                g = (g * 1.05).clamp(0.0, 255.0);
                                b = (b * 1.0).clamp(0.0, 255.0);
                            }
                        }
                        TileType::Dirt | TileType::Mud => {
                            if detail > 0.6 && detail < 0.68 {
                                r = (r * 0.75).clamp(0.0, 255.0);
                                g = (g * 0.72).clamp(0.0, 255.0);
                                b = (b * 0.7).clamp(0.0, 255.0);
                            }
                        }
                        TileType::Sand => {
                            if detail > 0.5 && detail < 0.58 {
                                r = (r * 0.88).clamp(0.0, 255.0);
                                g = (g * 0.86).clamp(0.0, 255.0);
                                b = (b * 0.82).clamp(0.0, 255.0);
                            }
                        }
                        _ => {}
                    }

                    data[index] = r.clamp(0.0, 255.0) as u8;
                    data[index + 1] = g.clamp(0.0, 255.0) as u8;
                    data[index + 2] = b.clamp(0.0, 255.0) as u8;
                    data[index + 3] = 255;
                }
            }
        }
    }

    Image::new(
        size,
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// Builds a normal map for the chunk (same resolution as color). Used for 2D lighting.
fn create_chunk_normal_image(chunk: &Chunk) -> Image {
    let tile_res = 8;
    let res = CHUNK_SIZE * tile_res;
    let size = Extent3d {
        width: res as u32,
        height: res as u32,
        depth_or_array_layers: 1,
    };
    let mut data = vec![0u8; res * res * 4];
    for y in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let tile = chunk.get_tile(x, y);
            let world_tile_x = chunk.position.x * CHUNK_SIZE as i32 + x as i32;
            let world_tile_y = chunk.position.y * CHUNK_SIZE as i32 + y as i32;
            // Slight normal variation from tile type and position (deterministic)
            let h = pixel_noise(world_tile_x, world_tile_y, 0, 0);
            let is_water = matches!(tile, TileType::Water | TileType::DeepWater);
            let tilt = match tile {
                TileType::Water | TileType::DeepWater => 0.0,
                TileType::Sand => h * 0.08,
                _ => h * 0.15,
            };
            let nx = tilt;
            let ny = pixel_noise(world_tile_x, world_tile_y, 1, 0) * 0.15;
            let nz = (1.0_f32).max(0.3);
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            let nx = nx / len;
            let ny = ny / len;
            let nz = nz / len;
            let r = ((nx * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;
            let g = ((ny * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;
            let b = ((nz * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;
            let a = if is_water { 0u8 } else { 255 };
            for ty in 0..tile_res {
                for tx in 0..tile_res {
                    let img_y = res - 1 - (y * tile_res + ty);
                    let img_x = x * tile_res + tx;
                    let index = (img_y * res + img_x) * 4;
                    data[index] = r;
                    data[index + 1] = g;
                    data[index + 2] = b;
                    data[index + 3] = a;
                }
            }
        }
    }
    Image::new(
        size,
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// Spawns a full-screen dark overlay with "Loading..." text, hidden once textures are ready.
fn spawn_loading_overlay(mut commands: Commands) {
    commands.spawn((
        LoadingOverlay,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.008, 0.008, 0.024)), // matches ClearColor
        GlobalZIndex(500),
    )).with_children(|parent| {
        parent.spawn((
            Text::new("Loading..."),
            TextFont { font_size: 32.0, ..default() },
            TextColor(Color::srgba(0.9, 0.85, 0.6, 1.0)),
        ));
    });
}

/// One-shot system: once the first tile texture finishes async loading, despawn all
/// chunks so `manage_chunks` respawns them with real textures instead of flat colors.
/// Also despawns the loading overlay.
fn refresh_chunks_on_texture_load(
    mut commands: Commands,
    mut ready: ResMut<TileTexturesReady>,
    assets: Res<crate::assets::GameAssets>,
    image_assets: Res<Assets<Image>>,
    mut world_state: ResMut<WorldState>,
    chunks_query: Query<(Entity, &Chunk)>,
    objects_query: Query<(Entity, &ChunkObject)>,
    overlay_query: Query<Entity, With<LoadingOverlay>>,
) {
    if ready.0 {
        return; // already refreshed once
    }
    // Check if a representative tile texture has loaded
    if image_assets.get(&assets.forest_grass).is_none() {
        return; // still loading
    }
    ready.0 = true;

    // Despawn the loading overlay
    for entity in overlay_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Despawn all current chunks — manage_chunks will respawn them with textures
    for (entity, chunk) in chunks_query.iter() {
        world_state.loaded_chunks.remove(&chunk.position);
        commands.entity(entity).despawn();
    }
    for (entity, _) in objects_query.iter() {
        commands.entity(entity).despawn();
    }
}

#[derive(SystemParam)]
struct ManageChunksRenderParams<'w> {
    chunk_materials: ResMut<'w, Assets<LitChunkMaterial>>,
    sprite_materials: ResMut<'w, Assets<LitSpriteMaterial>>,
    sprite_material_cache: ResMut<'w, LitSpriteMaterialCache>,
    chunk_image_cache: ResMut<'w, ChunkImageCache>,
}

fn manage_chunks(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut render: ManageChunksRenderParams,
    mut perf: ResMut<crate::debug_perf::DebugPerfTiming>,
    assets: Res<crate::assets::GameAssets>,
    quad_mesh: Res<LitQuadMesh>,
    mut world_state: ResMut<WorldState>,
    mut dungeon_registry: ResMut<DungeonRegistry>,
    mut loaded_chunk_cache: ResMut<LoadedChunkCache>,
    mut chunk_gen_async: ResMut<ChunkGenAsync>,
    season_cycle: Res<SeasonCycle>,
    player_query: Query<&Transform, With<Player>>,
    chunks_query: Query<(Entity, &Chunk)>,
    objects_query: Query<(Entity, &ChunkObject)>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    if dungeon_registry.current_dungeon.is_some() {
        return;
    }

    let start = Instant::now();

    let player_chunk = IVec2::new(
        (player_transform.translation.x / CHUNK_WORLD_SIZE).floor() as i32,
        (player_transform.translation.y / CHUNK_WORLD_SIZE).floor() as i32,
    );

    // Drain async chunk results and spawn on main thread (non-blocking)
    let drained: Vec<(IVec2, Chunk)> = {
        if let Ok(mut q) = chunk_gen_async.results.lock() {
            q.drain(..).collect()
        } else {
            Vec::new()
        }
    };
    for (chunk_pos, chunk) in drained {
        spawn_chunk_from_data(
            &mut commands,
            &mut images,
            &mut render.chunk_materials,
            &mut render.sprite_materials,
            &mut render.sprite_material_cache,
            &mut render.chunk_image_cache,
            &assets,
            &quad_mesh,
            &mut dungeon_registry,
            chunk_pos,
            chunk,
            world_state.seed,
            &world_state.generator,
            &season_cycle,
            player_chunk,
        );
        world_state.loaded_chunks.insert(chunk_pos);
        chunk_gen_async.requested.remove(&chunk_pos);
    }

    // Load new chunks: from cache (sync), or request async
    for cy in (player_chunk.y - RENDER_DISTANCE)..=(player_chunk.y + RENDER_DISTANCE) {
        for cx in (player_chunk.x - RENDER_DISTANCE)..=(player_chunk.x + RENDER_DISTANCE) {
            let chunk_pos = IVec2::new(cx, cy);
            if world_state.loaded_chunks.contains(&chunk_pos) {
                continue;
            }
            if let Some(tiles) = loaded_chunk_cache.0.remove(&chunk_pos) {
                let center_x = (chunk_pos.x * CHUNK_SIZE as i32 + CHUNK_SIZE as i32 / 2) as f64;
                let center_y = (chunk_pos.y * CHUNK_SIZE as i32 + CHUNK_SIZE as i32 / 2) as f64;
                let biome = world_state.generator.biome_at(center_x, center_y);
                let chunk = Chunk::from_tiles(chunk_pos, &tiles, biome);
                spawn_chunk_from_data(
                    &mut commands,
                    &mut images,
                    &mut render.chunk_materials,
                    &mut render.sprite_materials,
                    &mut render.sprite_material_cache,
                    &mut render.chunk_image_cache,
                    &assets,
                    &quad_mesh,
                    &mut dungeon_registry,
                    chunk_pos,
                    chunk,
                    world_state.seed,
                    &world_state.generator,
                    &season_cycle,
                    player_chunk,
                );
                world_state.loaded_chunks.insert(chunk_pos);
            } else if chunk_gen_async.requested.contains(&chunk_pos) {
                // Already requested, wait for result
            } else if chunk_gen_async.request_tx.send((world_state.seed, chunk_pos)).is_ok() {
                chunk_gen_async.requested.insert(chunk_pos);
            }
        }
    }

    // Unload distant chunks
    let unload_distance = RENDER_DISTANCE + 2;
    let mut chunks_to_remove = Vec::new();

    for (entity, chunk) in chunks_query.iter() {
        let dx = (chunk.position.x - player_chunk.x).abs();
        let dy = (chunk.position.y - player_chunk.y).abs();
        if dx > unload_distance || dy > unload_distance {
            chunks_to_remove.push((entity, chunk.position));
        }
    }

    for (entity, pos) in &chunks_to_remove {
        commands.entity(*entity).despawn();
        world_state.loaded_chunks.remove(pos);
    }

    // Unload objects belonging to despawned chunks
    if !chunks_to_remove.is_empty() {
        let removed_positions: HashSet<IVec2> = chunks_to_remove.iter().map(|(_, p)| *p).collect();
        for (entity, chunk_obj) in objects_query.iter() {
            if removed_positions.contains(&chunk_obj.chunk_pos) {
                commands.entity(entity).despawn();
            }
        }
    }

    perf.chunk_manage_ms = start.elapsed().as_secs_f32() * 1000.0;
}

/// Shows a small interaction hint ("E") above the nearest gatherable
/// WorldObject when the player is within 32px.  The hint entity is
/// despawned/recreated each frame to avoid stale markers.
fn show_interaction_hints(
    mut commands: Commands,
    hint_query: Query<Entity, With<InteractionHint>>,
    player_query: Query<&Transform, With<Player>>,
    object_query: Query<(&Transform, &WorldObject)>,
) {
    // Despawn all existing hint entities
    for entity in hint_query.iter() {
        commands.entity(entity).despawn();
    }

    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    // Find nearest gatherable world object within 32px
    let mut nearest: Option<(&Transform, f32)> = None;
    for (obj_tf, _obj) in object_query.iter() {
        let dist = player_pos.distance(obj_tf.translation.truncate());
        if dist <= 32.0 {
            if nearest.is_none() || dist < nearest.unwrap().1 {
                nearest = Some((obj_tf, dist));
            }
        }
    }

    if let Some((obj_tf, _)) = nearest {
        // Spawn hint text 12px above the object
        let hint_x = obj_tf.translation.x;
        let hint_y = obj_tf.translation.y + 12.0;

        commands.spawn((
            InteractionHint,
            Text2d::new("E"),
            TextFont {
                font_size: 10.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 0.6, 0.9)),
            Transform::from_xyz(hint_x, hint_y, 10.0),
        ));
    }
}
