use bevy::prelude::*;
use crate::inventory::Inventory;
use crate::crafting::{CraftingSystem, CraftingTier};
use crate::daynight::DayNightCycle;
use crate::building::{BuildingState, ChestStorage, ChestUI, CraftingStation};
use crate::player::{Player, Health, Hunger, ActiveBuff, BuffType, ArmorSlots};
use crate::saveload::SaveMessage;
use crate::season::SeasonCycle;
use crate::weather::WeatherSystem;
use crate::npc::{TradeMenu, Trader, HermitDialogueDisplay, NpcDialogueDisplay};
use crate::controls::ControlsOverlay;
use crate::lore::{LoreRegistry, LoreMessage};
use crate::experiment::{ExperimentSlots, ExperimentMessage};
use crate::techtree::TechTree;
use crate::world::generation::Biome;
use crate::world::chunk::Chunk;
use crate::world::{CHUNK_WORLD_SIZE};
use crate::mainmenu::MainMenuActive;
use crate::theme::EtherealTheme;
use crate::audio::SoundEvent;
use crate::fishing::{FishingState, FishingPhase, FishType};
use crate::quests::QuestLog;
use crate::pets::Pet;
use crate::status_effects::{ActiveStatusEffects, StatusEffectType};
use crate::skills::{SkillLevels, SkillType};

#[derive(Resource, Default)]
pub struct PauseState {
    pub paused: bool,
}

/// Run condition: returns `true` when the game is NOT paused and the main menu is not active.
pub fn not_paused(pause: Res<PauseState>, menu: Res<MainMenuActive>) -> bool {
    !pause.paused && !menu.active
}

#[derive(Resource, Default)]
pub struct CurrentBiome {
    pub biome: Option<Biome>,
    pub display_timer: f32,
}

/// Biomes the player has entered at least once (for first-time discovery feedback).
#[derive(Resource, Default)]
pub struct ExploredBiomes {
    pub set: std::collections::HashSet<Biome>,
}

/// Smoothed display values for health/hunger bars (lerp over ~0.2s).
#[derive(Resource)]
pub struct BarDisplayState {
    pub health_frac: f32,
    pub hunger_frac: f32,
}

impl Default for BarDisplayState {
    fn default() -> Self {
        Self { health_frac: 1.0, hunger_frac: 1.0 }
    }
}

#[derive(Component)]
pub struct BiomeBannerText;

pub struct HudPlugin;

/// Tracks the "Caught a Fish!" flash timer for the fishing HUD.
#[derive(Resource, Default)]
pub struct FishingCatchFlash {
    pub fish_name: String,
    pub timer: f32,
}

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FloatingTextRequest>()
            .insert_resource(PauseState::default())
            .insert_resource(CurrentBiome::default())
            .insert_resource(ExploredBiomes::default())
            .insert_resource(BarDisplayState::default())
            .insert_resource(FloatingTextQueue::default())
            .insert_resource(FishingCatchFlash::default())
            .add_systems(Startup, spawn_hud)
            .add_systems(Update, (
                toggle_pause,
                adjust_volume_when_paused,
                update_hud,
                update_status_hud,
                update_npc_hud,
                update_feedback_hud,
                update_inventory_panel,
                update_graphical_hotbar,
                track_player_biome,
                update_biome_banner,
                receive_floating_text_requests,
                drain_floating_text_queue,
                floating_text_system,
                update_fishing_hud,
                update_quest_log_hud,
                update_status_effects_hud,
                update_skill_hud,
            ));
    }
}

#[derive(Component)]
pub struct HudText;

#[derive(Component)]
pub struct CraftingHudText;

#[derive(Component)]
pub struct StatusHudText;

#[derive(Component)]
pub struct NpcHudText;

#[derive(Component)]
pub struct FeedbackHudText;

#[derive(Component)]
pub struct InventoryPanelText;

#[derive(Component)]
pub struct HealthBarFill;

#[derive(Component)]
pub struct HungerBarFill;

#[derive(Component)]
pub struct HotbarSlotUI {
    pub index: usize,
}

#[derive(Component)]
pub struct HotbarSlotColor;

#[derive(Component)]
pub struct HotbarSlotLabel;

#[derive(Component)]
pub struct HotbarTooltipText;

#[derive(Component)]
pub struct FishingHudText;

#[derive(Component)]
pub struct QuestLogHudText;

#[derive(Component)]
pub struct StatusEffectsHudText;

#[derive(Component)]
pub struct SkillHudText;

