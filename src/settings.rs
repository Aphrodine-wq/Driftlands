use crate::audio::GameAudio;
use crate::theme::EtherealTheme;
use bevy::input::gamepad::GamepadButton;
use bevy::prelude::*;
use bevy::window::MonitorSelection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        let settings = GameSettings::load();
        app.insert_resource(settings)
            .insert_resource(SettingsMenuState::default())
            .add_systems(Startup, spawn_settings_ui)
            .add_systems(
                Update,
                (
                    settings_menu_navigation,
                    update_settings_display,
                    update_fps_counter,
                    sync_audio_from_settings,
                    apply_fullscreen,
                    toggle_fullscreen_f11,
                    apply_resolution,
                ),
            );
    }
}

// ---------------------------------------------------------------------------
// GameSettings resource (persisted to settings.json)
// ---------------------------------------------------------------------------

#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct GameSettings {
    pub sfx_volume: f32,
    pub music_volume: f32,
    pub screen_shake: bool,
    pub show_minimap: bool,
    pub show_fps: bool,
    #[serde(default)]
    pub fullscreen: bool,
    #[serde(default)]
    pub resolution_index: usize,
    #[serde(default)]
    pub keybinds: KeyBinds,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            sfx_volume: 0.8,
            music_volume: 0.5,
            screen_shake: true,
            show_minimap: true,
            show_fps: false,
            fullscreen: false,
            resolution_index: 0,
            keybinds: KeyBinds::default(),
        }
    }
}

impl GameSettings {
    fn settings_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            let dir = config_dir.join("driftlands");
            if fs::create_dir_all(&dir).is_ok() {
                return dir.join("settings.json");
            }
        }
        // Fallback to CWD-relative path if platform config dir unavailable
        PathBuf::from("settings.json")
    }

    pub fn load() -> Self {
        let path = Self::settings_path();
        if let Ok(data) = fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let path = Self::settings_path();
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&path, data);
        }
    }
}

// ---------------------------------------------------------------------------
// KeyBinds (persisted via string-based KeyCode serialization)
// ---------------------------------------------------------------------------

#[derive(Clone, Serialize, Deserialize)]
pub struct KeyBinds {
    #[serde(serialize_with = "ser_key", deserialize_with = "de_key")]
    pub move_up: KeyCode,
    #[serde(serialize_with = "ser_key", deserialize_with = "de_key")]
    pub move_down: KeyCode,
    #[serde(serialize_with = "ser_key", deserialize_with = "de_key")]
    pub move_left: KeyCode,
    #[serde(serialize_with = "ser_key", deserialize_with = "de_key")]
    pub move_right: KeyCode,
    #[serde(serialize_with = "ser_key", deserialize_with = "de_key")]
    pub interact: KeyCode,
    #[serde(serialize_with = "ser_key", deserialize_with = "de_key")]
    pub inventory: KeyCode,
    #[serde(serialize_with = "ser_key", deserialize_with = "de_key")]
    pub crafting: KeyCode,
    #[serde(serialize_with = "ser_key", deserialize_with = "de_key")]
    pub building: KeyCode,
    #[serde(serialize_with = "ser_key", deserialize_with = "de_key")]
    pub journal: KeyCode,
    #[serde(serialize_with = "ser_key", deserialize_with = "de_key")]
    pub skills: KeyCode,
    #[serde(serialize_with = "ser_key", deserialize_with = "de_key")]
    pub dodge: KeyCode,
    #[serde(serialize_with = "ser_key", deserialize_with = "de_key")]
    pub save: KeyCode,
    #[serde(serialize_with = "ser_key", deserialize_with = "de_key")]
    pub load: KeyCode,
}

fn ser_key<S: serde::Serializer>(key: &KeyCode, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(keycode_display(*key))
}

