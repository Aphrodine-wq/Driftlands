use bevy::prelude::*;
use bevy::ui::Interaction;
use bevy::input::gamepad::GamepadButton;

use crate::daynight::DayNightCycle;
use crate::audio::SoundEvent;
use crate::hud::{PauseMenuPanel, PauseState};
use crate::mainmenu::MainMenuActive;
use crate::saveload::{
    delete_save_slot, list_save_slots, ActiveSaveSlot, LoadRequested, SaveSlotMeta, SaveTrigger,
    MAX_SAVE_SLOTS,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveSlotBrowserContext {
    /// Main menu -> Continue -> load selected slot
    MainContinue,
    /// Pause menu -> Save Game -> save selected slot
    PauseSave,
    /// Pause menu -> Load Game -> load selected slot
    PauseLoad,
}

#[derive(Resource, Debug)]
pub struct SaveSlotBrowserState {
    pub open: bool,
    pub context: SaveSlotBrowserContext,
    /// Which UI element is selected:
    /// - `0..MAX_SAVE_SLOTS` => slot rows
    /// - `MAX_SAVE_SLOTS` => delete row
    pub selected_focus: usize,
    pub confirm_delete: bool,
    pub delete_target_slot: usize,
}

impl Default for SaveSlotBrowserState {
    fn default() -> Self {
        Self {
            open: false,
            context: SaveSlotBrowserContext::MainContinue,
            selected_focus: 0,
            confirm_delete: false,
            delete_target_slot: 0,
        }
    }
}

#[derive(Component)]
struct SaveSlotBrowserRoot;

#[derive(Component)]
struct SaveSlotRow {
    slot_index: usize,
}

#[derive(Component)]
struct DeleteRow;
#[derive(Component)]
struct DeleteRowPanel;

#[derive(Component)]
struct SlotRowText {
    slot_index: usize,
}

#[derive(Component)]
struct SelectedMetaText;

pub struct SaveSlotBrowserPlugin;

impl Plugin for SaveSlotBrowserPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SaveSlotBrowserState>()
            .add_systems(Startup, spawn_save_slot_browser_ui)
            .add_systems(
                Update,
                (
                    save_slot_browser_input,
                    update_save_slot_browser_visuals,
                    update_selected_meta,
                ),
            );
    }
}

fn spawn_save_slot_browser_ui(mut commands: Commands, theme: Res<crate::theme::EtherealTheme>) {
    // Root overlay panel (full-screen-ish, but we center the actual browser panel).
    commands.spawn((
        SaveSlotBrowserRoot,
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            top: Val::Percent(50.0),
            left: Val::Percent(50.0),
            margin: UiRect {
                left: Val::Px(-240.0),
                top: Val::Px(-260.0),
                ..default()
            },
            width: Val::Px(480.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(16.0)),
            border: UiRect::all(Val::Px(2.0)),
            row_gap: Val::Px(4.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.02, 0.06, 0.96)),
        BorderColor(Color::srgba(0.4, 0.35, 0.2, 0.9)),
        GlobalZIndex(250),
    ))
    .with_children(|root| {
        root.spawn((
            Text::new("SAVE SLOTS"),
            TextFont { font_size: 24.0, ..default() },
            TextColor(theme.accent_gold),
            Node {
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
        ));

        // Slot list + delete row
        let list_height = (MAX_SAVE_SLOTS as f32 + 1.0) * 28.0;
        root.spawn(Node {
            width: Val::Px(420.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            height: Val::Px(list_height),
            ..default()
        })
        .with_children(|list| {
            for slot in 0..MAX_SAVE_SLOTS {
                list.spawn((
                    SaveSlotRow { slot_index: slot },
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(24.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.06, 0.06, 0.1, 0.7)),
                    BorderColor(Color::srgba(0.2, 0.2, 0.3, 0.5)),
                ))
                .with_children(|row| {
                    row.spawn((
                        SlotRowText { slot_index: slot },
                        Text::new(""),
                        TextFont { font_size: 13.0, ..default() },
                        TextColor(theme.hud_label_color()),
                    ));
                });
            }

            list.spawn((
                DeleteRow,
                DeleteRowPanel,
                Button,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(24.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.02, 0.02, 0.6)),
                BorderColor(Color::srgba(0.4, 0.2, 0.2, 0.6)),
            ))
            .with_children(|row| {
                row.spawn((
                    Text::new("DELETE SELECTED"),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(theme.critical),
                ));
            });
        });

        root.spawn((
            SelectedMetaText,
            Text::new(""),
            TextFont { font_size: 12.0, ..default() },
            TextColor(theme.accent_slate),
            Node { margin: UiRect::top(Val::Px(10.0)), ..default() },
        ));

        root.spawn((
            Text::new("Up/Down: Select  Enter: Confirm  Esc: Cancel"),
            TextFont { font_size: 11.0, ..default() },
            TextColor(Color::srgba(0.5, 0.5, 0.55, 0.9)),
            Node { margin: UiRect::top(Val::Px(8.0)), ..default() },
        ));
    });
}

