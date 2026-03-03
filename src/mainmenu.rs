use bevy::prelude::*;
use crate::saveload::LoadRequested;

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

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MainMenuActive::default())
            .add_systems(Startup, spawn_main_menu)
            .add_systems(Update, handle_main_menu_input);
    }
}

fn spawn_main_menu(mut commands: Commands) {
    // Root node — full-screen centered column
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
            BackgroundColor(Color::srgba(0.02, 0.02, 0.06, 0.95)),
            // High z-index so it covers the game world
            ZIndex(100),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("DRIFTLANDS"),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.75, 0.3)),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            // Subtitle
            parent.spawn((
                Text::new("A world of discovery awaits"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                },
            ));

            // Menu options
            parent.spawn((
                Text::new("[N]  New Game"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(12.0)),
                    ..default()
                },
            ));

            parent.spawn((
                Text::new("[L]  Load Game"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(12.0)),
                    ..default()
                },
            ));

            parent.spawn((
                Text::new("[Q]  Quit"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Footer
            parent.spawn((
                Text::new("v0.5 Early Access"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.4, 0.5)),
            ));
        });
}

fn handle_main_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut menu: ResMut<MainMenuActive>,
    mut menu_ui_query: Query<&mut Visibility, With<MainMenuUI>>,
    mut exit_writer: EventWriter<AppExit>,
    mut load_requested: ResMut<LoadRequested>,
) {
    if !menu.active {
        // Ensure UI is hidden when menu is inactive
        for mut vis in menu_ui_query.iter_mut() {
            *vis = Visibility::Hidden;
        }
        return;
    }

    // Ensure UI is visible
    for mut vis in menu_ui_query.iter_mut() {
        *vis = Visibility::Visible;
    }

    if keyboard.just_pressed(KeyCode::KeyN) {
        // New Game — dismiss the menu; world is already generated at startup
        menu.active = false;
    }

    if keyboard.just_pressed(KeyCode::KeyL) {
        // Load Game — dismiss menu and request a load from the saveload system
        menu.active = false;
        load_requested.requested = true;
    }

    if keyboard.just_pressed(KeyCode::KeyQ) {
        exit_writer.send(AppExit::Success);
    }
}
