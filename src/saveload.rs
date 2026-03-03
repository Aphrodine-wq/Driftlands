use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;

use crate::player::{Player, Health, Hunger, ArmorSlots};
use crate::inventory::{Inventory, InventorySlot, ItemType};
use crate::world::chunk::{Chunk, CHUNK_SIZE};
use crate::world::tile::TileType;
use crate::world::WorldState;
use crate::world::TILE_SIZE;
use crate::daynight::DayNightCycle;
use crate::building::{Building, BuildingType, ChestStorage, CraftingStation, Door, Roof};
use crate::techtree::TechTree;
use crate::lore::LoreRegistry;
use crate::death::SpawnPoint;
use crate::minimap::ExploredChunks;
use crate::farming::{FarmPlot, CropType};
use crate::tutorial::TutorialState;

pub struct SaveLoadPlugin;

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SaveMessage::default())
            .insert_resource(LoadRequested::default())
            .add_systems(Update, (handle_save_input, handle_load_input, update_save_message));
    }
}

const SAVE_PATH: &str = "saves/world.bin";

#[derive(Resource, Default)]
pub struct SaveMessage {
    pub text: String,
    pub timer: f32,
}

/// Set `requested = true` to trigger a load on the next frame (used by the main menu).
#[derive(Resource, Default)]
pub struct LoadRequested {
    pub requested: bool,
}

// --- Serializable State ---

#[derive(Serialize, Deserialize)]
struct SaveData {
    player_pos: [f32; 3],
    health: f32,
    max_health: f32,
    hunger: f32,
    max_hunger: f32,
    inventory_slots: Vec<Option<SaveSlot>>,
    chunks: Vec<SaveChunk>,
    buildings: Vec<SaveBuilding>,
    day_time: f32,
    day_count: u32,
    // US-006: World seed
    #[serde(default)]
    seed: u32,
    // US-007: Tech tree
    #[serde(default)]
    tech_tree_unlocks: Vec<String>,
    #[serde(default)]
    research_points: u32,
    // US-007: Armor slots (stored as Debug string of ItemType)
    #[serde(default)]
    armor_helmet: Option<String>,
    #[serde(default)]
    armor_chest: Option<String>,
    #[serde(default)]
    armor_shield: Option<String>,
    // US-007: Lore entries (stored as the collected entry strings)
    #[serde(default)]
    lore_entries: Vec<String>,
    // US-007: Spawn point
    #[serde(default = "default_spawn_point")]
    spawn_point: (f32, f32),
    // US-007: Explored chunks
    #[serde(default)]
    explored_chunks: Vec<(i32, i32)>,
    // US-026: Chest contents
    #[serde(default)]
    chests: Vec<SaveChestData>,
    // US-026: Farm plots
    #[serde(default)]
    farms: Vec<SaveFarmData>,
    // US-031: Tutorial hints shown
    #[serde(default)]
    tutorial_shown_hints: Vec<String>,
}

fn default_spawn_point() -> (f32, f32) {
    use crate::world::TILE_SIZE;
    (TILE_SIZE * 16.0, TILE_SIZE * 16.0)
}

#[derive(Serialize, Deserialize)]
struct SaveSlot {
    item: ItemType,
    count: u32,
    durability: Option<u32>,
}

#[derive(Serialize, Deserialize)]
struct SaveChunk {
    pos_x: i32,
    pos_y: i32,
    tiles: Vec<Vec<TileType>>,
}

#[derive(Serialize, Deserialize)]
struct SaveBuilding {
    building_type: SaveBuildingType,
    pos: [f32; 3],
    is_door_open: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone)]
struct SaveChestData {
    x: f32,
    y: f32,
    slots: Vec<Option<SaveItemSlot>>,
}

#[derive(Serialize, Deserialize, Clone)]
struct SaveItemSlot {
    item: ItemType,
    count: u32,
    durability: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone)]