fn spawn_hud(mut commands: Commands, theme: Res<EtherealTheme>) {
    // Root UI container
    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    })
    .with_children(|parent| {
        // Status area: Top-Left (HP/Hunger bars + stats)
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(theme.panel_bg()),
            BorderColor(theme.panel_border(false)),
        ))
        .with_children(|status_root| {
            // HP Bar
            status_root.spawn((
                Node {
                    width: Val::Px(160.0),
                    height: Val::Px(12.0),
                    margin: UiRect::bottom(Val::Px(4.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.3, 0.05, 0.05, 0.6)),
                BorderColor(Color::srgba(0.5, 0.2, 0.2, 0.5)),
            ))
            .with_children(|bar| {
                bar.spawn((
                    HealthBarFill,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(theme.healing),
                ));
            });

            // Hunger Bar
            status_root.spawn((
                Node {
                    width: Val::Px(160.0),
                    height: Val::Px(12.0),
                    margin: UiRect::bottom(Val::Px(6.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.2, 0.15, 0.02, 0.6)),
                BorderColor(Color::srgba(0.4, 0.35, 0.1, 0.5)),
            ))
            .with_children(|bar| {
                bar.spawn((
                    HungerBarFill,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(theme.accent_gold),
                ));
            });

            // Status Text
            status_root.spawn((
                StatusHudText,
                Text::new(""),
                TextFont { font_size: 13.0, ..default() },
                TextColor(theme.hud_label_color()),
            ));
        });

        // Main HUD text (day info, build mode) - small, top-left under status
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(104.0),
                left: Val::Px(14.0),
                padding: UiRect::all(Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(theme.panel_bg()),
            BorderColor(theme.panel_border(false)),
        ))
        .with_children(|panel| {
            panel.spawn((
                HudText,
                Text::new(""),
                TextFont { font_size: 12.0, ..default() },
                TextColor(theme.hud_label_color()),
            ));
        });

        // Graphical Hotbar: Bottom-Center — 9 colored slots
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(14.0),
                left: Val::Percent(50.0),
                margin: UiRect::left(Val::Px(-210.0)), // Center: 9 slots * 42px + gaps / 2
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(6.0),
                padding: UiRect::all(Val::Px(5.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(theme.panel_bg()),
            BorderColor(theme.panel_border(false)),
        ))
        .with_children(|hotbar| {
            for i in 0..9 {
                hotbar.spawn((
                    HotbarSlotUI { index: i },
                    Node {
                        width: Val::Px(40.0),
                        height: Val::Px(40.0),
                        border: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::End,
                        padding: UiRect::all(Val::Px(2.0)),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.8)),
                    BorderColor(Color::srgba(0.25, 0.25, 0.35, 0.5)),
                ))
                .with_children(|slot| {
                    // Colored item indicator
                    slot.spawn((
                        HotbarSlotColor,
                        Node {
                            width: Val::Px(30.0),
                            height: Val::Px(24.0),
                            margin: UiRect::bottom(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                    ));
                    // Label (item name / count)
                    slot.spawn((
                        HotbarSlotLabel,
                        Text::new(""),
                        TextFont { font_size: 10.0, ..default() },
                        TextColor(theme.hud_label_color()),
                    ));
                });
            }
        });

        // Hotbar tooltip: selected item name (below hotbar)
        parent.spawn((
            HotbarTooltipText,
            Text::new(""),
            TextFont { font_size: 12.0, ..default() },
            TextColor(theme.hud_label_color()),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(58.0),
                left: Val::Percent(50.0),
                margin: UiRect::left(Val::Px(-100.0)),
                max_width: Val::Px(200.0),
                ..default()
            },
        ));

        // Crafting Menu: Right
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                right: Val::Px(12.0),
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                max_width: Val::Px(320.0),
                ..default()
            },
            BackgroundColor(theme.panel_bg()),
            BorderColor(theme.panel_border(false)),
        ))
        .with_children(|panel| {
            panel.spawn((
                CraftingHudText,
                Text::new(""),
                TextFont { font_size: 13.0, ..default() },
                TextColor(theme.hud_label_color()),
            ));
        });

        // NPC / Experiment Panel: Far-Right
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                right: Val::Px(346.0),
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(theme.panel_bg()),
            BorderColor(theme.panel_border(false)),
        ))
        .with_children(|panel| {
            panel.spawn((
                NpcHudText,
                Text::new(""),
                TextFont { font_size: 13.0, ..default() },
                TextColor(theme.hud_primary_text()),
            ));
        });

        // Feedback: Bottom-Left
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(52.0),
                left: Val::Px(12.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(theme.panel_bg()),
            BorderColor(theme.panel_border(false)),
        ))
        .with_children(|panel| {
            panel.spawn((
                FeedbackHudText,
                Text::new(""),
                TextFont { font_size: 14.0, ..default() },
                TextColor(theme.hud_primary_text()),
            ));
        });

        // Inventory Panel: Center
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(80.0),
                left: Val::Percent(20.0),
                padding: UiRect::all(Val::Px(12.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(theme.panel_bg()),
            BorderColor(theme.panel_border(false)),
        ))
        .with_children(|panel| {
            panel.spawn((
                InventoryPanelText,
                Text::new(""),
                TextFont { font_size: 13.0, ..default() },
                TextColor(theme.hud_label_color()),
            ));
        });

        // Biome Banner: Center-Top (no panel — just floating text)
        parent.spawn((
            BiomeBannerText,
            Text::new(""),
            TextFont { font_size: 32.0, ..default() },
            TextColor(theme.hud_primary_text()),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(20.0),
                left: Val::Percent(45.0),
                ..default()
            },
        ));

        // Fishing HUD: Bottom-center above hotbar
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(68.0),
                left: Val::Percent(50.0),
                margin: UiRect::left(Val::Px(-160.0)),
                max_width: Val::Px(320.0),
                padding: UiRect::all(Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(theme.panel_bg()),
            BorderColor(theme.panel_border(false)),
        ))
        .with_children(|panel| {
            panel.spawn((
                FishingHudText,
                Text::new(""),
                TextFont { font_size: 13.0, ..default() },
                TextColor(theme.hud_primary_text()),
            ));
        });

        // Quest Log HUD: Center panel (like inventory, toggled by J)
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(80.0),
                left: Val::Percent(25.0),
                padding: UiRect::all(Val::Px(12.0)),
                border: UiRect::all(Val::Px(2.0)),
                max_width: Val::Px(400.0),
                ..default()
            },
            BackgroundColor(theme.panel_bg()),
            BorderColor(theme.panel_border(false)),
        ))
        .with_children(|panel| {
            panel.spawn((
                QuestLogHudText,
                Text::new(""),
                TextFont { font_size: 13.0, ..default() },
                TextColor(theme.hud_label_color()),
            ));
        });

        // Status Effects HUD: Top-left below status panel
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(160.0),
                left: Val::Px(14.0),
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|panel| {
            panel.spawn((
                StatusEffectsHudText,
                Text::new(""),
                TextFont { font_size: 12.0, ..default() },
                TextColor(theme.hud_primary_text()),
            ));
        });

        // Skill Panel: Center-Left (toggled by K)
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(160.0),
                left: Val::Percent(5.0),
                padding: UiRect::all(Val::Px(12.0)),
                border: UiRect::all(Val::Px(2.0)),
                max_width: Val::Px(340.0),
                ..default()
            },
            BackgroundColor(theme.panel_bg()),
            BorderColor(theme.panel_border(false)),
        ))
        .with_children(|panel| {
            panel.spawn((
                SkillHudText,
                Text::new(""),
                TextFont { font_size: 13.0, ..default() },
                TextColor(theme.hud_label_color()),
            ));
        });
    });
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
    if menu.active { return; }
    if keyboard.just_pressed(KeyCode::Escape) {
        if chest_ui.is_open || trade_menu.is_open || controls_overlay.is_visible {
            return;
        }
        pause_state.paused = !pause_state.paused;
        cycle.paused = pause_state.paused;
    }
}