fn update_selected_meta(
    state: Res<SaveSlotBrowserState>,
    active_slot: Res<ActiveSaveSlot>,
    mut meta_text: Query<&mut Text, With<SelectedMetaText>>,
) {
    let Ok(mut meta_text) = meta_text.get_single_mut() else { return };

    if !state.open {
        **meta_text = String::new();
        return;
    }

    if state.confirm_delete {
        **meta_text = format!(
            "Delete Slot {}? Enter/South to confirm  |  Esc/East to cancel",
            state.delete_target_slot + 1
        );
        return;
    }

    let selected_slot = if state.selected_focus == MAX_SAVE_SLOTS {
        active_slot.index
    } else {
        state.selected_focus
    };

    let slot_meta_opt = list_save_slots(MAX_SAVE_SLOTS)
        .into_iter()
        .find(|m| m.slot_index == selected_slot);

    if let Some(m) = slot_meta_opt {
        **meta_text = m.format_detail();
    } else {
        **meta_text = format!("Selected slot {}", selected_slot + 1);
    }
}

// Helper extension for details formatting.
trait FormatSlotMeta {
    fn format_detail(&self) -> String;
}

impl FormatSlotMeta for SaveSlotMeta {
    fn format_detail(&self) -> String {
        if !self.exists {
            format!("Slot {} is empty", self.slot_index + 1)
        } else {
            let size = self.size_bytes.map(|s| format!("{s} bytes")).unwrap_or_else(|| "Unknown size".to_string());
            let modified = self.modified_unix.map(|s| s.to_string()).unwrap_or_else(|| "Unknown time".to_string());
            format!(
                "Slot {}: {}  Last modified: {} (unix secs)",
                self.slot_index + 1,
                size,
                modified
            )
        }
    }
}

fn update_save_slot_browser_visuals(
    state: Res<SaveSlotBrowserState>,
    mut root_query: Query<&mut Node, With<SaveSlotBrowserRoot>>,
    mut slot_row_query: Query<(
        &SaveSlotRow,
        &mut BackgroundColor,
        &mut BorderColor,
        &SlotRowText,
        &mut Text,
    )>,
    mut delete_row_query: Query<(&mut BackgroundColor, &mut BorderColor), With<DeleteRowPanel>>,
) {
    let Ok(mut root) = root_query.get_single_mut() else { return };

    root.display = if state.open { Display::Flex } else { Display::None };

    if !state.open {
        return;
    }

    // Refresh texts + row highlight.
    let slots = list_save_slots(MAX_SAVE_SLOTS);
    for (row, mut bg, mut border, row_text, mut text) in slot_row_query.iter_mut() {
        let meta = slots.iter().find(|m| m.slot_index == row.slot_index);
        let label = match meta {
            Some(m) if m.exists => format!("Slot {:>2}  ({} bytes)", row.slot_index + 1, m.size_bytes.unwrap_or(0)),
            _ => format!("Slot {:>2}  (empty)", row.slot_index + 1),
        };
        **text = label;

        let selected = row_text.slot_index == state.selected_focus;
        *bg = BackgroundColor(if selected {
            Color::srgba(0.12, 0.10, 0.06, 0.95)
        } else {
            Color::srgba(0.06, 0.06, 0.1, 0.7)
        });
        *border = BorderColor(if selected {
            Color::srgba(0.9, 0.75, 0.3, 0.8)
        } else {
            Color::srgba(0.2, 0.2, 0.3, 0.5)
        });
    }

    // Delete row highlight
    let delete_selected = state.selected_focus == MAX_SAVE_SLOTS;
    if let Ok((mut bg, mut border)) = delete_row_query.get_single_mut() {
        *bg = BackgroundColor(if delete_selected {
            Color::srgba(0.25, 0.08, 0.08, 0.85)
        } else {
            Color::srgba(0.08, 0.02, 0.02, 0.6)
        });
        *border = BorderColor(if delete_selected {
            Color::srgba(0.9, 0.35, 0.35, 0.8)
        } else {
            Color::srgba(0.4, 0.2, 0.2, 0.6)
        });
    }
}