struct SaveFarmData {
    x: f32,
    y: f32,
    crop_name: String,
    growth: f32,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
enum SaveBuildingType {
    WoodFloor,
    WoodWall,
    WoodDoor,
    WoodRoof,
    WoodFence,
    StoneFloor,
    StoneWall,
    StoneDoor,
    StoneRoof,
    MetalWall,
    MetalDoor,
    Bed,
    Chest,
    Workbench,
    Forge,
    Campfire,
    AdvancedForge,
    AncientWorkstation,
}

impl From<BuildingType> for SaveBuildingType {
    fn from(bt: BuildingType) -> Self {
        match bt {
            BuildingType::WoodFloor => SaveBuildingType::WoodFloor,
            BuildingType::WoodWall => SaveBuildingType::WoodWall,
            BuildingType::WoodDoor => SaveBuildingType::WoodDoor,
            BuildingType::WoodRoof => SaveBuildingType::WoodRoof,
            BuildingType::WoodFence => SaveBuildingType::WoodFence,
            BuildingType::StoneFloor => SaveBuildingType::StoneFloor,
            BuildingType::StoneWall => SaveBuildingType::StoneWall,
            BuildingType::StoneDoor => SaveBuildingType::StoneDoor,
            BuildingType::StoneRoof => SaveBuildingType::StoneRoof,
            BuildingType::MetalWall => SaveBuildingType::MetalWall,
            BuildingType::MetalDoor => SaveBuildingType::MetalDoor,
            BuildingType::Bed => SaveBuildingType::Bed,
            BuildingType::Chest => SaveBuildingType::Chest,
            BuildingType::Workbench => SaveBuildingType::Workbench,
            BuildingType::Forge => SaveBuildingType::Forge,
            BuildingType::Campfire => SaveBuildingType::Campfire,
            BuildingType::AdvancedForge => SaveBuildingType::AdvancedForge,
            BuildingType::AncientWorkstation => SaveBuildingType::AncientWorkstation,
        }
    }
}

impl From<SaveBuildingType> for BuildingType {
    fn from(sbt: SaveBuildingType) -> Self {
        match sbt {
            SaveBuildingType::WoodFloor => BuildingType::WoodFloor,
            SaveBuildingType::WoodWall => BuildingType::WoodWall,
            SaveBuildingType::WoodDoor => BuildingType::WoodDoor,
            SaveBuildingType::WoodRoof => BuildingType::WoodRoof,
            SaveBuildingType::WoodFence => BuildingType::WoodFence,
            SaveBuildingType::StoneFloor => BuildingType::StoneFloor,
            SaveBuildingType::StoneWall => BuildingType::StoneWall,
            SaveBuildingType::StoneDoor => BuildingType::StoneDoor,
            SaveBuildingType::StoneRoof => BuildingType::StoneRoof,
            SaveBuildingType::MetalWall => BuildingType::MetalWall,
            SaveBuildingType::MetalDoor => BuildingType::MetalDoor,
            SaveBuildingType::Bed => BuildingType::Bed,
            SaveBuildingType::Chest => BuildingType::Chest,
            SaveBuildingType::Workbench => BuildingType::Workbench,
            SaveBuildingType::Forge => BuildingType::Forge,
            SaveBuildingType::Campfire => BuildingType::Campfire,
            SaveBuildingType::AdvancedForge => BuildingType::AdvancedForge,
            SaveBuildingType::AncientWorkstation => BuildingType::AncientWorkstation,
        }
    }
}

fn handle_save_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<(&Transform, &Health, &Hunger), With<Player>>,
    inventory: Res<Inventory>,
    chunk_query: Query<&Chunk>,
    building_query: Query<(&Building, &Transform, Option<&Door>), Without<Player>>,
    cycle: Res<DayNightCycle>,
    mut save_msg: ResMut<SaveMessage>,
    world_state: Res<WorldState>,
    tech_tree: Res<TechTree>,
    armor: Res<ArmorSlots>,
    lore_registry: Res<LoreRegistry>,
    spawn_point: Res<SpawnPoint>,
    explored: Res<ExploredChunks>,
    chest_query: Query<(&ChestStorage, &Transform), Without<Player>>,
    farm_query: Query<(&FarmPlot, &Transform), Without<Player>>,
    tutorial_state: Res<TutorialState>,
) {
    if !keyboard.just_pressed(KeyCode::F5) {
        return;
    }

    let Ok((player_tf, health, hunger)) = player_query.get_single() else { return };

    let save_data = SaveData {
        player_pos: [player_tf.translation.x, player_tf.translation.y, player_tf.translation.z],
        health: health.current,
        max_health: health.max,
        hunger: hunger.current,
        max_hunger: hunger.max,
        inventory_slots: inventory.slots.iter().map(|s| {
            s.as_ref().map(|slot| SaveSlot { item: slot.item, count: slot.count, durability: slot.durability })
        }).collect(),
        chunks: chunk_query.iter().map(|chunk| {
            let mut tiles = Vec::new();
            for y in 0..CHUNK_SIZE {
                let mut row = Vec::new();
                for x in 0..CHUNK_SIZE {
                    row.push(chunk.get_tile(x, y));
                }
                tiles.push(row);
            }
            SaveChunk { pos_x: chunk.position.x, pos_y: chunk.position.y, tiles }
        }).collect(),
        buildings: building_query.iter().map(|(building, tf, door)| {
            SaveBuilding {
                building_type: building.building_type.into(),
                pos: [tf.translation.x, tf.translation.y, tf.translation.z],
                is_door_open: door.map(|d| d.is_open),
            }
        }).collect(),
        day_time: cycle.time_of_day,
        day_count: cycle.day_count,
        // US-006: World seed
        seed: world_state.seed,
        // US-007: Tech tree
        tech_tree_unlocks: tech_tree.unlocked_recipes.iter().cloned().collect(),
        research_points: tech_tree.research_points,
        // US-007: Armor slots
        armor_helmet: armor.helmet.map(|i| format!("{:?}", i)),
        armor_chest: armor.chest.map(|i| format!("{:?}", i)),
        armor_shield: armor.shield.map(|i| format!("{:?}", i)),
        // US-007: Lore entries
        lore_entries: lore_registry.collected_entries.clone(),
        // US-007: Spawn point
        spawn_point: (spawn_point.position.x, spawn_point.position.y),
        // US-007: Explored chunks
        explored_chunks: explored.chunks.iter().map(|v| (v.x, v.y)).collect(),
        // US-026: Chest contents
        chests: chest_query.iter().map(|(chest, tf)| {
            SaveChestData {
                x: tf.translation.x,
                y: tf.translation.y,
                slots: chest.slots.iter().map(|s| {
                    s.as_ref().map(|slot| SaveItemSlot {
                        item: slot.item,
                        count: slot.count,
                        durability: slot.durability,
                    })
                }).collect(),
            }
        }).collect(),
        // US-031: Tutorial hints
        tutorial_shown_hints: tutorial_state.shown_hints.iter().cloned().collect(),
        // US-026: Farm plots
        farms: farm_query.iter().map(|(plot, tf)| {
            SaveFarmData {
                x: tf.translation.x,
                y: tf.translation.y,
                crop_name: match plot.crop {
                    Some(CropType::Wheat) => "Wheat".to_string(),
                    Some(CropType::Carrot) => "Carrot".to_string(),
                    None => "None".to_string(),
                },
                growth: plot.growth,
            }
        }).collect(),
    };

    // Create directory
    if let Err(e) = fs::create_dir_all("saves") {
        save_msg.text = format!("Save failed: {}", e);
        save_msg.timer = 2.0;
        return;
    }

    match bincode::serialize(&save_data) {
        Ok(bytes) => {
            if let Err(e) = fs::write(SAVE_PATH, bytes) {
                save_msg.text = format!("Save failed: {}", e);
            } else {
                save_msg.text = "Game Saved!".to_string();
            }
        }
        Err(e) => {
            save_msg.text = format!("Save failed: {}", e);
        }
    }
    save_msg.timer = 2.0;
}

