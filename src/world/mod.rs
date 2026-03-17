pub mod chunk;
pub mod generation;
pub mod tile;

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use chunk::{Chunk, CHUNK_SIZE};
use generation::{WorldGenerator, Biome};
use std::collections::HashSet;
use crate::player::Player;
use crate::dungeon::{DungeonRegistry, should_spawn_entrance};
use crate::hud::not_paused;
use crate::npc;

pub const TILE_SIZE: f32 = 16.0;
pub const CHUNK_WORLD_SIZE: f32 = TILE_SIZE * CHUNK_SIZE as f32;
pub const RENDER_DISTANCE: i32 = 5;

pub struct WorldPlugin;

/// Marker component for interaction hint text entities.
#[derive(Component)]
pub struct InteractionHint;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        let seed = rand::random::<u32>();
        app.insert_resource(WorldState::new(seed))
            .add_systems(Startup, spawn_initial_chunks)
            .add_systems(Update, (
                manage_chunks,
                show_interaction_hints.run_if(not_paused),
            ));
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
        }
    }

    pub fn min_tool_tier(&self) -> u32 {
        match self {
            WorldObjectType::IronVein | WorldObjectType::CrystalNode
            | WorldObjectType::ObsidianNode | WorldObjectType::FrozenOreDeposit => 2,
            WorldObjectType::AncientRuin => 3,
            _ => 0,
        }
    }

    pub fn size(&self) -> Vec2 {
        match self {
            WorldObjectType::OakTree => Vec2::new(14.0, 20.0),
            WorldObjectType::PineTree => Vec2::new(10.0, 22.0),
            WorldObjectType::Rock => Vec2::new(12.0, 10.0),
            WorldObjectType::Bush => Vec2::new(12.0, 10.0),
            WorldObjectType::Cactus => Vec2::new(8.0, 18.0),
            WorldObjectType::IceCrystal => Vec2::new(10.0, 14.0),
            WorldObjectType::Mushroom => Vec2::new(8.0, 8.0),
            WorldObjectType::GiantMushroom => Vec2::new(16.0, 24.0),
            WorldObjectType::ReedClump => Vec2::new(10.0, 14.0),
            WorldObjectType::SulfurDeposit => Vec2::new(12.0, 8.0),
            WorldObjectType::CrystalNode => Vec2::new(12.0, 14.0),
            WorldObjectType::AlpineFlower => Vec2::new(6.0, 6.0),
            WorldObjectType::IronVein => Vec2::new(14.0, 10.0),
            WorldObjectType::CoalDeposit => Vec2::new(12.0, 10.0),
            WorldObjectType::AncientRuin => Vec2::new(16.0, 16.0),
            WorldObjectType::SupplyCrate => Vec2::new(10.0, 8.0),
            WorldObjectType::RuinWall => Vec2::new(16.0, 20.0),
            WorldObjectType::BerryBush => Vec2::new(10.0, 9.0),
            WorldObjectType::SandstoneRock => Vec2::new(14.0, 11.0),
            WorldObjectType::OasisPalm => Vec2::new(12.0, 20.0),
            WorldObjectType::FrozenOreDeposit => Vec2::new(12.0, 10.0),
            WorldObjectType::IceFormation => Vec2::new(14.0, 16.0),
            WorldObjectType::SulfurVent => Vec2::new(10.0, 12.0),
            WorldObjectType::ObsidianNode => Vec2::new(12.0, 10.0),
            WorldObjectType::GlowingSpore => Vec2::new(6.0, 8.0),
            WorldObjectType::BioLuminescentGel => Vec2::new(8.0, 6.0),
            WorldObjectType::CrystalCluster => Vec2::new(14.0, 16.0),
            WorldObjectType::EchoStone => Vec2::new(10.0, 12.0),
            WorldObjectType::Driftwood => Vec2::new(14.0, 6.0),
            WorldObjectType::ShellDeposit => Vec2::new(8.0, 6.0),
            WorldObjectType::SeaweedPatch => Vec2::new(10.0, 8.0),
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
    assets: Res<crate::assets::GameAssets>,
    mut world_state: ResMut<WorldState>,
    mut dungeon_registry: ResMut<DungeonRegistry>,
) {
    for cy in -RENDER_DISTANCE..=RENDER_DISTANCE {
        for cx in -RENDER_DISTANCE..=RENDER_DISTANCE {
            let chunk_pos = IVec2::new(cx, cy);
            spawn_chunk(&mut commands, &mut images, &assets, &mut world_state, &mut dungeon_registry, chunk_pos);
        }
    }
}

fn spawn_chunk(
    commands: &mut Commands,
    images: &mut ResMut<Assets<Image>>,
    assets: &Res<crate::assets::GameAssets>,
    world_state: &mut ResMut<WorldState>,
    dungeon_registry: &mut ResMut<DungeonRegistry>,
    chunk_pos: IVec2,
) {
    if world_state.loaded_chunks.contains(&chunk_pos) {
        return;
    }

    let chunk = world_state.generator.generate_chunk(chunk_pos);
    let image = create_chunk_image(&chunk);
    let image_handle = images.add(image);

    let world_x = chunk_pos.x as f32 * CHUNK_WORLD_SIZE + CHUNK_WORLD_SIZE / 2.0;
    let world_y = chunk_pos.y as f32 * CHUNK_WORLD_SIZE + CHUNK_WORLD_SIZE / 2.0;

    // Use chunk data for object spawning BEFORE moving into ECS
    let seed = world_state.seed;
    spawn_chunk_objects(commands, assets, chunk_pos, &chunk, seed, dungeon_registry);

    // Spawn terrain chunk (moves chunk into ECS)
    commands.spawn((
        chunk,
        Sprite {
            image: image_handle,
            custom_size: Some(Vec2::new(CHUNK_WORLD_SIZE, CHUNK_WORLD_SIZE)),
            ..default()
        },
        Transform::from_xyz(world_x, world_y, 0.0),
    ));

    world_state.loaded_chunks.insert(chunk_pos);
}

/// Returns a tree color variant based on a hash value (3 green shades).
fn tree_color_variant(hash: u32) -> Color {
    match hash % 3 {
        0 => Color::srgb(0.10, 0.35, 0.08), // darker
        1 => Color::srgb(0.15, 0.45, 0.12), // normal
        _ => Color::srgb(0.20, 0.55, 0.16), // lighter
    }
}

/// Returns a rock size variant based on a hash value (2 sizes).
fn rock_size_variant(hash: u32) -> Vec2 {
    match hash % 2 {
        0 => Vec2::new(8.0, 7.0),  // small
        _ => Vec2::new(12.0, 10.0), // normal
    }
}

fn spawn_chunk_objects(
    commands: &mut Commands,
    assets: &Res<crate::assets::GameAssets>,
    chunk_pos: IVec2,
    chunk: &Chunk,
    seed: u32,
    dungeon_registry: &mut ResMut<DungeonRegistry>,
) {
    let biome = chunk.biome;

    // Track whether we have already placed one entrance in this chunk so we
    // don't cluster multiple portals.
    let mut entrance_placed_this_chunk = false;

    for y in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let tile = chunk.get_tile(x, y);
            if !tile.is_walkable() {
                continue;
            }

            let world_tile_x = chunk_pos.x * CHUNK_SIZE as i32 + x as i32;
            let world_tile_y = chunk_pos.y * CHUNK_SIZE as i32 + y as i32;

            let wx = world_tile_x as f32 * TILE_SIZE + TILE_SIZE / 2.0;
            let wy = world_tile_y as f32 * TILE_SIZE + TILE_SIZE / 2.0;

            let hash = WorldGenerator::position_hash(world_tile_x, world_tile_y, seed);
            let hash2 = WorldGenerator::position_hash(world_tile_x, world_tile_y, seed.wrapping_add(50));
            // Variant hash for color/size variety (uses a different offset)
            let variant_hash = WorldGenerator::position_hash(world_tile_x, world_tile_y, seed.wrapping_add(123));
            let density_roll = hash % 100;
            let variant_roll = hash2 % 100;

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
                    npc::spawn_hermit(commands, wx, wy, chunk_pos);
                    continue;
                }
            }

            match biome {
                Biome::Forest => {
                    // Highest tree density (8-12 per chunk via ~8-12% coverage)
                    if density_roll < 8 {
                        let obj = if variant_roll < 50 { WorldObjectType::OakTree } else { WorldObjectType::PineTree };
                        let color = tree_color_variant(variant_hash);
                        spawn_world_object_with_overrides(commands, assets, obj, wx, wy, chunk_pos, Some(color), None);
                    } else if density_roll < 11 {
                        // Berry bushes (biome-exclusive)
                        spawn_world_object(commands, assets, WorldObjectType::BerryBush, wx, wy, chunk_pos);
                    } else if density_roll < 13 {
                        // Mushrooms on forest floor
                        spawn_world_object(commands, assets, WorldObjectType::Mushroom, wx, wy, chunk_pos);
                    } else if density_roll < 15 {
                        let size = rock_size_variant(variant_hash);
                        spawn_world_object_with_overrides(commands, assets, WorldObjectType::Rock, wx, wy, chunk_pos, None, Some(size));
                    } else if density_roll == 99 {
                        spawn_world_object(commands, assets, WorldObjectType::SupplyCrate, wx, wy, chunk_pos);
                    }
                }
                Biome::Coastal => {
                    // Driftwood, shell deposits, and seaweed near water
                    if density_roll < 3 {
                        spawn_world_object(commands, assets, WorldObjectType::Driftwood, wx, wy, chunk_pos);
                    } else if density_roll < 5 {
                        spawn_world_object(commands, assets, WorldObjectType::ShellDeposit, wx, wy, chunk_pos);
                    } else if density_roll < 7 {
                        spawn_world_object(commands, assets, WorldObjectType::SeaweedPatch, wx, wy, chunk_pos);
                    } else if density_roll < 9 {
                        let size = rock_size_variant(variant_hash);
                        spawn_world_object_with_overrides(commands, assets, WorldObjectType::Rock, wx, wy, chunk_pos, None, Some(size));
                    } else if density_roll < 10 {
                        spawn_world_object(commands, assets, WorldObjectType::Bush, wx, wy, chunk_pos);
                    }
                }
                Biome::Swamp => {
                    if density_roll < 6 {
                        spawn_world_object(commands, assets, WorldObjectType::ReedClump, wx, wy, chunk_pos);
                    } else if density_roll < 10 {
                        let color = Color::srgb(0.7, 0.2, 0.3);
                        spawn_world_object_with_overrides(commands, assets, WorldObjectType::Bush, wx, wy, chunk_pos, Some(color), None);
                    } else if density_roll < 12 {
                        let color = tree_color_variant(variant_hash);
                        spawn_world_object_with_overrides(commands, assets, WorldObjectType::OakTree, wx, wy, chunk_pos, Some(color), None);
                    }
                }
                Biome::Desert => {
                    // Cacti, sandstone rocks, and rare oasis palms
                    if density_roll < 3 {
                        spawn_world_object(commands, assets, WorldObjectType::Cactus, wx, wy, chunk_pos);
                    } else if density_roll < 6 {
                        spawn_world_object(commands, assets, WorldObjectType::SandstoneRock, wx, wy, chunk_pos);
                    } else if density_roll < 7 {
                        let size = rock_size_variant(variant_hash);
                        spawn_world_object_with_overrides(commands, assets, WorldObjectType::Rock, wx, wy, chunk_pos, None, Some(size));
                    } else if density_roll == 98 {
                        // Rare oasis palm (with implied water tiles nearby)
                        spawn_world_object(commands, assets, WorldObjectType::OasisPalm, wx, wy, chunk_pos);
                    } else if density_roll == 99 {
                        spawn_world_object(commands, assets, WorldObjectType::RuinWall, wx, wy, chunk_pos);
                    }
                }
                Biome::Tundra => {
                    // Sparse trees, ice formations, and frozen ore deposits
                    if density_roll < 2 {
                        // Sparse pine trees
                        let color = Color::srgb(0.15, 0.3, 0.2);
                        spawn_world_object_with_overrides(commands, assets, WorldObjectType::PineTree, wx, wy, chunk_pos, Some(color), None);
                    } else if density_roll < 5 {
                        spawn_world_object(commands, assets, WorldObjectType::IceFormation, wx, wy, chunk_pos);
                    } else if density_roll < 7 {
                        spawn_world_object(commands, assets, WorldObjectType::IceCrystal, wx, wy, chunk_pos);
                    } else if density_roll < 9 {
                        let size = rock_size_variant(variant_hash);
                        spawn_world_object_with_overrides(commands, assets, WorldObjectType::Rock, wx, wy, chunk_pos, None, Some(size));
                    } else if density_roll < 10 {
                        spawn_world_object(commands, assets, WorldObjectType::FrozenOreDeposit, wx, wy, chunk_pos);
                    }
                }
                Biome::Volcanic => {
                    // Obsidian nodes, sulfur vents (yellow), no trees
                    if density_roll < 1 {
                        spawn_world_object(commands, assets, WorldObjectType::AncientRuin, wx, wy, chunk_pos);
                    } else if density_roll < 4 {
                        spawn_world_object(commands, assets, WorldObjectType::ObsidianNode, wx, wy, chunk_pos);
                    } else if density_roll < 7 {
                        // Sulfur vents with bright yellow color
                        spawn_world_object(commands, assets, WorldObjectType::SulfurVent, wx, wy, chunk_pos);
                    } else if density_roll < 9 {
                        spawn_world_object(commands, assets, WorldObjectType::SulfurDeposit, wx, wy, chunk_pos);
                    } else if density_roll < 11 {
                        let size = rock_size_variant(variant_hash);
                        spawn_world_object_with_overrides(commands, assets, WorldObjectType::Rock, wx, wy, chunk_pos, None, Some(size));
                    } else if density_roll < 13 {
                        spawn_world_object(commands, assets, WorldObjectType::CoalDeposit, wx, wy, chunk_pos);
                    }
                }
                Biome::Fungal => {
                    // Giant mushrooms (larger sprites), glowing spores, bio-luminescent gel
                    if density_roll < 4 {
                        spawn_world_object(commands, assets, WorldObjectType::GiantMushroom, wx, wy, chunk_pos);
                    } else if density_roll < 8 {
                        spawn_world_object(commands, assets, WorldObjectType::Mushroom, wx, wy, chunk_pos);
                    } else if density_roll < 11 {
                        spawn_world_object(commands, assets, WorldObjectType::GlowingSpore, wx, wy, chunk_pos);
                    } else if density_roll < 13 {
                        spawn_world_object(commands, assets, WorldObjectType::BioLuminescentGel, wx, wy, chunk_pos);
                    }
                }
                Biome::CrystalCave => {
                    // Crystal clusters, gemstone nodes, and echo stones
                    if density_roll < 4 {
                        spawn_world_object(commands, assets, WorldObjectType::CrystalCluster, wx, wy, chunk_pos);
                    } else if density_roll < 7 {
                        spawn_world_object(commands, assets, WorldObjectType::CrystalNode, wx, wy, chunk_pos);
                    } else if density_roll < 9 {
                        spawn_world_object(commands, assets, WorldObjectType::EchoStone, wx, wy, chunk_pos);
                    } else if density_roll < 11 {
                        let size = rock_size_variant(variant_hash);
                        spawn_world_object_with_overrides(commands, assets, WorldObjectType::Rock, wx, wy, chunk_pos, None, Some(size));
                    } else if density_roll < 13 {
                        spawn_world_object(commands, assets, WorldObjectType::IronVein, wx, wy, chunk_pos);
                    }
                }
                Biome::Mountain => {
                    if density_roll < 1 {
                        spawn_world_object(commands, assets, WorldObjectType::AncientRuin, wx, wy, chunk_pos);
                    } else if density_roll < 5 {
                        let size = rock_size_variant(variant_hash);
                        spawn_world_object_with_overrides(commands, assets, WorldObjectType::Rock, wx, wy, chunk_pos, None, Some(size));
                    } else if density_roll < 7 {
                        spawn_world_object(commands, assets, WorldObjectType::AlpineFlower, wx, wy, chunk_pos);
                    } else if density_roll < 9 {
                        spawn_world_object(commands, assets, WorldObjectType::IronVein, wx, wy, chunk_pos);
                    } else if density_roll < 11 {
                        spawn_world_object(commands, assets, WorldObjectType::CoalDeposit, wx, wy, chunk_pos);
                    } else if density_roll == 99 {
                        spawn_world_object(commands, assets, WorldObjectType::RuinWall, wx, wy, chunk_pos);
                    }
                }
            }
        }
    }
}

