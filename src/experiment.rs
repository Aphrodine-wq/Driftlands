use bevy::prelude::*;
use crate::inventory::{Inventory, ItemType};
use crate::crafting::CraftingSystem;
use crate::building::BuildingState;

pub struct ExperimentPlugin;

impl Plugin for ExperimentPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ExperimentSlots::default())
            .insert_resource(ExperimentMessage::default())
            .add_systems(Update, (
                toggle_experiment_ui,
                assign_experiment_slot,
                attempt_combination,
                tick_experiment_message,
            ));
    }
}

// ── Hidden recipes ────────────────────────────────────────────────────────────

struct ExperimentRecipe {
    slot_a: ItemType,
    slot_b: ItemType,
    output: ItemType,
}

const HIDDEN_RECIPES: [ExperimentRecipe; 5] = [
    ExperimentRecipe { slot_a: ItemType::Berry,        slot_b: ItemType::MushroomCap,  output: ItemType::HealthPotion  },
    ExperimentRecipe { slot_a: ItemType::IceShard,     slot_b: ItemType::Sulfur,       output: ItemType::Gemstone      },
    ExperimentRecipe { slot_a: ItemType::Spore,        slot_b: ItemType::CactusFiber,  output: ItemType::SpeedPotion   },
    ExperimentRecipe { slot_a: ItemType::Coal,         slot_b: ItemType::RareHerb,     output: ItemType::StrengthPotion},
    ExperimentRecipe { slot_a: ItemType::ObsidianShard, slot_b: ItemType::CrystalShard, output: ItemType::AncientCore  },
];

// ── Resources ─────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct ExperimentSlots {
    pub slot_a: Option<ItemType>,
    pub slot_b: Option<ItemType>,
    pub is_open: bool,
}

#[derive(Resource, Default)]
pub struct ExperimentMessage {
    pub text: String,
    pub timer: f32,
}

// ── Systems ───────────────────────────────────────────────────────────────────

fn toggle_experiment_ui(
    keyboard: Res<ButtonInput<KeyCode>>,
    building_state: Res<BuildingState>,
    crafting: Res<CraftingSystem>,
    mut slots: ResMut<ExperimentSlots>,
) {
    if building_state.active || crafting.is_open {
        return;
    }

    if keyboard.just_pressed(KeyCode::KeyX) {
        slots.is_open = !slots.is_open;
    }
}

/// Number keys 1 and 2 assign the currently selected hotbar item to slot A or B.
fn assign_experiment_slot(
    keyboard: Res<ButtonInput<KeyCode>>,
    inventory: Res<Inventory>,
    mut slots: ResMut<ExperimentSlots>,
) {
    if !slots.is_open {
        return;
    }

    let selected_item = inventory.selected_item().map(|s| s.item);

    if keyboard.just_pressed(KeyCode::Digit1) {
        slots.slot_a = selected_item;
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        slots.slot_b = selected_item;
    }
}

fn attempt_combination(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut slots: ResMut<ExperimentSlots>,
    mut inventory: ResMut<Inventory>,
    mut message: ResMut<ExperimentMessage>,
) {
    if !slots.is_open {
        return;
    }

    if !keyboard.just_pressed(KeyCode::Enter) {
        return;
    }

    let (Some(a), Some(b)) = (slots.slot_a, slots.slot_b) else {
        message.text = "Place items in both slots first.".to_string();
        message.timer = 2.0;
        return;
    };

    // Check that the player actually has both items
    if !inventory.has_items(a, 1) || !inventory.has_items(b, 1) {
        message.text = "You don't have those items!".to_string();
        message.timer = 2.0;
        return;
    }

    // Search hidden recipes — order-agnostic
    let result = HIDDEN_RECIPES.iter().find(|r| {
        (r.slot_a == a && r.slot_b == b) || (r.slot_a == b && r.slot_b == a)
    });

    // Always consume the inputs
    inventory.remove_items(a, 1);
    // If a == b we need to remove another one of the same item
    if a == b {
        inventory.remove_items(a, 1);
    } else {
        inventory.remove_items(b, 1);
    }

    // Clear slots
    slots.slot_a = None;
    slots.slot_b = None;

    match result {
        Some(recipe) => {
            inventory.add_item(recipe.output, 1);
            message.text = format!("Discovery! You created: {}!", recipe.output.display_name());
            message.timer = 2.0;
        }
        None => {
            message.text = "Nothing happened...".to_string();
            message.timer = 2.0;
        }
    }
}

fn tick_experiment_message(
    time: Res<Time>,
    mut message: ResMut<ExperimentMessage>,
) {
    if message.timer > 0.0 {
        message.timer -= time.delta_secs();
        if message.timer <= 0.0 {
            message.text.clear();
        }
    }
}
