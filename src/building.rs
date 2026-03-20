use bevy::prelude::*;
use crate::player::{Player, CurrentFloor};
use crate::inventory::{Inventory, InventorySlot, ItemType};
use crate::world::{TILE_SIZE, CHUNK_WORLD_SIZE};
use crate::world::chunk::{Chunk, CHUNK_SIZE};
use crate::crafting::CraftingTier;
use crate::audio::SoundEvent;
use crate::particles::SpawnParticlesEvent;
use crate::animation::{SpriteAnimation, SpriteAnimationKind};
use crate::quests::{QuestProgressEvent, QuestType};

pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BuildingState::default())
            .insert_resource(ChestUI::default())
            .add_systems(Update, (
                toggle_build_mode,
                cycle_building_type,
                place_building,
                update_build_preview,
                door_interaction,
                roof_transparency,
                stair_ladder_use,
                building_visibility_by_floor,
                destroy_building,
                chest_interaction,
                chest_transfer,
            ));
    }
}

#[derive(Resource)]
pub struct BuildingState {
    pub active: bool,
    pub selected_type: BuildingType,
    pub placement_valid: bool,
}

impl Default for BuildingState {
    fn default() -> Self {
        Self {
            active: false,
            selected_type: BuildingType::WoodFloor,
            placement_valid: true,
        }
    }
}

#[derive(Component)]
pub struct Building {
    pub building_type: BuildingType,
}

#[derive(Component)]
pub struct Door {
    pub is_open: bool,
}

#[derive(Component)]
pub struct Roof;

/// Which floor (0 = ground, 1 = first floor) this building is on. Used for visibility and placement.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct FloorLayer(pub u8);

/// Marker for stairs and ladders; used for E-key interaction to change player floor.
#[derive(Component)]
pub struct StairsOrLadder;

#[derive(Component)]
pub struct CraftingStation {
    pub tier: CraftingTier,
}

#[derive(Component)]
pub struct BuildPreview;

#[derive(Component)]
pub struct ChestStorage {
    pub slots: Vec<Option<InventorySlot>>,
}

impl ChestStorage {
    pub fn new() -> Self {
        Self { slots: vec![None; 18] }
    }
}

