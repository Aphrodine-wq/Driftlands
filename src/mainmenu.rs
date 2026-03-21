use bevy::prelude::*;
use bevy::ui::Interaction;
use bevy::input::gamepad::GamepadButton;
use crate::audio::SoundEvent;

pub struct MainMenuPlugin;

/// When `active` is true, the main menu is shown and gameplay is suppressed.
#[derive(Resource)]
pub struct MainMenuActive {
    pub active: bool,
}

impl Default for MainMenuActive {
    fn default() -> Self {
        Self { active: true }
    }
}

/// Marker for all main-menu UI entities so they can be shown/hidden together.
#[derive(Component)]
pub struct MainMenuUI;

/// Tracks which menu button is currently selected.
#[derive(Resource)]
struct MenuSelection {
    index: usize,
    count: usize,
}

impl Default for MenuSelection {
    fn default() -> Self {
        Self { index: 0, count: 4 }
    }
}

/// Marker for selectable menu buttons.
#[derive(Component)]
struct MenuButton {
    index: usize,
}

const MENU_BG: Color = Color::srgb(0.012, 0.012, 0.035);
const MENU_ACCENT: Color = Color::srgb(0.9, 0.75, 0.3);
const MENU_TEXT: Color = Color::srgb(0.75, 0.75, 0.8);
const MENU_DIM: Color = Color::srgb(0.35, 0.35, 0.45);
const BTN_BG: Color = Color::srgba(0.08, 0.08, 0.14, 0.9);
const BTN_BORDER: Color = Color::srgba(0.3, 0.28, 0.2, 0.5);
const BTN_SELECTED_BG: Color = Color::srgba(0.12, 0.10, 0.06, 0.95);
const BTN_SELECTED_BORDER: Color = Color::srgba(0.9, 0.75, 0.3, 0.8);

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MainMenuActive::default())
            .insert_resource(MenuSelection::default())
            .add_systems(Startup, spawn_main_menu)
            .add_systems(Update, (handle_main_menu_input, update_menu_visuals));
    }
}

fn spawn_main_menu(mut commands: Commands) {
    // Full-screen root
    commands
        .spawn((
            MainMenuUI,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(MENU_BG),
            ZIndex(100),
        ))
        .with_children(|root| {
            // Decorative top line
            root.spawn((
                Node {
                    width: Val::Px(360.0),
                    height: Val::Px(1.0),
                    margin: UiRect::bottom(Val::Px(24.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.9, 0.75, 0.3, 0.3)),
            ));

            // Title
            root.spawn((
                Text::new("DRIFTLANDS"),
                TextFont { font_size: 68.0, ..default() },
                TextColor(MENU_ACCENT),
                Node { margin: UiRect::bottom(Val::Px(6.0)), ..default() },
            ));

            // Subtitle
            root.spawn((
                Text::new("A world of discovery awaits"),
                TextFont { font_size: 15.0, ..default() },
                TextColor(MENU_DIM),
                Node { margin: UiRect::bottom(Val::Px(16.0)), ..default() },
            ));

            // Decorative bottom line
            root.spawn((
                Node {
                    width: Val::Px(360.0),
                    height: Val::Px(1.0),
                    margin: UiRect::bottom(Val::Px(32.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.9, 0.75, 0.3, 0.3)),
            ));

            // Button container
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(10.0),
                margin: UiRect::bottom(Val::Px(36.0)),
                ..default()
            })
            .with_children(|buttons| {
                spawn_menu_button(buttons, "New Game", 0);
                spawn_menu_button(buttons, "Continue", 1);
                spawn_menu_button(buttons, "Settings", 2);
                spawn_menu_button(buttons, "Quit", 3);
            });

            // Controls hint
            root.spawn((
                Text::new("Arrow Keys / W S to navigate  |  Enter to select"),
                TextFont { font_size: 12.0, ..default() },
                TextColor(MENU_DIM),
                Node { margin: UiRect::bottom(Val::Px(20.0)), ..default() },
            ));

            // Version & credits
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|footer| {
                footer.spawn((
                    Text::new("v0.6 Early Access"),
                    TextFont { font_size: 13.0, ..default() },
                    TextColor(MENU_DIM),
                    Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() },
                ));
                footer.spawn((
                    Text::new("Built with Bevy + Rust"),
                    TextFont { font_size: 11.0, ..default() },
                    TextColor(Color::srgba(0.3, 0.3, 0.4, 0.6)),
                ));
            });
        });
}