fn de_key<'de, D: serde::Deserializer<'de>>(d: D) -> Result<KeyCode, D::Error> {
    let s: String = serde::Deserialize::deserialize(d)?;
    Ok(string_to_keycode(&s).unwrap_or(KeyCode::Escape))
}

fn string_to_keycode(s: &str) -> Option<KeyCode> {
    match s {
        "A" => Some(KeyCode::KeyA),
        "B" => Some(KeyCode::KeyB),
        "C" => Some(KeyCode::KeyC),
        "D" => Some(KeyCode::KeyD),
        "E" => Some(KeyCode::KeyE),
        "F" => Some(KeyCode::KeyF),
        "G" => Some(KeyCode::KeyG),
        "H" => Some(KeyCode::KeyH),
        "I" => Some(KeyCode::KeyI),
        "J" => Some(KeyCode::KeyJ),
        "K" => Some(KeyCode::KeyK),
        "L" => Some(KeyCode::KeyL),
        "M" => Some(KeyCode::KeyM),
        "N" => Some(KeyCode::KeyN),
        "O" => Some(KeyCode::KeyO),
        "P" => Some(KeyCode::KeyP),
        "Q" => Some(KeyCode::KeyQ),
        "R" => Some(KeyCode::KeyR),
        "S" => Some(KeyCode::KeyS),
        "T" => Some(KeyCode::KeyT),
        "U" => Some(KeyCode::KeyU),
        "V" => Some(KeyCode::KeyV),
        "W" => Some(KeyCode::KeyW),
        "X" => Some(KeyCode::KeyX),
        "Y" => Some(KeyCode::KeyY),
        "Z" => Some(KeyCode::KeyZ),
        "0" => Some(KeyCode::Digit0),
        "1" => Some(KeyCode::Digit1),
        "2" => Some(KeyCode::Digit2),
        "3" => Some(KeyCode::Digit3),
        "4" => Some(KeyCode::Digit4),
        "5" => Some(KeyCode::Digit5),
        "6" => Some(KeyCode::Digit6),
        "7" => Some(KeyCode::Digit7),
        "8" => Some(KeyCode::Digit8),
        "9" => Some(KeyCode::Digit9),
        "Space" => Some(KeyCode::Space),
        "Tab" => Some(KeyCode::Tab),
        "Enter" => Some(KeyCode::Enter),
        "Esc" => Some(KeyCode::Escape),
        "Up" => Some(KeyCode::ArrowUp),
        "Down" => Some(KeyCode::ArrowDown),
        "Left" => Some(KeyCode::ArrowLeft),
        "Right" => Some(KeyCode::ArrowRight),
        "L-Shift" => Some(KeyCode::ShiftLeft),
        "R-Shift" => Some(KeyCode::ShiftRight),
        "L-Ctrl" => Some(KeyCode::ControlLeft),
        "R-Ctrl" => Some(KeyCode::ControlRight),
        "F1" => Some(KeyCode::F1),
        "F2" => Some(KeyCode::F2),
        "F3" => Some(KeyCode::F3),
        "F4" => Some(KeyCode::F4),
        "F5" => Some(KeyCode::F5),
        "F6" => Some(KeyCode::F6),
        "F7" => Some(KeyCode::F7),
        "F8" => Some(KeyCode::F8),
        "F9" => Some(KeyCode::F9),
        "F10" => Some(KeyCode::F10),
        "F11" => Some(KeyCode::F11),
        "F12" => Some(KeyCode::F12),
        _ => None,
    }
}

impl Default for KeyBinds {
    fn default() -> Self {
        Self {
            move_up: KeyCode::KeyW,
            move_down: KeyCode::KeyS,
            move_left: KeyCode::KeyA,
            move_right: KeyCode::KeyD,
            interact: KeyCode::KeyE,
            inventory: KeyCode::Tab,
            crafting: KeyCode::KeyC,
            building: KeyCode::KeyB,
            journal: KeyCode::KeyJ,
            skills: KeyCode::KeyK,
            dodge: KeyCode::Space,
            save: KeyCode::F5,
            load: KeyCode::F9,
        }
    }
}

