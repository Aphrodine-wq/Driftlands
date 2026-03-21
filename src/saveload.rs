use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::player::{Player, Health, Hunger, ArmorSlots, CurrentFloor};
use crate::inventory::{Inventory, InventorySlot, ItemType};
use crate::world::chunk::{Chunk, CHUNK_SIZE};
use crate::world::tile::TileType;
use crate::world::{LoadedChunkCache, WorldState};
use crate::world::TILE_SIZE;
use crate::daynight::DayNightCycle;
use crate::building::{Building, BuildingType, ChestStorage, CraftingStation, Door, FloorLayer, Roof, StairsOrLadder};
use crate::techtree::TechTree;
use crate::lore::LoreRegistry;
use crate::death::SpawnPoint;
use crate::minimap::ExploredChunks;
use crate::farming::{FarmPlot, CropType};
use crate::tutorial::TutorialState;
use crate::quests::QuestLog;
use crate::pets::{Pet, PetData, PetType, PetSystem};
use crate::skills::SkillLevels;

pub struct SaveLoadPlugin;

/// Which save slot the player is currently “targeting” for quick-save/quick-load and menu actions.
#[derive(Resource, Debug, Clone, Copy)]
pub struct ActiveSaveSlot {
    pub index: usize,
}

impl Default for ActiveSaveSlot {
    fn default() -> Self {
        Self { index: 0 }
    }
}

/// Lightweight metadata used by the save-slot browser UI.
#[derive(Clone, Debug)]
pub struct SaveSlotMeta {
    pub slot_index: usize,
    pub exists: bool,
    pub size_bytes: Option<u64>,
    pub modified_unix: Option<u64>,
}

const SAVE_DIR: &str = "saves";
pub const MAX_SAVE_SLOTS: usize = 10;

fn legacy_save_path() -> PathBuf {
    PathBuf::from(SAVE_DIR).join("world.bin")
}

fn slot_save_path(slot_index: usize) -> PathBuf {
    PathBuf::from(SAVE_DIR).join(format!("slot_{slot_index}.bin"))
}

/// When Some, the load application system will apply it (avoids system param limit).
#[derive(Resource, Default)]
struct PendingLoad(Option<SaveData>);

/// Holds expansion save data collected by pre_save_patch, read by handle_save_input.
/// Also bundles the background save channel to keep handle_save_input at 16 params.
#[derive(Resource)]
struct SavePatchData {
    pub quest_progress: Vec<(String, u32, bool, bool)>,
    pub active_pet: Option<PetData>,
    pub tutorial_shown_hints: Vec<String>,
    pub skill_levels: Vec<(String, u32, u32)>,
    /// Channel for receiving background save thread results.
    /// Wrapped in Mutex to satisfy Sync requirement for Bevy Resource.
    pub save_rx: Mutex<Option<mpsc::Receiver<Result<(), String>>>>,
}

impl Default for SavePatchData {
    fn default() -> Self {
        Self {
            quest_progress: Vec::new(),
            active_pet: None,
            tutorial_shown_hints: Vec::new(),
            skill_levels: Vec::new(),
            save_rx: Mutex::new(None),
        }
    }
}

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SaveMessage::default())
            .insert_resource(LoadRequested::default())
            .insert_resource(PendingLoad::default())
            .insert_resource(SavePatchData::default())
            .insert_resource(SaveTrigger::default())
            .insert_resource(AutoSaveTimer::default())
            .insert_resource(ActiveSaveSlot::default())
            .add_systems(Startup, migrate_legacy_single_slot_save)
            .add_systems(
                Update,
                (
                    pre_save_patch,
                    check_manual_save,
                    autosave_timer,
                    handle_save_input.after(check_manual_save).after(autosave_timer),
                    handle_load_input,
                    apply_pending_load_1.after(handle_load_input),
                    apply_pending_load_2.after(apply_pending_load_1),
                    apply_pending_load_3.after(apply_pending_load_2),
                    apply_pending_load_4.after(apply_pending_load_3),
                ),
            )
            .add_systems(
                Update,
                (
                    check_save_complete,
                    apply_pending_load_5.after(apply_pending_load_4),
                    update_save_message,
                ),
            );
    }
}

