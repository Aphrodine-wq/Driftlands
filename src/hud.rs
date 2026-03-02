use bevy::prelude::*;
use crate::inventory::{Inventory, ItemType};
use crate::crafting::CraftingSystem;
use crate::daynight::DayNightCycle;
use crate::building::BuildingState;
use crate::player::{Player, Health, Hunger};
use crate::saveload::SaveMessage;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hud)
            .add_systems(Update, (update_hud, update_status_hud));
    }
}

#[derive(Component)]
pub struct HudText;

#[derive(Component)]
pub struct CraftingHudText;

#[derive(Component)]
pub struct StatusHudText;

fn spawn_hud(mut commands: Commands) {
    // Status HUD (HP/Hunger) — top-left
    commands.spawn((
        StatusHudText,
        Text::new(""),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    // Main HUD — below status
    commands.spawn((
        HudText,
        Text::new("Driftlands"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(60.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    // Crafting menu — right side
    commands.spawn((
        CraftingHudText,
        Text::new(""),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
    ));
}

fn update_status_hud(
    player_query: Query<(&Health, &Hunger), With<Player>>,
    mut status_query: Query<(&mut Text, &mut TextColor), With<StatusHudText>>,
    save_msg: Res<SaveMessage>,
) {
    let Ok((health, hunger)) = player_query.get_single() else { return };
    let Ok((mut text, mut color)) = status_query.get_single_mut() else { return };

    let hp_color = if health.current < health.max * 0.25 { "!!" } else { "" };
    let hunger_color = if hunger.current < hunger.max * 0.3 { "!!" } else { "" };

    let mut lines = vec![
        format!("HP: {:.0}/{:.0} {}", health.current, health.max, hp_color),
        format!("Hunger: {:.0}/{:.0} {}", hunger.current, hunger.max, hunger_color),
    ];

    if !save_msg.text.is_empty() {
        lines.push(String::new());
        lines.push(save_msg.text.clone());
    }

    **text = lines.join("\n");

    if health.current < health.max * 0.25 {
        *color = TextColor(Color::srgb(1.0, 0.2, 0.2));
    } else if hunger.current < hunger.max * 0.3 {
        *color = TextColor(Color::srgb(1.0, 1.0, 0.3));
    } else {
        *color = TextColor(Color::WHITE);
    }
}

fn update_hud(
    inventory: Res<Inventory>,
    crafting: Res<CraftingSystem>,
    cycle: Res<DayNightCycle>,
    building_state: Res<BuildingState>,
    mut hud_query: Query<&mut Text, (With<HudText>, Without<CraftingHudText>, Without<StatusHudText>)>,
    mut craft_hud_query: Query<&mut Text, (With<CraftingHudText>, Without<HudText>, Without<StatusHudText>)>,
) {
    // Main HUD
    if let Ok(mut text) = hud_query.get_single_mut() {
        let mut lines = vec![
            format!("Day {} | {} | {:.0}%",
                cycle.day_count,
                cycle.phase_name(),
                cycle.time_of_day * 100.0,
            ),
            String::new(),
        ];

        if building_state.active {
            lines.push(format!("[BUILD MODE] {} | [Q] Cycle | [RClick] Place", building_state.selected_type.name()));
            lines.push(String::new());
        }

        lines.push("-- Hotbar --".into());
        for i in 0..inventory.hotbar_size {
            let marker = if i == inventory.selected_slot { ">" } else { " " };
            let slot_text = match &inventory.slots[i] {
                Some(slot) => {
                    if let Some(dur) = slot.durability {
                        let max_dur = slot.item.max_durability().unwrap_or(dur);
                        format!("{} [{}] {} ({}/{})", marker, i + 1, slot.item.display_name(), dur, max_dur)
                    } else {
                        format!("{} [{}] {} x{}", marker, i + 1, slot.item.display_name(), slot.count)
                    }
                }
                None => format!("{} [{}] ---", marker, i + 1),
            };
            lines.push(slot_text);
        }

        // Show non-hotbar items if inventory is open
        if inventory.is_open {
            lines.push(String::new());
            lines.push("-- Inventory --".into());
            for i in inventory.hotbar_size..inventory.slots.len() {
                if let Some(slot) = &inventory.slots[i] {
                    lines.push(format!("  {} x{}", slot.item.display_name(), slot.count));
                }
            }
        }

        lines.push(String::new());
        lines.push("[WASD] Move  [LClick] Gather/Attack  [B] Build  [C] Craft  [I] Inventory".into());
        lines.push("[E] Open Door  [F5] Save  [F9] Load".into());

        **text = lines.join("\n");
    }

    // Crafting HUD
    if let Ok(mut text) = craft_hud_query.get_single_mut() {
        if !crafting.is_open {
            **text = String::new();
            return;
        }

        // Determine workbench access (simplified: check inventory for now)
        let near_workbench = inventory.has_items(ItemType::Workbench, 1);
        let available = crafting.available_recipes(near_workbench);

        let mut lines = vec!["== CRAFTING ==".to_string(), String::new()];

        for (display_idx, &recipe_idx) in available.iter().enumerate() {
            let recipe = &crafting.recipes[recipe_idx];
            let selected = if display_idx == crafting.selected_recipe { "> " } else { "  " };
            let can_craft = if crafting.can_craft(recipe_idx, &inventory) { "" } else { " [missing]" };

            lines.push(format!("{}{} {} {}", selected, recipe.tier.label(), recipe.name, can_craft));

            if display_idx == crafting.selected_recipe {
                for (item, count) in &recipe.inputs {
                    let has = inventory.has_items(*item, *count);
                    let mark = if has { "+" } else { "x" };
                    lines.push(format!("    {} {} x{}", mark, item.display_name(), count));
                }
            }
        }

        lines.push(String::new());
        lines.push("[Up/Down] Select  [Enter] Craft  [C] Close".into());

        **text = lines.join("\n");
    }
}