// ---------------------------------------------------------------------------
// Settings UI state
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
pub struct SettingsMenuState {
    pub is_open: bool,
    pub selected: usize,
    pub rebinding: bool,
}

const SETTINGS_COUNT: usize = 8; // volume/toggle/display items (+ reset row)
const KEYBIND_COUNT: usize = 13;
const BACK_COUNT: usize = 1;
const TOTAL_ITEMS: usize = SETTINGS_COUNT + KEYBIND_COUNT + BACK_COUNT;

// Available window resolutions (width x height)
pub const RESOLUTIONS: [(u32, u32); 4] = [(1280, 720), (1600, 900), (1920, 1080), (2560, 1440)];

// Labels for each settings row
const ITEM_LABELS: [&str; TOTAL_ITEMS] = [
    "SFX Volume",
    "Music Volume",
    "Screen Shake",
    "Show Minimap",
    "Show FPS",
    "Fullscreen",
    "Resolution",
    "Reset to Defaults",
    "Move Up",
    "Move Down",
    "Move Left",
    "Move Right",
    "Interact",
    "Inventory",
    "Crafting",
    "Building",
    "Journal",
    "Skills",
    "Dodge",
    "Quick Save",
    "Quick Load",
    "Back to Menu",
];

// ---------------------------------------------------------------------------
// UI components
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct SettingsPanel;

#[derive(Component)]
struct SettingsRow {
    index: usize,
}

#[derive(Component)]
struct SettingsValueText {
    index: usize,
}

#[derive(Component)]
pub struct FpsText;

// ---------------------------------------------------------------------------
// Spawn settings UI (hidden by default)
// ---------------------------------------------------------------------------

fn spawn_settings_ui(mut commands: Commands, theme: Res<EtherealTheme>) {
    // Settings panel — centered overlay
    commands.spawn((
        SettingsPanel,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(50.0),
            left: Val::Percent(50.0),
            margin: UiRect {
                left: Val::Px(-200.0),
                top: Val::Px(-280.0),
                ..default()
            },
            width: Val::Px(400.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            row_gap: Val::Px(2.0),
            border: UiRect::all(Val::Px(2.0)),
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.02, 0.06, 0.95)),
        BorderColor(Color::srgba(0.4, 0.35, 0.2, 0.8)),
        GlobalZIndex(200),
    )).with_children(|panel| {
        // Title
        panel.spawn((
            Text::new("SETTINGS"),
            TextFont { font_size: 24.0, ..default() },
            TextColor(theme.accent_gold),
            Node {
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
        ));

        // Settings rows
        for (i, label) in ITEM_LABELS.iter().enumerate() {
            // Section separator before keybinds
            if i == SETTINGS_COUNT {
                panel.spawn((
                    Text::new("-- Keybinds --"),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(theme.accent_slate),
                    Node {
                        margin: UiRect::vertical(Val::Px(4.0)),
                        ..default()
                    },
                ));
            }

            panel.spawn((
                SettingsRow { index: i },
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(24.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.06, 0.06, 0.1, 0.8)),
                BorderColor(Color::srgba(0.2, 0.2, 0.3, 0.4)),
            )).with_children(|row| {
                row.spawn((
                    Text::new(label.to_string()),
                    TextFont { font_size: 13.0, ..default() },
                    TextColor(Color::srgba(0.8, 0.78, 0.7, 1.0)),
                ));
                row.spawn((
                    SettingsValueText { index: i },
                    Text::new(""),
                    TextFont { font_size: 13.0, ..default() },
                    TextColor(Color::srgba(0.9, 0.85, 0.6, 1.0)),
                ));
            });
        }

        // Hint text
        panel.spawn((
            Text::new("Up/Down: navigate  |  Left/Right: adjust  |  Enter: select/rebind  |  ESC: back"),
            TextFont { font_size: 11.0, ..default() },
            TextColor(Color::srgba(0.5, 0.5, 0.55, 0.8)),
            Node {
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            },
        ));
    });

    // FPS counter (top-right, hidden by default)
    commands.spawn((
        FpsText,
        Text::new(""),
        TextFont {
            font_size: 12.0,
            ..default()
        },
        TextColor(Color::srgba(0.6, 0.6, 0.5, 0.7)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(4.0),
            right: Val::Px(8.0),
            ..default()
        },
        GlobalZIndex(100),
    ));
}

