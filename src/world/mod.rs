pub mod chunk;
pub mod generation;
pub mod tile;

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use chunk::{Chunk, CHUNK_SIZE};
use generation::{WorldGenerator, Biome};
use std::collections::HashSet;
use tile::TileType;

use crate::player::Player;
use crate::dungeon::{DungeonRegistry, should_spawn_entrance, spawn_entrance};

pub const TILE_SIZE: f32 = 16.0;
pub const CHUNK_WORLD_SIZE: f32 = TILE_SIZE * CHUNK_SIZE as f32;
pub const RENDER_DISTANCE: i32 = 5;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldState::new(42))
            .add_systems(Startup, spawn_initial_chunks)
            .add_systems(Update, manage_chunks);
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
        }
    }

    pub fn color(&self) -> Color {
        match self {
            WorldObjectType::OakTree => Color::srgb(0.15, 0.45, 0.12),
            WorldObjectType::PineTree => Color::srgb(0.1, 0.35, 0.15),
            WorldObjectType::Rock => Color::srgb(0.5, 0.5, 0.5),
            WorldObjectType::Bush => Color::srgb(0.2, 0.55, 0.18),
            WorldObjectType::Cactus => Color::srgb(0.3, 0.6, 0.2),
            WorldObjectType::IceCrystal => Color::srgb(0.7, 0.85, 0.95),
            WorldObjectType::Mushroom => Color::srgb(0.7, 0.3, 0.3),
            WorldObjectType::GiantMushroom => Color::srgb(0.5, 0.2, 0.6),
            WorldObjectType::ReedClump => Color::srgb(0.4, 0.5, 0.25),
            WorldObjectType::SulfurDeposit => Color::srgb(0.8, 0.75, 0.2),
            WorldObjectType::CrystalNode => Color::srgb(0.6, 0.5, 0.8),
            WorldObjectType::AlpineFlower => Color::srgb(0.8, 0.4, 0.7),
            WorldObjectType::IronVein => Color::srgb(0.35, 0.3, 0.3),
            WorldObjectType::CoalDeposit => Color::srgb(0.15, 0.12, 0.12),
            WorldObjectType::AncientRuin => Color::srgb(0.6, 0.5, 0.2),
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
    mut world_state: ResMut<WorldState>,
    mut dungeon_registry: ResMut<DungeonRegistry>,
) {
    for cy in -RENDER_DISTANCE..=RENDER_DISTANCE {
        for cx in -RENDER_DISTANCE..=RENDER_DISTANCE {
            let chunk_pos = IVec2::new(cx, cy);
            spawn_chunk(&mut commands, &mut images, &mut world_state, &mut dungeon_registry, chunk_pos);
        }
    }
}

fn spawn_chunk(
    commands: &mut Commands,
    images: &mut ResMut<Assets<Image>>,
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

    // Spawn terrain chunk
    commands.spawn((
        chunk,
        Sprite {
            image: image_handle,
            custom_size: Some(Vec2::new(CHUNK_WORLD_SIZE, CHUNK_WORLD_SIZE)),
            ..default()
        },
        Transform::from_xyz(world_x, world_y, 0.0),
    ));

    // Spawn world objects on this chunk
    let seed = world_state.seed;
    spawn_chunk_objects(commands, chunk_pos, &world_state.generator, seed, dungeon_registry);

    world_state.loaded_chunks.insert(chunk_pos);
}

fn spawn_chunk_objects(
    commands: &mut Commands,
    chunk_pos: IVec2,
    generator: &WorldGenerator,
    seed: u32,
    dungeon_registry: &mut ResMut<DungeonRegistry>,
) {
    let chunk = generator.generate_chunk(chunk_pos);
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
            let density_roll = hash % 100;
            let variant_roll = hash2 % 100;

            // --- Dungeon entrance for eligible biomes ---
            if !entrance_placed_this_chunk
                && matches!(biome, Biome::Mountain | Biome::Volcanic | Biome::CrystalCave)
                && should_spawn_entrance(world_tile_x, world_tile_y, seed)
            {
                spawn_entrance(commands, dungeon_registry, wx, wy, chunk_pos);
                entrance_placed_this_chunk = true;
                // Skip placing a world object on the same tile.
                continue;
            }

            match biome {
                Biome::Forest => {
                    if density_roll < 6 {
                        let obj = if variant_roll < 50 { WorldObjectType::OakTree } else { WorldObjectType::PineTree };
                        spawn_world_object(commands, obj, wx, wy, chunk_pos);
                    } else if density_roll < 10 {
                        spawn_world_object(commands, WorldObjectType::Bush, wx, wy, chunk_pos);
                    } else if density_roll < 12 {
                        spawn_world_object(commands, WorldObjectType::Rock, wx, wy, chunk_pos);
                    }
                }
                Biome::Coastal => {
                    if density_roll < 3 {
                        spawn_world_object(commands, WorldObjectType::Rock, wx, wy, chunk_pos);
                    } else if density_roll < 5 {
                        spawn_world_object(commands, WorldObjectType::Bush, wx, wy, chunk_pos);
                    }
                }
                Biome::Swamp => {
                    if density_roll < 6 {
                        spawn_world_object(commands, WorldObjectType::ReedClump, wx, wy, chunk_pos);
                    } else if density_roll < 10 {
                        spawn_world_object(commands, WorldObjectType::Bush, wx, wy, chunk_pos);
                    } else if density_roll < 12 {
                        spawn_world_object(commands, WorldObjectType::OakTree, wx, wy, chunk_pos);
                    }
                }
                Biome::Desert => {
                    if density_roll < 3 {
                        spawn_world_object(commands, WorldObjectType::Cactus, wx, wy, chunk_pos);
                    } else if density_roll < 5 {
                        spawn_world_object(commands, WorldObjectType::Rock, wx, wy, chunk_pos);
                    }
                }
                Biome::Tundra => {
                    if density_roll < 3 {
                        spawn_world_object(commands, WorldObjectType::IceCrystal, wx, wy, chunk_pos);
                    } else if density_roll < 5 {
                        spawn_world_object(commands, WorldObjectType::Rock, wx, wy, chunk_pos);
                    }
                }
                Biome::Volcanic => {
                    if density_roll < 1 {
                        spawn_world_object(commands, WorldObjectType::AncientRuin, wx, wy, chunk_pos);
                    } else if density_roll < 4 {
                        spawn_world_object(commands, WorldObjectType::SulfurDeposit, wx, wy, chunk_pos);
                    } else if density_roll < 6 {
                        spawn_world_object(commands, WorldObjectType::Rock, wx, wy, chunk_pos);
                    } else if density_roll < 8 {
                        spawn_world_object(commands, WorldObjectType::CoalDeposit, wx, wy, chunk_pos);
                    }
                }
                Biome::Fungal => {
                    if density_roll < 6 {
                        spawn_world_object(commands, WorldObjectType::Mushroom, wx, wy, chunk_pos);
                    } else if density_roll < 9 {
                        spawn_world_object(commands, WorldObjectType::GiantMushroom, wx, wy, chunk_pos);
                    }
                }
                Biome::CrystalCave => {
                    if density_roll < 5 {
                        spawn_world_object(commands, WorldObjectType::CrystalNode, wx, wy, chunk_pos);
                    } else if density_roll < 7 {
                        spawn_world_object(commands, WorldObjectType::Rock, wx, wy, chunk_pos);
                    } else if density_roll < 9 {
                        spawn_world_object(commands, WorldObjectType::IronVein, wx, wy, chunk_pos);
                    }
                }
                Biome::Mountain => {
                    if density_roll < 1 {
                        spawn_world_object(commands, WorldObjectType::AncientRuin, wx, wy, chunk_pos);
                    } else if density_roll < 5 {
                        spawn_world_object(commands, WorldObjectType::Rock, wx, wy, chunk_pos);
                    } else if density_roll < 7 {
                        spawn_world_object(commands, WorldObjectType::AlpineFlower, wx, wy, chunk_pos);
                    } else if density_roll < 9 {
                        spawn_world_object(commands, WorldObjectType::IronVein, wx, wy, chunk_pos);
                    } else if density_roll < 11 {
                        spawn_world_object(commands, WorldObjectType::CoalDeposit, wx, wy, chunk_pos);
                    }
                }
            }
        }
    }
}