#[derive(Resource, Default)]
pub struct SaveMessage {
    pub text: String,
    pub timer: f32,
}

/// Set `requested = true` to trigger a load on the next frame (used by the main menu).
#[derive(Resource, Default)]
pub struct LoadRequested {
    pub requested: bool,
    pub slot_index: usize,
}

/// Intermediate flag so both manual F5 and autosave can trigger the save system
/// without adding extra params to handle_save_input (which is at the 16-param ceiling).
#[derive(Resource, Default)]
pub struct SaveTrigger {
    pub requested: bool,
    pub slot_index: usize,
}

/// Counts down and triggers an autosave periodically.
#[derive(Resource)]
pub struct AutoSaveTimer {
    pub timer: f32,
    pub interval: f32,
}

impl Default for AutoSaveTimer {
    fn default() -> Self {
        Self { timer: 300.0, interval: 300.0 } // 5 minutes
    }
}

/// Returns per-slot metadata for UI.
pub fn list_save_slots(max_slots: usize) -> Vec<SaveSlotMeta> {
    (0..max_slots).map(|slot_index| slot_meta(slot_index)).collect()
}

pub fn slot_meta(slot_index: usize) -> SaveSlotMeta {
    let path = slot_save_path(slot_index);
    if let Ok(meta) = fs::metadata(&path) {
        let size_bytes = Some(meta.len());
        let modified_unix = meta.modified().ok().and_then(to_unix_seconds);
        SaveSlotMeta {
            slot_index,
            exists: true,
            size_bytes,
            modified_unix,
        }
    } else {
        SaveSlotMeta {
            slot_index,
            exists: false,
            size_bytes: None,
            modified_unix: None,
        }
    }
}

fn to_unix_seconds(t: SystemTime) -> Option<u64> {
    t.duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs())
}

pub fn delete_save_slot(slot_index: usize) -> Result<(), String> {
    let path = slot_save_path(slot_index);
    if !path.exists() {
        return Ok(());
    }
    fs::remove_file(&path).map_err(|e| format!("Failed to delete slot {slot_index}: {e}"))?;
    Ok(())
}

