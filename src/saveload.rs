use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;

use crate::player::{Player, Health, Hunger};
use crate::inventory::{Inventory, InventorySlot, ItemType};
use crate::world::chunk::{Chunk, CHUNK_SIZE};
use crate::world::tile::TileType;
use crate::daynight::DayNightCycle;
use crate::building::{Building, BuildingType, Door, Roof};

pub struct SaveLoadPlugin;

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SaveMessage::default())
            .add_systems(Update, (handle_save_input, handle_load_input, update_save_message));
    }
}

const SAVE_PATH: &str = "saves/world.bin";

#[derive(Resource, Default)]
pub struct SaveMessage {
    pub text: String,
    pub timer: f32,
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

fn handle_load_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Transform, &mut Health, &mut Hunger), With<Player>>,
    mut inventory: ResMut<Inventory>,
    mut cycle: ResMut<DayNightCycle>,
    mut commands: Commands,
    building_entities: Query<Entity, With<Building>>,
    mut save_msg: ResMut<SaveMessage>,
) {
    if !keyboard.just_pressed(KeyCode::F9) {
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
    }

    // Restore day/night
    cycle.time_of_day = save_data.day_time;
    cycle.day_count = save_data.day_count;

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