fn spawn_world_object(
    commands: &mut Commands,
    obj_type: WorldObjectType,
    x: f32,
    y: f32,
    chunk_pos: IVec2,
) {
    commands.spawn((
        WorldObject {
            object_type: obj_type,
            health: obj_type.max_health(),
        },
        ChunkObject { chunk_pos },
        Sprite {
            color: obj_type.color(),
            custom_size: Some(obj_type.size()),
            ..default()
        },
        Transform::from_xyz(x, y, 2.0),
    ));
}

fn create_chunk_image(chunk: &Chunk) -> Image {
    let size = Extent3d {
        width: CHUNK_SIZE as u32,
        height: CHUNK_SIZE as u32,
        depth_or_array_layers: 1,
    };

    let mut data = vec![0u8; CHUNK_SIZE * CHUNK_SIZE * 4];

    for y in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let tile = chunk.get_tile(x, y);
            let color = tile.color();
            // Image y=0 is top, world y=0 is bottom, so flip
            let img_y = CHUNK_SIZE - 1 - y;
            let index = (img_y * CHUNK_SIZE + x) * 4;
            data[index] = color[0];
            data[index + 1] = color[1];
            data[index + 2] = color[2];
            data[index + 3] = color[3];
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
                spawn_chunk(&mut commands, &mut images, &mut world_state, &mut dungeon_registry, chunk_pos);
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
