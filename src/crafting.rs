use bevy::prelude::*;
use crate::inventory::{Inventory, ItemType};
use crate::player::Player;

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
}

impl CraftingTier {
    pub fn label(&self) -> &str {
        match self {
            CraftingTier::Hand => "[Hand]",
            CraftingTier::Workbench => "[Bench]",
            CraftingTier::Forge => "[Forge]",
            CraftingTier::Campfire => "[Fire]",
        }
    }
}

pub struct Recipe {
    pub name: &'static str,
    pub inputs: Vec<(ItemType, u32)>,
    pub output: (ItemType, u32),
    pub tier: CraftingTier,
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
                },
                Recipe {
                    name: "Wood Plank (x4)",
                    inputs: vec![(ItemType::Wood, 2)],
                    output: (ItemType::WoodPlank, 4),
                    tier: CraftingTier::Hand,
                },
                Recipe {
                    name: "Rope",
                    inputs: vec![(ItemType::PlantFiber, 3)],
                    output: (ItemType::Rope, 1),
                    tier: CraftingTier::Hand,
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
                },
                Recipe {
                    name: "Wood Floor",
                    inputs: vec![(ItemType::WoodPlank, 4)],
                    output: (ItemType::WoodFloor, 1),
                    tier: CraftingTier::Hand,
                },
                Recipe {
                    name: "Workbench",
                    inputs: vec![
                        (ItemType::WoodPlank, 8),
                        (ItemType::Stick, 4),
                    ],
                    output: (ItemType::Workbench, 1),
                    tier: CraftingTier::Hand,
                },
                Recipe {
                    name: "Stone Block",
                    inputs: vec![(ItemType::Stone, 4)],
                    output: (ItemType::StoneBlock, 1),
                    tier: CraftingTier::Hand,
                },
                // Workbench tier (10 new recipes)
                Recipe {
                    name: "Wood Wall",
                    inputs: vec![(ItemType::WoodPlank, 4), (ItemType::Stick, 2)],
                    output: (ItemType::WoodWall, 1),
                    tier: CraftingTier::Workbench,
                },
                Recipe {
                    name: "Wood Door",
                    inputs: vec![(ItemType::WoodPlank, 6)],
                    output: (ItemType::WoodDoor, 1),
                    tier: CraftingTier::Workbench,
                },
                Recipe {
                    name: "Wood Roof",
                    inputs: vec![(ItemType::WoodPlank, 6), (ItemType::Stick, 4)],
                    output: (ItemType::WoodRoof, 1),
                    tier: CraftingTier::Workbench,
                },
                Recipe {
                    name: "Wood Fence",
                    inputs: vec![(ItemType::Stick, 6)],
                    output: (ItemType::WoodFence, 2),
                    tier: CraftingTier::Workbench,
                },
                Recipe {
                    name: "Chest",
                    inputs: vec![(ItemType::WoodPlank, 8)],
                    output: (ItemType::Chest, 1),
                    tier: CraftingTier::Workbench,
                },
                Recipe {
                    name: "Torch",
                    inputs: vec![(ItemType::Stick, 1), (ItemType::PlantFiber, 2)],
                    output: (ItemType::Torch, 4),
                    tier: CraftingTier::Workbench,
                },
                Recipe {
                    name: "Wood Sword",
                    inputs: vec![(ItemType::WoodPlank, 3), (ItemType::Stick, 1)],
                    output: (ItemType::WoodSword, 1),
                    tier: CraftingTier::Workbench,
                },
                Recipe {
                    name: "Wood Shield",
                    inputs: vec![(ItemType::WoodPlank, 6), (ItemType::Rope, 1)],
                    output: (ItemType::WoodShield, 1),
                    tier: CraftingTier::Workbench,
                },
                Recipe {
                    name: "Wood Bow",
                    inputs: vec![(ItemType::Stick, 3), (ItemType::Rope, 2)],
                    output: (ItemType::WoodBow, 1),
                    tier: CraftingTier::Workbench,
                },
                Recipe {
                    name: "Arrow (x8)",
                    inputs: vec![(ItemType::Stick, 2), (ItemType::Flint, 1)],
                    output: (ItemType::Arrow, 8),
                    tier: CraftingTier::Workbench,
                },
                // Workbench — Tier 3 stations
                Recipe {
                    name: "Forge",
                    inputs: vec![(ItemType::StoneBlock, 10), (ItemType::IronOre, 5), (ItemType::Coal, 3)],
                    output: (ItemType::Forge, 1),
                    tier: CraftingTier::Workbench,
                },
                Recipe {
                    name: "Anvil",
                    inputs: vec![(ItemType::IronIngot, 8), (ItemType::StoneBlock, 4)],
                    output: (ItemType::Anvil, 1),
                    tier: CraftingTier::Workbench,
                },
                // Forge tier recipes
                Recipe {
                    name: "Iron Ingot",
                    inputs: vec![(ItemType::IronOre, 2), (ItemType::Coal, 1)],
                    output: (ItemType::IronIngot, 1),
                    tier: CraftingTier::Forge,
                },
                Recipe {
                    name: "Steel Alloy",
                    inputs: vec![(ItemType::IronIngot, 2), (ItemType::Coal, 2)],
                    output: (ItemType::SteelAlloy, 1),
                    tier: CraftingTier::Forge,
                },
                Recipe {
                    name: "Iron Axe",
                    inputs: vec![(ItemType::IronIngot, 3), (ItemType::Stick, 2)],
                    output: (ItemType::IronAxe, 1),
                    tier: CraftingTier::Forge,
                },
                Recipe {
                    name: "Iron Pickaxe",
                    inputs: vec![(ItemType::IronIngot, 3), (ItemType::Stick, 2)],
                    output: (ItemType::IronPickaxe, 1),
                    tier: CraftingTier::Forge,
                },
                Recipe {
                    name: "Iron Sword",
                    inputs: vec![(ItemType::IronIngot, 4), (ItemType::Stick, 1)],
                    output: (ItemType::IronSword, 1),
                    tier: CraftingTier::Forge,
                },
                Recipe {
                    name: "Iron Shield",
                    inputs: vec![(ItemType::IronIngot, 5), (ItemType::WoodPlank, 2)],
                    output: (ItemType::IronShield, 1),
                    tier: CraftingTier::Forge,
                },
                Recipe {
                    name: "Iron Helmet",
                    inputs: vec![(ItemType::IronIngot, 4)],
                    output: (ItemType::IronHelmet, 1),
                    tier: CraftingTier::Forge,
                },
                Recipe {
                    name: "Iron Chestplate",
                    inputs: vec![(ItemType::IronIngot, 6)],
                    output: (ItemType::IronChestplate, 1),
                    tier: CraftingTier::Forge,
                },
                Recipe {
                    name: "Stone Wall",
                    inputs: vec![(ItemType::StoneBlock, 4)],
                    output: (ItemType::StoneWall, 1),
                    tier: CraftingTier::Forge,
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
                },
                // Hand tier — seeds (craft from plant fiber / gathered items)
                Recipe {
                    name: "Wheat Seed (x3)",
                    inputs: vec![(ItemType::PlantFiber, 2)],
                    output: (ItemType::WheatSeed, 3),
                    tier: CraftingTier::Hand,
                },
                Recipe {
                    name: "Carrot Seed (x3)",
                    inputs: vec![(ItemType::PlantFiber, 2), (ItemType::Berry, 1)],
                    output: (ItemType::CarrotSeed, 3),
                    tier: CraftingTier::Hand,
                },
                // Campfire cooking recipes
                Recipe {
                    name: "Cooked Berry",
                    inputs: vec![(ItemType::Berry, 2)],
                    output: (ItemType::CookedBerry, 1),
                    tier: CraftingTier::Campfire,
                },
                Recipe {
                    name: "Baked Wheat",
                    inputs: vec![(ItemType::Wheat, 2)],
                    output: (ItemType::BakedWheat, 1),
                    tier: CraftingTier::Campfire,
                },
                Recipe {
                    name: "Cooked Carrot",
                    inputs: vec![(ItemType::Carrot, 1)],
                    output: (ItemType::CookedCarrot, 1),
                    tier: CraftingTier::Campfire,
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

    /// Get indices of recipes available given current tier access
    pub fn available_recipes(&self, near_workbench: bool, near_forge: bool, near_campfire: bool) -> Vec<usize> {
        self.recipes.iter().enumerate()
            .filter(|(_, r)| match r.tier {
                CraftingTier::Hand => true,
                CraftingTier::Workbench => near_workbench,
                CraftingTier::Forge => near_forge,
                CraftingTier::Campfire => near_campfire,
            })
            .map(|(i, _)| i)
            .collect()
    }
}

fn handle_crafting_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut crafting: ResMut<CraftingSystem>,
    mut inventory: ResMut<Inventory>,
) {
    if keyboard.just_pressed(KeyCode::KeyC) {
        crafting.is_open = !crafting.is_open;
        crafting.selected_recipe = 0;
    }

    if !crafting.is_open {
        return;
    }

    // Check crafting station access (simplified: check inventory for station items)
    let near_workbench = inventory.has_items(ItemType::Workbench, 1);
    let near_forge = inventory.has_items(ItemType::Forge, 1);
    let near_campfire = inventory.has_items(ItemType::Campfire, 1);

    let available = crafting.available_recipes(near_workbench, near_forge, near_campfire);

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
        crafting.craft(actual_idx, &mut inventory);
    }
}