/// Parse an ItemType from its Debug string representation (e.g. "IronHelmet" -> ItemType::IronHelmet).
/// Used to restore armor slots from save data.
fn parse_armor_item(s: &str) -> Option<ItemType> {
    match s {
        "IronHelmet" => Some(ItemType::IronHelmet),
        "IronChestplate" => Some(ItemType::IronChestplate),
        "SteelArmor" => Some(ItemType::SteelArmor),
        "AncientArmor" => Some(ItemType::AncientArmor),
        "WoodShield" => Some(ItemType::WoodShield),
        "IronShield" => Some(ItemType::IronShield),
        _ => {
            warn!("Unknown armor item in save: '{}'", s);
            None
        }
    }
}

fn handle_load_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Transform, &mut Health, &mut Hunger), With<Player>>,
    mut inventory: ResMut<Inventory>,
    mut cycle: ResMut<DayNightCycle>,
    mut commands: Commands,
    building_entities: Query<Entity, With<Building>>,
    farm_entities: Query<Entity, With<FarmPlot>>,
    mut save_msg: ResMut<SaveMessage>,
    mut world_state: ResMut<WorldState>,
    mut tech_tree: ResMut<TechTree>,
    mut armor: ResMut<ArmorSlots>,
    mut lore_registry: ResMut<LoreRegistry>,
    mut spawn_point: ResMut<SpawnPoint>,
    mut explored: ResMut<ExploredChunks>,
    mut load_requested: ResMut<LoadRequested>,
    mut tutorial_state: ResMut<TutorialState>,
) {
    // Trigger on F9 key or programmatic request (e.g. from main menu)
    if load_requested.requested {
        load_requested.requested = false;
    } else if !keyboard.just_pressed(KeyCode::F9) {
        return;
    }

    if !Path::new(SAVE_PATH).exists() {
        save_msg.text = "No save found".to_string();
        save_msg.timer = 2.0;
        return;
    }

    let bytes = match fs::read(SAVE_PATH) {
        Ok(b) => b,
        Err(e) => {
            save_msg.text = format!("Load failed: {}", e);
            save_msg.timer = 2.0;
            return;
        }
    };

    let save_data: SaveData = match bincode::deserialize(&bytes) {
        Ok(d) => d,
        Err(e) => {
            save_msg.text = format!("Load failed: {}", e);
            save_msg.timer = 2.0;
            return;
        }
    };

    // Restore player
    if let Ok((mut tf, mut health, mut hunger)) = player_query.get_single_mut() {
        tf.translation = Vec3::new(save_data.player_pos[0], save_data.player_pos[1], save_data.player_pos[2]);
        health.current = save_data.health;
        health.max = save_data.max_health;
        hunger.current = save_data.hunger;
        hunger.max = save_data.max_hunger;
        hunger.starvation_timer = 0.0;
    }

    // Restore inventory
    inventory.slots = save_data.inventory_slots.iter().map(|s| {
        s.as_ref().map(|slot| InventorySlot { item: slot.item, count: slot.count, durability: slot.durability })
    }).collect();
    // Pad if needed
    while inventory.slots.len() < 36 {
        inventory.slots.push(None);
    }

    // Remove existing buildings
    for entity in building_entities.iter() {
        commands.entity(entity).despawn();
    }

    // Restore buildings
    for sb in &save_data.buildings {
        let bt: BuildingType = sb.building_type.into();
        let mut entity_commands = commands.spawn((
            Building { building_type: bt },
            Sprite {
                color: bt.color(),
                custom_size: Some(bt.size()),
                ..default()
            },
            Transform::from_xyz(sb.pos[0], sb.pos[1], sb.pos[2]),
        ));

        if matches!(bt, BuildingType::WoodDoor | BuildingType::StoneDoor | BuildingType::MetalDoor) {
            entity_commands.insert(Door { is_open: sb.is_door_open.unwrap_or(false) });
        }
        if matches!(bt, BuildingType::WoodRoof | BuildingType::StoneRoof) {
            entity_commands.insert(Roof);
        }
        if let Some(tier) = bt.crafting_tier() {
            entity_commands.insert(CraftingStation { tier });
        }
        if matches!(bt, BuildingType::Chest) {
            // US-026: Restore chest contents by matching position
            let mut chest = ChestStorage::new();
            for sc in &save_data.chests {
                if (sc.x - sb.pos[0]).abs() < 1.0 && (sc.y - sb.pos[1]).abs() < 1.0 {
                    chest.slots = sc.slots.iter().map(|s| {
                        s.as_ref().map(|slot| InventorySlot {
                            item: slot.item,
                            count: slot.count,
                            durability: slot.durability,
                        })
                    }).collect();
                    // Pad to 18 slots if needed
                    while chest.slots.len() < 18 {
                        chest.slots.push(None);
                    }
                    break;
                }
            }
            entity_commands.insert(chest);
        }
    }

    // US-026: Remove existing farm plots
    for entity in farm_entities.iter() {
        commands.entity(entity).despawn();
    }

    // US-026: Restore farm plots
    for sf in &save_data.farms {
        let crop = match sf.crop_name.as_str() {
            "Wheat" => Some(CropType::Wheat),
            "Carrot" => Some(CropType::Carrot),
            _ => None,
        };
        let plot = FarmPlot { crop, growth: sf.growth };
        let color = match crop {
            Some(ct) => {
                if sf.growth >= 1.0 {
                    ct.mature_color()
                } else {
                    ct.growing_color()
                }
            }
            None => Color::srgb(0.45, 0.28, 0.12),
        };
        commands.spawn((
            plot,
            Sprite {
                color,
                custom_size: Some(Vec2::new(TILE_SIZE - 2.0, TILE_SIZE - 2.0)),
                ..default()
            },
            Transform::from_xyz(sf.x, sf.y, 1.5),
        ));
    }

    // Restore day/night
    cycle.time_of_day = save_data.day_time;
    cycle.day_count = save_data.day_count;

    // US-006: Restore world seed so regenerated chunks match the saved world.
    // The seed must be set BEFORE any chunk regeneration occurs (managed by
    // WorldState/manage_chunks on subsequent frames).
    if save_data.seed != 0 {
        world_state.seed = save_data.seed;
        world_state.generator = crate::world::generation::WorldGenerator::new(save_data.seed);
        // Clear loaded_chunks so chunks will be regenerated with the restored seed
        world_state.loaded_chunks.clear();
    }

    // US-007: Restore tech tree
    tech_tree.unlocked_recipes = save_data.tech_tree_unlocks.into_iter().collect();
    tech_tree.research_points = save_data.research_points;

    // US-007: Restore armor slots
    armor.helmet = save_data.armor_helmet.as_deref().and_then(parse_armor_item);
    armor.chest = save_data.armor_chest.as_deref().and_then(parse_armor_item);
    armor.shield = save_data.armor_shield.as_deref().and_then(parse_armor_item);

    // US-007: Restore lore entries
    lore_registry.collected_entries = save_data.lore_entries;

    // US-007: Restore spawn point
    spawn_point.position = Vec3::new(save_data.spawn_point.0, save_data.spawn_point.1, 10.0);

    // US-007: Restore explored chunks
    explored.chunks = save_data.explored_chunks.iter().map(|&(x, y)| IVec2::new(x, y)).collect();

    // US-031: Restore tutorial state
    tutorial_state.shown_hints = save_data.tutorial_shown_hints.into_iter().collect();
    tutorial_state.spawn_hint_queued = true; // Don't re-show spawn hint
    tutorial_state.seen_pickup = tutorial_state.shown_hints.contains("first_gather");
    tutorial_state.seen_craft = tutorial_state.shown_hints.contains("first_craft");
    tutorial_state.seen_build = tutorial_state.shown_hints.contains("first_nightfall")
        || tutorial_state.shown_hints.contains("first_craft");

    save_msg.text = "Game Loaded!".to_string();
    save_msg.timer = 2.0;
}

fn update_save_message(
    time: Res<Time>,
    mut save_msg: ResMut<SaveMessage>,
) {
    if save_msg.timer > 0.0 {
        save_msg.timer -= time.delta_secs();
        if save_msg.timer <= 0.0 {
            save_msg.text.clear();
        }
    }
}
