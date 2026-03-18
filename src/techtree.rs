use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
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

/// Recipes that can be unlocked by spending research points (recipe_key, cost).
pub const RP_UNLOCK_RECIPES: &[(&str, u32)] = &[
    ("advanced_forge", 15),
    ("alchemy_lab", 12),
    ("steel_sword", 8),
    ("steel_axe", 8),
    ("steel_pickaxe", 8),
    ("steel_armor", 10),
    ("health_potion", 5),
    ("speed_potion", 5),
    ("strength_potion", 5),
    ("ancient_workstation", 25),
    ("ancient_blade", 20),
    ("ancient_armor", 20),
    ("ancient_pickaxe", 20),
];

/// Prerequisite mappings: recipe_key -> list of recipe_keys that must be unlocked first.
/// Only recipes that have prerequisites are listed here.
pub fn tech_prerequisites() -> HashMap<&'static str, Vec<&'static str>> {
    let mut map = HashMap::new();
    // Steel gear requires advanced_forge
    map.insert("steel_sword", vec!["advanced_forge"]);
    map.insert("steel_axe", vec!["advanced_forge"]);
    map.insert("steel_pickaxe", vec!["advanced_forge"]);
    map.insert("steel_armor", vec!["advanced_forge"]);
    // Ancient gear requires ancient_workstation
    map.insert("ancient_blade", vec!["ancient_workstation"]);
    map.insert("ancient_armor", vec!["ancient_workstation"]);
    map.insert("ancient_pickaxe", vec!["ancient_workstation"]);
    // Potions require alchemy_lab
    map.insert("health_potion", vec!["alchemy_lab"]);
    map.insert("speed_potion", vec!["alchemy_lab"]);
    map.insert("strength_potion", vec!["alchemy_lab"]);
    map
}

#[derive(Resource)]
pub struct TechTree {
    /// Names of recipes that have been unlocked via research / blueprints.
    pub unlocked_recipes: HashSet<String>,
    /// Accumulated research points.
    pub research_points: u32,
    /// Prerequisite map: recipe_key -> required recipe_keys.
    pub prerequisites: HashMap<String, Vec<String>>,
}

impl Default for TechTree {
    fn default() -> Self {
        let prereqs = tech_prerequisites();
        Self {
            unlocked_recipes: HashSet::new(),
            research_points: 0,
            prerequisites: prereqs.into_iter()
                .map(|(k, v)| (k.to_string(), v.into_iter().map(|s| s.to_string()).collect()))
                .collect(),
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

    /// Human-readable unlock requirement for a recipe (for UI). Returns empty if unlocked.
    pub fn unlock_hint(&self, recipe_key: Option<&str>) -> String {
        let Some(key) = recipe_key else { return String::new() };
        if self.unlocked_recipes.contains(key) {
            return String::new();
        }
        if let Some(&(_, cost)) = RP_UNLOCK_RECIPES.iter().find(|(k, _)| *k == key) {
            return format!("{} RP", cost);
        }
        "Blueprint".to_string()
    }

    /// RP cost to unlock this recipe, if it is RP-gated.
    pub fn rp_cost(&self, recipe_key: &str) -> Option<u32> {
        RP_UNLOCK_RECIPES.iter().find(|(k, _)| *k == recipe_key).map(|(_, c)| *c)
    }

    /// Spend research points to unlock a recipe. Returns true if unlocked.
    /// Also checks prerequisites are met before allowing the unlock.
    pub fn spend_rp_to_unlock(&mut self, recipe_key: &str) -> bool {
        if self.unlocked_recipes.contains(recipe_key) {
            return false;
        }
        if !self.can_unlock(recipe_key) {
            return false;
        }
        if let Some(&(_, cost)) = RP_UNLOCK_RECIPES.iter().find(|(k, _)| *k == recipe_key) {
            if self.research_points >= cost {
                self.research_points -= cost;
                self.unlocked_recipes.insert(recipe_key.to_string());
                return true;
            }
        }
        false
    }

    /// Returns true if all prerequisites for this recipe are unlocked.
    pub fn can_unlock(&self, recipe_key: &str) -> bool {
        if let Some(prereqs) = self.prerequisites.get(recipe_key) {
            prereqs.iter().all(|p| self.unlocked_recipes.contains(p))
        } else {
            true
        }
    }

    /// Returns a human-readable string describing unmet prerequisites for this recipe.
    /// Returns empty string if all prerequisites are met or there are none.
    pub fn prerequisite_hint(&self, recipe_key: &str) -> String {
        if let Some(prereqs) = self.prerequisites.get(recipe_key) {
            let missing: Vec<&String> = prereqs.iter()
                .filter(|p| !self.unlocked_recipes.contains(p.as_str()))
                .collect();
            if missing.is_empty() {
                String::new()
            } else {
                let names: Vec<String> = missing.iter()
                    .map(|k| k.replace('_', " "))
                    .collect();
                format!("Requires: {}", names.join(", "))
            }
        } else {
            String::new()
        }
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