// ---------------------------------------------------------------------------
// Settings navigation and input
// ---------------------------------------------------------------------------

fn settings_menu_navigation(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepad_buttons: Res<ButtonInput<GamepadButton>>,
    mut state: ResMut<SettingsMenuState>,
    mut settings: ResMut<GameSettings>,
    mut panel_query: Query<&mut Node, (With<SettingsPanel>, Without<crate::hud::PauseMenuPanel>)>,
    mut pause_panel_query: Query<
        &mut Node,
        (With<crate::hud::PauseMenuPanel>, Without<SettingsPanel>),
    >,
    mut row_query: Query<(&SettingsRow, &mut BackgroundColor, &mut BorderColor)>,
    pause_state: Res<crate::hud::PauseState>,
) {
    if !state.is_open {
        return;
    }

    let cancel_pressed =
        keyboard.just_pressed(KeyCode::Escape) || gamepad_buttons.just_pressed(GamepadButton::East);
    let confirm_pressed = gamepad_buttons.just_pressed(GamepadButton::South);

    // Rebinding mode: capture next key press
    if state.rebinding {
        if cancel_pressed {
            state.rebinding = false;
            return;
        }
        for &key in keyboard.get_just_pressed() {
            if key == KeyCode::Escape {
                continue;
            }
            // Swap detection: if another keybind already uses this key, swap them
            let old_key = get_keybind(&settings.keybinds, state.selected);
            for other_idx in SETTINGS_COUNT..(SETTINGS_COUNT + KEYBIND_COUNT) {
                if other_idx != state.selected && get_keybind(&settings.keybinds, other_idx) == key
                {
                    set_keybind(&mut settings.keybinds, other_idx, old_key);
                    break;
                }
            }
            set_keybind(&mut settings.keybinds, state.selected, key);
            state.rebinding = false;
            break;
        }
        return;
    }

    // ESC: close settings, show pause menu
    if cancel_pressed {
        state.is_open = false;
        if let Ok(mut node) = panel_query.get_single_mut() {
            node.display = Display::None;
        }
        if let Ok(mut node) = pause_panel_query.get_single_mut() {
            node.display = if pause_state.paused {
                Display::Flex
            } else {
                Display::None
            };
        }
        settings.save();
        return;
    }

    // Navigate up/down (arrow keys + D-pad)
    if keyboard.just_pressed(KeyCode::ArrowUp)
        || gamepad_buttons.just_pressed(GamepadButton::DPadUp)
    {
        if state.selected > 0 {
            state.selected -= 1;
        }
    }
    if keyboard.just_pressed(KeyCode::ArrowDown)
        || gamepad_buttons.just_pressed(GamepadButton::DPadDown)
    {
        if state.selected < TOTAL_ITEMS - 1 {
            state.selected += 1;
        }
    }

    // Adjust values (arrow keys in settings + D-pad)
    let left = keyboard.just_pressed(KeyCode::ArrowLeft)
        || gamepad_buttons.just_pressed(GamepadButton::DPadLeft);
    let right = keyboard.just_pressed(KeyCode::ArrowRight)
        || gamepad_buttons.just_pressed(GamepadButton::DPadRight);
    let enter = keyboard.just_pressed(KeyCode::Enter) || confirm_pressed;

    let mut do_close = false;
    match state.selected {
        0 => {
            // SFX Volume (round to nearest 5% to avoid float drift)
            if left {
                settings.sfx_volume = ((settings.sfx_volume - 0.05).max(0.0) * 20.0).round() / 20.0;
            }
            if right {
                settings.sfx_volume = ((settings.sfx_volume + 0.05).min(1.0) * 20.0).round() / 20.0;
            }
        }
        1 => {
            // Music Volume
            if left {
                settings.music_volume =
                    ((settings.music_volume - 0.05).max(0.0) * 20.0).round() / 20.0;
            }
            if right {
                settings.music_volume =
                    ((settings.music_volume + 0.05).min(1.0) * 20.0).round() / 20.0;
            }
        }
        2 => {
            // Screen Shake
            if left || right || enter {
                settings.screen_shake = !settings.screen_shake;
            }
        }
        3 => {
            // Show Minimap
            if left || right || enter {
                settings.show_minimap = !settings.show_minimap;
            }
        }
        4 => {
            // Show FPS
            if left || right || enter {
                settings.show_fps = !settings.show_fps;
            }
        }
        5 => {
            // Fullscreen
            if left || right || enter {
                settings.fullscreen = !settings.fullscreen;
            }
        }
        6 => {
            // Resolution (cycle through presets, left/right)
            if left && settings.resolution_index > 0 {
                settings.resolution_index -= 1;
            }
            if right && settings.resolution_index < RESOLUTIONS.len() - 1 {
                settings.resolution_index += 1;
            }
        }
        7 => {
            // Reset to defaults
            if enter {
                *settings = GameSettings::default();
                settings.save();
            }
        }
        i if i >= SETTINGS_COUNT && i < SETTINGS_COUNT + KEYBIND_COUNT => {
            // Keybind: enter rebind mode
            if enter {
                state.rebinding = true;
            }
        }
        i if i == SETTINGS_COUNT + KEYBIND_COUNT => {
            // Back to Menu
            if enter {
                do_close = true;
            }
        }
        _ => {}
    }

    if do_close {
        state.is_open = false;
        if let Ok(mut node) = panel_query.get_single_mut() {
            node.display = Display::None;
        }
        if let Ok(mut node) = pause_panel_query.get_single_mut() {
            node.display = if pause_state.paused {
                Display::Flex
            } else {
                Display::None
            };
        }
        settings.save();
        return;
    }

    // Update visual highlights
    for (row, mut bg, mut border) in row_query.iter_mut() {
        if row.index == state.selected && state.rebinding {
            // Rebinding: bright amber pulse so the player knows we're waiting for input
            *bg = BackgroundColor(Color::srgba(0.25, 0.18, 0.04, 0.95));
            *border = BorderColor(Color::srgba(1.0, 0.85, 0.2, 1.0));
        } else if row.index == state.selected {
            *bg = BackgroundColor(Color::srgba(0.15, 0.13, 0.08, 0.95));
            *border = BorderColor(Color::srgba(0.9, 0.75, 0.3, 0.9));
        } else {
            *bg = BackgroundColor(Color::srgba(0.06, 0.06, 0.1, 0.8));
            *border = BorderColor(Color::srgba(0.2, 0.2, 0.3, 0.4));
        }
    }
}

