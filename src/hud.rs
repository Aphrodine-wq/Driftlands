use bevy::prelude::*;
use crate::inventory::Inventory;
use crate::crafting::{CraftingSystem, CraftingTier};
use crate::daynight::DayNightCycle;
use crate::building::{BuildingState, ChestStorage, ChestUI, CraftingStation};
use crate::player::{Player, Health, Hunger, ActiveBuff, BuffType, ArmorSlots};
use crate::saveload::SaveMessage;
use crate::season::SeasonCycle;
use crate::weather::WeatherSystem;
use crate::npc::{TradeMenu, Trader, HermitDialogueDisplay};
use crate::controls::ControlsOverlay;
use crate::lore::{LoreRegistry, LoreMessage};
use crate::experiment::{ExperimentSlots, ExperimentMessage};
use crate::techtree::TechTree;
use crate::world::generation::Biome;
use crate::world::chunk::Chunk;
use crate::world::{CHUNK_WORLD_SIZE};
use crate::mainmenu::MainMenuActive;

#[derive(Resource, Default)]
pub struct PauseState {
    pub paused: bool,
}

/// Run condition: returns `true` when the game is NOT paused and the main menu is not active.
/// Use with `.run_if(not_paused)` to gate gameplay systems.
pub fn not_paused(pause: Res<PauseState>, menu: Res<MainMenuActive>) -> bool {
    !pause.paused && !menu.active
}

/// Tracks the biome the player is currently standing in.
#[derive(Resource, Default)]
pub struct CurrentBiome {
    pub biome: Option<Biome>,
    /// Countdown timer for displaying the biome name banner.
    pub display_timer: f32,
}

/// Marker for the biome name banner text entity.
#[derive(Component)]
pub struct BiomeBannerText;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PauseState::default())
            .insert_resource(CurrentBiome::default())
            .add_systems(Startup, spawn_hud)
            .add_systems(Update, (
                toggle_pause,
                update_hud,
                update_status_hud,
                update_npc_hud,
                update_feedback_hud,
                update_inventory_panel,
                track_player_biome,
                update_biome_banner,
                floating_text_system,
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

/// Inventory grid panel (US-012) — toggled with Tab/I.
#[derive(Component)]
pub struct InventoryPanelText;

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

    // Inventory grid panel (US-012) — center of screen
    commands.spawn((
        InventoryPanelText,
        Text::new(""),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.95, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(80.0),
            left: Val::Percent(30.0),
            ..default()
        },
    ));

    // Biome name banner — large centered text (US-029)
    commands.spawn((
        BiomeBannerText,
        Text::new(""),
        TextFont {
            font_size: 36.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(30.0),
            left: Val::Percent(50.0),
            ..default()
        },
    ));
}

fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pause_state: ResMut<PauseState>,
    mut cycle: ResMut<crate::daynight::DayNightCycle>,
    chest_ui: Res<ChestUI>,
    trade_menu: Res<TradeMenu>,
    menu: Res<MainMenuActive>,
    controls_overlay: Res<ControlsOverlay>,
) {
    // Don't toggle pause while main menu is active
    if menu.active {
        return;
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        // Don't toggle pause when closing a modal UI with Escape
        if chest_ui.is_open || trade_menu.is_open || controls_overlay.is_visible {
            return;
        }
        pause_state.paused = !pause_state.paused;
        cycle.paused = pause_state.paused;
    }
}