fn migrate_legacy_single_slot_save() {
    // First launch / upgrade path:
    // - If `saves/world.bin` exists but `saves/slot_0.bin` doesn't, import it.
    // - This keeps existing saves working without requiring users to do anything manually.
    if !fs::metadata(SAVE_DIR).is_ok() {
        let _ = fs::create_dir_all(SAVE_DIR);
    }

    let legacy = legacy_save_path();
    let slot0 = slot_save_path(0);
    if legacy.exists() && !slot0.exists() {
        if let Err(e) = fs::copy(&legacy, &slot0) {
            warn!("Save migration failed: {e}");
        } else {
            info!("Imported legacy save into slot 0");
        }
    }
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
    // Player floor (verticality)
    #[serde(default)]
    current_floor: u8,
    // Expansion: Pet data
    #[serde(default)]
    active_pet: Option<crate::pets::PetData>,
    // Expansion: Quest progress (quest_id, progress, completed, claimed)
    #[serde(default)]
    quest_progress: Vec<(String, u32, bool, bool)>,
    // Expansion: Skill levels (skill_key, level, xp)
    #[serde(default)]
    skill_levels: Vec<(String, u32, u32)>,
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
    #[serde(default)]
    floor_layer: u8,
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
    WoodStairs,
    StoneStairs,
    Ladder,
    WoodHalfWall,
    WoodWallWindow,
    BrickWall,
    ReinforcedStoneWall,
    EnchantingTable,
    FishSmoker,
    PetHouse,
    DisplayCase,
    // Wave 6 — New Furniture
    Lantern,
    Bookshelf,
    WeaponRack,
    CookingPot,
    RainCollector,
    TrophyMount,
    // Wave 6 — Automation
    AutoSmelter,
    CropSprinkler,
    AlarmBell,
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
            BuildingType::WoodStairs => SaveBuildingType::WoodStairs,
            BuildingType::StoneStairs => SaveBuildingType::StoneStairs,
            BuildingType::Ladder => SaveBuildingType::Ladder,
            BuildingType::WoodHalfWall => SaveBuildingType::WoodHalfWall,
            BuildingType::WoodWallWindow => SaveBuildingType::WoodWallWindow,
            BuildingType::BrickWall => SaveBuildingType::BrickWall,
            BuildingType::ReinforcedStoneWall => SaveBuildingType::ReinforcedStoneWall,
            BuildingType::EnchantingTable => SaveBuildingType::EnchantingTable,
            BuildingType::FishSmoker => SaveBuildingType::FishSmoker,
            BuildingType::PetHouse => SaveBuildingType::PetHouse,
            BuildingType::DisplayCase => SaveBuildingType::DisplayCase,
            BuildingType::Lantern => SaveBuildingType::Lantern,
            BuildingType::Bookshelf => SaveBuildingType::Bookshelf,
            BuildingType::WeaponRack => SaveBuildingType::WeaponRack,
            BuildingType::CookingPot => SaveBuildingType::CookingPot,
            BuildingType::RainCollector => SaveBuildingType::RainCollector,
            BuildingType::TrophyMount => SaveBuildingType::TrophyMount,
            BuildingType::AutoSmelter => SaveBuildingType::AutoSmelter,
            BuildingType::CropSprinkler => SaveBuildingType::CropSprinkler,
            BuildingType::AlarmBell => SaveBuildingType::AlarmBell,
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
            SaveBuildingType::WoodStairs => BuildingType::WoodStairs,
            SaveBuildingType::StoneStairs => BuildingType::StoneStairs,
            SaveBuildingType::Ladder => BuildingType::Ladder,
            SaveBuildingType::WoodHalfWall => BuildingType::WoodHalfWall,
            SaveBuildingType::WoodWallWindow => BuildingType::WoodWallWindow,
            SaveBuildingType::BrickWall => BuildingType::BrickWall,
            SaveBuildingType::ReinforcedStoneWall => BuildingType::ReinforcedStoneWall,
            SaveBuildingType::EnchantingTable => BuildingType::EnchantingTable,
            SaveBuildingType::FishSmoker => BuildingType::FishSmoker,
            SaveBuildingType::PetHouse => BuildingType::PetHouse,
            SaveBuildingType::DisplayCase => BuildingType::DisplayCase,
            SaveBuildingType::Lantern => BuildingType::Lantern,
            SaveBuildingType::Bookshelf => BuildingType::Bookshelf,
            SaveBuildingType::WeaponRack => BuildingType::WeaponRack,
            SaveBuildingType::CookingPot => BuildingType::CookingPot,
            SaveBuildingType::RainCollector => BuildingType::RainCollector,
            SaveBuildingType::TrophyMount => BuildingType::TrophyMount,
            SaveBuildingType::AutoSmelter => BuildingType::AutoSmelter,
            SaveBuildingType::CropSprinkler => BuildingType::CropSprinkler,
            SaveBuildingType::AlarmBell => BuildingType::AlarmBell,
        }
    }
}

/// Collects quest, pet, tutorial, and skill data into a resource before the save system runs.
fn pre_save_patch(
    quest_log: Res<QuestLog>,
    pet_query: Query<&Pet>,
    tutorial_state: Res<TutorialState>,
    skill_levels: Res<SkillLevels>,
    mut patch: ResMut<SavePatchData>,
) {
    patch.quest_progress = quest_log.to_save_data();
    patch.active_pet = pet_query.get_single().ok().map(|pet| PetData {
        pet_type_name: pet.pet_type.display_name().to_string(),
        happiness: pet.happiness,
    });
    patch.tutorial_shown_hints = tutorial_state.shown_hints.iter().cloned().collect();
    patch.skill_levels = skill_levels.to_save_data();
}

