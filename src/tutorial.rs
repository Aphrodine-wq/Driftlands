use bevy::prelude::*;
use std::collections::HashSet;
use crate::hud::not_paused;

pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TutorialState::default())
            .add_systems(Startup, spawn_tutorial_hint_ui)
            .add_systems(Update, (
                check_tutorial_triggers,
                update_tutorial_hint_display,
            ).chain().run_if(not_paused));
    }
}

/// Tracks which tutorial hints have been shown and the currently active hint.
#[derive(Resource)]
pub struct TutorialState {
    /// Set of hint IDs that have already been shown (persisted via save).
    pub shown_hints: HashSet<String>,
    /// The currently displayed hint text (empty if none).
    pub active_hint: String,
    /// Timer for how long the current hint remains visible.
    pub hint_timer: f32,
    /// Whether the initial spawn hint has been queued (set on first frame).
    pub spawn_hint_queued: bool,
    /// Tracks whether we have seen at least one item pickup (first gather complete).
    pub seen_pickup: bool,
    /// Tracks whether we have seen at least one craft.
    pub seen_craft: bool,
    /// Tracks whether we have seen at least one build.
    pub seen_build: bool,
}

impl Default for TutorialState {
    fn default() -> Self {
        Self {
            shown_hints: HashSet::new(),
            active_hint: String::new(),
            hint_timer: 0.0,
            spawn_hint_queued: false,
            seen_pickup: false,
            seen_craft: false,
            seen_build: false,
        }
    }
}

/// Marker for the tutorial hint UI text entity.
#[derive(Component)]
pub struct TutorialHintText;

fn spawn_tutorial_hint_ui(mut commands: Commands) {
    // Semi-transparent text box at top-center of screen
    commands.spawn((
        TutorialHintText,
        Text::new(""),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 0.85, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(50.0),
            left: Val::Percent(25.0),
            right: Val::Percent(25.0),
            ..default()
        },
    ));
}

/// Shows a hint if it hasn't been shown before. Returns true if a new hint was triggered.
fn try_show_hint(state: &mut TutorialState, hint_id: &str, hint_text: &str, duration: f32) -> bool {
    if state.shown_hints.contains(hint_id) {
        return false;
    }
    // Don't interrupt a currently showing hint
    if state.hint_timer > 0.0 {
        return false;
    }
    state.shown_hints.insert(hint_id.to_string());
    state.active_hint = hint_text.to_string();
    state.hint_timer = duration;
    true
}

/// Watches for tutorial trigger conditions each frame.
fn check_tutorial_triggers(
    mut tutorial: ResMut<TutorialState>,
    inventory: Res<crate::inventory::Inventory>,
    cycle: Res<crate::daynight::DayNightCycle>,
    building_query: Query<&crate::building::Building>,
) {
    // 1. Spawn hint: show on first frame of gameplay
    if !tutorial.spawn_hint_queued {
        tutorial.spawn_hint_queued = true;
        try_show_hint(
            &mut tutorial,
            "spawn",
            "WASD to move, hold LMB near trees to gather",
            8.0,
        );
    }

    // 2. After first gather: detect by checking if player has any gathered items.
    //    The simplest signal is that the inventory has gained items since spawn.
    //    We track this via seen_pickup: once inventory has any item, trigger.
    if !tutorial.seen_pickup {
        let has_items = inventory.slots.iter().any(|s| s.is_some());
        if has_items {
            tutorial.seen_pickup = true;
            try_show_hint(
                &mut tutorial,
                "first_gather",
                "Press C to open crafting. Try making a Workbench!",
                8.0,
            );
        }
    }

    // 3. After first craft: detect by checking if crafting menu has been used.
    //    We can detect this indirectly: if the player has crafted items (items that
    //    only come from crafting, like WoodPlank, Stick x4, Rope, etc.).
    //    Simplest: check if player has any crafted item (Stick, WoodPlank, Rope, Workbench, etc.)
    if !tutorial.seen_craft && tutorial.seen_pickup {
        let crafted_items = [
            crate::inventory::ItemType::WoodPlank,
            crate::inventory::ItemType::Rope,
            crate::inventory::ItemType::WoodAxe,
            crate::inventory::ItemType::WoodPickaxe,
            crate::inventory::ItemType::StoneAxe,
            crate::inventory::ItemType::Workbench,
            crate::inventory::ItemType::WoodSword,
            crate::inventory::ItemType::WoodFloor,
            crate::inventory::ItemType::WoodWall,
        ];
        let has_crafted = crafted_items.iter().any(|item| inventory.count_items(*item) > 0);
        if has_crafted {
            tutorial.seen_craft = true;
            try_show_hint(
                &mut tutorial,
                "first_craft",
                "Press B to enter build mode. Q to cycle, RMB to place.",
                8.0,
            );
        }
    }

    // 4. After first build: detect by checking if any Building entities exist.
    //    Then show the nightfall hint on first nightfall transition.
    if !tutorial.seen_build && tutorial.seen_craft {
        let has_built = !building_query.is_empty();
        if has_built {
            tutorial.seen_build = true;
        }
    }

    // 5. Nightfall hint: after first build, show on first nightfall (time > 0.78)
    if tutorial.seen_build {
        let is_night = cycle.time_of_day >= 0.78 && cycle.time_of_day <= 0.85;
        if is_night {
            try_show_hint(
                &mut tutorial,
                "first_nightfall",
                "Night is coming... craft a weapon!",
                8.0,
            );
        }
    }
}

/// Renders the active tutorial hint with fade-in/fade-out.
fn update_tutorial_hint_display(
    time: Res<Time>,
    mut tutorial: ResMut<TutorialState>,
    mut hint_query: Query<(&mut Text, &mut TextColor), With<TutorialHintText>>,
) {
    let Ok((mut text, mut color)) = hint_query.get_single_mut() else { return };

    if tutorial.hint_timer > 0.0 {
        **text = tutorial.active_hint.clone();

        // Fade in for first 0.5s, full opacity in middle, fade out in last 1.0s
        let alpha = if tutorial.hint_timer > 7.5 {
            // Fade in (first 0.5s of 8s duration)
            1.0 - (tutorial.hint_timer - 7.5) * 2.0
        } else if tutorial.hint_timer < 1.0 {
            // Fade out (last 1.0s)
            tutorial.hint_timer
        } else {
            1.0
        };

        *color = TextColor(Color::srgba(1.0, 1.0, 0.85, alpha.clamp(0.0, 0.9)));

        tutorial.hint_timer -= time.delta_secs();

        if tutorial.hint_timer <= 0.0 {
            tutorial.hint_timer = 0.0;
            tutorial.active_hint.clear();
        }
    } else {
        // No active hint — hide
        *color = TextColor(Color::srgba(1.0, 1.0, 0.85, 0.0));
        **text = String::new();
    }
}
