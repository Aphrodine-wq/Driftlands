use bevy::prelude::*;
use crate::crafting::CraftingSystem;
use crate::building::ChestUI;
use crate::npc::TradeMenu;
use crate::experiment::ExperimentSlots;
use crate::mainmenu::MainMenuActive;

/// Resource tracking whether the controls overlay is visible.
#[derive(Resource, Default)]
pub struct ControlsOverlay {
    pub is_visible: bool,
}

/// Marker for the controls overlay UI entity.
#[derive(Component)]
struct ControlsOverlayPanel;

/// Marker for the text node inside the overlay.
#[derive(Component)]
struct ControlsOverlayText;

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ControlsOverlay::default())
            .add_systems(Startup, spawn_controls_overlay)
            .add_systems(Update, (
                toggle_controls_overlay,
                update_controls_overlay,
            ));
    }
}

fn spawn_controls_overlay(mut commands: Commands) {
    // Semi-transparent background panel on the left side
    commands.spawn((
        ControlsOverlayPanel,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(60.0),
            left: Val::Px(0.0),
            padding: UiRect::all(Val::Px(12.0)),
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
    )).with_children(|parent| {
        parent.spawn((
            ControlsOverlayText,
            Text::new(""),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.95)),
        ));
    });
}

fn toggle_controls_overlay(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut overlay: ResMut<ControlsOverlay>,
    menu: Res<MainMenuActive>,
) {
    if menu.active {
        return;
    }

    // F1 toggles
    if keyboard.just_pressed(KeyCode::F1) {
        overlay.is_visible = !overlay.is_visible;
    }

    // Escape dismisses (only when overlay is visible)
    if overlay.is_visible && keyboard.just_pressed(KeyCode::Escape) {
        overlay.is_visible = false;
    }
}

fn update_controls_overlay(
    overlay: Res<ControlsOverlay>,
    crafting: Res<CraftingSystem>,
    trade_menu: Res<TradeMenu>,
    chest_ui: Res<ChestUI>,
    experiment_slots: Res<ExperimentSlots>,
    game_settings: Res<crate::settings::GameSettings>,
    mut panel_query: Query<&mut Node, With<ControlsOverlayPanel>>,
    mut text_query: Query<&mut Text, With<ControlsOverlayText>>,
) {
    let Ok(mut node) = panel_query.get_single_mut() else { return };
    let Ok(mut text) = text_query.get_single_mut() else { return };

    let menus_open = crafting.is_open
        || trade_menu.is_open
        || chest_ui.is_open
        || experiment_slots.is_open;

    let should_show = overlay.is_visible && !menus_open;

    if !should_show {
        node.display = Display::None;
        **text = String::new();
        return;
    }

    node.display = Display::Flex;

    let kb = &game_settings.keybinds;
    let k = crate::settings::keycode_display;

    let content = format!(
        "=== CONTROLS (F1 to close) ===\n\
         \n\
         --- Movement ---\n\
         \x20 {}/{}/{}/{}    Move\n\
         \x20 +/-         Zoom In/Out\n\
         \x20 F11         Toggle Fullscreen\n\
         \n\
         --- Combat ---\n\
         \x20 LMB         Attack / Gather\n\
         \x20 {}           Dodge Roll\n\
         \x20 R           Equip Armor/Shield\n\
         \n\
         --- Building ---\n\
         \x20 {}           Toggle Build Mode\n\
         \x20 Q           Cycle Building Type\n\
         \x20 RMB         Place Building\n\
         \x20 {}           Open/Close Door\n\
         \n\
         --- Crafting ---\n\
         \x20 {}           Open/Close Crafting\n\
         \x20 Up/Down     Select Recipe\n\
         \x20 Enter       Craft Selected\n\
         \x20 X           Experiment Table\n\
         \n\
         --- Inventory ---\n\
         \x20 {}           Open/Close Inventory\n\
         \x20 1-9         Select Hotbar Slot\n\
         \x20 RMB         Use / Eat / Place Item\n\
         \n\
         --- Info ---\n\
         \x20 {}           Quest Log\n\
         \x20 {}           Skills\n\
         \n\
         --- Other ---\n\
         \x20 {}           Interact (NPC/Bed)\n\
         \x20 {}          Save Slot (Selected)\n\
         \x20 {}          Load Slot (Selected)\n\
         \x20 Escape      Pause / Close Menu\n\
         \x20 F1          Toggle This Overlay",
        k(kb.move_up), k(kb.move_left), k(kb.move_down), k(kb.move_right),
        k(kb.dodge),
        k(kb.building),
        k(kb.interact),
        k(kb.crafting),
        k(kb.inventory),
        k(kb.journal),
        k(kb.skills),
        k(kb.interact),
        k(kb.save),
        k(kb.load),
    );

    **text = content;
}