fn spawn_menu_button(parent: &mut ChildBuilder, label: &str, index: usize) {
    parent.spawn((
        MenuButton { index },
        Button,
        Node {
            width: Val::Px(260.0),
            height: Val::Px(46.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(BTN_BG),
        BorderColor(BTN_BORDER),
    ))
    .with_children(|btn| {
        btn.spawn((
            Text::new(label),
            TextFont { font_size: 21.0, ..default() },
            TextColor(MENU_TEXT),
        ));
    });
}

fn update_menu_visuals(
    selection: Res<MenuSelection>,
    menu: Res<MainMenuActive>,
    mut button_query: Query<(&MenuButton, &mut BackgroundColor, &mut BorderColor)>,
) {
    if !menu.active { return; }

    for (btn, mut bg, mut border) in button_query.iter_mut() {
        if btn.index == selection.index {
            *bg = BackgroundColor(BTN_SELECTED_BG);
            *border = BorderColor(BTN_SELECTED_BORDER);
        } else {
            *bg = BackgroundColor(BTN_BG);
            *border = BorderColor(BTN_BORDER);
        }
    }
}

fn handle_main_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepad_buttons: Res<ButtonInput<GamepadButton>>,
    mut menu: ResMut<MainMenuActive>,
    mut selection: ResMut<MenuSelection>,
    mut menu_ui_query: Query<&mut Visibility, With<MainMenuUI>>,
    mut exit_writer: EventWriter<AppExit>,
    active_slot: Res<crate::saveload::ActiveSaveSlot>,
    mut save_slot_browser_state: ResMut<crate::saveslots::SaveSlotBrowserState>,
    menu_button_query: Query<(&MenuButton, &Interaction)>,
    mut settings_state: ResMut<crate::settings::SettingsMenuState>,
    mut settings_panel_query: Query<&mut Node, With<crate::settings::SettingsPanel>>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    if !menu.active {
        for mut vis in menu_ui_query.iter_mut() {
            *vis = Visibility::Hidden;
        }
        return;
    }

    for mut vis in menu_ui_query.iter_mut() {
        *vis = Visibility::Visible;
    }

    // When the save-slot browser is open, ignore the underlying menu.
    if save_slot_browser_state.open {
        return;
    }
    if settings_state.is_open {
        return;
    }

    // Mouse click selects and activates immediately.
    let mut mouse_activated = false;
    for (btn, interaction) in menu_button_query.iter() {
        if *interaction == Interaction::Pressed {
            selection.index = btn.index;
            mouse_activated = true;
            break;
        }
    }

    // Navigation
    if keyboard.just_pressed(KeyCode::ArrowUp)
        || keyboard.just_pressed(KeyCode::KeyW)
        || gamepad_buttons.just_pressed(GamepadButton::DPadUp)
    {
        if selection.index > 0 {
            selection.index -= 1;
        } else {
            selection.index = selection.count - 1;
        }
    }
    if keyboard.just_pressed(KeyCode::ArrowDown)
        || keyboard.just_pressed(KeyCode::KeyS)
        || gamepad_buttons.just_pressed(GamepadButton::DPadDown)
    {
        selection.index = (selection.index + 1) % selection.count;
    }

    // Selection via Enter or direct key
    let activated = mouse_activated
        || keyboard.just_pressed(KeyCode::Enter)
        || keyboard.just_pressed(KeyCode::Space)
        || gamepad_buttons.just_pressed(GamepadButton::South);

    if activated {
        match selection.index {
            0 => {
                sound_events.send(SoundEvent::MenuOpen);
                menu.active = false
            } // New Game
            1 => {
                sound_events.send(SoundEvent::MenuOpen);
                save_slot_browser_state.open = true;
                save_slot_browser_state.context = crate::saveslots::SaveSlotBrowserContext::MainContinue;
                save_slot_browser_state.selected_focus = active_slot.index;
                save_slot_browser_state.confirm_delete = false;
                save_slot_browser_state.delete_target_slot = active_slot.index;
            } // Continue
            2 => {
                sound_events.send(SoundEvent::MenuOpen);
                // Settings
                settings_state.is_open = true;
                settings_state.selected = 0;
                if let Ok(mut node) = settings_panel_query.get_single_mut() {
                    node.display = Display::Flex;
                }
            }
            3 => {
                sound_events.send(SoundEvent::MenuOpen);
                exit_writer.send(AppExit::Success);
            } // Quit
            _ => {}
        }
    }

    // Direct keyboard shortcuts still work
    if keyboard.just_pressed(KeyCode::KeyN) { menu.active = false; }
    if keyboard.just_pressed(KeyCode::KeyL) {
        save_slot_browser_state.open = true;
        save_slot_browser_state.context = crate::saveslots::SaveSlotBrowserContext::MainContinue;
        save_slot_browser_state.selected_focus = active_slot.index;
        save_slot_browser_state.confirm_delete = false;
        save_slot_browser_state.delete_target_slot = active_slot.index;
    }
    if keyboard.just_pressed(KeyCode::KeyQ) { exit_writer.send(AppExit::Success); }
}