const BAR_LERP_SPEED: f32 = 5.0; // ~0.2s to catch up

fn update_status_hud(
    time: Res<Time>,
    player_query: Query<(&Health, &Hunger, Option<&ActiveBuff>), With<Player>>,
    mut status_query: Query<&mut Text, With<StatusHudText>>,
    mut health_fill_query: Query<&mut Node, (With<HealthBarFill>, Without<HungerBarFill>)>,
    mut hunger_fill_query: Query<&mut Node, (With<HungerBarFill>, Without<HealthBarFill>)>,
    mut bar_display: ResMut<BarDisplayState>,
    save_msg: Res<SaveMessage>,
    armor: Res<ArmorSlots>,
    inventory: Res<Inventory>,
    pet_query: Query<&Pet>,
    _theme: Res<EtherealTheme>,
) {
    let Ok((health, hunger, active_buff)) = player_query.get_single() else { return };

    let target_health = (health.current / health.max).clamp(0.0, 1.0);
    let target_hunger = (hunger.current / hunger.max).clamp(0.0, 1.0);
    let dt = time.delta_secs();
    bar_display.health_frac += (target_health - bar_display.health_frac) * (BAR_LERP_SPEED * dt).min(1.0);
    bar_display.hunger_frac += (target_hunger - bar_display.hunger_frac) * (BAR_LERP_SPEED * dt).min(1.0);

    if let Ok(mut node) = health_fill_query.get_single_mut() {
        node.width = Val::Percent(bar_display.health_frac * 100.0);
    }
    if let Ok(mut node) = hunger_fill_query.get_single_mut() {
        node.width = Val::Percent(bar_display.hunger_frac * 100.0);
    }

    let Ok(mut text) = status_query.get_single_mut() else { return };

    let atk = inventory.selected_item()
        .and_then(|s| s.item.weapon_damage())
        .unwrap_or(5.0);

    let mut lines = vec![
        format!("{:.0}/{:.0} HP | {:.0}/{:.0} FOOD", health.current, health.max, hunger.current, hunger.max),
        format!("ARMOR: {} | ATK: {:.0}", armor.total_armor(), atk),
    ];

    if let Some(buff) = active_buff {
        let buff_name = match buff.buff_type {
            BuffType::Speed => "Speed",
            BuffType::Strength => "Strength",
            BuffType::Regen => "Regen",
        };
        lines.push(format!("[{}] +{:.0}% ({:.0}s)", buff_name, (buff.magnitude - 1.0) * 100.0, buff.remaining));
    }

    // 1C: Pet Status
    if let Ok(pet) = pet_query.get_single() {
        let max_h = pet.pet_type.max_happiness();
        let frac = (pet.happiness / max_h).clamp(0.0, 1.0);
        let bar_len = 8;
        let filled = (frac * bar_len as f32).round() as usize;
        let bar: String = "=".repeat(filled) + &"-".repeat(bar_len - filled);
        let warning = if pet.happiness < 30.0 { " !!UNHAPPY!!" } else { "" };
        lines.push(format!("Pet: {} [{}] {:.0}%{}", pet.pet_type.display_name(), bar, frac * 100.0, warning));
    }

    if !save_msg.text.is_empty() {
        lines.push(save_msg.text.clone());
    }

    **text = lines.join("\n");
}