fn save_slot_browser_input(
    mut state: ResMut<SaveSlotBrowserState>,
    mut active_slot: ResMut<ActiveSaveSlot>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepad_buttons: Res<ButtonInput<GamepadButton>>,
    mut load_requested: ResMut<LoadRequested>,
    mut save_trigger: ResMut<SaveTrigger>,
    mut pause_state: ResMut<PauseState>,
    mut cycle: ResMut<DayNightCycle>,
    mut main_menu: ResMut<MainMenuActive>,
    mut pause_panel_query: Query<&mut Node, With<PauseMenuPanel>>,
    mut slot_row_query: Query<(&SaveSlotRow, &Interaction)>,
    mut delete_row_query: Query<&Interaction, With<DeleteRow>>,
    mut save_msg: ResMut<crate::saveload::SaveMessage>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    if !state.open {
        return;
    }

    let confirm = keyboard.just_pressed(KeyCode::Enter)
        || keyboard.just_pressed(KeyCode::Space)
        || gamepad_buttons.just_pressed(GamepadButton::South);
    let cancel = keyboard.just_pressed(KeyCode::Escape)
        || gamepad_buttons.just_pressed(GamepadButton::East);

    // Mouse click handling (select + confirm in one step).
    for (row, interaction) in slot_row_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            state.selected_focus = row.slot_index;
            active_slot.index = row.slot_index;
            if !state.confirm_delete {
                match state.context {
                    SaveSlotBrowserContext::MainContinue => {
                        load_requested.requested = true;
                        load_requested.slot_index = row.slot_index;
                        main_menu.active = false;
                        sound_events.send(SoundEvent::MenuOpen);
                        state.open = false;
                    }
                    SaveSlotBrowserContext::PauseLoad => {
                        pause_state.paused = false;
                        cycle.paused = false;
                        if let Ok(mut panel) = pause_panel_query.get_single_mut() {
                            panel.display = Display::None;
                        }
                        load_requested.requested = true;
                        load_requested.slot_index = row.slot_index;
                        sound_events.send(SoundEvent::MenuOpen);
                        state.open = false;
                    }
                    SaveSlotBrowserContext::PauseSave => {
                        save_trigger.requested = true;
                        save_trigger.slot_index = row.slot_index;
                        sound_events.send(SoundEvent::MenuOpen);
                        state.open = false;
                    }
                }
            }
            return;
        }
    }

    if let Ok(interaction) = delete_row_query.get_single_mut() {
        if *interaction == Interaction::Pressed {
            if state.confirm_delete {
                // In confirmation mode, ignore delete row presses (Enter/Esc decides).
            } else {
                state.confirm_delete = true;
                state.delete_target_slot = active_slot.index;
            }
            return;
        }
    }

    // If we are confirming a delete, Enter/Esc are the only actions we care about.
    if state.confirm_delete {
        if cancel {
            state.confirm_delete = false;
            return;
        }

        if confirm {
            let target = state.delete_target_slot;
            match delete_save_slot(target) {
                Ok(()) => {
                    save_msg.text = format!("Deleted slot {}", target + 1);
                    save_msg.timer = 2.0;
                }
                Err(e) => {
                    save_msg.text = e;
                    save_msg.timer = 2.0;
                }
            }
            sound_events.send(SoundEvent::MenuOpen);

            // Move selection to first existing slot (or keep if empty).
            let slots = list_save_slots(MAX_SAVE_SLOTS);
            if let Some(first) = slots.iter().take(MAX_SAVE_SLOTS).find(|m| m.exists) {
                state.selected_focus = first.slot_index;
                active_slot.index = first.slot_index;
            } else {
                state.selected_focus = 0;
                active_slot.index = 0;
            }

            state.confirm_delete = false;
        }
        return;
    }

    // Cancel/Back
    if cancel {
        state.open = false;
        state.confirm_delete = false;
        sound_events.send(SoundEvent::MenuClose);
        return;
    }

    // Navigation among rows.
    let up = keyboard.just_pressed(KeyCode::ArrowUp)
        || keyboard.just_pressed(KeyCode::KeyW)
        || gamepad_buttons.just_pressed(GamepadButton::DPadUp);
    let down = keyboard.just_pressed(KeyCode::ArrowDown)
        || keyboard.just_pressed(KeyCode::KeyS)
        || gamepad_buttons.just_pressed(GamepadButton::DPadDown);

    if up {
        if state.selected_focus > 0 {
            state.selected_focus -= 1;
        }
        if state.selected_focus < MAX_SAVE_SLOTS {
            active_slot.index = state.selected_focus;
        }
    }
    if down {
        let max_focus = MAX_SAVE_SLOTS; // includes delete row
        if state.selected_focus < max_focus {
            state.selected_focus += 1;
        }
        if state.selected_focus < MAX_SAVE_SLOTS {
            active_slot.index = state.selected_focus;
        }
    }

    // Activate
    if confirm {
        if state.selected_focus == MAX_SAVE_SLOTS {
            state.confirm_delete = true;
            state.delete_target_slot = active_slot.index;
            return;
        }

        active_slot.index = state.selected_focus;
        match state.context {
            SaveSlotBrowserContext::MainContinue => {
                load_requested.requested = true;
                load_requested.slot_index = state.selected_focus;
                main_menu.active = false;
            }
            SaveSlotBrowserContext::PauseLoad => {
                pause_state.paused = false;
                cycle.paused = false;
                if let Ok(mut panel) = pause_panel_query.get_single_mut() {
                    panel.display = Display::None;
                }
                load_requested.requested = true;
                load_requested.slot_index = state.selected_focus;
            }
            SaveSlotBrowserContext::PauseSave => {
                save_trigger.requested = true;
                save_trigger.slot_index = state.selected_focus;
            }
        }

        sound_events.send(SoundEvent::MenuOpen);
        state.open = false;
    }
}

