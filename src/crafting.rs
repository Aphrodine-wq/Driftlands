use bevy::prelude::*;
use crate::inventory::{Inventory, ItemType};
use crate::player::Player;
use crate::techtree::TechTree;
use crate::building::CraftingStation;
use crate::audio::SoundEvent;

pub struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CraftingSystem::new())
            .add_systems(Update, handle_crafting_input);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CraftingTier {
    Hand,
    Workbench,
    Forge,
    Campfire,
    AdvancedForge,
    Ancient,
}

impl CraftingTier {
    pub fn label(&self) -> &str {
        match self {
            CraftingTier::Hand => "[Hand]",
            CraftingTier::Workbench => "[Bench]",
            CraftingTier::Forge => "[Forge]",
            CraftingTier::Campfire => "[Fire]",
            CraftingTier::AdvancedForge => "[AdvForge]",
            CraftingTier::Ancient => "[Ancient]",
        }
    }
}

pub struct Recipe {
    pub name: &'static str,
    pub inputs: Vec<(ItemType, u32)>,
    pub output: (ItemType, u32),
    pub tier: CraftingTier,
    /// If set, this recipe requires the named tech to be unlocked in the TechTree.
    pub tech_key: Option<&'static str>,
}

#[derive(Resource)]
pub struct CraftingSystem {
    pub recipes: Vec<Recipe>,
    pub is_open: bool,
    pub selected_recipe: usize,
}