#[derive(Resource, Default)]
pub struct ChestUI {
    pub is_open: bool,
    pub target_entity: Option<Entity>,
    pub selected_slot: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BuildingType {
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

impl BuildingType {
    pub fn next(self) -> Self {
        match self {
            BuildingType::WoodFloor => BuildingType::WoodWall,
            BuildingType::WoodWall => BuildingType::WoodDoor,
            BuildingType::WoodDoor => BuildingType::WoodRoof,
            BuildingType::WoodRoof => BuildingType::WoodFence,
            BuildingType::WoodFence => BuildingType::Workbench,
            BuildingType::Workbench => BuildingType::Campfire,
            BuildingType::Campfire => BuildingType::Chest,
            BuildingType::Chest => BuildingType::StoneFloor,
            BuildingType::StoneFloor => BuildingType::StoneWall,
            BuildingType::StoneWall => BuildingType::StoneDoor,
            BuildingType::StoneDoor => BuildingType::StoneRoof,
            BuildingType::StoneRoof => BuildingType::Forge,
            BuildingType::Forge => BuildingType::MetalWall,
            BuildingType::MetalWall => BuildingType::MetalDoor,
            BuildingType::MetalDoor => BuildingType::AdvancedForge,
            BuildingType::AdvancedForge => BuildingType::AncientWorkstation,
            BuildingType::AncientWorkstation => BuildingType::Bed,
            BuildingType::Bed => BuildingType::WoodStairs,
            BuildingType::WoodStairs => BuildingType::StoneStairs,
            BuildingType::StoneStairs => BuildingType::Ladder,
            BuildingType::Ladder => BuildingType::WoodHalfWall,
            BuildingType::WoodHalfWall => BuildingType::WoodWallWindow,
            BuildingType::WoodWallWindow => BuildingType::BrickWall,
            BuildingType::BrickWall => BuildingType::ReinforcedStoneWall,
            BuildingType::ReinforcedStoneWall => BuildingType::EnchantingTable,
            BuildingType::EnchantingTable => BuildingType::FishSmoker,
            BuildingType::FishSmoker => BuildingType::PetHouse,
            BuildingType::PetHouse => BuildingType::DisplayCase,
            BuildingType::DisplayCase => BuildingType::Lantern,
            BuildingType::Lantern => BuildingType::Bookshelf,
            BuildingType::Bookshelf => BuildingType::WeaponRack,
            BuildingType::WeaponRack => BuildingType::CookingPot,
            BuildingType::CookingPot => BuildingType::RainCollector,
            BuildingType::RainCollector => BuildingType::TrophyMount,
            BuildingType::TrophyMount => BuildingType::AutoSmelter,
            BuildingType::AutoSmelter => BuildingType::CropSprinkler,
            BuildingType::CropSprinkler => BuildingType::AlarmBell,
            BuildingType::AlarmBell => BuildingType::WoodFloor,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            BuildingType::WoodFloor => "Wood Floor",
            BuildingType::WoodWall => "Wood Wall",
            BuildingType::WoodDoor => "Wood Door",
            BuildingType::WoodRoof => "Wood Roof",
            BuildingType::WoodFence => "Wood Fence",
            BuildingType::StoneFloor => "Stone Floor",
            BuildingType::StoneWall => "Stone Wall",
            BuildingType::StoneDoor => "Stone Door",
            BuildingType::StoneRoof => "Stone Roof",
            BuildingType::MetalWall => "Metal Wall",
            BuildingType::MetalDoor => "Metal Door",
            BuildingType::Bed => "Bed",
            BuildingType::Chest => "Chest",
            BuildingType::Workbench => "Workbench",
            BuildingType::Forge => "Forge",
            BuildingType::Campfire => "Campfire",
            BuildingType::AdvancedForge => "Advanced Forge",
            BuildingType::AncientWorkstation => "Ancient Workstation",
            BuildingType::WoodStairs => "Wood Stairs",
            BuildingType::StoneStairs => "Stone Stairs",
            BuildingType::Ladder => "Ladder",
            BuildingType::WoodHalfWall => "Wood Half Wall",
            BuildingType::WoodWallWindow => "Wood Wall (Window)",
            BuildingType::BrickWall => "Brick Wall",
            BuildingType::ReinforcedStoneWall => "Reinforced Stone Wall",
            BuildingType::EnchantingTable => "Enchanting Table",
            BuildingType::FishSmoker => "Fish Smoker",
            BuildingType::PetHouse => "Pet House",
            BuildingType::DisplayCase => "Display Case",
            BuildingType::Lantern => "Lantern",
            BuildingType::Bookshelf => "Bookshelf",
            BuildingType::WeaponRack => "Weapon Rack",
            BuildingType::CookingPot => "Cooking Pot",
            BuildingType::RainCollector => "Rain Collector",
            BuildingType::TrophyMount => "Trophy Mount",
            BuildingType::AutoSmelter => "Auto-Smelter",
            BuildingType::CropSprinkler => "Crop Sprinkler",
            BuildingType::AlarmBell => "Alarm Bell",
        }
    }

    pub fn required_item(&self) -> ItemType {
        match self {
            BuildingType::WoodFloor => ItemType::WoodFloor,
            BuildingType::WoodWall => ItemType::WoodWall,
            BuildingType::WoodDoor => ItemType::WoodDoor,
            BuildingType::WoodRoof => ItemType::WoodRoof,
            BuildingType::WoodFence => ItemType::WoodFence,
            BuildingType::StoneFloor => ItemType::StoneFloor,
            BuildingType::StoneWall => ItemType::StoneWall,
            BuildingType::StoneDoor => ItemType::StoneDoor,
            BuildingType::StoneRoof => ItemType::StoneRoof,
            BuildingType::MetalWall => ItemType::MetalWall,
            BuildingType::MetalDoor => ItemType::MetalDoor,
            BuildingType::Bed => ItemType::Bed,
            BuildingType::Chest => ItemType::Chest,
            BuildingType::Workbench => ItemType::Workbench,
            BuildingType::Forge => ItemType::Forge,
            BuildingType::Campfire => ItemType::Campfire,
            BuildingType::AdvancedForge => ItemType::AdvancedForge,
            BuildingType::AncientWorkstation => ItemType::AncientWorkstation,
            BuildingType::WoodStairs => ItemType::WoodStairs,
            BuildingType::StoneStairs => ItemType::StoneStairs,
            BuildingType::Ladder => ItemType::Ladder,
            BuildingType::WoodHalfWall => ItemType::WoodHalfWall,
            BuildingType::WoodWallWindow => ItemType::WoodWallWindow,
            BuildingType::BrickWall => ItemType::BrickWall,
            BuildingType::ReinforcedStoneWall => ItemType::ReinforcedStoneWall,
            BuildingType::EnchantingTable => ItemType::EnchantingTable,
            BuildingType::FishSmoker => ItemType::FishSmoker,
            BuildingType::PetHouse => ItemType::PetHouse,
            BuildingType::DisplayCase => ItemType::DisplayCase,
            BuildingType::Lantern => ItemType::Lantern,
            BuildingType::Bookshelf => ItemType::Bookshelf,
            BuildingType::WeaponRack => ItemType::WeaponRack,
            BuildingType::CookingPot => ItemType::CookingPot,
            BuildingType::RainCollector => ItemType::RainCollector,
            BuildingType::TrophyMount => ItemType::TrophyMount,
            BuildingType::AutoSmelter => ItemType::AutoSmelterItem,
            BuildingType::CropSprinkler => ItemType::CropSprinklerItem,
            BuildingType::AlarmBell => ItemType::AlarmBellItem,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            BuildingType::WoodFloor => Color::srgb(0.6, 0.4, 0.2),
            BuildingType::WoodWall => Color::srgb(0.5, 0.3, 0.15),
            BuildingType::WoodDoor => Color::srgb(0.55, 0.35, 0.2),
            BuildingType::WoodRoof => Color::srgb(0.35, 0.2, 0.1),
            BuildingType::WoodFence => Color::srgb(0.5, 0.35, 0.2),
            BuildingType::StoneFloor => Color::srgb(0.55, 0.55, 0.55),
            BuildingType::StoneWall => Color::srgb(0.5, 0.5, 0.5),
            BuildingType::StoneDoor => Color::srgb(0.55, 0.52, 0.5),
            BuildingType::StoneRoof => Color::srgb(0.4, 0.4, 0.4),
            BuildingType::MetalWall => Color::srgb(0.6, 0.62, 0.65),
            BuildingType::MetalDoor => Color::srgb(0.58, 0.6, 0.63),
            BuildingType::Bed => Color::srgb(0.7, 0.3, 0.3),
            BuildingType::Chest => Color::srgb(0.55, 0.4, 0.2),
            BuildingType::Workbench => Color::srgb(0.45, 0.30, 0.15),
            BuildingType::Forge => Color::srgb(0.6, 0.2, 0.08),
            BuildingType::Campfire => Color::srgb(0.9, 0.6, 0.15),
            BuildingType::AdvancedForge => Color::srgb(0.3, 0.35, 0.45),
            BuildingType::AncientWorkstation => Color::srgb(0.45, 0.25, 0.75),
            BuildingType::WoodStairs => Color::srgb(0.5, 0.32, 0.18),
            BuildingType::StoneStairs => Color::srgb(0.52, 0.52, 0.5),
            BuildingType::Ladder => Color::srgb(0.45, 0.35, 0.2),
            BuildingType::WoodHalfWall => Color::srgb(0.48, 0.32, 0.18),
            BuildingType::WoodWallWindow => Color::srgb(0.5, 0.33, 0.18),
            BuildingType::BrickWall => Color::srgb(0.65, 0.35, 0.28),
            BuildingType::ReinforcedStoneWall => Color::srgb(0.45, 0.45, 0.48),
            BuildingType::EnchantingTable => Color::srgb(0.55, 0.3, 0.8),
            BuildingType::FishSmoker => Color::srgb(0.7, 0.5, 0.25),
            BuildingType::PetHouse => Color::srgb(0.55, 0.4, 0.2),
            BuildingType::DisplayCase => Color::srgb(0.6, 0.65, 0.7),
            BuildingType::Lantern => Color::srgb(0.95, 0.85, 0.4),
            BuildingType::Bookshelf => Color::srgb(0.5, 0.3, 0.15),
            BuildingType::WeaponRack => Color::srgb(0.55, 0.45, 0.35),
            BuildingType::CookingPot => Color::srgb(0.35, 0.35, 0.35),
            BuildingType::RainCollector => Color::srgb(0.4, 0.55, 0.7),
            BuildingType::TrophyMount => Color::srgb(0.6, 0.45, 0.25),
            BuildingType::AutoSmelter => Color::srgb(0.7, 0.35, 0.1),
            BuildingType::CropSprinkler => Color::srgb(0.3, 0.6, 0.75),
            BuildingType::AlarmBell => Color::srgb(0.8, 0.7, 0.2),
        }
    }

    pub fn size(&self) -> Vec2 {
        match self {
            BuildingType::WoodFloor => Vec2::new(TILE_SIZE, TILE_SIZE),
            BuildingType::WoodWall => Vec2::new(TILE_SIZE, 24.0),
            BuildingType::WoodDoor => Vec2::new(10.0, 20.0),
            BuildingType::WoodRoof => Vec2::new(TILE_SIZE, TILE_SIZE),
            BuildingType::WoodFence => Vec2::new(TILE_SIZE, 12.0),
            BuildingType::StoneFloor => Vec2::new(TILE_SIZE, TILE_SIZE),
            BuildingType::StoneWall => Vec2::new(TILE_SIZE, 24.0),
            BuildingType::StoneDoor => Vec2::new(10.0, 20.0),
            BuildingType::StoneRoof => Vec2::new(TILE_SIZE, TILE_SIZE),
            BuildingType::MetalWall => Vec2::new(TILE_SIZE, 24.0),
            BuildingType::MetalDoor => Vec2::new(10.0, 20.0),
            BuildingType::Bed => Vec2::new(TILE_SIZE, TILE_SIZE),
            BuildingType::Chest => Vec2::new(TILE_SIZE * 0.75, TILE_SIZE * 0.75),
            BuildingType::Campfire => Vec2::new(12.0, 12.0),
            BuildingType::Workbench | BuildingType::Forge |
            BuildingType::AdvancedForge |
            BuildingType::AncientWorkstation => Vec2::new(TILE_SIZE, TILE_SIZE),
            BuildingType::WoodStairs | BuildingType::StoneStairs => Vec2::new(TILE_SIZE, TILE_SIZE),
            BuildingType::Ladder => Vec2::new(8.0, TILE_SIZE),
            BuildingType::WoodHalfWall => Vec2::new(TILE_SIZE, 12.0),
            BuildingType::WoodWallWindow => Vec2::new(TILE_SIZE, 24.0),
            BuildingType::BrickWall | BuildingType::ReinforcedStoneWall => Vec2::new(TILE_SIZE, 24.0),
            BuildingType::EnchantingTable | BuildingType::FishSmoker |
            BuildingType::PetHouse | BuildingType::DisplayCase => Vec2::new(TILE_SIZE, TILE_SIZE),
            BuildingType::Lantern => Vec2::new(8.0, 8.0),
            BuildingType::Bookshelf => Vec2::new(TILE_SIZE, TILE_SIZE),
            BuildingType::WeaponRack => Vec2::new(TILE_SIZE, TILE_SIZE),
            BuildingType::CookingPot => Vec2::new(TILE_SIZE, TILE_SIZE),
            BuildingType::RainCollector => Vec2::new(TILE_SIZE, TILE_SIZE),
            BuildingType::TrophyMount => Vec2::new(TILE_SIZE * 0.75, TILE_SIZE * 0.75),
            BuildingType::AutoSmelter => Vec2::new(TILE_SIZE, TILE_SIZE),
            BuildingType::CropSprinkler => Vec2::new(TILE_SIZE * 0.75, TILE_SIZE * 0.75),
            BuildingType::AlarmBell => Vec2::new(TILE_SIZE * 0.75, TILE_SIZE * 0.75),
        }
    }

    pub fn z_depth(&self) -> f32 {
        match self {
            BuildingType::WoodFloor => 1.0,
            BuildingType::WoodWall => 3.0,
            BuildingType::WoodDoor => 3.0,
            BuildingType::WoodRoof => 15.0,
            BuildingType::WoodFence => 2.0,
            BuildingType::StoneFloor | BuildingType::Bed => 1.0,
            BuildingType::StoneWall | BuildingType::MetalWall => 3.0,
            BuildingType::StoneDoor | BuildingType::MetalDoor => 3.0,
            BuildingType::StoneRoof => 15.0,
            BuildingType::Chest => 2.0,
            BuildingType::Workbench | BuildingType::Forge |
            BuildingType::Campfire | BuildingType::AdvancedForge |
            BuildingType::AncientWorkstation => 2.0,
            BuildingType::WoodStairs | BuildingType::StoneStairs | BuildingType::Ladder => 2.0,
            BuildingType::WoodHalfWall | BuildingType::WoodWallWindow => 2.5,
            BuildingType::BrickWall | BuildingType::ReinforcedStoneWall => 3.0,
            BuildingType::EnchantingTable | BuildingType::FishSmoker |
            BuildingType::PetHouse | BuildingType::DisplayCase => 2.0,
            BuildingType::Lantern | BuildingType::Bookshelf | BuildingType::WeaponRack |
            BuildingType::CookingPot | BuildingType::RainCollector | BuildingType::TrophyMount |
            BuildingType::AutoSmelter | BuildingType::CropSprinkler | BuildingType::AlarmBell => 2.0,
        }
    }

    /// Returns true for stairs and ladders (vertical traversal).
    pub fn is_stairs_or_ladder(&self) -> bool {
        matches!(self, BuildingType::WoodStairs | BuildingType::StoneStairs | BuildingType::Ladder)
    }

    /// Returns materials returned when destroyed (50% of recipe, min 1)
    pub fn salvage(&self) -> Vec<(ItemType, u32)> {
        match self {
            BuildingType::WoodFloor => vec![(ItemType::WoodPlank, 2)],
            BuildingType::WoodWall => vec![(ItemType::WoodPlank, 2), (ItemType::Stick, 1)],
            BuildingType::WoodDoor => vec![(ItemType::WoodPlank, 3)],
            BuildingType::WoodRoof => vec![(ItemType::WoodPlank, 3), (ItemType::Stick, 2)],
            BuildingType::WoodFence => vec![(ItemType::Stick, 3)],
            BuildingType::StoneFloor => vec![(ItemType::StoneBlock, 2)],
            BuildingType::StoneWall => vec![(ItemType::StoneBlock, 2)],
            BuildingType::StoneDoor => vec![(ItemType::StoneBlock, 3), (ItemType::IronIngot, 1)],
            BuildingType::StoneRoof => vec![(ItemType::StoneBlock, 3)],
            BuildingType::MetalWall => vec![(ItemType::SteelAlloy, 2), (ItemType::IronIngot, 1)],
            BuildingType::MetalDoor => vec![(ItemType::SteelAlloy, 3)],
            BuildingType::Bed => vec![(ItemType::WoodPlank, 3), (ItemType::PlantFiber, 2)],
            BuildingType::Chest => vec![(ItemType::WoodPlank, 3), (ItemType::IronIngot, 1)],
            BuildingType::Workbench => vec![(ItemType::WoodPlank, 4), (ItemType::Stick, 2)],
            BuildingType::Forge => vec![(ItemType::StoneBlock, 5), (ItemType::IronOre, 2), (ItemType::Coal, 1)],
            BuildingType::Campfire => vec![(ItemType::Stone, 2), (ItemType::Stick, 1), (ItemType::Wood, 1)],
            BuildingType::AdvancedForge => vec![(ItemType::SteelAlloy, 5), (ItemType::CrystalShard, 2), (ItemType::ObsidianShard, 1)],
            BuildingType::AncientWorkstation => vec![(ItemType::AncientCore, 2), (ItemType::Gemstone, 2), (ItemType::SteelAlloy, 5)],
            BuildingType::WoodStairs => vec![(ItemType::WoodPlank, 2), (ItemType::Stick, 2)],
            BuildingType::StoneStairs => vec![(ItemType::StoneBlock, 2)],
            BuildingType::Ladder => vec![(ItemType::Stick, 3), (ItemType::Rope, 1)],
            BuildingType::WoodHalfWall => vec![(ItemType::WoodPlank, 2), (ItemType::Stick, 1)],
            BuildingType::WoodWallWindow => vec![(ItemType::WoodPlank, 4), (ItemType::Stick, 2)],
            BuildingType::BrickWall => vec![(ItemType::Brick, 4)],
            BuildingType::ReinforcedStoneWall => vec![(ItemType::ReinforcedStoneBlock, 3), (ItemType::IronIngot, 1)],
            BuildingType::EnchantingTable => vec![(ItemType::IronIngot, 4), (ItemType::CrystalShard, 2), (ItemType::Gemstone, 1)],
            BuildingType::FishSmoker => vec![(ItemType::StoneBlock, 3), (ItemType::IronIngot, 1)],
            BuildingType::PetHouse => vec![(ItemType::WoodPlank, 5), (ItemType::Rope, 1)],
            BuildingType::DisplayCase => vec![(ItemType::WoodPlank, 4), (ItemType::CrystalShard, 1)],
            BuildingType::Lantern => vec![(ItemType::IronIngot, 1), (ItemType::Torch, 1)],
            BuildingType::Bookshelf => vec![(ItemType::WoodPlank, 4)],
            BuildingType::WeaponRack => vec![(ItemType::IronIngot, 2), (ItemType::WoodPlank, 1)],
            BuildingType::CookingPot => vec![(ItemType::IronIngot, 2), (ItemType::Stone, 1)],
            BuildingType::RainCollector => vec![(ItemType::IronIngot, 2), (ItemType::WoodPlank, 1)],
            BuildingType::TrophyMount => vec![(ItemType::WoodPlank, 1), (ItemType::IronIngot, 1)],
            BuildingType::AutoSmelter => vec![(ItemType::IronIngot, 4), (ItemType::Stone, 2)],
            BuildingType::CropSprinkler => vec![(ItemType::IronIngot, 2), (ItemType::WoodPlank, 1)],
            BuildingType::AlarmBell => vec![(ItemType::IronIngot, 1), (ItemType::Gemstone, 1)],
        }
    }

    /// Returns the CraftingTier this building provides, if it is a crafting station.
    pub fn crafting_tier(&self) -> Option<CraftingTier> {
        match self {
            BuildingType::Workbench => Some(CraftingTier::Workbench),
            BuildingType::Forge => Some(CraftingTier::Forge),
            BuildingType::Campfire => Some(CraftingTier::Campfire),
            BuildingType::AdvancedForge => Some(CraftingTier::AdvancedForge),
            BuildingType::AncientWorkstation => Some(CraftingTier::Ancient),
            BuildingType::WoodStairs | BuildingType::StoneStairs | BuildingType::Ladder => None,
            BuildingType::EnchantingTable => Some(CraftingTier::Ancient),
            BuildingType::FishSmoker => Some(CraftingTier::Campfire),
            BuildingType::WoodHalfWall | BuildingType::WoodWallWindow | BuildingType::BrickWall | BuildingType::ReinforcedStoneWall => None,
            BuildingType::PetHouse | BuildingType::DisplayCase => None,
            BuildingType::CookingPot => Some(CraftingTier::Campfire),
            BuildingType::Lantern | BuildingType::Bookshelf | BuildingType::WeaponRack |
            BuildingType::RainCollector | BuildingType::TrophyMount |
            BuildingType::AutoSmelter | BuildingType::CropSprinkler | BuildingType::AlarmBell => None,
            _ => None,
        }
    }
}

/// Returns the appropriate sprite for a building type, using real PNG sprites.
pub fn building_sprite(bt: BuildingType, assets: &crate::assets::GameAssets) -> Sprite {
    let (texture, use_tint) = match bt {
        BuildingType::WoodWall => (Some(assets.wood_wall.clone()), false),
        BuildingType::WoodFence => (Some(assets.wood_fence.clone()), false),
        BuildingType::WoodFloor => (Some(assets.wood_floor.clone()), false),
        BuildingType::WoodDoor => (Some(assets.wood_door.clone()), false),
        BuildingType::WoodRoof => (Some(assets.roof_thatch.clone()), false),
        BuildingType::WoodHalfWall => (Some(assets.wood_half_wall.clone()), false),
        BuildingType::WoodWallWindow => (Some(assets.wood_wall_window.clone()), false),
        BuildingType::WoodStairs => (Some(assets.wood_stairs.clone()), false),
        BuildingType::StoneFloor => (Some(assets.stone_floor.clone()), false),
        BuildingType::StoneWall => (Some(assets.stone_wall.clone()), false),
        BuildingType::StoneDoor => (Some(assets.stone_door_building.clone()), false),
        BuildingType::StoneRoof => (Some(assets.stone_roof.clone()), false),
        BuildingType::StoneStairs => (Some(assets.stone_stairs.clone()), false),
        BuildingType::MetalWall => (Some(assets.metal_wall.clone()), false),
        BuildingType::MetalDoor => (Some(assets.metal_door.clone()), false),
        BuildingType::BrickWall => (Some(assets.brick_wall.clone()), false),
        BuildingType::ReinforcedStoneWall => (Some(assets.reinforced_wall.clone()), false),
        BuildingType::Campfire => (Some(assets.campfire.clone()), false),
        BuildingType::Workbench => (Some(assets.workbench.clone()), false),
        BuildingType::Forge => (Some(assets.forge.clone()), false),
        BuildingType::AdvancedForge => (Some(assets.advanced_forge.clone()), false),
        BuildingType::AncientWorkstation => (Some(assets.ancient_workstation.clone()), false),
        BuildingType::Chest => (Some(assets.chest_building.clone()), false),
        BuildingType::Bed => (Some(assets.bed.clone()), false),
        BuildingType::Ladder => (Some(assets.ladder.clone()), false),
        BuildingType::EnchantingTable => (Some(assets.enchanting_table.clone()), false),
        BuildingType::FishSmoker => (Some(assets.fish_smoker.clone()), false),
        BuildingType::PetHouse => (Some(assets.pet_house.clone()), false),
        BuildingType::DisplayCase => (Some(assets.display_case.clone()), false),
        BuildingType::Lantern => (Some(assets.lantern.clone()), false),
        BuildingType::Bookshelf => (Some(assets.bookshelf.clone()), false),
        BuildingType::WeaponRack => (Some(assets.weapon_rack.clone()), false),
        BuildingType::CookingPot => (Some(assets.cooking_pot.clone()), false),
        BuildingType::RainCollector => (Some(assets.rain_collector.clone()), false),
        BuildingType::TrophyMount => (Some(assets.trophy_mount.clone()), false),
        BuildingType::AutoSmelter => (Some(assets.auto_smelter.clone()), false),
        BuildingType::CropSprinkler => (Some(assets.crop_sprinkler.clone()), false),
        BuildingType::AlarmBell => (Some(assets.alarm_bell.clone()), false),
    };

    let mut sprite = Sprite {
        color: if use_tint { bt.color() } else { Color::WHITE },
        custom_size: Some(bt.size()),
        ..default()
    };
    if let Some(tex) = texture {
        sprite.image = tex;
    } else {
        sprite.color = bt.color();
    }

    // Campfire: use a runtime-built texture atlas if available.
    if matches!(bt, BuildingType::Campfire) {
        if let (Some(atlas_image), Some(atlas_layout)) = (
            assets.campfire_anim_atlas_image.as_ref(),
            assets.campfire_anim_atlas_layout.as_ref(),
        ) {
            sprite.image = atlas_image.clone();
            sprite.texture_atlas = Some(TextureAtlas { layout: atlas_layout.clone(), index: 0 });
        }
    }
    sprite
}

fn toggle_build_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut building_state: ResMut<BuildingState>,
    game_settings: Res<crate::settings::GameSettings>,
) {
    if keyboard.just_pressed(game_settings.keybinds.building) {
        building_state.active = !building_state.active;
    }
}

fn cycle_building_type(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut building_state: ResMut<BuildingState>,
) {
    if !building_state.active {
        return;
    }
    if keyboard.just_pressed(KeyCode::KeyQ) {
        building_state.selected_type = building_state.selected_type.next();
    }
}

fn update_build_preview(
    mut commands: Commands,
    mut building_state: ResMut<BuildingState>,
    player_query: Query<&Transform, With<Player>>,
    preview_query: Query<Entity, With<BuildPreview>>,
    chunk_query: Query<&Chunk>,
    building_query: Query<&Transform, (With<Building>, Without<Player>, Without<BuildPreview>)>,
) {
    for entity in preview_query.iter() {
        commands.entity(entity).despawn();
    }

    if !building_state.active {
        return;
    }

    let Ok(player_tf) = player_query.get_single() else { return };

    let snapped_x = (player_tf.translation.x / TILE_SIZE).round() * TILE_SIZE;
    let snapped_y = (player_tf.translation.y / TILE_SIZE).round() * TILE_SIZE;

    let bt = building_state.selected_type;

    // --- Placement validation ---
    let mut valid = true;

    // Check if tile at preview position is walkable
    let chunk_x = (snapped_x / CHUNK_WORLD_SIZE).floor() as i32;
    let chunk_y = (snapped_y / CHUNK_WORLD_SIZE).floor() as i32;
    let tile_x = ((snapped_x / TILE_SIZE).floor() as i32 - chunk_x * CHUNK_SIZE as i32).clamp(0, CHUNK_SIZE as i32 - 1) as usize;
    let tile_y = ((snapped_y / TILE_SIZE).floor() as i32 - chunk_y * CHUNK_SIZE as i32).clamp(0, CHUNK_SIZE as i32 - 1) as usize;

    for chunk in chunk_query.iter() {
        if chunk.position.x == chunk_x && chunk.position.y == chunk_y {
            if !chunk.get_tile(tile_x, tile_y).is_walkable() {
                valid = false;
            }
            break;
        }
    }

    // Check if any existing building overlaps the preview position (within 8px)
    if valid {
        for tf in building_query.iter() {
            let dist = Vec2::new(snapped_x, snapped_y).distance(tf.translation.truncate());
            if dist < 8.0 {
                valid = false;
                break;
            }
        }
    }

    building_state.placement_valid = valid;

    let color = if valid {
        Color::srgba(1.0, 1.0, 1.0, 0.4)
    } else {
        Color::srgba(0.8, 0.2, 0.2, 0.5)
    };

    commands.spawn((
        BuildPreview,
        Sprite {
            color,
            custom_size: Some(bt.size()),
            ..default()
        },
        Transform::from_xyz(snapped_x, snapped_y, bt.z_depth() + 0.1),
    ));
}

fn place_building(
    mut commands: Commands,
    building_state: Res<BuildingState>,
    mut inventory: ResMut<Inventory>,
    player_query: Query<(&Transform, &CurrentFloor), With<Player>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut sound_events: EventWriter<SoundEvent>,
    mut particle_events: EventWriter<SpawnParticlesEvent>,
    assets: Res<crate::assets::GameAssets>,
    mut quest_events: EventWriter<QuestProgressEvent>,
) {
    if !building_state.active || !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    let Ok((player_tf, current_floor)) = player_query.get_single() else { return };
    let bt = building_state.selected_type;

    // Invalid placement feedback: wrong spot or missing materials
    if !building_state.placement_valid {
        sound_events.send(SoundEvent::PlaceInvalid);
        return;
    }
    if !inventory.has_items(bt.required_item(), 1) {
        sound_events.send(SoundEvent::PlaceInvalid);
        return;
    }

    let snapped_x = (player_tf.translation.x / TILE_SIZE).round() * TILE_SIZE;
    let snapped_y = (player_tf.translation.y / TILE_SIZE).round() * TILE_SIZE;

    inventory.remove_items(bt.required_item(), 1);
    sound_events.send(SoundEvent::Build);
    quest_events.send(QuestProgressEvent { quest_type: QuestType::PlaceBuilding, amount: 1 });

    // Place success: dust/shimmer particles
    particle_events.send(SpawnParticlesEvent {
        position: Vec2::new(snapped_x, snapped_y),
        color: Color::srgba(0.75, 0.7, 0.6, 0.9),
        count: 6,
    });

    let mut entity_commands = commands.spawn((
        Building { building_type: bt },
        FloorLayer(current_floor.0),
        building_sprite(bt, &assets),
        Transform::from_xyz(snapped_x, snapped_y, bt.z_depth()),
    ));

    if bt.is_stairs_or_ladder() {
        entity_commands.insert(StairsOrLadder);
    }
    if matches!(bt, BuildingType::WoodDoor | BuildingType::StoneDoor | BuildingType::MetalDoor) {
        entity_commands.insert(Door { is_open: false });
    }
    if matches!(bt, BuildingType::WoodRoof | BuildingType::StoneRoof) {
        entity_commands.insert(Roof);
    }
    if let Some(tier) = bt.crafting_tier() {
        entity_commands.insert(CraftingStation { tier });
        // Glow effect: slightly larger, low-alpha bright sprite behind the station
        let glow_size = bt.size() + Vec2::new(2.0, 2.0);
        let glow_color = match bt {
            BuildingType::Workbench => Color::srgba(0.7, 0.55, 0.35, 0.25),
            BuildingType::Forge => Color::srgba(1.0, 0.4, 0.15, 0.3),
            BuildingType::Campfire => Color::srgba(1.0, 0.75, 0.3, 0.35),
            BuildingType::AdvancedForge => Color::srgba(0.5, 0.55, 0.7, 0.3),
            BuildingType::AncientWorkstation => Color::srgba(0.65, 0.4, 1.0, 0.35),
            _ => Color::srgba(1.0, 1.0, 1.0, 0.2),
        };
        entity_commands.with_children(|parent| {
            parent.spawn((
                Sprite {
                    color: glow_color,
                    custom_size: Some(glow_size),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, -0.1),
            ));
        });
    }
    // Campfire: attach looping fire animation
    if matches!(bt, BuildingType::Campfire) {
        let frames = assets.campfire_anim_frames.clone();
        if !frames.is_empty() {
            entity_commands.insert(SpriteAnimation::new(
                SpriteAnimationKind::Campfire,
                frames,
                0.12,
                true,
            ));
        }
    }
    if matches!(bt, BuildingType::Chest) {
        entity_commands.insert(ChestStorage::new());
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

fn door_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    chest_ui: Res<ChestUI>,
    player_query: Query<&Transform, With<Player>>,
    mut door_query: Query<(&Transform, &mut Door, &mut Sprite), Without<Player>>,
    game_settings: Res<crate::settings::GameSettings>,
) {
    if !keyboard.just_pressed(game_settings.keybinds.interact) {
        return;
    }

    // Don't toggle doors while chest UI is open
    if chest_ui.is_open {
        return;
    }

    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    let mut nearest: Option<(f32, Entity)> = None;
    for (tf, _, _) in door_query.iter() {
        let dist = player_pos.distance(tf.translation.truncate());
        if dist <= 32.0 {
            if nearest.is_none() || dist < nearest.unwrap().0 {
                // We can't store Entity from the query easily, so we'll just toggle all in range
                nearest = Some((dist, Entity::PLACEHOLDER));
            }
        }
    }

    if nearest.is_none() {
        return;
    }

    // Toggle nearest door
    let mut best_dist = f32::MAX;
    let mut best_idx = None;
    for (i, (tf, _, _)) in door_query.iter().enumerate() {
        let dist = player_pos.distance(tf.translation.truncate());
        if dist <= 32.0 && dist < best_dist {
            best_dist = dist;
            best_idx = Some(i);
        }
    }

    if let Some(idx) = best_idx {
        for (i, (_, mut door, mut sprite)) in door_query.iter_mut().enumerate() {
            if i == idx {
                door.is_open = !door.is_open;
                if door.is_open {
                    sprite.color = Color::srgba(0.55, 0.35, 0.2, 0.4);
                    sprite.custom_size = Some(Vec2::new(4.0, 20.0));
                } else {
                    sprite.color = Color::srgb(0.55, 0.35, 0.2);
                    sprite.custom_size = Some(Vec2::new(10.0, 20.0));
                }
                break;
            }
        }
    }
}

fn roof_transparency(
    player_query: Query<&Transform, With<Player>>,
    mut roof_query: Query<(&Transform, &mut Sprite), (With<Roof>, Without<Player>)>,
) {
    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    for (tf, mut sprite) in roof_query.iter_mut() {
        let roof_pos = tf.translation.truncate();
        let dist = player_pos.distance(roof_pos);
        if dist < TILE_SIZE * 0.8 {
            sprite.color = Color::srgba(0.35, 0.2, 0.1, 0.3);
        } else {
            sprite.color = Color::srgb(0.35, 0.2, 0.1);
        }
    }
}

fn stair_ladder_use(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(Entity, &Transform, &mut CurrentFloor), With<Player>>,
    stair_query: Query<(&Transform, &StairsOrLadder), Without<Player>>,
    game_settings: Res<crate::settings::GameSettings>,
) {
    if !keyboard.just_pressed(game_settings.keybinds.interact) {
        return;
    }

    let Ok((_player_entity, player_tf, mut current_floor)) = player_query.get_single_mut() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();

    for (tf, _) in stair_query.iter() {
        let dist = player_pos.distance(tf.translation.truncate());
        if dist <= 24.0 {
            current_floor.0 = if current_floor.0 == 0 { 1 } else { 0 };
            return;
        }
    }
}

fn building_visibility_by_floor(
    player_query: Query<&CurrentFloor, With<Player>>,
    mut building_query: Query<(&FloorLayer, Option<&StairsOrLadder>, &mut Sprite), (With<Building>, Without<Player>)>,
) {
    let Ok(player_floor) = player_query.get_single() else {
        return;
    };

    for (floor, stairs, mut sprite) in building_query.iter_mut() {
        let visible = stairs.is_some() || floor.0 == player_floor.0;
        sprite.color.set_alpha(if visible { 1.0 } else { 0.0 });
    }
}

fn destroy_building(
    mut commands: Commands,
    building_state: Res<BuildingState>,
    mouse: Res<ButtonInput<MouseButton>>,
    player_query: Query<&Transform, With<Player>>,
    building_query: Query<(Entity, &Transform, &Building), Without<Player>>,
    mut inventory: ResMut<Inventory>,
    mut chest_ui: ResMut<ChestUI>,
    mut sound_events: EventWriter<crate::audio::SoundEvent>,
) {
    if !building_state.active || !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    // Find nearest building within 32px
    let mut nearest: Option<(Entity, f32, BuildingType)> = None;
    for (entity, tf, building) in building_query.iter() {
        let dist = player_pos.distance(tf.translation.truncate());
        if dist <= 32.0 {
            if nearest.is_none() || dist < nearest.as_ref().unwrap().1 {
                nearest = Some((entity, dist, building.building_type));
            }
        }
    }

    if let Some((entity, _, bt)) = nearest {
        // Close chest UI if we're destroying the chest that's open
        if chest_ui.target_entity == Some(entity) {
            chest_ui.is_open = false;
            chest_ui.target_entity = None;
            chest_ui.selected_slot = 0;
        }
        for (item, count) in bt.salvage() {
            inventory.add_item(item, count);
        }
        sound_events.send(crate::audio::SoundEvent::BuildBreak);
        commands.entity(entity).despawn_recursive();
    }
}

fn chest_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    building_state: Res<BuildingState>,
    crafting: Res<crate::crafting::CraftingSystem>,
    trade_menu: Res<crate::npc::TradeMenu>,
    player_query: Query<&Transform, With<Player>>,
    chest_query: Query<(Entity, &Transform), With<ChestStorage>>,
    mut chest_ui: ResMut<ChestUI>,
    game_settings: Res<crate::settings::GameSettings>,
) {
    // Close chest UI with E or Escape
    if chest_ui.is_open {
        if keyboard.just_pressed(game_settings.keybinds.interact) || keyboard.just_pressed(KeyCode::Escape) {
            chest_ui.is_open = false;
            chest_ui.target_entity = None;
            chest_ui.selected_slot = 0;
        }
        return;
    }

    // Don't open if other menus are active
    if building_state.active || crafting.is_open || trade_menu.is_open {
        return;
    }

    if !keyboard.just_pressed(game_settings.keybinds.interact) {
        return;
    }

    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    // Find nearest chest within 32px
    let mut nearest: Option<(Entity, f32)> = None;
    for (entity, tf) in chest_query.iter() {
        let dist = player_pos.distance(tf.translation.truncate());
        if dist <= 32.0 {
            if nearest.is_none() || dist < nearest.unwrap().1 {
                nearest = Some((entity, dist));
            }
        }
    }

    if let Some((entity, _)) = nearest {
        chest_ui.is_open = true;
        chest_ui.target_entity = Some(entity);
        chest_ui.selected_slot = 0;
    }
}

fn chest_transfer(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut chest_ui: ResMut<ChestUI>,
    mut inventory: ResMut<Inventory>,
    mut chest_query: Query<&mut ChestStorage>,
) {
    if !chest_ui.is_open {
        return;
    }

    let Some(target) = chest_ui.target_entity else { return };
    let Ok(mut chest) = chest_query.get_mut(target) else {
        // Chest entity no longer exists
        chest_ui.is_open = false;
        chest_ui.target_entity = None;
        return;
    };

    // Number keys 1-9: transfer from player hotbar slot to first empty chest slot
    let hotbar_keys = [
        KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3,
        KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6,
        KeyCode::Digit7, KeyCode::Digit8, KeyCode::Digit9,
    ];
    for (i, key) in hotbar_keys.iter().enumerate() {
        if keyboard.just_pressed(*key) {
            // Take from player hotbar slot i
            if let Some(slot_data) = inventory.slots[i].clone() {
                // Find first empty chest slot
                if let Some(empty_idx) = chest.slots.iter().position(|s| s.is_none()) {
                    chest.slots[empty_idx] = Some(slot_data);
                    inventory.slots[i] = None;
                }
            }
        }
    }

    // Arrow Up/Down: select chest slot
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        if chest_ui.selected_slot > 0 {
            chest_ui.selected_slot -= 1;
        }
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        if chest_ui.selected_slot < 17 {
            chest_ui.selected_slot += 1;
        }
    }

    // Enter: transfer selected chest item to player inventory
    if keyboard.just_pressed(KeyCode::Enter) {
        let idx = chest_ui.selected_slot;
        if let Some(slot_data) = chest.slots[idx].clone() {
            let remaining = inventory.add_item(slot_data.item, slot_data.count);
            if remaining == 0 {
                chest.slots[idx] = None;
            } else if remaining < slot_data.count {
                // Partially transferred
                chest.slots[idx] = Some(InventorySlot {
                    item: slot_data.item,
                    count: remaining,
                    durability: slot_data.durability,
                });
            }
            // If remaining == slot_data.count, inventory was full, nothing happens
        }
    }
}