fn set_keybind(keybinds: &mut KeyBinds, index: usize, key: KeyCode) {
    match index {
        8 => keybinds.move_up = key,
        9 => keybinds.move_down = key,
        10 => keybinds.move_left = key,
        11 => keybinds.move_right = key,
        12 => keybinds.interact = key,
        13 => keybinds.inventory = key,
        14 => keybinds.crafting = key,
        15 => keybinds.building = key,
        16 => keybinds.journal = key,
        17 => keybinds.skills = key,
        18 => keybinds.dodge = key,
        19 => keybinds.save = key,
        20 => keybinds.load = key,
        _ => {}
    }
}

fn get_keybind(keybinds: &KeyBinds, index: usize) -> KeyCode {
    match index {
        8 => keybinds.move_up,
        9 => keybinds.move_down,
        10 => keybinds.move_left,
        11 => keybinds.move_right,
        12 => keybinds.interact,
        13 => keybinds.inventory,
        14 => keybinds.crafting,
        15 => keybinds.building,
        16 => keybinds.journal,
        17 => keybinds.skills,
        18 => keybinds.dodge,
        19 => keybinds.save,
        20 => keybinds.load,
        _ => KeyCode::Escape,
    }
}

pub fn keycode_display(key: KeyCode) -> &'static str {
    match key {
        KeyCode::KeyA => "A",
        KeyCode::KeyB => "B",
        KeyCode::KeyC => "C",
        KeyCode::KeyD => "D",
        KeyCode::KeyE => "E",
        KeyCode::KeyF => "F",
        KeyCode::KeyG => "G",
        KeyCode::KeyH => "H",
        KeyCode::KeyI => "I",
        KeyCode::KeyJ => "J",
        KeyCode::KeyK => "K",
        KeyCode::KeyL => "L",
        KeyCode::KeyM => "M",
        KeyCode::KeyN => "N",
        KeyCode::KeyO => "O",
        KeyCode::KeyP => "P",
        KeyCode::KeyQ => "Q",
        KeyCode::KeyR => "R",
        KeyCode::KeyS => "S",
        KeyCode::KeyT => "T",
        KeyCode::KeyU => "U",
        KeyCode::KeyV => "V",
        KeyCode::KeyW => "W",
        KeyCode::KeyX => "X",
        KeyCode::KeyY => "Y",
        KeyCode::KeyZ => "Z",
        KeyCode::Digit0 => "0",
        KeyCode::Digit1 => "1",
        KeyCode::Digit2 => "2",
        KeyCode::Digit3 => "3",
        KeyCode::Digit4 => "4",
        KeyCode::Digit5 => "5",
        KeyCode::Digit6 => "6",
        KeyCode::Digit7 => "7",
        KeyCode::Digit8 => "8",
        KeyCode::Digit9 => "9",
        KeyCode::Space => "Space",
        KeyCode::Tab => "Tab",
        KeyCode::Enter => "Enter",
        KeyCode::Escape => "Esc",
        KeyCode::ArrowUp => "Up",
        KeyCode::ArrowDown => "Down",
        KeyCode::ArrowLeft => "Left",
        KeyCode::ArrowRight => "Right",
        KeyCode::ShiftLeft => "L-Shift",
        KeyCode::ShiftRight => "R-Shift",
        KeyCode::ControlLeft => "L-Ctrl",
        KeyCode::ControlRight => "R-Ctrl",
        KeyCode::F1 => "F1",
        KeyCode::F2 => "F2",
        KeyCode::F3 => "F3",
        KeyCode::F4 => "F4",
        KeyCode::F5 => "F5",
        KeyCode::F6 => "F6",
        KeyCode::F7 => "F7",
        KeyCode::F8 => "F8",
        KeyCode::F9 => "F9",
        KeyCode::F10 => "F10",
        KeyCode::F11 => "F11",
        KeyCode::F12 => "F12",
        _ => "?",
    }
}

