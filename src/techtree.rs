use bevy::prelude::*;
use std::collections::HashSet;
use rand::seq::SliceRandom;
use crate::combat::ResearchPointEvent;
use crate::inventory::{Inventory, ItemType};

pub struct TechTreePlugin;

impl Plugin for TechTreePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TechTree::default())
            .add_systems(Update, (
                accumulate_research_points,
                use_blueprint,
            ));
    }
}

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// All known recipe names (used as the pool of things to unlock).
/// These mirror the recipe keys used in crafting.rs.
const ALL_RECIPES: &[&str] = &[
    "advanced_forge",
    "alchemy_lab",
    "ancient_workstation",
    "steel_sword",
    "steel_axe",
    "steel_pickaxe",
    "steel_armor",
    "ancient_blade",
    "ancient_armor",
    "ancient_pickaxe",
    "health_potion",
    "speed_potion",
    "strength_potion",
];

#[derive(Resource)]
pub struct TechTree {
    /// Names of recipes that have been unlocked via research / blueprints.
    pub unlocked_recipes: HashSet<String>,
    /// Accumulated research points.
    pub research_points: u32,
}

impl Default for TechTree {
    fn default() -> Self {
        Self {
            unlocked_recipes: HashSet::new(),
            research_points: 0,
        }
    }
}

impl TechTree {
    /// Returns true if the given recipe name is unlocked.
    pub fn is_unlocked(&self, recipe: &str) -> bool {
        self.unlocked_recipes.contains(recipe)
    }

    /// Unlock a recipe by name.
    pub fn unlock(&mut self, recipe: impl Into<String>) {
        self.unlocked_recipes.insert(recipe.into());
    }

    /// Return a random recipe name that is not yet unlocked, or None if all
    /// recipes are already unlocked.
    pub fn random_locked_recipe(&self) -> Option<&'static str> {
        let locked: Vec<&'static str> = ALL_RECIPES
            .iter()
            .copied()
            .filter(|r| !self.unlocked_recipes.contains(*r))
            .collect();

        let mut rng = rand::thread_rng();
        locked.choose(&mut rng).copied()
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Listen for `ResearchPointEvent`s fired by combat and crafting and add the
/// points to the `TechTree` resource.
///
/// Combat awards +5 RP per regular kill and +20 RP per boss kill.
/// Crafting can fire this event to award +1 RP per craft (handled in
/// crafting.rs — this system just processes whatever events arrive).
fn accumulate_research_points(
    mut events: EventReader<ResearchPointEvent>,
    mut tech: ResMut<TechTree>,
) {
    for ev in events.read() {
        tech.research_points = tech.research_points.saturating_add(ev.amount);
    }
}

/// When the player right-clicks while holding a Blueprint item, consume one
/// Blueprint from the inventory and unlock a random locked recipe.
fn use_blueprint(
    mouse: Res<ButtonInput<MouseButton>>,
    mut inventory: ResMut<Inventory>,
    mut tech: ResMut<TechTree>,
) {
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    // Check that the currently selected hotbar slot holds a Blueprint.
    let is_blueprint = inventory
        .selected_item()
        .map(|s| s.item == ItemType::Blueprint)
        .unwrap_or(false);

    if !is_blueprint {
        return;
    }

    // Try to find a locked recipe to unlock.
    if let Some(recipe) = tech.random_locked_recipe() {
        let recipe_name = recipe.to_string();
        inventory.remove_items(ItemType::Blueprint, 1);
        tech.unlock(recipe_name.clone());
        info!("Blueprint used: unlocked recipe '{}'", recipe_name);
    } else {
        // All recipes already unlocked; still consume the blueprint.
        inventory.remove_items(ItemType::Blueprint, 1);
        info!("Blueprint used: all recipes already unlocked.");
    }
}
