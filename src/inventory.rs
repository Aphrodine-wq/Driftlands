use bevy::prelude::*;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_inventory)
            .add_systems(Update, (toggle_inventory, hotbar_selection));
    }
}

use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemType {
    Wood,
    Stone,
    PlantFiber,
    Stick,
    Flint,
    WoodPlank,
    Rope,
    Campfire,
    WoodFloor,
    Workbench,
    WoodAxe,
    WoodPickaxe,
    StoneAxe,
    StonePickaxe,
    Berry,
    WoodWall,
    WoodDoor,
    WoodRoof,
    WoodFence,
    Chest,
    Torch,
    WoodSword,
    WoodShield,
    WoodBow,
    Arrow,
    // Phase 3 biome resources
    CactusFiber,
    IceShard,
    MushroomCap,
    Spore,
    Reed,
    Sulfur,
    CrystalShard,
    AlpineHerb,
    Peat,
    ObsidianShard,
    // Tier 3 materials
    IronOre,
    Coal,
    StoneBlock,
    IronIngot,
    SteelAlloy,
    // Tier 3 equipment
    Forge,
    Anvil,
    IronAxe,
    IronPickaxe,
    IronSword,
    IronShield,
    IronHelmet,
    IronChestplate,
    StoneWall,
    // Farming tools & seeds
    Hoe,
    WheatSeed,
    CarrotSeed,
    // Crops
    Wheat,
    Carrot,
    // Cooked foods
    CookedBerry,
    BakedWheat,
    CookedCarrot,
}

impl ItemType {
    pub fn display_name(&self) -> &str {
        match self {
            ItemType::Wood => "Wood",
            ItemType::Stone => "Stone",
            ItemType::PlantFiber => "Plant Fiber",
            ItemType::Stick => "Stick",
            ItemType::Flint => "Flint",
            ItemType::WoodPlank => "Wood Plank",
            ItemType::Rope => "Rope",
            ItemType::Campfire => "Campfire",
            ItemType::WoodFloor => "Wood Floor",
            ItemType::Workbench => "Workbench",
            ItemType::WoodAxe => "Wood Axe",
            ItemType::WoodPickaxe => "Wood Pickaxe",
            ItemType::StoneAxe => "Stone Axe",
            ItemType::StonePickaxe => "Stone Pickaxe",
            ItemType::Berry => "Berry",
            ItemType::WoodWall => "Wood Wall",
            ItemType::WoodDoor => "Wood Door",
            ItemType::WoodRoof => "Wood Roof",
            ItemType::WoodFence => "Wood Fence",
            ItemType::Chest => "Chest",
            ItemType::Torch => "Torch",
            ItemType::WoodSword => "Wood Sword",
            ItemType::WoodShield => "Wood Shield",
            ItemType::WoodBow => "Wood Bow",
            ItemType::Arrow => "Arrow",
            ItemType::CactusFiber => "Cactus Fiber",
            ItemType::IceShard => "Ice Shard",
            ItemType::MushroomCap => "Mushroom Cap",
            ItemType::Spore => "Spore",
            ItemType::Reed => "Reed",
            ItemType::Sulfur => "Sulfur",
            ItemType::CrystalShard => "Crystal Shard",
            ItemType::AlpineHerb => "Alpine Herb",
            ItemType::Peat => "Peat",
            ItemType::ObsidianShard => "Obsidian",
            ItemType::IronOre => "Iron Ore",
            ItemType::Coal => "Coal",
            ItemType::StoneBlock => "Stone Block",
            ItemType::IronIngot => "Iron Ingot",
            ItemType::SteelAlloy => "Steel Alloy",
            ItemType::Forge => "Forge",
            ItemType::Anvil => "Anvil",
            ItemType::IronAxe => "Iron Axe",
            ItemType::IronPickaxe => "Iron Pickaxe",
            ItemType::IronSword => "Iron Sword",
            ItemType::IronShield => "Iron Shield",
            ItemType::IronHelmet => "Iron Helmet",
            ItemType::IronChestplate => "Iron Chestplate",
            ItemType::StoneWall => "Stone Wall",
            ItemType::Hoe => "Hoe",
            ItemType::WheatSeed => "Wheat Seed",
            ItemType::CarrotSeed => "Carrot Seed",
            ItemType::Wheat => "Wheat",
            ItemType::Carrot => "Carrot",
            ItemType::CookedBerry => "Cooked Berry",
            ItemType::BakedWheat => "Baked Wheat",
            ItemType::CookedCarrot => "Cooked Carrot",
        }
    }