// ---------------------------------------------------------------------------
// Display update
// ---------------------------------------------------------------------------

fn update_settings_display(
    state: Res<SettingsMenuState>,
    settings: Res<GameSettings>,
    mut value_query: Query<(&SettingsValueText, &mut Text)>,
) {
    if !state.is_open {
        return;
    }

    for (val, mut text) in value_query.iter_mut() {
        let display = match val.index {
            0 => format!("{}%", (settings.sfx_volume * 100.0).round() as i32),
            1 => format!("{}%", (settings.music_volume * 100.0).round() as i32),
            2 => {
                if settings.screen_shake {
                    "On".into()
                } else {
                    "Off".into()
                }
            }
            3 => {
                if settings.show_minimap {
                    "On".into()
                } else {
                    "Off".into()
                }
            }
            4 => {
                if settings.show_fps {
                    "On".into()
                } else {
                    "Off".into()
                }
            }
            5 => {
                if settings.fullscreen {
                    "On".into()
                } else {
                    "Off".into()
                }
            }
            6 => {
                let idx = settings.resolution_index.min(RESOLUTIONS.len() - 1);
                let (w, h) = RESOLUTIONS[idx];
                format!("{}x{}", w, h)
            }
            7 => "[ Reset ]".into(),
            i if i >= SETTINGS_COUNT && i < SETTINGS_COUNT + KEYBIND_COUNT => {
                if state.rebinding && i == state.selected {
                    "Press any key...".into()
                } else {
                    keycode_display(get_keybind(&settings.keybinds, i)).into()
                }
            }
            i if i == SETTINGS_COUNT + KEYBIND_COUNT => "[ Enter ]".into(),
            _ => String::new(),
        };
        **text = display;
    }
}