fn spawn_world_object(
    commands: &mut Commands,
    assets: &Res<crate::assets::GameAssets>,
    obj_type: WorldObjectType,
    x: f32,
    y: f32,
    chunk_pos: IVec2,
) {
    spawn_world_object_with_overrides(commands, assets, obj_type, x, y, chunk_pos, None, None);
}

fn spawn_world_object_with_overrides(
    commands: &mut Commands,
    assets: &Res<crate::assets::GameAssets>,
    obj_type: WorldObjectType,
    x: f32,
    y: f32,
    chunk_pos: IVec2,
    color_override: Option<Color>,
    size_override: Option<Vec2>,
) {
    let texture = match obj_type {
        WorldObjectType::OakTree => assets.oak_tree.clone(),
        WorldObjectType::PineTree => assets.pine_tree.clone(),
        WorldObjectType::Rock | WorldObjectType::SandstoneRock => assets.rock.clone(),
        _ => Handle::default(),
    };

    commands.spawn((
        WorldObject {
            object_type: obj_type,
            health: obj_type.max_health(),
        },
        ChunkObject { chunk_pos },
        Sprite {
            image: texture,
            color: color_override.unwrap_or(Color::WHITE),
            custom_size: Some(size_override.unwrap_or_else(|| obj_type.size())),
            ..default()
        },
        Transform::from_xyz(x, y, 2.0),
    ));
}

