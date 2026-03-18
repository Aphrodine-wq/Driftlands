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
    TomatoSeed,
    PumpkinSeed,
    // Crops
    Wheat,
    Carrot,
    Tomato,
    Pumpkin,
    // Cooked foods
    CookedBerry,
    BakedWheat,
    CookedCarrot,
    CookedTomato,
    BakedPumpkin,
    // Phase 4 — Tier 4 stations & materials
    AdvancedForge,
    AlchemyLab,
    RareHerb,
    Gemstone,
    AncientCore,
    // Phase 4 — Tier 4 equipment
    SteelSword,
    SteelAxe,
    SteelPickaxe,
    SteelArmor,
    HealthPotion,
    SpeedPotion,
    StrengthPotion,
    // Phase 4 — Tier 5 Ancient items
    AncientWorkstation,
    AncientBlade,
    AncientArmor,
    AncientPickaxe,
    // Phase 4 — Blueprint & biome boss drops (US-006 / US-007 / US-008)
    Blueprint,
    GuardianHeart,
    SwampEssence,
    WyrmScale,
    FrostGem,
    MagmaCore,
    FungalSporeEssence,
    CrystalHeart,
    // Phase 4 — NPC / lore items (US-012)
    JournalPage,
    // Phase 5 — Building tiers
    StoneFloor,
    StoneDoor,
    StoneRoof,
    MetalWall,
    MetalDoor,
    // Phase 5 — Bed
    Bed,
    // Vertical traversal (stairs & ladders)
    WoodStairs,
    StoneStairs,
    Ladder,
    // Phase 5 — Boss drops
    CoralEssence,
    TitanBone,
    // US-035 — Biome-exclusive resources
    SandstoneChip,
    Shell,
    Seaweed,
    BioGel,
    EchoStoneFragment,
    FrozenOre,
    // US-036 — Dungeon loot / spider drops
    CaveSlime,
    SpiderSilk,
    // Building palette: brick, reinforced stone, half walls, windows
    Brick,
    ReinforcedStoneBlock,
    WoodHalfWall,
    WoodWallWindow,
    BrickWall,
    ReinforcedStoneWall,
    // Expansion — New crop seeds
    CornSeed,
    PotatoSeed,
    MelonSeed,
    RiceSeed,
    PepperSeed,
    OnionSeed,
    FlaxSeed,
    SugarcaneSeed,
    // Expansion — New crops
    Corn,
    Potato,
    Melon,
    Rice,
    Pepper,
    Onion,
    Flax,
    Sugarcane,
    // Expansion — New cooked foods & processed crops
    RoastedCorn,
    BakedPotato,
    MelonSlice,
    CookedRice,
    RoastedPepper,
    CookedOnion,
    LinenCloth,
    Sugar,
    // Expansion — Fishing
    FishingRod,
    SteelFishingRod,
    FishBait,
    RawTrout,
    RawSalmon,
    RawCatfish,
    RawPufferfish,
    RawEel,
    RawCrab,
    CookedTrout,
    CookedSalmon,
    CookedCatfish,
    CookedEel,
    CrabMeat,
    // Expansion — Enchanting essences
    FireEssence,
    IceEssence,
    VenomEssence,
    LifeEssence,
    // Expansion — Enchanted weapons
    FlameBlade,
    FrostBlade,
    VenomBlade,
    LifestealBlade,
    // Expansion — Buildings
    EnchantingTable,
    FishSmoker,
    PetHouse,
    DisplayCase,
    // Expansion — Pet items
    PetCollar,
    PetFood,
    // Wave 6 — New Furniture
    Lantern,
    Bookshelf,
    WeaponRack,
    CookingPot,
    RainCollector,
    TrophyMount,
    // Wave 6 — Automation
    AutoSmelterItem,
    CropSprinklerItem,
    AlarmBellItem,
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
            ItemType::TomatoSeed => "Tomato Seed",
            ItemType::PumpkinSeed => "Pumpkin Seed",
            ItemType::Wheat => "Wheat",
            ItemType::Carrot => "Carrot",
            ItemType::Tomato => "Tomato",
            ItemType::Pumpkin => "Pumpkin",
            ItemType::CookedBerry => "Cooked Berry",
            ItemType::BakedWheat => "Baked Wheat",
            ItemType::CookedCarrot => "Cooked Carrot",
            ItemType::CookedTomato => "Cooked Tomato",
            ItemType::BakedPumpkin => "Baked Pumpkin",
            ItemType::AdvancedForge => "Advanced Forge",
            ItemType::AlchemyLab => "Alchemy Lab",
            ItemType::RareHerb => "Rare Herb",
            ItemType::Gemstone => "Gemstone",
            ItemType::AncientCore => "Ancient Core",
            ItemType::SteelSword => "Steel Sword",
            ItemType::SteelAxe => "Steel Axe",
            ItemType::SteelPickaxe => "Steel Pickaxe",
            ItemType::SteelArmor => "Steel Armor",
            ItemType::HealthPotion => "Health Potion",
            ItemType::SpeedPotion => "Speed Potion",
            ItemType::StrengthPotion => "Strength Potion",
            ItemType::AncientWorkstation => "Ancient Workstation",
            ItemType::AncientBlade => "Ancient Blade",
            ItemType::AncientArmor => "Ancient Armor",
            ItemType::AncientPickaxe => "Ancient Pickaxe",
            ItemType::Blueprint => "Blueprint",
            ItemType::GuardianHeart => "Guardian Heart",
            ItemType::SwampEssence => "Swamp Essence",
            ItemType::WyrmScale => "Wyrm Scale",
            ItemType::FrostGem => "Frost Gem",
            ItemType::MagmaCore => "Magma Core",
            ItemType::FungalSporeEssence => "Fungal Spore Essence",
            ItemType::CrystalHeart => "Crystal Heart",
            ItemType::JournalPage => "Journal Page",
            ItemType::StoneFloor => "Stone Floor",
            ItemType::StoneDoor => "Stone Door",
            ItemType::StoneRoof => "Stone Roof",
            ItemType::MetalWall => "Metal Wall",
            ItemType::MetalDoor => "Metal Door",
            ItemType::Bed => "Bed",
            ItemType::WoodStairs => "Wood Stairs",
            ItemType::StoneStairs => "Stone Stairs",
            ItemType::Ladder => "Ladder",
            ItemType::CoralEssence => "Coral Essence",
            ItemType::TitanBone => "Titan Bone",
            ItemType::SandstoneChip => "Sandstone Chip",
            ItemType::Shell => "Shell",
            ItemType::Seaweed => "Seaweed",
            ItemType::BioGel => "Bio-Luminescent Gel",
            ItemType::EchoStoneFragment => "Echo Stone Fragment",
            ItemType::FrozenOre => "Frozen Ore",
            ItemType::CaveSlime => "Cave Slime",
            ItemType::SpiderSilk => "Spider Silk",
            ItemType::Brick => "Brick",
            ItemType::ReinforcedStoneBlock => "Reinforced Stone Block",
            ItemType::WoodHalfWall => "Wood Half Wall",
            ItemType::WoodWallWindow => "Wood Wall (Window)",
            ItemType::BrickWall => "Brick Wall",
            ItemType::ReinforcedStoneWall => "Reinforced Stone Wall",
            ItemType::CornSeed => "Corn Seed",
            ItemType::PotatoSeed => "Potato Seed",
            ItemType::MelonSeed => "Melon Seed",
            ItemType::RiceSeed => "Rice Seed",
            ItemType::PepperSeed => "Pepper Seed",
            ItemType::OnionSeed => "Onion Seed",
            ItemType::FlaxSeed => "Flax Seed",
            ItemType::SugarcaneSeed => "Sugarcane Seed",
            ItemType::Corn => "Corn",
            ItemType::Potato => "Potato",
            ItemType::Melon => "Melon",
            ItemType::Rice => "Rice",
            ItemType::Pepper => "Pepper",
            ItemType::Onion => "Onion",
            ItemType::Flax => "Flax",
            ItemType::Sugarcane => "Sugarcane",
            ItemType::RoastedCorn => "Roasted Corn",
            ItemType::BakedPotato => "Baked Potato",
            ItemType::MelonSlice => "Melon Slice",
            ItemType::CookedRice => "Cooked Rice",
            ItemType::RoastedPepper => "Roasted Pepper",
            ItemType::CookedOnion => "Cooked Onion",
            ItemType::LinenCloth => "Linen Cloth",
            ItemType::Sugar => "Sugar",
            ItemType::FishingRod => "Fishing Rod",
            ItemType::SteelFishingRod => "Steel Fishing Rod",
            ItemType::FishBait => "Fish Bait",
            ItemType::RawTrout => "Raw Trout",
            ItemType::RawSalmon => "Raw Salmon",
            ItemType::RawCatfish => "Raw Catfish",
            ItemType::RawPufferfish => "Raw Pufferfish",
            ItemType::RawEel => "Raw Eel",
            ItemType::RawCrab => "Raw Crab",
            ItemType::CookedTrout => "Cooked Trout",
            ItemType::CookedSalmon => "Cooked Salmon",
            ItemType::CookedCatfish => "Cooked Catfish",
            ItemType::CookedEel => "Cooked Eel",
            ItemType::CrabMeat => "Crab Meat",
            ItemType::FireEssence => "Fire Essence",
            ItemType::IceEssence => "Ice Essence",
            ItemType::VenomEssence => "Venom Essence",
            ItemType::LifeEssence => "Life Essence",
            ItemType::FlameBlade => "Flame Blade",
            ItemType::FrostBlade => "Frost Blade",
            ItemType::VenomBlade => "Venom Blade",
            ItemType::LifestealBlade => "Lifesteal Blade",
            ItemType::EnchantingTable => "Enchanting Table",
            ItemType::FishSmoker => "Fish Smoker",
            ItemType::PetHouse => "Pet House",
            ItemType::DisplayCase => "Display Case",
            ItemType::PetCollar => "Pet Collar",
            ItemType::PetFood => "Pet Food",
            ItemType::Lantern => "Lantern",
            ItemType::Bookshelf => "Bookshelf",
            ItemType::WeaponRack => "Weapon Rack",
            ItemType::CookingPot => "Cooking Pot",
            ItemType::RainCollector => "Rain Collector",
            ItemType::TrophyMount => "Trophy Mount",
            ItemType::AutoSmelterItem => "Auto-Smelter",
            ItemType::CropSprinklerItem => "Crop Sprinkler",
            ItemType::AlarmBellItem => "Alarm Bell",
        }
    }

    pub fn max_stack(&self) -> u32 {
        match self {
            ItemType::WoodAxe | ItemType::WoodPickaxe |
            ItemType::StoneAxe | ItemType::StonePickaxe |
            ItemType::WoodSword | ItemType::WoodShield | ItemType::WoodBow |
            ItemType::IronAxe | ItemType::IronPickaxe | ItemType::IronSword |
            ItemType::IronShield | ItemType::IronHelmet | ItemType::IronChestplate |
            ItemType::Hoe |
            ItemType::SteelSword | ItemType::SteelAxe | ItemType::SteelPickaxe |
            ItemType::SteelArmor |
            ItemType::AncientBlade | ItemType::AncientArmor | ItemType::AncientPickaxe |
            ItemType::FishingRod | ItemType::SteelFishingRod |
            ItemType::FlameBlade | ItemType::FrostBlade | ItemType::VenomBlade | ItemType::LifestealBlade => 1,
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
            ItemType::SteelAxe => Some(300),
            ItemType::SteelPickaxe => Some(300),
            ItemType::SteelSword => Some(250),
            ItemType::AncientPickaxe => Some(500),
            ItemType::AncientBlade => Some(400),
            ItemType::FishingRod => Some(100),
            ItemType::SteelFishingRod => Some(250),
            ItemType::FlameBlade => Some(350),
            ItemType::FrostBlade => Some(350),
            ItemType::VenomBlade => Some(350),
            ItemType::LifestealBlade => Some(350),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn is_tool(&self) -> bool {
        self.max_durability().is_some()
    }

    pub fn weapon_damage(&self) -> Option<f32> {
        match self {
            ItemType::WoodSword => Some(10.0),
            ItemType::IronSword => Some(20.0),
            ItemType::SteelSword => Some(35.0),
            ItemType::AncientBlade => Some(50.0),
            ItemType::FlameBlade => Some(40.0),
            ItemType::FrostBlade => Some(38.0),
            ItemType::VenomBlade => Some(36.0),
            ItemType::LifestealBlade => Some(42.0),
            _ => None,
        }
    }

    pub fn armor_value(&self) -> u32 {
        match self {
            ItemType::IronHelmet => 3,
            ItemType::IronChestplate => 5,
            ItemType::SteelArmor => 8,
            ItemType::AncientArmor => 12,
            _ => 0,
        }
    }

    pub fn is_helmet(&self) -> bool {
        matches!(self, ItemType::IronHelmet)
    }

    pub fn is_chestplate(&self) -> bool {
        matches!(self, ItemType::IronChestplate | ItemType::SteelArmor | ItemType::AncientArmor)
    }

    pub fn shield_value(&self) -> u32 {
        match self {
            ItemType::WoodShield => 2,
            ItemType::IronShield => 5,
            _ => 0,
        }
    }

    pub fn is_shield(&self) -> bool {
        matches!(self, ItemType::WoodShield | ItemType::IronShield)
    }

    pub fn tool_tier(&self) -> u32 {
        match self {
            ItemType::WoodAxe | ItemType::WoodPickaxe => 1,
            ItemType::StoneAxe | ItemType::StonePickaxe => 2,
            ItemType::IronAxe | ItemType::IronPickaxe | ItemType::IronSword | ItemType::Hoe => 3,
            ItemType::SteelAxe | ItemType::SteelPickaxe | ItemType::SteelSword => 4,
            ItemType::AncientPickaxe | ItemType::AncientBlade => 5,
            ItemType::FlameBlade | ItemType::FrostBlade |
            ItemType::VenomBlade | ItemType::LifestealBlade => 5,
            _ => 0,
        }
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
        self.count_items(item) >= count
    }

    /// Returns the total count of a given item type across all inventory slots.
    pub fn count_items(&self, item: ItemType) -> u32 {
        self.slots.iter()
            .filter_map(|s| s.as_ref())
            .filter(|s| s.item == item)
            .map(|s| s.count)
            .sum()
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
