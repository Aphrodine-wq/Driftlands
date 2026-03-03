use bevy::prelude::*;
use crate::inventory::{Inventory, ItemType};
use crate::crafting::CraftingSystem;
use crate::daynight::DayNightCycle;
use crate::building::BuildingState;
use crate::player::{Player, Health, Hunger, ActiveBuff, BuffType, ArmorSlots};
use crate::saveload::SaveMessage;
use crate::season::SeasonCycle;
use crate::weather::WeatherSystem;
use crate::npc::{TradeMenu, Trader, HermitDialogueDisplay};
use crate::lore::{LoreRegistry, LoreMessage};
use crate::experiment::{ExperimentSlots, ExperimentMessage};
use crate::techtree::TechTree;

#[derive(Resource, Default)]
pub struct PauseState {
    pub paused: bool,
}

/// Run condition: returns `true` when the game is NOT paused.
/// Use with `.run_if(not_paused)` to gate gameplay systems.
pub fn not_paused(pause: Res<PauseState>) -> bool {
    !pause.paused
}

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PauseState::default())
            .add_systems(Startup, spawn_hud)
            .add_systems(Update, (
                toggle_pause,
                update_hud,
                update_status_hud,
                update_npc_hud,
                update_feedback_hud,
            ));
    }
}

#[derive(Component)]
pub struct HudText;

#[derive(Component)]
pub struct CraftingHudText;

#[derive(Component)]
pub struct StatusHudText;

/// The right-side panel used for the trade / experiment UIs.
#[derive(Component)]
pub struct NpcHudText;

/// Full-width bottom bar for lore / hermit / experiment feedback.
#[derive(Component)]
pub struct FeedbackHudText;

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

    // NPC / experiment panel — right side, below crafting
    commands.spawn((
        NpcHudText,
        Text::new(""),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.6)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(280.0),
            ..default()
        },
    ));

    // Feedback bar — bottom of screen
    commands.spawn((
        FeedbackHudText,
        Text::new(""),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.7)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(40.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}

fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pause_state: ResMut<PauseState>,
    mut cycle: ResMut<crate::daynight::DayNightCycle>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        pause_state.paused = !pause_state.paused;
        cycle.paused = pause_state.paused;
    }
}