    pub fn max_stack(&self) -> u32 {
        match self {
            ItemType::WoodAxe | ItemType::WoodPickaxe |
            ItemType::StoneAxe | ItemType::StonePickaxe |
            ItemType::WoodSword | ItemType::WoodShield | ItemType::WoodBow |
            ItemType::IronAxe | ItemType::IronPickaxe | ItemType::IronSword |
            ItemType::IronShield | ItemType::IronHelmet | ItemType::IronChestplate |
            ItemType::Hoe => 1,
            _ => 64,
        }
    }

    pub fn max_durability(&self) -> Option<u32> {
        match self {
            ItemType::WoodAxe => Some(50),
            ItemType::WoodPickaxe => Some(50),
            ItemType::StoneAxe => Some(100),
            ItemType::StonePickaxe => Some(100),
            ItemType::IronAxe => Some(200),
            ItemType::IronPickaxe => Some(200),
            ItemType::IronSword => Some(150),
            ItemType::Hoe => Some(75),
            _ => None,
        }
    }

    pub fn is_tool(&self) -> bool {
        self.max_durability().is_some()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InventorySlot {
    pub item: ItemType,
    pub count: u32,
    pub durability: Option<u32>,
}

#[derive(Resource)]
pub struct Inventory {
    pub slots: Vec<Option<InventorySlot>>,
    pub hotbar_size: usize,
    pub selected_slot: usize,
    pub is_open: bool,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            slots: vec![None; 36],
            hotbar_size: 9,
            selected_slot: 0,
            is_open: false,
        }
    }
}

impl Inventory {
    pub fn add_item(&mut self, item: ItemType, count: u32) -> u32 {
        let mut remaining = count;

        // Stack with existing slots first
        for slot in self.slots.iter_mut() {
            if remaining == 0 { break; }
            if let Some(ref mut s) = slot {
                if s.item == item {
                    let can_add = item.max_stack() - s.count;
                    let add = remaining.min(can_add);
                    s.count += add;
                    remaining -= add;
                }
            }
        }

        // Fill empty slots
        for slot in self.slots.iter_mut() {
            if remaining == 0 { break; }
            if slot.is_none() {
                let add = remaining.min(item.max_stack());
                *slot = Some(InventorySlot { item, count: add, durability: item.max_durability() });
                remaining -= add;
            }
        }

        remaining
    }

    pub fn has_items(&self, item: ItemType, count: u32) -> bool {
        let total: u32 = self.slots.iter()
            .filter_map(|s| s.as_ref())
            .filter(|s| s.item == item)
            .map(|s| s.count)
            .sum();
        total >= count
    }

    pub fn remove_items(&mut self, item: ItemType, count: u32) -> bool {
        if !self.has_items(item, count) {
            return false;
        }
        let mut remaining = count;
        for slot in self.slots.iter_mut() {
            if remaining == 0 { break; }
            if let Some(ref mut s) = slot {
                if s.item == item {
                    let remove = remaining.min(s.count);
                    s.count -= remove;
                    remaining -= remove;
                    if s.count == 0 {
                        *slot = None;
                    }
                }
            }
        }
        true
    }

    pub fn selected_item(&self) -> Option<&InventorySlot> {
        self.slots[self.selected_slot].as_ref()
    }

    /// Reduce durability of selected tool by 1. Returns true if tool broke.
    pub fn use_selected_tool(&mut self) -> bool {
        let idx = self.selected_slot;
        if let Some(ref mut slot) = self.slots[idx] {
            if let Some(ref mut dur) = slot.durability {
                *dur = dur.saturating_sub(1);
                if *dur == 0 {
                    self.slots[idx] = None;
                    return true;
                }
            }
        }
        false
    }
}

fn setup_inventory(mut commands: Commands) {
    commands.insert_resource(Inventory::default());
}

fn toggle_inventory(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut inventory: ResMut<Inventory>,
) {
    if keyboard.just_pressed(KeyCode::Tab) || keyboard.just_pressed(KeyCode::KeyI) {
        inventory.is_open = !inventory.is_open;
    }
}

fn hotbar_selection(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut inventory: ResMut<Inventory>,
) {
    let keys = [
        KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3,
        KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6,
        KeyCode::Digit7, KeyCode::Digit8, KeyCode::Digit9,
    ];
    for (i, key) in keys.iter().enumerate() {
        if keyboard.just_pressed(*key) {
            inventory.selected_slot = i;
        }
    }
}