/// Small system that detects F5 and sets the SaveTrigger flag.
fn check_manual_save(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut trigger: ResMut<SaveTrigger>,
    active_slot: Res<ActiveSaveSlot>,
) {
    if keyboard.just_pressed(KeyCode::F5) {
        trigger.requested = true;
        trigger.slot_index = active_slot.index;
    }
}

/// Counts down the autosave timer and sets the SaveTrigger flag when it expires.
fn autosave_timer(
    time: Res<Time>,
    pause: Res<crate::hud::PauseState>,
    menu: Res<crate::mainmenu::MainMenuActive>,
    mut timer: ResMut<AutoSaveTimer>,
    mut trigger: ResMut<SaveTrigger>,
    mut save_msg: ResMut<SaveMessage>,
    active_slot: Res<ActiveSaveSlot>,
) {
    // Don't autosave while paused or in main menu
    if pause.paused || menu.active {
        return;
    }
    timer.timer -= time.delta_secs();
    if timer.timer <= 0.0 {
        timer.timer = timer.interval;
        trigger.requested = true;
        trigger.slot_index = active_slot.index;
        save_msg.text = "Autosaving...".to_string();
        save_msg.timer = 5.0;
    }
}

fn handle_save_input(
    mut save_trigger: ResMut<SaveTrigger>,
    player_query: Query<(&Transform, &Health, &Hunger, &CurrentFloor), With<Player>>,
    inventory: Res<Inventory>,
    chunk_query: Query<&Chunk>,
    building_query: Query<(&Building, &Transform, Option<&Door>, Option<&FloorLayer>), Without<Player>>,
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
    save_patch: Res<SavePatchData>,
) {
    if !save_trigger.requested {
        return;
    }
    save_trigger.requested = false;

    let slot_index = save_trigger.slot_index;
    if slot_index >= MAX_SAVE_SLOTS {
        warn!("Ignoring save request for invalid slot index {slot_index}");
        return;
    }

    let Ok((player_tf, health, hunger, current_floor)) = player_query.get_single() else { return };

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
        buildings: building_query.iter().map(|(building, tf, door, floor)| {
            SaveBuilding {
                building_type: building.building_type.into(),
                pos: [tf.translation.x, tf.translation.y, tf.translation.z],
                is_door_open: door.map(|d| d.is_open),
                floor_layer: floor.map(|f| f.0).unwrap_or(0),
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
        // US-031: Tutorial hints (via save patch)
        tutorial_shown_hints: save_patch.tutorial_shown_hints.clone(),
        // US-026: Farm plots
        farms: farm_query.iter().map(|(plot, tf)| {
            SaveFarmData {
                x: tf.translation.x,
                y: tf.translation.y,
                crop_name: match plot.crop {
                    Some(CropType::Wheat) => "Wheat".to_string(),
                    Some(CropType::Carrot) => "Carrot".to_string(),
                    Some(CropType::Tomato) => "Tomato".to_string(),
                    Some(CropType::Pumpkin) => "Pumpkin".to_string(),
                    Some(CropType::Corn) => "Corn".to_string(),
                    Some(CropType::Potato) => "Potato".to_string(),
                    Some(CropType::Melon) => "Melon".to_string(),
                    Some(CropType::Rice) => "Rice".to_string(),
                    Some(CropType::Pepper) => "Pepper".to_string(),
                    Some(CropType::Onion) => "Onion".to_string(),
                    Some(CropType::Flax) => "Flax".to_string(),
                    Some(CropType::Sugarcane) => "Sugarcane".to_string(),
                    None => "None".to_string(),
                },
                growth: plot.growth,
            }
        }).collect(),
        current_floor: current_floor.0,
        active_pet: save_patch.active_pet.clone(),
        quest_progress: save_patch.quest_progress.clone(),
        skill_levels: save_patch.skill_levels.clone(),
    };

    // Prevent double-save if a background save is already in progress
    {
        let lock = save_patch.save_rx.lock().unwrap();
        if lock.is_some() {
            save_msg.text = "Save already in progress...".to_string();
            save_msg.timer = 1.0;
            return;
        }
    }

    // Create directory synchronously (fast, needed before thread)
    if let Err(e) = fs::create_dir_all(SAVE_DIR) {
        save_msg.text = format!("Save failed: {}", e);
        save_msg.timer = 2.0;
        return;
    }

    let save_path = slot_save_path(slot_index);

    // Serialize and write on a background thread to avoid stalling the main thread
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = match bincode::serialize(&save_data) {
            Ok(bytes) => {
                fs::write(&save_path, bytes)
                    .map_err(|e| format!("Save failed: {}", e))
            }
            Err(e) => Err(format!("Save failed: {}", e)),
        };
        let _ = tx.send(result);
    });

    *save_patch.save_rx.lock().unwrap() = Some(rx);
    save_msg.text = "Saving...".to_string();
    save_msg.timer = 5.0; // Will be overridden when save completes
}