fn adjust_volume_when_paused(
    pause_state: Res<PauseState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_audio: ResMut<crate::audio::GameAudio>,
) {
    if !pause_state.paused {
        return;
    }
    if keyboard.just_pressed(KeyCode::Minus) {
        game_audio.sfx_volume = (game_audio.sfx_volume - 0.1).max(0.0);
    }
    if keyboard.just_pressed(KeyCode::Equal) {
        game_audio.sfx_volume = (game_audio.sfx_volume + 0.1).min(1.0);
    }
}

fn update_hud(
    inventory: Res<Inventory>,
    crafting: Res<CraftingSystem>,
    cycle: Res<DayNightCycle>,
    building_state: Res<BuildingState>,
    season: Res<SeasonCycle>,
    weather: Res<WeatherSystem>,
    _lore_registry: Res<LoreRegistry>,
    pause_state: Res<PauseState>,
    tech_tree: Res<TechTree>,
    game_audio: Res<crate::audio::GameAudio>,
    mut hud_query: Query<&mut Text, (With<HudText>, Without<CraftingHudText>, Without<StatusHudText>, Without<NpcHudText>, Without<FeedbackHudText>, Without<InventoryPanelText>)>,
    mut craft_hud_query: Query<&mut Text, (With<CraftingHudText>, Without<HudText>, Without<StatusHudText>, Without<NpcHudText>, Without<FeedbackHudText>, Without<InventoryPanelText>)>,
    station_query: Query<(&CraftingStation, &Transform), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
) {
    if let Ok(mut text) = hud_query.get_single_mut() {
        if pause_state.paused {
            let sfx_pct = (game_audio.sfx_volume * 100.0) as u32;
            let lines = vec![
                "".to_string(),
                "=== PAUSED ===".to_string(),
                "".to_string(),
                "[ESC] Resume".to_string(),
                "[F5] Save Game".to_string(),
                format!("SFX Volume: {}%  [-] [+]", sfx_pct),
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

        let weather_str = match weather.current {
            crate::weather::Weather::Clear => "",
            crate::weather::Weather::Rain => " Rain",
            crate::weather::Weather::Snow => " Snow",
            crate::weather::Weather::Storm => " STORM",
            crate::weather::Weather::Fog => " Fog",
            crate::weather::Weather::Blizzard => " BLIZZARD",
        };

        // Weather forecast: show what's coming next
        let forecast_str = match weather.next_weather {
            Some(next) => format!(" -> {}", next.name()),
            None => String::new(),
        };

        let mut lines = Vec::new();

        if building_state.active {
            lines.push(format!("BUILD: {} | Q Cycle | RClick Place", building_state.selected_type.name()));
        }

        lines.push(format!(
            "Day {} {} | {}{}{}",
            cycle.day_count,
            cycle.phase_name(),
            season.current.name(),
            weather_str,
            forecast_str,
        ));

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
        let visible = crafting.recipes_visible_at_stations(near_workbench, near_forge, near_campfire, near_advanced_forge, near_ancient, &tech_tree);

        let mut lines = vec!["=== CRAFTING (C to close) ===".to_string()];
        lines.push(format!("RP: {}  [U] Unlock (when locked recipe shown)", tech_tree.research_points));
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

        for (display_idx, (recipe_idx, locked)) in visible.iter().enumerate() {
            let recipe = &crafting.recipes[*recipe_idx];
            let is_selected = display_idx == crafting.selected_recipe;
            let sel_marker = if is_selected { "> " } else { "  " };

            if *locked {
                let hint = tech_tree.unlock_hint(recipe.tech_key);
                lines.push(format!("{}[LOCKED] {}  Unlock: {}", sel_marker, recipe.name, hint));
                // Show prerequisite chain if selected and has unmet prerequisites
                if is_selected {
                    if let Some(key) = recipe.tech_key {
                        let prereq_hint = tech_tree.prerequisite_hint(key);
                        if !prereq_hint.is_empty() {
                            lines.push(format!("    {}", prereq_hint));
                        }
                    }
                }
            } else if is_selected && !*locked {
                let craftable = crafting.can_craft(*recipe_idx, &inventory);
                let craft_tag = if craftable { " [READY]" } else { "" };
                lines.push(format!("{}{}{} [SELECTED]", sel_marker, recipe.name, craft_tag));
            } else if !*locked {
                let can_craft = crafting.can_craft(*recipe_idx, &inventory);
                let status = if can_craft { "" } else { " [missing]" };
                lines.push(format!("{}{} {}{}", sel_marker, recipe.tier.label(), recipe.name, status));
            }

            if is_selected {
                for (item, count) in &recipe.inputs {
                    let have = inventory.count_items(*item);
                    let has_enough = have >= *count;
                    let mark = if has_enough { "\u{2713}" } else { "\u{2717}" };
                    lines.push(format!("    {} {} x{} (have {})", mark, item.display_name(), count, have));
                }
                let (out_item, out_count) = recipe.output;
                if out_count > 1 {
                    lines.push(format!("    -> {} x{}", out_item.display_name(), out_count));
                } else {
                    lines.push(format!("    -> {}", out_item.display_name()));
                }
            }
        }

        if visible.is_empty() {
            lines.push("  (no recipes at current stations)".to_string());
        }

        lines.push(String::new());
        lines.push("[Up/Down] Select  [Enter] Craft  [U] Unlock (locked)  [C] Close".into());

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

/// Shows ephemeral feedback messages: lore discoveries, hermit dialogue, NPC dialogue, experiment results.
fn update_feedback_hud(
    lore_msg: Res<LoreMessage>,
    hermit_display: Res<HermitDialogueDisplay>,
    npc_display: Res<NpcDialogueDisplay>,
    experiment_msg: Res<ExperimentMessage>,
    mut feedback_query: Query<&mut Text, With<FeedbackHudText>>,
) {
    let Ok(mut text) = feedback_query.get_single_mut() else { return };

    // Priority order: experiment > lore > hermit > npc
    if !experiment_msg.text.is_empty() {
        **text = experiment_msg.text.clone();
    } else if !lore_msg.text.is_empty() {
        **text = lore_msg.text.clone();
    } else if !hermit_display.text.is_empty() {
        **text = hermit_display.text.clone();
    } else if !npc_display.text.is_empty() {
        **text = npc_display.text.clone();
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

/// Updates the graphical hotbar slots: highlights selected, shows item color and label; updates tooltip.
fn update_graphical_hotbar(
    inventory: Res<Inventory>,
    mut slot_query: Query<(&HotbarSlotUI, &mut BorderColor, &Children)>,
    mut color_query: Query<&mut BackgroundColor, With<HotbarSlotColor>>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<HotbarSlotLabel>>,
        Query<&mut Text, With<HotbarTooltipText>>,
    )>,
) {
    // Update tooltip first (uses p1)
    {
        let mut tooltip_query = text_queries.p1();
        if let Ok(mut tooltip) = tooltip_query.get_single_mut() {
            let idx = inventory.selected_slot;
            **tooltip = inventory.slots.get(idx)
                .and_then(|s| s.as_ref())
                .map(|s| s.item.display_name().to_string())
                .unwrap_or_else(|| format!("Slot {}", idx + 1));
        }
    }

    // Collect slot data to avoid borrow conflicts
    let slot_data: Vec<(usize, bool, Vec<Entity>)> = slot_query.iter_mut()
        .map(|(slot_ui, _, children)| {
            (slot_ui.index, slot_ui.index == inventory.selected_slot, children.iter().copied().collect())
        })
        .collect();

    // Update borders
    for (slot_ui, mut border, _) in slot_query.iter_mut() {
        if slot_ui.index == inventory.selected_slot {
            *border = BorderColor(Color::srgba(0.9, 0.75, 0.3, 0.9));
        } else {
            *border = BorderColor(Color::srgba(0.25, 0.25, 0.35, 0.5));
        }
    }

    // Update child color blocks
    for (i, _, ref children) in &slot_data {
        for child in children {
            if let Ok(mut bg) = color_query.get_mut(*child) {
                if let Some(slot) = &inventory.slots[*i] {
                    let item_color = crate::gathering::dropped_item_color(slot.item);
                    let c = item_color.to_srgba();
                    *bg = BackgroundColor(Color::srgba(c.red, c.green, c.blue, 0.7));
                } else {
                    *bg = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0));
                }
            }
        }
    }

    // Update labels (uses p0)
    {
        let mut label_query = text_queries.p0();
        for (i, _, ref children) in &slot_data {
            for child in children {
                if let Ok(mut text) = label_query.get_mut(*child) {
                    if let Some(slot) = &inventory.slots[*i] {
                        let name: String = slot.item.display_name().chars().take(6).collect();
                        if slot.count > 1 {
                            **text = format!("{} x{}", name, slot.count);
                        } else {
                            **text = name;
                        }
                    } else {
                        **text = format!("{}", *i + 1);
                    }
                }
            }
        }
    }
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
/// the banner timer whenever the biome changes. First-time discovery plays a sound and longer banner.
fn track_player_biome(
    player_query: Query<&Transform, With<Player>>,
    chunk_query: Query<&Chunk>,
    mut current_biome: ResMut<CurrentBiome>,
    mut explored_biomes: ResMut<ExploredBiomes>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    let Ok(player_tf) = player_query.get_single() else { return };

    let chunk_x = (player_tf.translation.x / CHUNK_WORLD_SIZE).floor() as i32;
    let chunk_y = (player_tf.translation.y / CHUNK_WORLD_SIZE).floor() as i32;

    for chunk in chunk_query.iter() {
        if chunk.position.x == chunk_x && chunk.position.y == chunk_y {
            let new_biome = chunk.biome;
            if current_biome.biome != Some(new_biome) {
                current_biome.biome = Some(new_biome);
                let first_time = !explored_biomes.set.contains(&new_biome);
                if first_time {
                    explored_biomes.set.insert(new_biome);
                    sound_events.send(SoundEvent::Discovery);
                    current_biome.display_timer = 4.5; // Longer banner for first discovery
                } else {
                    current_biome.display_timer = 3.0;
                }
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
        let mut c = color.0.to_srgba();
        c.alpha = alpha;
        *color = TextColor(Color::Srgba(c));

        current_biome.display_timer -= time.delta_secs();
    } else {
        // Hide banner
        let mut c = color.0.to_srgba();
        c.alpha = 0.0;
        *color = TextColor(Color::Srgba(c));
        **text = String::new();
    }
}

// --- US-028: Floating Text ---

/// Request to show floating text; queued and shown with delay to avoid overlap.
#[derive(Event)]
pub struct FloatingTextRequest {
    pub text: String,
    pub position: Vec2,
    pub color: Color,
}

/// Queue for floating text so multiple requests in one frame don't overlap illegibly.
#[derive(Resource, Default)]
pub struct FloatingTextQueue {
    pub pending: Vec<(String, Vec2, Color)>,
    pub cooldown: f32,
}

/// World-space floating text that drifts upward and fades out.
/// Used for damage numbers, item pickup notifications, etc.
#[derive(Component)]
pub struct FloatingText {
    pub timer: f32,
    pub max_timer: f32,
    pub velocity: Vec2,
}

const FLOATING_TEXT_QUEUE_INTERVAL: f32 = 0.35;

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
            font_size: choose_floating_font_size(color),
            ..default()
        },
        TextColor(color),
        Transform::from_xyz(position.x, position.y + 8.0, 100.0),
    ));
}

fn receive_floating_text_requests(
    mut events: EventReader<FloatingTextRequest>,
    mut queue: ResMut<FloatingTextQueue>,
) {
    for ev in events.read() {
        queue.pending.push((ev.text.clone(), ev.position, ev.color));
    }
}

fn drain_floating_text_queue(
    time: Res<Time>,
    mut commands: Commands,
    mut queue: ResMut<FloatingTextQueue>,
) {
    queue.cooldown -= time.delta_secs();
    if queue.cooldown <= 0.0 {
        if let Some((text, position, color)) = queue.pending.first() {
            spawn_floating_text(&mut commands, text.as_str(), *position, *color);
            queue.pending.remove(0);
            queue.cooldown = FLOATING_TEXT_QUEUE_INTERVAL;
        }
    }
}

fn choose_floating_font_size(color: Color) -> f32 {
    let c = color.to_srgba();
    // Damage/heal numbers: larger
    if c.red > 0.8 && c.green < 0.6 {
        18.0
    // Rare/important pickups: bright gold/white
    } else if c.red > 0.85 && c.green > 0.75 {
        17.0
    } else {
        14.0
    }
}

// ---------------------------------------------------------------------------
// 1A: Fishing HUD
// ---------------------------------------------------------------------------

fn update_fishing_hud(
    time: Res<Time>,
    fishing: Res<FishingState>,
    mut catch_flash: ResMut<FishingCatchFlash>,
    mut fishing_hud_query: Query<&mut Text, With<FishingHudText>>,
    player_query: Query<&Transform, With<Player>>,
    mut floating_text_events: EventWriter<FloatingTextRequest>,
) {
    let Ok(mut text) = fishing_hud_query.get_single_mut() else { return };

    // Tick catch flash timer
    if catch_flash.timer > 0.0 {
        catch_flash.timer -= time.delta_secs();
    }

    match fishing.phase {
        FishingPhase::Idle => {
            if catch_flash.timer > 0.0 {
                **text = format!("Caught a {}!", catch_flash.fish_name);
            } else {
                **text = String::new();
            }
        }
        FishingPhase::Casting => {
            **text = "Casting...".to_string();
        }
        FishingPhase::Waiting => {
            let dots = match ((time.elapsed_secs() * 2.0) as u32) % 4 {
                0 => "",
                1 => ".",
                2 => "..",
                _ => "...",
            };
            **text = format!("Waiting for a bite{}", dots);
        }
        FishingPhase::Hooked => {
            **text = format!("FISH ON! Press [E]! ({:.1}s)", fishing.hook_window);
        }
        FishingPhase::Reeling => {
            let bar_len = 20;
            let filled = (fishing.reel_progress * bar_len as f32).round() as usize;
            let bar: String = "=".repeat(filled) + &"-".repeat(bar_len - filled);
            let fish_name = fishing.target_fish
                .map(|f| fish_type_name(f))
                .unwrap_or("???");
            let pct = (fishing.reel_progress * 100.0) as u32;
            **text = format!("Reeling: {} [{}] {}%", fish_name, bar, pct);
        }
        FishingPhase::Caught => {
            let fish_name = fishing.target_fish
                .map(|f| fish_type_name(f))
                .unwrap_or("Fish");
            catch_flash.fish_name = fish_name.to_string();
            catch_flash.timer = 2.0;
            **text = format!("Caught a {}!", fish_name);

            // Floating text on catch
            if let Ok(player_tf) = player_query.get_single() {
                floating_text_events.send(FloatingTextRequest {
                    text: format!("Caught {}!", fish_name),
                    position: player_tf.translation.truncate(),
                    color: Color::srgb(0.95, 0.85, 0.3),
                });
            }
        }
    }
}

fn fish_type_name(fish: FishType) -> &'static str {
    match fish {
        FishType::Trout => "Trout",
        FishType::Salmon => "Salmon",
        FishType::Catfish => "Catfish",
        FishType::Pufferfish => "Pufferfish",
        FishType::Eel => "Eel",
        FishType::Crab => "Crab",
    }
}

// ---------------------------------------------------------------------------
// 1B: Quest Log HUD
// ---------------------------------------------------------------------------

fn update_quest_log_hud(
    quest_log: Res<QuestLog>,
    dynamic_log: Res<crate::quests::DynamicQuestLog>,
    mut quest_hud_query: Query<&mut Text, With<QuestLogHudText>>,
) {
    let Ok(mut text) = quest_hud_query.get_single_mut() else { return };

    if !quest_log.is_open {
        **text = String::new();
        return;
    }

    let defs = crate::quests::quest_definitions();
    let mut lines = vec!["=== QUEST LOG [J] ===".to_string(), String::new()];

    let static_count = quest_log.quests.len();

    for (i, quest) in quest_log.quests.iter().enumerate() {
        let def = &defs[quest.def_index];
        let sel = if i == quest_log.selected { "> " } else { "  " };

        let status = if quest.claimed {
            format!("[X] {}  {}/{}  CLAIMED", def.name, quest.progress, def.target)
        } else if quest.completed {
            format!("[!] {}  {}/{}  CLAIM", def.name, quest.progress, def.target)
        } else {
            format!("[ ] {}  {}/{}", def.name, quest.progress, def.target)
        };

        lines.push(format!("{}{}", sel, status));

        // Show description for selected quest
        if i == quest_log.selected {
            lines.push(format!("    {}", def.description));
            if !def.reward.is_empty() {
                let rewards: Vec<String> = def.reward.iter()
                    .map(|(item, count)| {
                        if *count > 1 {
                            format!("{} x{}", item.display_name(), count)
                        } else {
                            item.display_name().to_string()
                        }
                    })
                    .collect();
                lines.push(format!("    Reward: {}", rewards.join(", ")));
            }
            if def.rp_reward > 0 {
                lines.push(format!("    +{} Research Points", def.rp_reward));
            }
        }
    }

    // Dynamic quests section
    if !dynamic_log.quests.is_empty() {
        lines.push(String::new());
        lines.push("--- DYNAMIC QUESTS ---".to_string());

        for (di, dq) in dynamic_log.quests.iter().enumerate() {
            let global_idx = static_count + di;
            let sel = if global_idx == quest_log.selected { "> " } else { "  " };

            let status = if dq.claimed {
                format!("[DYNAMIC] [X]  {}/{}  CLAIMED", dq.progress, dq.target_count)
            } else if dq.completed {
                format!("[DYNAMIC] [!]  {}/{}  CLAIM", dq.progress, dq.target_count)
            } else {
                format!("[DYNAMIC] [ ]  {}/{}", dq.progress, dq.target_count)
            };

            lines.push(format!("{}{}", sel, status));

            if global_idx == quest_log.selected {
                lines.push(format!("    {}", dq.description));
                lines.push(format!("    Expires: Day {}", dq.expiry_day));
                if !dq.reward_items.is_empty() {
                    let rewards: Vec<String> = dq.reward_items.iter()
                        .map(|(item, count)| {
                            if *count > 1 {
                                format!("{} x{}", item.display_name(), count)
                            } else {
                                item.display_name().to_string()
                            }
                        })
                        .collect();
                    lines.push(format!("    Reward: {}", rewards.join(", ")));
                }
            }
        }
    }

    lines.push(String::new());
    lines.push("[Up/Down] Select  [Enter] Claim  [J] Close".to_string());

    **text = lines.join("\n");
}

// ---------------------------------------------------------------------------
// 1D: Status Effects HUD
// ---------------------------------------------------------------------------

fn update_status_effects_hud(
    player_query: Query<Option<&ActiveStatusEffects>, With<Player>>,
    mut effects_hud_query: Query<&mut Text, With<StatusEffectsHudText>>,
) {
    let Ok(mut text) = effects_hud_query.get_single_mut() else { return };
    let Ok(maybe_effects) = player_query.get_single() else {
        **text = String::new();
        return;
    };

    let Some(active) = maybe_effects else {
        **text = String::new();
        return;
    };

    if active.effects.is_empty() {
        **text = String::new();
        return;
    }

    let parts: Vec<String> = active.effects.iter().map(|e| {
        let name = match e.effect_type {
            StatusEffectType::Poison => "POISON",
            StatusEffectType::Burn => "BURN",
            StatusEffectType::Freeze => "FREEZE",
            StatusEffectType::Bleed => "BLEED",
            StatusEffectType::Stun => "STUN",
            StatusEffectType::Regen => "REGEN",
            StatusEffectType::WellFed => "WELL FED",
        };
        if e.stacks > 1 {
            format!("[{} x{} {:.0}s]", name, e.stacks, e.remaining_secs)
        } else {
            format!("[{} {:.0}s]", name, e.remaining_secs)
        }
    }).collect();

    **text = parts.join(" ");
}

// ---------------------------------------------------------------------------
// 4A: Skills HUD
// ---------------------------------------------------------------------------

fn update_skill_hud(
    skill_levels: Res<SkillLevels>,
    mut skill_hud_query: Query<&mut Text, With<SkillHudText>>,
) {
    let Ok(mut text) = skill_hud_query.get_single_mut() else { return };

    if !skill_levels.skills_open {
        **text = String::new();
        return;
    }

    let mut lines = vec!["=== SKILLS [K] ===".to_string(), String::new()];

    let skills = [
        SkillType::Gathering,
        SkillType::Combat,
        SkillType::Fishing,
        SkillType::Farming,
        SkillType::Crafting,
        SkillType::Building,
    ];

    for skill in &skills {
        let data = skill_levels.get(*skill);
        let frac = data.progress_fraction();
        let pct = (frac * 100.0) as u32;
        let filled = (frac * 8.0) as usize;
        let bar: String = "=".repeat(filled) + &"-".repeat(8 - filled);
        lines.push(format!(
            "{:<10} Lv {:>2}  [{}] {:>3}%",
            skill.display_name(),
            data.level,
            bar,
            pct,
        ));
    }

    lines.push(String::new());
    lines.push("[K] Close".to_string());

    **text = lines.join("\n");
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

        // Fade alpha based on remaining time (slower at start, faster at end)
        let t = (ft.timer / ft.max_timer).clamp(0.0, 1.0);
        let alpha = t * t;
        let c = color.0.to_srgba();
        color.0 = Color::srgba(c.red, c.green, c.blue, alpha);

        // Despawn when done
        if ft.timer <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