// ---------------------------------------------------------------------------
// FPS counter
// ---------------------------------------------------------------------------

fn update_fps_counter(
    time: Res<Time>,
    settings: Res<GameSettings>,
    mut query: Query<(&mut Text, &mut Node), With<FpsText>>,
    mut smoothed: Local<f32>,
) {
    let Ok((mut text, mut node)) = query.get_single_mut() else {
        return;
    };

    if settings.show_fps {
        node.display = Display::Flex;
        let raw = 1.0 / time.delta_secs().max(0.001);
        // Exponential moving average (weight 0.05 = ~20 frame smoothing)
        if *smoothed <= 0.0 {
            *smoothed = raw;
        } else {
            *smoothed = *smoothed * 0.95 + raw * 0.05;
        }
        **text = format!("FPS: {:.0}", *smoothed);
    } else {
        node.display = Display::None;
    }
}

// ---------------------------------------------------------------------------
// Sync settings into audio resource
// ---------------------------------------------------------------------------

fn sync_audio_from_settings(
    settings: Res<GameSettings>,
    mut audio: ResMut<GameAudio>,
    mut minimap_state: ResMut<crate::minimap::MinimapState>,
) {
    if !settings.is_changed() {
        return;
    }
    audio.sfx_volume = settings.sfx_volume;
    audio.music_volume = settings.music_volume;
    minimap_state.minimap_visible = settings.show_minimap;
}

// ---------------------------------------------------------------------------
// Fullscreen
// ---------------------------------------------------------------------------

fn apply_fullscreen(settings: Res<GameSettings>, mut windows: Query<&mut Window>) {
    if !settings.is_changed() {
        return;
    }
    let Ok(mut window) = windows.get_single_mut() else {
        return;
    };
    window.mode = if settings.fullscreen {
        bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Current)
    } else {
        bevy::window::WindowMode::Windowed
    };
}

fn toggle_fullscreen_f11(keyboard: Res<ButtonInput<KeyCode>>, mut settings: ResMut<GameSettings>) {
    if keyboard.just_pressed(KeyCode::F11) {
        settings.fullscreen = !settings.fullscreen;
    }
}

// ---------------------------------------------------------------------------
// Resolution
// ---------------------------------------------------------------------------

fn apply_resolution(settings: Res<GameSettings>, mut windows: Query<&mut Window>) {
    if !settings.is_changed() {
        return;
    }
    let Ok(mut window) = windows.get_single_mut() else {
        return;
    };
    if !settings.fullscreen {
        let idx = settings.resolution_index.min(RESOLUTIONS.len() - 1);
        let (w, h) = RESOLUTIONS[idx];
        window.resolution.set(w as f32, h as f32);
    }
}