/// Polls the background save thread for completion and updates the save message.
fn check_save_complete(
    save_patch: Res<SavePatchData>,
    mut save_msg: ResMut<SaveMessage>,
) {
    let mut lock = save_patch.save_rx.lock().unwrap();
    let Some(ref rx) = *lock else { return };
    match rx.try_recv() {
        Ok(Ok(())) => {
            save_msg.text = "Game Saved!".to_string();
            save_msg.timer = 2.0;
            *lock = None;
        }
        Ok(Err(err_msg)) => {
            save_msg.text = err_msg;
            save_msg.timer = 2.0;
            *lock = None;
        }
        Err(mpsc::TryRecvError::Empty) => {
            // Still saving — do nothing
        }
        Err(mpsc::TryRecvError::Disconnected) => {
            save_msg.text = "Save failed: thread crashed".to_string();
            save_msg.timer = 2.0;
            *lock = None;
        }
    }
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
    game_settings: Res<crate::settings::GameSettings>,
    active_slot: Res<ActiveSaveSlot>,
    mut save_msg: ResMut<SaveMessage>,
    mut load_requested: ResMut<LoadRequested>,
    mut pending_load: ResMut<PendingLoad>,
) {
    let slot_index = if load_requested.requested {
        load_requested.requested = false;
        load_requested.slot_index
    } else {
        if !keyboard.just_pressed(game_settings.keybinds.load) {
            return;
        }
        active_slot.index
    };

    if slot_index >= MAX_SAVE_SLOTS {
        save_msg.text = "Invalid save slot".to_string();
        save_msg.timer = 2.0;
        return;
    }

    let path = slot_save_path(slot_index);
    if !path.exists() {
        save_msg.text = format!("No save found in slot {}", slot_index + 1);
        save_msg.timer = 2.0;
        return;
    }

    let bytes = match fs::read(&path) {
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

    pending_load.0 = Some(save_data);
}

fn apply_pending_load_1(
    mut pending: ResMut<PendingLoad>,
    mut player_query: Query<(&mut Transform, &mut Health, &mut Hunger, &mut CurrentFloor), With<Player>>,
    mut inventory: ResMut<Inventory>,
    mut cycle: ResMut<DayNightCycle>,
    mut world_state: ResMut<WorldState>,
    mut loaded_chunk_cache: ResMut<LoadedChunkCache>,
    mut chunk_gen_async: ResMut<crate::world::ChunkGenAsync>,
) {
    let Some(save_data) = pending.0.take() else {
        return;
    };

    if let Ok((mut tf, mut health, mut hunger, mut current_floor)) = player_query.get_single_mut() {
        tf.translation = Vec3::new(save_data.player_pos[0], save_data.player_pos[1], save_data.player_pos[2]);
        health.current = save_data.health;
        health.max = save_data.max_health;
        hunger.current = save_data.hunger;
        hunger.max = save_data.max_hunger;
        hunger.starvation_timer = 0.0;
        current_floor.0 = save_data.current_floor;
    }

    inventory.slots = save_data.inventory_slots.iter().map(|s| {
        s.as_ref().map(|slot| InventorySlot { item: slot.item, count: slot.count, durability: slot.durability })
    }).collect();
    while inventory.slots.len() < 36 {
        inventory.slots.push(None);
    }

    cycle.time_of_day = save_data.day_time;
    cycle.day_count = save_data.day_count;

    if save_data.seed != 0 {
        world_state.seed = save_data.seed;
        world_state.generator = crate::world::generation::WorldGenerator::new(save_data.seed);
        world_state.loaded_chunks.clear();
    }

    loaded_chunk_cache.0.clear();
    for sc in &save_data.chunks {
        loaded_chunk_cache
            .0
            .insert(IVec2::new(sc.pos_x, sc.pos_y), sc.tiles.clone());
    }

    // Clear async chunk state so pending results from old seed are not spawned
    chunk_gen_async.requested.clear();
    if let Ok(mut q) = chunk_gen_async.results.lock() {
        q.clear();
    }

    pending.0 = Some(save_data);
}

fn apply_pending_load_2(
    mut pending: ResMut<PendingLoad>,
    mut commands: Commands,
    building_entities: Query<Entity, With<Building>>,
) {
    let Some(save_data) = pending.0.take() else {
        return;
    };

    for entity in building_entities.iter() {
        commands.entity(entity).despawn();
    }

    for sb in &save_data.buildings {
        let bt: BuildingType = sb.building_type.into();
        let mut entity_commands = commands.spawn((
            Building { building_type: bt },
            FloorLayer(sb.floor_layer),
            Sprite {
                color: bt.color(),
                custom_size: Some(bt.size()),
                ..default()
            },
            Transform::from_xyz(sb.pos[0], sb.pos[1], sb.pos[2]),
        ));

        if bt.is_stairs_or_ladder() {
            entity_commands.insert(StairsOrLadder);
        }
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
                    while chest.slots.len() < 18 {
                        chest.slots.push(None);
                    }
                    break;
                }
            }
            entity_commands.insert(chest);
        }
        // Wave 6 — Automation components
        if matches!(bt, BuildingType::AutoSmelter) {
            entity_commands.insert(crate::automation::AutoSmelter { timer: 0.0 });
        }
        if matches!(bt, BuildingType::CropSprinkler) {
            entity_commands.insert(crate::automation::CropSprinkler);
        }
        if matches!(bt, BuildingType::AlarmBell) {
            entity_commands.insert(crate::automation::AlarmBell { cooldown: 0.0 });
        }
    }

    pending.0 = Some(save_data);
}

