use bevy::prelude::*;
use crate::inventory::{Inventory, ItemType};
use crate::player::Player;
use crate::building::{Building, BuildingType};

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
}

impl CraftingTier {
    pub fn label(&self) -> &str {
        match self {
            CraftingTier::Hand => "[Hand]",
            CraftingTier::Workbench => "[Bench]",
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
    pub fn available_recipes(&self, near_workbench: bool) -> Vec<usize> {
        self.recipes.iter().enumerate()
            .filter(|(_, r)| match r.tier {
                CraftingTier::Hand => true,
                CraftingTier::Workbench => near_workbench,
            })
            .map(|(i, _)| i)
            .collect()
    }
}

fn handle_crafting_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut crafting: ResMut<CraftingSystem>,
    mut inventory: ResMut<Inventory>,
    player_query: Query<&Transform, With<Player>>,
    workbench_query: Query<&Transform, With<Building>>,
    building_query: Query<&Building>,
) {
    if keyboard.just_pressed(KeyCode::KeyC) {
        crafting.is_open = !crafting.is_open;
        crafting.selected_recipe = 0;
    }

    if !crafting.is_open {
        return;
    }

    // Check if near a workbench
    let near_workbench = if let Ok(player_tf) = player_query.get_single() {
        let player_pos = player_tf.translation.truncate();
        workbench_query.iter().any(|tf| {
            let dist = player_pos.distance(tf.translation.truncate());
            if dist <= 48.0 {
                // Check it's actually a workbench (not just any building)
                // We check all buildings at this position
                true
            } else {
                false
            }
        }) && building_query.iter().any(|b| b.building_type == BuildingType::WoodFloor)
            // Actually, workbenches are placed via the building system as Workbench type
            // but currently workbench is an inventory item placed as a building.
            // For now, check if any workbench item is placed nearby.
            // The current building system only has WoodFloor/Wall/Door/Roof.
            // Workbench is crafted as an item but not yet placeable as a building.
            // We'll use a simpler approach: check if player has a workbench in inventory
            // OR is near a placed workbench. Since workbench placement isn't implemented yet,
            // check inventory for now.
            || inventory.has_items(ItemType::Workbench, 1)
    } else {
        false
    };

    let available = crafting.available_recipes(near_workbench);

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