impl CraftingSystem {
    pub fn new() -> Self {
        Self {
            recipes: vec![
                // Hand tier (existing 10)
                Recipe {
                    name: "Stick (x4)",
                    inputs: vec![(ItemType::Wood, 1)],
                    output: (ItemType::Stick, 4),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Wood Plank (x4)",
                    inputs: vec![(ItemType::Wood, 2)],
                    output: (ItemType::WoodPlank, 4),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Rope",
                    inputs: vec![(ItemType::PlantFiber, 3)],
                    output: (ItemType::Rope, 1),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Campfire",
                    inputs: vec![
                        (ItemType::Stone, 5),
                        (ItemType::Stick, 3),
                        (ItemType::Wood, 2),
                    ],
                    output: (ItemType::Campfire, 1),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Wood Axe",
                    inputs: vec![
                        (ItemType::Stick, 2),
                        (ItemType::Flint, 1),
                        (ItemType::Rope, 1),
                    ],
                    output: (ItemType::WoodAxe, 1),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Wood Pickaxe",
                    inputs: vec![
                        (ItemType::Stick, 2),
                        (ItemType::Flint, 2),
                        (ItemType::Rope, 1),
                    ],
                    output: (ItemType::WoodPickaxe, 1),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Stone Axe",
                    inputs: vec![
                        (ItemType::Stick, 2),
                        (ItemType::Stone, 3),
                        (ItemType::Rope, 1),
                    ],
                    output: (ItemType::StoneAxe, 1),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Stone Pickaxe",
                    inputs: vec![
                        (ItemType::Stick, 2),
                        (ItemType::Stone, 3),
                        (ItemType::Rope, 1),
                    ],
                    output: (ItemType::StonePickaxe, 1),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Wood Floor",
                    inputs: vec![(ItemType::WoodPlank, 4)],
                    output: (ItemType::WoodFloor, 1),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Workbench",
                    inputs: vec![
                        (ItemType::WoodPlank, 8),
                        (ItemType::Stick, 4),
                    ],
                    output: (ItemType::Workbench, 1),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Stone Block",
                    inputs: vec![(ItemType::Stone, 4)],
                    output: (ItemType::StoneBlock, 1),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                // Workbench tier (10 new recipes)
                Recipe {
                    name: "Wood Wall",
                    inputs: vec![(ItemType::WoodPlank, 4), (ItemType::Stick, 2)],
                    output: (ItemType::WoodWall, 1),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Wood Door",
                    inputs: vec![(ItemType::WoodPlank, 6)],
                    output: (ItemType::WoodDoor, 1),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Wood Roof",
                    inputs: vec![(ItemType::WoodPlank, 6), (ItemType::Stick, 4)],
                    output: (ItemType::WoodRoof, 1),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Wood Fence",
                    inputs: vec![(ItemType::Stick, 6)],
                    output: (ItemType::WoodFence, 2),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Wood Stairs",
                    inputs: vec![(ItemType::WoodPlank, 4), (ItemType::Stick, 4)],
                    output: (ItemType::WoodStairs, 1),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Ladder",
                    inputs: vec![(ItemType::Stick, 6), (ItemType::Rope, 2)],
                    output: (ItemType::Ladder, 1),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Chest",
                    inputs: vec![(ItemType::WoodPlank, 8)],
                    output: (ItemType::Chest, 1),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Torch",
                    inputs: vec![(ItemType::Stick, 1), (ItemType::PlantFiber, 2)],
                    output: (ItemType::Torch, 4),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Wood Sword",
                    inputs: vec![(ItemType::WoodPlank, 3), (ItemType::Stick, 1)],
                    output: (ItemType::WoodSword, 1),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Wood Shield",
                    inputs: vec![(ItemType::WoodPlank, 6), (ItemType::Rope, 1)],
                    output: (ItemType::WoodShield, 1),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Wood Bow",
                    inputs: vec![(ItemType::Stick, 3), (ItemType::Rope, 2)],
                    output: (ItemType::WoodBow, 1),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Arrow (x8)",
                    inputs: vec![(ItemType::Stick, 2), (ItemType::Flint, 1)],
                    output: (ItemType::Arrow, 8),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                // Workbench — Tier 3 stations
                Recipe {
                    name: "Forge",
                    inputs: vec![(ItemType::StoneBlock, 10), (ItemType::IronOre, 5), (ItemType::Coal, 3)],
                    output: (ItemType::Forge, 1),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Anvil",
                    inputs: vec![(ItemType::IronIngot, 8), (ItemType::StoneBlock, 4)],
                    output: (ItemType::Anvil, 1),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                // Forge tier recipes
                Recipe {
                    name: "Iron Ingot",
                    inputs: vec![(ItemType::IronOre, 2), (ItemType::Coal, 1)],
                    output: (ItemType::IronIngot, 1),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Steel Alloy",
                    inputs: vec![(ItemType::IronIngot, 2), (ItemType::Coal, 2)],
                    output: (ItemType::SteelAlloy, 1),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Iron Axe",
                    inputs: vec![(ItemType::IronIngot, 3), (ItemType::Stick, 2)],
                    output: (ItemType::IronAxe, 1),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Iron Pickaxe",
                    inputs: vec![(ItemType::IronIngot, 3), (ItemType::Stick, 2)],
                    output: (ItemType::IronPickaxe, 1),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Iron Sword",
                    inputs: vec![(ItemType::IronIngot, 4), (ItemType::Stick, 1)],
                    output: (ItemType::IronSword, 1),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Iron Shield",
                    inputs: vec![(ItemType::IronIngot, 5), (ItemType::WoodPlank, 2)],
                    output: (ItemType::IronShield, 1),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Iron Helmet",
                    inputs: vec![(ItemType::IronIngot, 4)],
                    output: (ItemType::IronHelmet, 1),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Iron Chestplate",
                    inputs: vec![(ItemType::IronIngot, 6)],
                    output: (ItemType::IronChestplate, 1),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Stone Wall",
                    inputs: vec![(ItemType::StoneBlock, 4)],
                    output: (ItemType::StoneWall, 1),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Stone Stairs",
                    inputs: vec![(ItemType::StoneBlock, 4)],
                    output: (ItemType::StoneStairs, 1),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                // Hand tier — Hoe
                Recipe {
                    name: "Hoe",
                    inputs: vec![
                        (ItemType::Stick, 2),
                        (ItemType::Flint, 2),
                        (ItemType::Rope, 1),
                    ],
                    output: (ItemType::Hoe, 1),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                // Hand tier — seeds (craft from plant fiber / gathered items)
                Recipe {
                    name: "Wheat Seed (x3)",
                    inputs: vec![(ItemType::PlantFiber, 2)],
                    output: (ItemType::WheatSeed, 3),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Carrot Seed (x3)",
                    inputs: vec![(ItemType::PlantFiber, 2), (ItemType::Berry, 1)],
                    output: (ItemType::CarrotSeed, 3),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                // Campfire cooking recipes
                Recipe {
                    name: "Cooked Berry",
                    inputs: vec![(ItemType::Berry, 2)],
                    output: (ItemType::CookedBerry, 1),
                    tier: CraftingTier::Campfire,
                    tech_key: None,
                },
                Recipe {
                    name: "Baked Wheat",
                    inputs: vec![(ItemType::Wheat, 2)],
                    output: (ItemType::BakedWheat, 1),
                    tier: CraftingTier::Campfire,
                    tech_key: None,
                },
                Recipe {
                    name: "Cooked Carrot",
                    inputs: vec![(ItemType::Carrot, 1)],
                    output: (ItemType::CookedCarrot, 1),
                    tier: CraftingTier::Campfire,
                    tech_key: None,
                },
                // Forge tier — Tier 4 stations
                Recipe {
                    name: "Advanced Forge",
                    inputs: vec![
                        (ItemType::SteelAlloy, 10),
                        (ItemType::CrystalShard, 5),
                        (ItemType::ObsidianShard, 3),
                    ],
                    output: (ItemType::AdvancedForge, 1),
                    tier: CraftingTier::Forge,
                    tech_key: Some("advanced_forge"),
                },
                Recipe {
                    name: "Alchemy Lab",
                    inputs: vec![
                        (ItemType::StoneBlock, 8),
                        (ItemType::CrystalShard, 4),
                        (ItemType::Sulfur, 2),
                    ],
                    output: (ItemType::AlchemyLab, 1),
                    tier: CraftingTier::Forge,
                    tech_key: Some("alchemy_lab"),
                },
                // AdvancedForge tier — Tier 4 equipment
                Recipe {
                    name: "Steel Sword",
                    inputs: vec![
                        (ItemType::SteelAlloy, 4),
                        (ItemType::IronIngot, 2),
                        (ItemType::Stick, 1),
                    ],
                    output: (ItemType::SteelSword, 1),
                    tier: CraftingTier::AdvancedForge,
                    tech_key: Some("steel_sword"),
                },
                Recipe {
                    name: "Steel Axe",
                    inputs: vec![
                        (ItemType::SteelAlloy, 4),
                        (ItemType::IronIngot, 2),
                        (ItemType::Stick, 2),
                    ],
                    output: (ItemType::SteelAxe, 1),
                    tier: CraftingTier::AdvancedForge,
                    tech_key: Some("steel_axe"),
                },
                Recipe {
                    name: "Steel Pickaxe",
                    inputs: vec![
                        (ItemType::SteelAlloy, 4),
                        (ItemType::IronIngot, 2),
                        (ItemType::Stick, 2),
                    ],
                    output: (ItemType::SteelPickaxe, 1),
                    tier: CraftingTier::AdvancedForge,
                    tech_key: Some("steel_pickaxe"),
                },
                Recipe {
                    name: "Steel Armor",
                    inputs: vec![
                        (ItemType::SteelAlloy, 8),
                        (ItemType::IronIngot, 4),
                    ],
                    output: (ItemType::SteelArmor, 1),
                    tier: CraftingTier::AdvancedForge,
                    tech_key: Some("steel_armor"),
                },
                Recipe {
                    name: "Health Potion",
                    inputs: vec![
                        (ItemType::RareHerb, 2),
                        (ItemType::AlpineHerb, 3),
                        (ItemType::CrystalShard, 1),
                    ],
                    output: (ItemType::HealthPotion, 1),
                    tier: CraftingTier::AdvancedForge,
                    tech_key: Some("health_potion"),
                },
                Recipe {
                    name: "Speed Potion",
                    inputs: vec![
                        (ItemType::RareHerb, 2),
                        (ItemType::Spore, 2),
                        (ItemType::CrystalShard, 1),
                    ],
                    output: (ItemType::SpeedPotion, 1),
                    tier: CraftingTier::AdvancedForge,
                    tech_key: Some("speed_potion"),
                },
                Recipe {
                    name: "Strength Potion",
                    inputs: vec![
                        (ItemType::RareHerb, 2),
                        (ItemType::Sulfur, 1),
                        (ItemType::Gemstone, 1),
                    ],
                    output: (ItemType::StrengthPotion, 1),
                    tier: CraftingTier::AdvancedForge,
                    tech_key: Some("strength_potion"),
                },
                // Ancient tier — Tier 5 stations & equipment
                Recipe {
                    name: "Ancient Workstation",
                    inputs: vec![
                        (ItemType::AncientCore, 5),
                        (ItemType::Gemstone, 5),
                        (ItemType::SteelAlloy, 10),
                    ],
                    output: (ItemType::AncientWorkstation, 1),
                    tier: CraftingTier::AdvancedForge,
                    tech_key: Some("ancient_workstation"),
                },
                Recipe {
                    name: "Ancient Blade",
                    inputs: vec![
                        (ItemType::AncientCore, 3),
                        (ItemType::Gemstone, 2),
                        (ItemType::SteelAlloy, 5),
                    ],
                    output: (ItemType::AncientBlade, 1),
                    tier: CraftingTier::Ancient,
                    tech_key: Some("ancient_blade"),
                },
                Recipe {
                    name: "Ancient Armor",
                    inputs: vec![
                        (ItemType::AncientCore, 4),
                        (ItemType::Gemstone, 3),
                        (ItemType::SteelArmor, 1),
                    ],
                    output: (ItemType::AncientArmor, 1),
                    tier: CraftingTier::Ancient,
                    tech_key: Some("ancient_armor"),
                },
                Recipe {
                    name: "Ancient Pickaxe",
                    inputs: vec![
                        (ItemType::AncientCore, 3),
                        (ItemType::Gemstone, 2),
                        (ItemType::SteelAlloy, 4),
                    ],
                    output: (ItemType::AncientPickaxe, 1),
                    tier: CraftingTier::Ancient,
                    tech_key: Some("ancient_pickaxe"),
                },
                // Phase 5 — Stone building recipes (Forge tier)
                Recipe {
                    name: "Stone Floor",
                    inputs: vec![(ItemType::StoneBlock, 4)],
                    output: (ItemType::StoneFloor, 1),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Stone Door",
                    inputs: vec![(ItemType::StoneBlock, 6), (ItemType::IronIngot, 2)],
                    output: (ItemType::StoneDoor, 1),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Stone Roof",
                    inputs: vec![(ItemType::StoneBlock, 6)],
                    output: (ItemType::StoneRoof, 1),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                // Phase 5 — Metal building recipes (AdvancedForge tier)
                Recipe {
                    name: "Metal Wall",
                    inputs: vec![(ItemType::SteelAlloy, 4), (ItemType::IronIngot, 2)],
                    output: (ItemType::MetalWall, 1),
                    tier: CraftingTier::AdvancedForge,
                    tech_key: None,
                },
                Recipe {
                    name: "Metal Door",
                    inputs: vec![(ItemType::SteelAlloy, 6)],
                    output: (ItemType::MetalDoor, 1),
                    tier: CraftingTier::AdvancedForge,
                    tech_key: None,
                },
                // Phase 5 — Bed recipe (Hand tier)
                Recipe {
                    name: "Bed",
                    inputs: vec![(ItemType::WoodPlank, 6), (ItemType::PlantFiber, 4)],
                    output: (ItemType::Bed, 1),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                // --- Extra recipes toward 100+ ---
                Recipe {
                    name: "Cooked Berry (x2)",
                    inputs: vec![(ItemType::Berry, 3)],
                    output: (ItemType::CookedBerry, 2),
                    tier: CraftingTier::Campfire,
                    tech_key: None,
                },
                Recipe {
                    name: "Baked Wheat (x2)",
                    inputs: vec![(ItemType::Wheat, 2)],
                    output: (ItemType::BakedWheat, 2),
                    tier: CraftingTier::Campfire,
                    tech_key: None,
                },
                Recipe {
                    name: "Cooked Carrot (x2)",
                    inputs: vec![(ItemType::Carrot, 2)],
                    output: (ItemType::CookedCarrot, 2),
                    tier: CraftingTier::Campfire,
                    tech_key: None,
                },
                Recipe {
                    name: "Arrow (x16)",
                    inputs: vec![(ItemType::Stick, 4), (ItemType::Flint, 2)],
                    output: (ItemType::Arrow, 16),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Torch (x8)",
                    inputs: vec![(ItemType::Stick, 2), (ItemType::PlantFiber, 4)],
                    output: (ItemType::Torch, 8),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Rope (x3)",
                    inputs: vec![(ItemType::PlantFiber, 9)],
                    output: (ItemType::Rope, 3),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Wood Plank (x8)",
                    inputs: vec![(ItemType::Wood, 4)],
                    output: (ItemType::WoodPlank, 8),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Stick (x8)",
                    inputs: vec![(ItemType::Wood, 2)],
                    output: (ItemType::Stick, 8),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Stone Block (x2)",
                    inputs: vec![(ItemType::Stone, 8)],
                    output: (ItemType::StoneBlock, 2),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Iron Ingot (x3)",
                    inputs: vec![(ItemType::IronOre, 6), (ItemType::Coal, 3)],
                    output: (ItemType::IronIngot, 3),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Steel Alloy (x2)",
                    inputs: vec![(ItemType::IronIngot, 4), (ItemType::Coal, 4)],
                    output: (ItemType::SteelAlloy, 2),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Health Potion",
                    inputs: vec![(ItemType::RareHerb, 2), (ItemType::Berry, 4), (ItemType::CrystalShard, 1)],
                    output: (ItemType::HealthPotion, 1),
                    tier: CraftingTier::AdvancedForge,
                    tech_key: Some("health_potion"),
                },
                Recipe {
                    name: "Speed Potion",
                    inputs: vec![(ItemType::RareHerb, 1), (ItemType::AlpineHerb, 2), (ItemType::Spore, 2)],
                    output: (ItemType::SpeedPotion, 1),
                    tier: CraftingTier::AdvancedForge,
                    tech_key: Some("speed_potion"),
                },
                Recipe {
                    name: "Strength Potion",
                    inputs: vec![(ItemType::RareHerb, 2), (ItemType::Sulfur, 1), (ItemType::IronIngot, 1)],
                    output: (ItemType::StrengthPotion, 1),
                    tier: CraftingTier::AdvancedForge,
                    tech_key: Some("strength_potion"),
                },
                Recipe {
                    name: "Crossbow Bolt (x12)",
                    inputs: vec![(ItemType::IronIngot, 1), (ItemType::Stick, 2)],
                    output: (ItemType::Arrow, 12),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Wood Fence (x4)",
                    inputs: vec![(ItemType::Stick, 12)],
                    output: (ItemType::WoodFence, 4),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Chest (x2)",
                    inputs: vec![(ItemType::WoodPlank, 16)],
                    output: (ItemType::Chest, 2),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Wood Floor (x2)",
                    inputs: vec![(ItemType::WoodPlank, 8)],
                    output: (ItemType::WoodFloor, 2),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Stone Floor (x2)",
                    inputs: vec![(ItemType::StoneBlock, 4)],
                    output: (ItemType::StoneFloor, 2),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Wood Wall (x2)",
                    inputs: vec![(ItemType::WoodPlank, 8), (ItemType::Stick, 4)],
                    output: (ItemType::WoodWall, 2),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Stone Wall (x2)",
                    inputs: vec![(ItemType::StoneBlock, 8)],
                    output: (ItemType::StoneWall, 2),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Wood Door (x2)",
                    inputs: vec![(ItemType::WoodPlank, 12)],
                    output: (ItemType::WoodDoor, 2),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Stone Door (x2)",
                    inputs: vec![(ItemType::StoneBlock, 6), (ItemType::IronIngot, 2)],
                    output: (ItemType::StoneDoor, 2),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Wood Roof (x2)",
                    inputs: vec![(ItemType::WoodPlank, 12), (ItemType::Stick, 8)],
                    output: (ItemType::WoodRoof, 2),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Stone Roof (x2)",
                    inputs: vec![(ItemType::StoneBlock, 6)],
                    output: (ItemType::StoneRoof, 2),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Campfire (x2)",
                    inputs: vec![(ItemType::Stone, 10), (ItemType::Stick, 6), (ItemType::Wood, 4)],
                    output: (ItemType::Campfire, 2),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Wheat Seed (x6)",
                    inputs: vec![(ItemType::PlantFiber, 4)],
                    output: (ItemType::WheatSeed, 6),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Carrot Seed (x4)",
                    inputs: vec![(ItemType::PlantFiber, 3), (ItemType::Carrot, 1)],
                    output: (ItemType::CarrotSeed, 4),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Iron Pickaxe (x2)",
                    inputs: vec![(ItemType::IronIngot, 6), (ItemType::Stick, 4)],
                    output: (ItemType::IronPickaxe, 2),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Iron Axe (x2)",
                    inputs: vec![(ItemType::IronIngot, 6), (ItemType::Stick, 4)],
                    output: (ItemType::IronAxe, 2),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Iron Sword (x2)",
                    inputs: vec![(ItemType::IronIngot, 8), (ItemType::Stick, 2)],
                    output: (ItemType::IronSword, 2),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Iron Helmet (x2)",
                    inputs: vec![(ItemType::IronIngot, 8)],
                    output: (ItemType::IronHelmet, 2),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Iron Chestplate (x2)",
                    inputs: vec![(ItemType::IronIngot, 12)],
                    output: (ItemType::IronChestplate, 2),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Wood Stairs (x2)",
                    inputs: vec![(ItemType::WoodPlank, 8), (ItemType::Stick, 8)],
                    output: (ItemType::WoodStairs, 2),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Stone Stairs (x2)",
                    inputs: vec![(ItemType::StoneBlock, 8)],
                    output: (ItemType::StoneStairs, 2),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Ladder (x2)",
                    inputs: vec![(ItemType::Stick, 12), (ItemType::Rope, 4)],
                    output: (ItemType::Ladder, 2),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Metal Wall (x2)",
                    inputs: vec![(ItemType::SteelAlloy, 8), (ItemType::IronIngot, 4)],
                    output: (ItemType::MetalWall, 2),
                    tier: CraftingTier::AdvancedForge,
                    tech_key: None,
                },
                Recipe {
                    name: "Metal Door (x2)",
                    inputs: vec![(ItemType::SteelAlloy, 12)],
                    output: (ItemType::MetalDoor, 2),
                    tier: CraftingTier::AdvancedForge,
                    tech_key: None,
                },
                Recipe {
                    name: "Forge (x2)",
                    inputs: vec![(ItemType::StoneBlock, 20), (ItemType::IronOre, 10), (ItemType::Coal, 6)],
                    output: (ItemType::Forge, 2),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Brick (x4)",
                    inputs: vec![(ItemType::StoneBlock, 2), (ItemType::Coal, 1)],
                    output: (ItemType::Brick, 4),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Reinforced Stone Block",
                    inputs: vec![(ItemType::StoneBlock, 2), (ItemType::IronIngot, 1)],
                    output: (ItemType::ReinforcedStoneBlock, 1),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Wood Half Wall",
                    inputs: vec![(ItemType::WoodPlank, 2), (ItemType::Stick, 1)],
                    output: (ItemType::WoodHalfWall, 1),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Wood Wall (Window)",
                    inputs: vec![(ItemType::WoodPlank, 4), (ItemType::Stick, 2)],
                    output: (ItemType::WoodWallWindow, 1),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Brick Wall",
                    inputs: vec![(ItemType::Brick, 4)],
                    output: (ItemType::BrickWall, 1),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Reinforced Stone Wall",
                    inputs: vec![(ItemType::ReinforcedStoneBlock, 3), (ItemType::IronIngot, 1)],
                    output: (ItemType::ReinforcedStoneWall, 1),
                    tier: CraftingTier::AdvancedForge,
                    tech_key: None,
                },
                Recipe {
                    name: "Tomato Seed (x3)",
                    inputs: vec![(ItemType::PlantFiber, 2), (ItemType::Berry, 1)],
                    output: (ItemType::TomatoSeed, 3),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Pumpkin Seed (x2)",
                    inputs: vec![(ItemType::PlantFiber, 4)],
                    output: (ItemType::PumpkinSeed, 2),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Cooked Tomato (x2)",
                    inputs: vec![(ItemType::Tomato, 2)],
                    output: (ItemType::CookedTomato, 2),
                    tier: CraftingTier::Campfire,
                    tech_key: None,
                },
                Recipe {
                    name: "Baked Pumpkin",
                    inputs: vec![(ItemType::Pumpkin, 1)],
                    output: (ItemType::BakedPumpkin, 1),
                    tier: CraftingTier::Campfire,
                    tech_key: None,
                },
                // ── Expansion: New crop seeds ──
                Recipe {
                    name: "Corn Seed (x3)",
                    inputs: vec![(ItemType::PlantFiber, 2), (ItemType::Berry, 1)],
                    output: (ItemType::CornSeed, 3),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Potato Seed (x3)",
                    inputs: vec![(ItemType::PlantFiber, 3)],
                    output: (ItemType::PotatoSeed, 3),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Melon Seed (x2)",
                    inputs: vec![(ItemType::PlantFiber, 3), (ItemType::Berry, 2)],
                    output: (ItemType::MelonSeed, 2),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Rice Seed (x3)",
                    inputs: vec![(ItemType::PlantFiber, 2), (ItemType::Reed, 1)],
                    output: (ItemType::RiceSeed, 3),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Pepper Seed (x3)",
                    inputs: vec![(ItemType::PlantFiber, 2), (ItemType::Berry, 1)],
                    output: (ItemType::PepperSeed, 3),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Onion Seed (x3)",
                    inputs: vec![(ItemType::PlantFiber, 3)],
                    output: (ItemType::OnionSeed, 3),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Flax Seed (x3)",
                    inputs: vec![(ItemType::PlantFiber, 3), (ItemType::Reed, 1)],
                    output: (ItemType::FlaxSeed, 3),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Sugarcane Seed (x2)",
                    inputs: vec![(ItemType::PlantFiber, 4)],
                    output: (ItemType::SugarcaneSeed, 2),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                // ── Expansion: New cooked foods (Campfire) ──
                Recipe {
                    name: "Roasted Corn",
                    inputs: vec![(ItemType::Corn, 2)],
                    output: (ItemType::RoastedCorn, 1),
                    tier: CraftingTier::Campfire,
                    tech_key: None,
                },
                Recipe {
                    name: "Baked Potato",
                    inputs: vec![(ItemType::Potato, 1)],
                    output: (ItemType::BakedPotato, 1),
                    tier: CraftingTier::Campfire,
                    tech_key: None,
                },
                Recipe {
                    name: "Melon Slice",
                    inputs: vec![(ItemType::Melon, 1)],
                    output: (ItemType::MelonSlice, 2),
                    tier: CraftingTier::Campfire,
                    tech_key: None,
                },
                Recipe {
                    name: "Cooked Rice",
                    inputs: vec![(ItemType::Rice, 2)],
                    output: (ItemType::CookedRice, 1),
                    tier: CraftingTier::Campfire,
                    tech_key: None,
                },
                Recipe {
                    name: "Roasted Pepper",
                    inputs: vec![(ItemType::Pepper, 2)],
                    output: (ItemType::RoastedPepper, 1),
                    tier: CraftingTier::Campfire,
                    tech_key: None,
                },
                Recipe {
                    name: "Cooked Onion",
                    inputs: vec![(ItemType::Onion, 1)],
                    output: (ItemType::CookedOnion, 1),
                    tier: CraftingTier::Campfire,
                    tech_key: None,
                },
                Recipe {
                    name: "Sugar",
                    inputs: vec![(ItemType::Sugarcane, 2)],
                    output: (ItemType::Sugar, 1),
                    tier: CraftingTier::Campfire,
                    tech_key: None,
                },
                // ── Expansion: Cooked fish (Campfire) ──
                Recipe {
                    name: "Cooked Trout",
                    inputs: vec![(ItemType::RawTrout, 1)],
                    output: (ItemType::CookedTrout, 1),
                    tier: CraftingTier::Campfire,
                    tech_key: None,
                },
                Recipe {
                    name: "Cooked Salmon",
                    inputs: vec![(ItemType::RawSalmon, 1)],
                    output: (ItemType::CookedSalmon, 1),
                    tier: CraftingTier::Campfire,
                    tech_key: None,
                },
                Recipe {
                    name: "Cooked Catfish",
                    inputs: vec![(ItemType::RawCatfish, 1)],
                    output: (ItemType::CookedCatfish, 1),
                    tier: CraftingTier::Campfire,
                    tech_key: None,
                },
                Recipe {
                    name: "Cooked Eel",
                    inputs: vec![(ItemType::RawEel, 1)],
                    output: (ItemType::CookedEel, 1),
                    tier: CraftingTier::Campfire,
                    tech_key: None,
                },
                Recipe {
                    name: "Crab Meat",
                    inputs: vec![(ItemType::RawCrab, 1)],
                    output: (ItemType::CrabMeat, 1),
                    tier: CraftingTier::Campfire,
                    tech_key: None,
                },
                // ── Expansion: Workbench recipes ──
                Recipe {
                    name: "Fishing Rod",
                    inputs: vec![(ItemType::Stick, 3), (ItemType::Rope, 2)],
                    output: (ItemType::FishingRod, 1),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Pet Collar",
                    inputs: vec![(ItemType::Rope, 2), (ItemType::IronIngot, 1)],
                    output: (ItemType::PetCollar, 1),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Linen Cloth",
                    inputs: vec![(ItemType::Flax, 3)],
                    output: (ItemType::LinenCloth, 1),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Fish Bait (x5)",
                    inputs: vec![(ItemType::PlantFiber, 2), (ItemType::Berry, 1)],
                    output: (ItemType::FishBait, 5),
                    tier: CraftingTier::Hand,
                    tech_key: None,
                },
                Recipe {
                    name: "Pet Food (x3)",
                    inputs: vec![(ItemType::Berry, 2), (ItemType::Wheat, 1)],
                    output: (ItemType::PetFood, 3),
                    tier: CraftingTier::Campfire,
                    tech_key: None,
                },
                // ── Expansion: Forge recipes ──
                Recipe {
                    name: "Enchanting Table",
                    inputs: vec![(ItemType::IronIngot, 8), (ItemType::CrystalShard, 4), (ItemType::Gemstone, 2)],
                    output: (ItemType::EnchantingTable, 1),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Steel Fishing Rod",
                    inputs: vec![(ItemType::SteelAlloy, 2), (ItemType::Rope, 2), (ItemType::Stick, 2)],
                    output: (ItemType::SteelFishingRod, 1),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Fish Smoker",
                    inputs: vec![(ItemType::StoneBlock, 6), (ItemType::IronIngot, 2), (ItemType::Coal, 2)],
                    output: (ItemType::FishSmoker, 1),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Display Case",
                    inputs: vec![(ItemType::WoodPlank, 8), (ItemType::CrystalShard, 2)],
                    output: (ItemType::DisplayCase, 1),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Pet House",
                    inputs: vec![(ItemType::WoodPlank, 10), (ItemType::Rope, 2), (ItemType::PlantFiber, 4)],
                    output: (ItemType::PetHouse, 1),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                // ── Expansion: Enchanting essences (AdvancedForge) ──
                Recipe {
                    name: "Fire Essence",
                    inputs: vec![(ItemType::Sulfur, 3), (ItemType::MagmaCore, 1)],
                    output: (ItemType::FireEssence, 1),
                    tier: CraftingTier::AdvancedForge,
                    tech_key: None,
                },
                Recipe {
                    name: "Ice Essence",
                    inputs: vec![(ItemType::IceShard, 3), (ItemType::FrostGem, 1)],
                    output: (ItemType::IceEssence, 1),
                    tier: CraftingTier::AdvancedForge,
                    tech_key: None,
                },
                Recipe {
                    name: "Venom Essence",
                    inputs: vec![(ItemType::Spore, 3), (ItemType::SwampEssence, 1)],
                    output: (ItemType::VenomEssence, 1),
                    tier: CraftingTier::AdvancedForge,
                    tech_key: None,
                },
                Recipe {
                    name: "Life Essence",
                    inputs: vec![(ItemType::RareHerb, 3), (ItemType::GuardianHeart, 1)],
                    output: (ItemType::LifeEssence, 1),
                    tier: CraftingTier::AdvancedForge,
                    tech_key: None,
                },
                // ── Expansion: Enchanted weapons (Ancient tier) ──
                Recipe {
                    name: "Flame Blade",
                    inputs: vec![(ItemType::SteelSword, 1), (ItemType::FireEssence, 1)],
                    output: (ItemType::FlameBlade, 1),
                    tier: CraftingTier::Ancient,
                    tech_key: None,
                },
                Recipe {
                    name: "Frost Blade",
                    inputs: vec![(ItemType::SteelSword, 1), (ItemType::IceEssence, 1)],
                    output: (ItemType::FrostBlade, 1),
                    tier: CraftingTier::Ancient,
                    tech_key: None,
                },
                Recipe {
                    name: "Venom Blade",
                    inputs: vec![(ItemType::SteelSword, 1), (ItemType::VenomEssence, 1)],
                    output: (ItemType::VenomBlade, 1),
                    tier: CraftingTier::Ancient,
                    tech_key: None,
                },
                Recipe {
                    name: "Lifesteal Blade",
                    inputs: vec![(ItemType::SteelSword, 1), (ItemType::LifeEssence, 1)],
                    output: (ItemType::LifestealBlade, 1),
                    tier: CraftingTier::Ancient,
                    tech_key: None,
                },
                // ── Wave 6: New Furniture ──
                Recipe {
                    name: "Lantern",
                    inputs: vec![(ItemType::IronIngot, 3), (ItemType::Torch, 1)],
                    output: (ItemType::Lantern, 1),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Bookshelf",
                    inputs: vec![(ItemType::WoodPlank, 8)],
                    output: (ItemType::Bookshelf, 1),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Weapon Rack",
                    inputs: vec![(ItemType::IronIngot, 4), (ItemType::WoodPlank, 2)],
                    output: (ItemType::WeaponRack, 1),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Cooking Pot",
                    inputs: vec![(ItemType::IronIngot, 5), (ItemType::Stone, 2)],
                    output: (ItemType::CookingPot, 1),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Rain Collector",
                    inputs: vec![(ItemType::IronIngot, 4), (ItemType::WoodPlank, 2)],
                    output: (ItemType::RainCollector, 1),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                Recipe {
                    name: "Trophy Mount",
                    inputs: vec![(ItemType::WoodPlank, 3), (ItemType::IronIngot, 1)],
                    output: (ItemType::TrophyMount, 1),
                    tier: CraftingTier::Workbench,
                    tech_key: None,
                },
                // ── Wave 6: Automation ──
                Recipe {
                    name: "Auto-Smelter",
                    inputs: vec![(ItemType::IronIngot, 8), (ItemType::Stone, 4), (ItemType::Coal, 1)],
                    output: (ItemType::AutoSmelterItem, 1),
                    tier: CraftingTier::AdvancedForge,
                    tech_key: None,
                },
                Recipe {
                    name: "Crop Sprinkler",
                    inputs: vec![(ItemType::IronIngot, 4), (ItemType::WoodPlank, 2)],
                    output: (ItemType::CropSprinklerItem, 1),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
                Recipe {
                    name: "Alarm Bell",
                    inputs: vec![(ItemType::IronIngot, 3), (ItemType::Gemstone, 1)],
                    output: (ItemType::AlarmBellItem, 1),
                    tier: CraftingTier::Forge,
                    tech_key: None,
                },
            ],
            is_open: false,
            selected_recipe: 0,
        }
    }

    pub fn can_craft(&self, recipe_index: usize, inventory: &Inventory) -> bool {
        if recipe_index >= self.recipes.len() {
            return false;
        }
        let recipe = &self.recipes[recipe_index];
        recipe.inputs.iter().all(|(item, count)| inventory.has_items(*item, *count))
    }

    pub fn craft(&self, recipe_index: usize, inventory: &mut Inventory) -> bool {
        if !self.can_craft(recipe_index, inventory) {
            return false;
        }
        let recipe = &self.recipes[recipe_index];
        for (item, count) in &recipe.inputs {
            inventory.remove_items(*item, *count);
        }
        let (output_item, output_count) = recipe.output;
        inventory.add_item(output_item, output_count);
        true
    }

    /// Get indices of recipes available given current tier access and tech tree unlocks.
    /// Recipes that pass station proximity (used for UI to show locked + unlocked).
    pub fn recipes_visible_at_stations(
        &self,
        near_workbench: bool,
        near_forge: bool,
        near_campfire: bool,
        near_advanced_forge: bool,
        near_ancient: bool,
        tech_tree: &TechTree,
    ) -> Vec<(usize, bool)> {
        self.recipes.iter().enumerate()
            .filter_map(|(i, r)| {
                let tier_ok = match r.tier {
                    CraftingTier::Hand => true,
                    CraftingTier::Workbench => near_workbench,
                    CraftingTier::Forge => near_forge,
                    CraftingTier::Campfire => near_campfire,
                    CraftingTier::AdvancedForge => near_advanced_forge,
                    CraftingTier::Ancient => near_ancient,
                };
                if !tier_ok {
                    return None;
                }
                let locked = r.tech_key.map(|k| !tech_tree.is_unlocked(k)).unwrap_or(false);
                Some((i, locked))
            })
            .collect()
    }

    pub fn available_recipes(
        &self,
        near_workbench: bool,
        near_forge: bool,
        near_campfire: bool,
        near_advanced_forge: bool,
        near_ancient: bool,
        tech_tree: &TechTree,
    ) -> Vec<usize> {
        self.recipes_visible_at_stations(
            near_workbench, near_forge, near_campfire,
            near_advanced_forge, near_ancient, tech_tree,
        )
        .into_iter()
        .filter(|(_, locked)| !locked)
        .map(|(i, _)| i)
        .collect()
    }
}

/// Proximity radius (in pixels) within which the player can use a placed crafting station.
const STATION_RANGE: f32 = 64.0;

fn handle_crafting_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut crafting: ResMut<CraftingSystem>,
    mut inventory: ResMut<Inventory>,
    mut tech_tree: ResMut<TechTree>,
    station_query: Query<(&CraftingStation, &Transform)>,
    player_query: Query<&Transform, With<Player>>,
    mut sound_events: EventWriter<SoundEvent>,
    mut particle_events: EventWriter<crate::particles::SpawnParticlesEvent>,
) {
    if keyboard.just_pressed(KeyCode::KeyC) {
        crafting.is_open = !crafting.is_open;
        crafting.selected_recipe = 0;
    }

    if !crafting.is_open {
        return;
    }

    let mut near_workbench = false;
    let mut near_forge = false;
    let mut near_campfire = false;
    let mut near_advanced_forge = false;
    let mut near_ancient = false;

    if let Ok(player_tf) = player_query.get_single() {
        let player_pos = player_tf.translation.truncate();
        for (station, tf) in station_query.iter() {
            let dist = player_pos.distance(tf.translation.truncate());
            if dist <= STATION_RANGE {
                match station.tier {
                    CraftingTier::Workbench => near_workbench = true,
                    CraftingTier::Forge => near_forge = true,
                    CraftingTier::Campfire => near_campfire = true,
                    CraftingTier::AdvancedForge => near_advanced_forge = true,
                    CraftingTier::Ancient => near_ancient = true,
                    CraftingTier::Hand => {}
                }
            }
        }
    }

    let visible = crafting.recipes_visible_at_stations(
        near_workbench, near_forge, near_campfire,
        near_advanced_forge, near_ancient, &tech_tree,
    );

    if visible.is_empty() {
        return;
    }

    if crafting.selected_recipe >= visible.len() {
        crafting.selected_recipe = 0;
    }

    if keyboard.just_pressed(KeyCode::ArrowUp) && crafting.selected_recipe > 0 {
        crafting.selected_recipe -= 1;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) && crafting.selected_recipe < visible.len() - 1 {
        crafting.selected_recipe += 1;
    }

    let (focused_recipe_idx, focused_locked) = visible[crafting.selected_recipe];

    if keyboard.just_pressed(KeyCode::KeyU) && focused_locked {
        if let Some(key) = crafting.recipes[focused_recipe_idx].tech_key {
            if tech_tree.spend_rp_to_unlock(key) {
                sound_events.send(SoundEvent::Craft);
            }
        }
    }

    if keyboard.just_pressed(KeyCode::Enter) && !focused_locked {
        if crafting.craft(focused_recipe_idx, &mut inventory) {
            sound_events.send(SoundEvent::Craft);
            if let Ok(player_tf) = player_query.get_single() {
                particle_events.send(crate::particles::SpawnParticlesEvent {
                    position: player_tf.translation.truncate(),
                    color: Color::srgba(0.85, 0.75, 0.5, 0.9),
                    count: 5,
                });
            }
        }
    }
}