fn apply_pending_load_3(
    mut pending: ResMut<PendingLoad>,
    mut commands: Commands,
    farm_entities: Query<Entity, With<FarmPlot>>,
) {
    let Some(save_data) = pending.0.take() else {
        return;
    };

    for entity in farm_entities.iter() {
        commands.entity(entity).despawn();
    }

    for sf in &save_data.farms {
        let crop = match sf.crop_name.as_str() {
            "Wheat" => Some(CropType::Wheat),
            "Carrot" => Some(CropType::Carrot),
            "Tomato" => Some(CropType::Tomato),
            "Pumpkin" => Some(CropType::Pumpkin),
            "Corn" => Some(CropType::Corn),
            "Potato" => Some(CropType::Potato),
            "Melon" => Some(CropType::Melon),
            "Rice" => Some(CropType::Rice),
            "Pepper" => Some(CropType::Pepper),
            "Onion" => Some(CropType::Onion),
            "Flax" => Some(CropType::Flax),
            "Sugarcane" => Some(CropType::Sugarcane),
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

    pending.0 = Some(save_data);
}

fn apply_pending_load_4(
    mut pending: ResMut<PendingLoad>,
    mut tech_tree: ResMut<TechTree>,
    mut armor: ResMut<ArmorSlots>,
    mut lore_registry: ResMut<LoreRegistry>,
    mut spawn_point: ResMut<SpawnPoint>,
    mut explored: ResMut<ExploredChunks>,
    mut tutorial_state: ResMut<TutorialState>,
    mut save_msg: ResMut<SaveMessage>,
) {
    let Some(save_data) = pending.0.take() else {
        return;
    };

    tech_tree.unlocked_recipes = save_data.tech_tree_unlocks.into_iter().collect();
    tech_tree.research_points = save_data.research_points;

    armor.helmet = save_data.armor_helmet.as_deref().and_then(parse_armor_item);
    armor.chest = save_data.armor_chest.as_deref().and_then(parse_armor_item);
    armor.shield = save_data.armor_shield.as_deref().and_then(parse_armor_item);

    lore_registry.collected_entries = save_data.lore_entries;

    spawn_point.position = Vec3::new(save_data.spawn_point.0, save_data.spawn_point.1, 10.0);

    explored.chunks = save_data.explored_chunks.iter().map(|&(x, y)| IVec2::new(x, y)).collect();

    tutorial_state.shown_hints = save_data.tutorial_shown_hints.into_iter().collect();
    tutorial_state.spawn_hint_queued = true;
    tutorial_state.seen_pickup = tutorial_state.shown_hints.contains("first_gather");
    tutorial_state.seen_craft = tutorial_state.shown_hints.contains("first_craft");
    tutorial_state.seen_build = tutorial_state.shown_hints.contains("first_nightfall")
        || tutorial_state.shown_hints.contains("first_craft");

    save_msg.text = "Game Loaded!".to_string();
    save_msg.timer = 2.0;
}

fn apply_pending_load_5(
    mut pending: ResMut<PendingLoad>,
    mut commands: Commands,
    mut quest_log: ResMut<QuestLog>,
    mut pet_system: ResMut<PetSystem>,
    existing_pets: Query<Entity, With<Pet>>,
    mut skill_levels: ResMut<SkillLevels>,
    assets: Res<crate::assets::GameAssets>,
) {
    let Some(save_data) = pending.0.take() else {
        return;
    };

    // Restore quest progress
    if !save_data.quest_progress.is_empty() {
        *quest_log = QuestLog::from_save_data(&save_data.quest_progress);
    }

    // Restore skill levels
    if !save_data.skill_levels.is_empty() {
        skill_levels.restore_from_save_data(&save_data.skill_levels);
    }

    // Despawn any existing pets
    for entity in existing_pets.iter() {
        commands.entity(entity).despawn_recursive();
    }
    pet_system.active_pet = false;

    // Restore pet if saved
    if let Some(ref pet_data) = save_data.active_pet {
        let pet_type = match pet_data.pet_type_name.as_str() {
            "Wolf" => Some(PetType::Wolf),
            "Cat" => Some(PetType::Cat),
            "Hawk" => Some(PetType::Hawk),
            "Bear" => Some(PetType::Bear),
            _ => None,
        };

        if let Some(pt) = pet_type {
            let pet_image = match pt {
                PetType::Wolf => assets.pet_wolf.clone(),
                PetType::Cat => assets.pet_cat.clone(),
                PetType::Hawk => assets.pet_hawk.clone(),
                PetType::Bear => assets.pet_bear.clone(),
            };
            commands.spawn((
                Pet {
                    pet_type: pt,
                    happiness: pet_data.happiness,
                    attack_cooldown: 0.0,
                },
                Sprite {
                    image: pet_image,
                    custom_size: Some(pt.size()),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 6.0), // Will snap to player via pet_follow
            ));
            pet_system.active_pet = true;
        }
    }

    // Don't consume save_data — it's already taken
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