fn create_chunk_image(chunk: &Chunk) -> Image {
    let tile_res = 8; // 8x8 pixels per tile for a textured look
    let res = CHUNK_SIZE * tile_res;
    let size = Extent3d {
        width: res as u32,
        height: res as u32,
        depth_or_array_layers: 1,
    };

    let mut data = vec![0u8; res * res * 4];
    let biome = chunk.biome;

    for y in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let tile = chunk.get_tile(x, y);
            let base_color = tile.biome_color(biome);
            
            for ty in 0..tile_res {
                for tx in 0..tile_res {
                    // Image y=0 is top, world y=0 is bottom, so flip
                    let img_y = res - 1 - (y * tile_res + ty);
                    let img_x = x * tile_res + tx;
                    let index = (img_y * res + img_x) * 4;
                    
                    // Add some noise to the base color
                    let noise = (rand::random::<f32>() - 0.5) * 15.0;
                    data[index] = (base_color[0] as f32 + noise).clamp(0.0, 255.0) as u8;
                    data[index + 1] = (base_color[1] as f32 + noise).clamp(0.0, 255.0) as u8;
                    data[index + 2] = (base_color[2] as f32 + noise).clamp(0.0, 255.0) as u8;
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

fn manage_chunks(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    assets: Res<crate::assets::GameAssets>,
    mut world_state: ResMut<WorldState>,
    mut dungeon_registry: ResMut<DungeonRegistry>,
    player_query: Query<&Transform, With<Player>>,
    chunks_query: Query<(Entity, &Chunk)>,
    objects_query: Query<(Entity, &ChunkObject)>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    // Don't load/unload surface chunks while the player is inside a dungeon –
    // the player's Y position is deep underground and would trigger mass loading.
    if dungeon_registry.current_dungeon.is_some() {
        return;
    }

    let player_chunk = IVec2::new(
        (player_transform.translation.x / CHUNK_WORLD_SIZE).floor() as i32,
        (player_transform.translation.y / CHUNK_WORLD_SIZE).floor() as i32,
    );

    // Load new chunks
    for cy in (player_chunk.y - RENDER_DISTANCE)..=(player_chunk.y + RENDER_DISTANCE) {
        for cx in (player_chunk.x - RENDER_DISTANCE)..=(player_chunk.x + RENDER_DISTANCE) {
            let chunk_pos = IVec2::new(cx, cy);
            if !world_state.loaded_chunks.contains(&chunk_pos) {
                spawn_chunk(&mut commands, &mut images, &assets, &mut world_state, &mut dungeon_registry, chunk_pos);
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
