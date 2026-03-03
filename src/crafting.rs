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
    pub fn available_recipes(
        &self,
        near_workbench: bool,
        near_forge: bool,
        near_campfire: bool,
        near_advanced_forge: bool,
        near_ancient: bool,
        tech_tree: &TechTree,
    ) -> Vec<usize> {
        self.recipes.iter().enumerate()
            .filter(|(_, r)| {
                // First check crafting station proximity
                let tier_ok = match r.tier {
                    CraftingTier::Hand => true,
                    CraftingTier::Workbench => near_workbench,
                    CraftingTier::Forge => near_forge,
                    CraftingTier::Campfire => near_campfire,
                    CraftingTier::AdvancedForge => near_advanced_forge,
                    CraftingTier::Ancient => near_ancient,
                };
                if !tier_ok {
                    return false;
                }
                // Then check tech tree unlock requirement
                if let Some(key) = r.tech_key {
                    return tech_tree.is_unlocked(key);
                }
                true
            })
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
    tech_tree: Res<TechTree>,
    station_query: Query<(&CraftingStation, &Transform)>,
    player_query: Query<&Transform, With<Player>>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    if keyboard.just_pressed(KeyCode::KeyC) {
        crafting.is_open = !crafting.is_open;
        crafting.selected_recipe = 0;
    }

    if !crafting.is_open {
        return;
    }

    // Check crafting station access by proximity to placed buildings
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

    let available = crafting.available_recipes(near_workbench, near_forge, near_campfire, near_advanced_forge, near_ancient, &tech_tree);

    if available.is_empty() {
        return;
    }

    // Clamp selected to available range
    if crafting.selected_recipe >= available.len() {
        crafting.selected_recipe = 0;
    }

    if keyboard.just_pressed(KeyCode::ArrowUp) && crafting.selected_recipe > 0 {
        crafting.selected_recipe -= 1;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown)
        && crafting.selected_recipe < available.len() - 1
    {
        crafting.selected_recipe += 1;
    }

    if keyboard.just_pressed(KeyCode::Enter) {
        let actual_idx = available[crafting.selected_recipe];
        if crafting.craft(actual_idx, &mut inventory) {
            sound_events.send(SoundEvent::Craft);
        }
    }
}