fn update_status_hud(
    player_query: Query<(&Health, &Hunger, Option<&ActiveBuff>), With<Player>>,
    mut status_query: Query<(&mut Text, &mut TextColor), With<StatusHudText>>,
    save_msg: Res<SaveMessage>,
    armor: Res<ArmorSlots>,
    inventory: Res<Inventory>,
) {
    let Ok((health, hunger, active_buff)) = player_query.get_single() else { return };
    let Ok((mut text, mut color)) = status_query.get_single_mut() else { return };

    let hp_color = if health.current < health.max * 0.25 { "!!" } else { "" };
    let hunger_color = if hunger.current < hunger.max * 0.3 { "!!" } else { "" };

    let atk = inventory.selected_item()
        .and_then(|s| s.item.weapon_damage())
        .unwrap_or(5.0);

    let mut lines = vec![
        format!("HP: {:.0}/{:.0} {}  Armor: {}  ATK: {:.0}", health.current, health.max, hp_color, armor.total_armor(), atk),
        format!("Hunger: {:.0}/{:.0} {}", hunger.current, hunger.max, hunger_color),
    ];

    // Show active buff in status area
    if let Some(buff) = active_buff {
        let buff_name = match buff.buff_type {
            BuffType::Speed => "Speed",
            BuffType::Strength => "Strength",
        };
        lines.push(format!("[BUFF] {} +{:.0}% ({:.0}s)", buff_name, (buff.magnitude - 1.0) * 100.0, buff.remaining));
    }

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
    season: Res<SeasonCycle>,
    weather: Res<WeatherSystem>,
    lore_registry: Res<LoreRegistry>,
    pause_state: Res<PauseState>,
    tech_tree: Res<TechTree>,
    mut hud_query: Query<&mut Text, (With<HudText>, Without<CraftingHudText>, Without<StatusHudText>, Without<NpcHudText>, Without<FeedbackHudText>)>,
    mut craft_hud_query: Query<&mut Text, (With<CraftingHudText>, Without<HudText>, Without<StatusHudText>, Without<NpcHudText>, Without<FeedbackHudText>)>,
) {
    // Main HUD
    if let Ok(mut text) = hud_query.get_single_mut() {
        if pause_state.paused {
            let lines = vec![
                "".to_string(),
                "=== PAUSED ===".to_string(),
                "".to_string(),
                "[ESC] Resume".to_string(),
                "[F5] Save Game".to_string(),
                "".to_string(),
                "--- Controls ---".to_string(),
                "[WASD] Move".to_string(),
                "[LClick] Gather / Attack".to_string(),
                "[RClick] Use Item / Eat / Place".to_string(),
                "[B] Build Mode  [Q] Cycle Type".to_string(),
                "[C] Crafting Menu".to_string(),
                "[I/Tab] Inventory".to_string(),
                "[E] Interact (Door/NPC/Bed)".to_string(),
                "[R] Equip Armor/Shield".to_string(),
                "[X] Experiment Table".to_string(),
                "[+/-] Zoom".to_string(),
                "[F5] Save  [F9] Load".to_string(),
            ];
            **text = lines.join("\n");
            return;
        }

        let hour = ((cycle.time_of_day * 24.0) as u32) % 24;
        let time_str = if hour < 12 {
            format!("{}AM", if hour == 0 { 12 } else { hour })
        } else {
            format!("{}PM", if hour == 12 { 12 } else { hour - 12 })
        };
        let season_day = ((cycle.day_count.saturating_sub(1)) % 5) + 1;
        let weather_color = match weather.current {
            crate::weather::Weather::Clear => "",
            crate::weather::Weather::Rain => "[Rain]",
            crate::weather::Weather::Snow => "[Snow]",
            crate::weather::Weather::Storm => "!!STORM!!",
        };

        let mut lines = vec![
            format!("Day {} | {} {} | {} (Day {}/5) | {}",
                cycle.day_count,
                cycle.phase_name(),
                time_str,
                season.current.name(),
                season_day,
                if weather_color.is_empty() { weather.current.name().to_string() } else { weather_color.to_string() },
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
        lines.push(format!(
            "Lore: {}/{}",
            lore_registry.collected_entries.len(),
            lore_registry.total_entries
        ));
        lines.push("[WASD] Move  [LClick] Gather/Attack  [B] Build  [C] Craft  [I] Inventory".into());
        lines.push("[E] Interact  [X] Experiment  [F5] Save  [F9] Load".into());

        **text = lines.join("\n");
    }

    // Crafting HUD
    if let Ok(mut text) = craft_hud_query.get_single_mut() {
        if !crafting.is_open {
            **text = String::new();
            return;
        }

        let near_workbench = inventory.has_items(ItemType::Workbench, 1);
        let near_forge = inventory.has_items(ItemType::Forge, 1);
        let near_campfire = inventory.has_items(ItemType::Campfire, 1);
        let near_advanced_forge = inventory.has_items(ItemType::AdvancedForge, 1);
        let near_ancient = inventory.has_items(ItemType::AncientWorkstation, 1);
        let available = crafting.available_recipes(near_workbench, near_forge, near_campfire, near_advanced_forge, near_ancient, &tech_tree);

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

/// Renders the trader trade menu or the experiment UI on the secondary right-side panel.
fn update_npc_hud(
    trade_menu: Res<TradeMenu>,
    experiment_slots: Res<ExperimentSlots>,
    inventory: Res<Inventory>,
    trader_query: Query<&Trader>,
    mut npc_hud_query: Query<&mut Text, With<NpcHudText>>,
) {
    let Ok(mut text) = npc_hud_query.get_single_mut() else { return };

    // Experiment UI takes priority if open
    if experiment_slots.is_open {
        let slot_a_name = experiment_slots.slot_a
            .map(|i| i.display_name().to_string())
            .unwrap_or_else(|| "---".to_string());
        let slot_b_name = experiment_slots.slot_b
            .map(|i| i.display_name().to_string())
            .unwrap_or_else(|| "---".to_string());

        let lines = vec![
            "== EXPERIMENT TABLE ==".to_string(),
            String::new(),
            format!("Slot A: {}", slot_a_name),
            format!("Slot B: {}", slot_b_name),
            String::new(),
            "[1] Assign selected item to Slot A".to_string(),
            "[2] Assign selected item to Slot B".to_string(),
            "[Enter] Attempt combination".to_string(),
            "[X] Close".to_string(),
        ];
        **text = lines.join("\n");
        return;
    }

    // Trade menu
    if trade_menu.is_open {
        if let Some(entity) = trade_menu.trader_entity {
            if let Ok(trader) = trader_query.get(entity) {
                let mut lines = vec![
                    "== WANDERING TRADER ==".to_string(),
                    String::new(),
                ];

                for (i, offer) in trader.offers.iter().enumerate() {
                    let marker = if i == trade_menu.selected_offer { "> " } else { "  " };
                    let status = if offer.sold {
                        " [SOLD]".to_string()
                    } else {
                        let can_afford = inventory.has_items(offer.cost_item, offer.cost_count);
                        if can_afford { String::new() } else { " [need more]".to_string() }
                    };
                    lines.push(format!(
                        "{}{}  for {} x{}{}",
                        marker,
                        offer.item_for_sale.display_name(),
                        offer.cost_item.display_name(),
                        offer.cost_count,
                        status,
                    ));
                }

                lines.push(String::new());
                lines.push("[Up/Down] Select  [Enter] Buy  [Esc] Close".to_string());
                **text = lines.join("\n");
                return;
            }
        }
    }

    **text = String::new();
}

/// Shows ephemeral feedback messages: lore discoveries, hermit dialogue, experiment results.
fn update_feedback_hud(
    lore_msg: Res<LoreMessage>,
    hermit_display: Res<HermitDialogueDisplay>,
    experiment_msg: Res<ExperimentMessage>,
    mut feedback_query: Query<&mut Text, With<FeedbackHudText>>,
) {
    let Ok(mut text) = feedback_query.get_single_mut() else { return };

    // Priority order: experiment > lore > hermit
    if !experiment_msg.text.is_empty() {
        **text = experiment_msg.text.clone();
    } else if !lore_msg.text.is_empty() {
        **text = lore_msg.text.clone();
    } else if !hermit_display.text.is_empty() {
        **text = hermit_display.text.clone();
    } else {
        **text = String::new();
    }
}