/// Build an ASCII bar: 20 characters wide using Unicode block chars.
/// `fraction` should be in 0.0..=1.0.
fn ascii_bar(fraction: f32) -> String {
    let width = 20;
    let filled = ((fraction.clamp(0.0, 1.0) * width as f32).round()) as usize;
    let empty = width - filled;
    let mut bar = String::with_capacity(width + 2);
    bar.push('[');
    for _ in 0..filled {
        bar.push('\u{2588}'); // █
    }
    for _ in 0..empty {
        bar.push('\u{2591}'); // ░
    }
    bar.push(']');
    bar
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

    let hp_frac = if health.max > 0.0 { health.current / health.max } else { 0.0 };
    let hunger_frac = if hunger.max > 0.0 { hunger.current / hunger.max } else { 0.0 };

    let hp_warn = if health.current < health.max * 0.25 { " !!" } else { "" };
    let hunger_warn = if hunger.current < hunger.max * 0.3 { " !!" } else { "" };

    let atk = inventory.selected_item()
        .and_then(|s| s.item.weapon_damage())
        .unwrap_or(5.0);

    let mut lines = vec![
        format!("HP   {} {:.0}/{:.0}{}  Armor: {}  ATK: {:.0}",
            ascii_bar(hp_frac), health.current, health.max, hp_warn, armor.total_armor(), atk),
        format!("FOOD {} {:.0}/{:.0}{}",
            ascii_bar(hunger_frac), hunger.current, hunger.max, hunger_warn),
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
    mut hud_query: Query<&mut Text, (With<HudText>, Without<CraftingHudText>, Without<StatusHudText>, Without<NpcHudText>, Without<FeedbackHudText>, Without<InventoryPanelText>)>,
    mut craft_hud_query: Query<&mut Text, (With<CraftingHudText>, Without<HudText>, Without<StatusHudText>, Without<NpcHudText>, Without<FeedbackHudText>, Without<InventoryPanelText>)>,
    station_query: Query<(&CraftingStation, &Transform), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
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

        // Visual hotbar: each slot is fixed-width with brackets and selection indicator
        let mut hotbar_top = String::new();
        let mut hotbar_sel = String::new();
        for i in 0..inventory.hotbar_size {
            let is_selected = i == inventory.selected_slot;
            let content = match &inventory.slots[i] {
                Some(slot) => {
                    // Abbreviate item name to first 4 chars
                    let name: String = slot.item.display_name().chars().take(4).collect();
                    if let Some(dur) = slot.durability {
                        let max_dur = slot.item.max_durability().unwrap_or(dur);
                        format!("{}:{}/{}", name, dur, max_dur)
                    } else if slot.count > 1 {
                        format!("{}x{}", name, slot.count)
                    } else {
                        name
                    }
                }
                None => "    ".to_string(),
            };
            // Pad or truncate content to 9 chars for uniform slot width
            let padded = format!("{:<9}", content);
            let padded = if padded.len() > 9 {
                padded.chars().take(9).collect::<String>()
            } else {
                padded
            };
            if is_selected {
                hotbar_top.push_str(&format!(">>{}:{}<< ", i + 1, padded));
                hotbar_sel.push_str(&format!("  ^^^{}   ", " ".repeat(padded.len().saturating_sub(3))));
            } else {
                hotbar_top.push_str(&format!("[{}:{}] ", i + 1, padded));
                hotbar_sel.push_str(&format!(" {}  ", " ".repeat(padded.len() + 2)));
            }
        }
        lines.push(hotbar_top.trim_end().to_string());
        lines.push(hotbar_sel.trim_end().to_string());

        // When the inventory panel is open, show a hint on the main HUD
        if inventory.is_open {
            lines.push(String::new());
            lines.push("[Inventory open — see grid panel]".into());
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

    // Crafting HUD (US-013 — improved with ingredient availability)
    if let Ok(mut text) = craft_hud_query.get_single_mut() {
        if !crafting.is_open {
            **text = String::new();
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
                if dist <= 64.0 {
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

        let mut lines = vec!["=== CRAFTING (C to close) ===".to_string()];

        // Show available station tiers
        {
            let mut stations = vec!["Hand"];
            if near_workbench { stations.push("Workbench"); }
            if near_campfire { stations.push("Campfire"); }
            if near_forge { stations.push("Forge"); }
            if near_advanced_forge { stations.push("AdvForge"); }
            if near_ancient { stations.push("Ancient"); }
            lines.push(format!("Stations: {}", stations.join(", ")));
        }
        lines.push(String::new());

        for (display_idx, &recipe_idx) in available.iter().enumerate() {
            let recipe = &crafting.recipes[recipe_idx];
            let is_selected = display_idx == crafting.selected_recipe;
            let sel_marker = if is_selected { "> " } else { "  " };

            if is_selected {
                let craftable = crafting.can_craft(recipe_idx, &inventory);
                let craft_tag = if craftable { " [READY]" } else { "" };
                lines.push(format!("{}{}{} [SELECTED]", sel_marker, recipe.name, craft_tag));
            } else {
                let can_craft = crafting.can_craft(recipe_idx, &inventory);
                let status = if can_craft { "" } else { " [missing]" };
                lines.push(format!("{}{} {}{}", sel_marker, recipe.tier.label(), recipe.name, status));
            }

            // Show ingredients for selected recipe, or compact view for all
            if is_selected {
                for (item, count) in &recipe.inputs {
                    let have = inventory.count_items(*item);
                    let has_enough = have >= *count;
                    let mark = if has_enough { "\u{2713}" } else { "\u{2717}" }; // checkmark / cross
                    lines.push(format!("    {} {} x{} (have {})", mark, item.display_name(), count, have));
                }
                // Show output
                let (out_item, out_count) = recipe.output;
                if out_count > 1 {
                    lines.push(format!("    -> {} x{}", out_item.display_name(), out_count));
                } else {
                    lines.push(format!("    -> {}", out_item.display_name()));
                }
            }
        }

        if available.is_empty() {
            lines.push("  (no recipes available)".to_string());
        }

        lines.push(String::new());
        lines.push("[Up/Down] Select  [Enter] Craft  [C] Close".into());

        **text = lines.join("\n");
    }
}

/// Renders the chest UI, trader trade menu, or experiment UI on the secondary right-side panel.
fn update_npc_hud(
    trade_menu: Res<TradeMenu>,
    experiment_slots: Res<ExperimentSlots>,
    chest_ui: Res<ChestUI>,
    inventory: Res<Inventory>,
    trader_query: Query<&Trader>,
    chest_query: Query<&ChestStorage>,
    mut npc_hud_query: Query<&mut Text, With<NpcHudText>>,
) {
    let Ok(mut text) = npc_hud_query.get_single_mut() else { return };

    // Chest UI takes highest priority if open
    if chest_ui.is_open {
        if let Some(entity) = chest_ui.target_entity {
            if let Ok(chest) = chest_query.get(entity) {
                let mut lines = vec![
                    "=== CHEST ===".to_string(),
                    String::new(),
                ];

                for (i, slot) in chest.slots.iter().enumerate() {
                    let marker = if i == chest_ui.selected_slot { "> " } else { "  " };
                    let slot_text = match slot {
                        Some(s) => {
                            if let Some(dur) = s.durability {
                                let max_dur = s.item.max_durability().unwrap_or(dur);
                                format!("{}{:2}. {} ({}/{})", marker, i + 1, s.item.display_name(), dur, max_dur)
                            } else {
                                format!("{}{:2}. {} x{}", marker, i + 1, s.item.display_name(), s.count)
                            }
                        }
                        None => format!("{}{:2}. (empty)", marker, i + 1),
                    };
                    lines.push(slot_text);
                }

                lines.push(String::new());
                lines.push("1-9: Store hotbar item | Up/Down+Enter: Take | E: Close".to_string());
                **text = lines.join("\n");
                return;
            }
        }
    }

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

/// US-012 — Renders a grid-style inventory panel when the inventory is open.
fn update_inventory_panel(
    inventory: Res<Inventory>,
    mut panel_query: Query<&mut Text, With<InventoryPanelText>>,
) {
    let Ok(mut text) = panel_query.get_single_mut() else { return };

    if !inventory.is_open {
        **text = String::new();
        return;
    }

    let mut lines = Vec::new();
    lines.push("=== INVENTORY (Tab to close) ===".to_string());
    lines.push(String::new());

    let slots_per_row = 9;
    for row_start in (0..inventory.slots.len()).step_by(slots_per_row) {
        let row_end = (row_start + slots_per_row).min(inventory.slots.len());
        let mut row = String::new();
        for i in row_start..row_end {
            let slot_num = i + 1;
            let cell = match &inventory.slots[i] {
                Some(slot) => {
                    // Abbreviate name to 8 chars
                    let name: String = slot.item.display_name().chars().take(8).collect();
                    if let Some(dur) = slot.durability {
                        let max_dur = slot.item.max_durability().unwrap_or(dur);
                        format!("{:02}: {:<8} {}/{}", slot_num, name, dur, max_dur)
                    } else if slot.count > 1 {
                        format!("{:02}: {:<8} x{}", slot_num, name, slot.count)
                    } else {
                        format!("{:02}: {:<8}    ", slot_num, name)
                    }
                }
                None => {
                    format!("{:02}: --------    ", slot_num)
                }
            };
            // Mark selected slot
            let prefix = if i == inventory.selected_slot { ">" } else { " " };
            row.push_str(&format!("{}[{}] ", prefix, cell));
        }
        lines.push(row.trim_end().to_string());
    }

    lines.push(String::new());
    lines.push("[1-9] Select hotbar  [Tab/I] Close".to_string());

    **text = lines.join("\n");
}

/// Returns a human-readable name for a biome.
fn biome_display_name(biome: Biome) -> &'static str {
    match biome {
        Biome::Forest => "Forest",
        Biome::Coastal => "Coastal",
        Biome::Swamp => "Swamp",
        Biome::Desert => "Desert",
        Biome::Tundra => "Tundra",
        Biome::Volcanic => "Volcanic Wastes",
        Biome::Fungal => "Fungal Groves",
        Biome::CrystalCave => "Crystal Caverns",
        Biome::Mountain => "Mountains",
    }
}

/// Determines which biome the player is currently standing in and starts
/// the banner timer whenever the biome changes.
fn track_player_biome(
    player_query: Query<&Transform, With<Player>>,
    chunk_query: Query<&Chunk>,
    mut current_biome: ResMut<CurrentBiome>,
) {
    let Ok(player_tf) = player_query.get_single() else { return };

    let chunk_x = (player_tf.translation.x / CHUNK_WORLD_SIZE).floor() as i32;
    let chunk_y = (player_tf.translation.y / CHUNK_WORLD_SIZE).floor() as i32;

    for chunk in chunk_query.iter() {
        if chunk.position.x == chunk_x && chunk.position.y == chunk_y {
            let new_biome = chunk.biome;
            if current_biome.biome != Some(new_biome) {
                current_biome.biome = Some(new_biome);
                current_biome.display_timer = 3.0; // Show banner for 3 seconds
            }
            return;
        }
    }
}

/// Fades the biome banner text over 3 seconds and hides it when done.
fn update_biome_banner(
    time: Res<Time>,
    mut current_biome: ResMut<CurrentBiome>,
    mut banner_query: Query<(&mut Text, &mut TextColor), With<BiomeBannerText>>,
) {
    let Ok((mut text, mut color)) = banner_query.get_single_mut() else { return };

    if current_biome.display_timer > 0.0 {
        // Set text to biome name
        if let Some(biome) = current_biome.biome {
            **text = biome_display_name(biome).to_string();
        }

        // Fade: full opacity for first 2 seconds, then fade out over the last 1 second
        let alpha = if current_biome.display_timer > 1.0 {
            1.0
        } else {
            current_biome.display_timer
        };
        *color = TextColor(Color::srgba(1.0, 1.0, 1.0, alpha));

        current_biome.display_timer -= time.delta_secs();
    } else {
        // Hide banner
        *color = TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0));
        **text = String::new();
    }
}

// --- US-028: Floating Text ---

/// World-space floating text that drifts upward and fades out.
/// Used for damage numbers, item pickup notifications, etc.
#[derive(Component)]
pub struct FloatingText {
    pub timer: f32,
    pub max_timer: f32,
    pub velocity: Vec2,
}

/// Spawns a floating text entity in world space at the given position.
pub fn spawn_floating_text(commands: &mut Commands, text: &str, position: Vec2, color: Color) {
    commands.spawn((
        FloatingText {
            timer: 1.5,
            max_timer: 1.5,
            velocity: Vec2::new(0.0, 30.0),
        },
        Text2d::new(text.to_string()),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(color),
        Transform::from_xyz(position.x, position.y + 8.0, 100.0),
    ));
}

/// Moves floating text upward, fades alpha, and despawns when expired.
fn floating_text_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut FloatingText, &mut Transform, &mut TextColor)>,
) {
    let dt = time.delta_secs();
    for (entity, mut ft, mut tf, mut color) in query.iter_mut() {
        // Move upward
        tf.translation.x += ft.velocity.x * dt;
        tf.translation.y += ft.velocity.y * dt;

        // Decrease timer
        ft.timer -= dt;

        // Fade alpha based on remaining time
        let alpha = (ft.timer / ft.max_timer).clamp(0.0, 1.0);
        let c = color.0.to_srgba();
        color.0 = Color::srgba(c.red, c.green, c.blue, alpha);

        // Despawn when done
        if ft.timer <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
