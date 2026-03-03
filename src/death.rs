use bevy::prelude::*;
use crate::player::{Player, Health, Hunger};
use crate::inventory::{Inventory, InventorySlot};
use crate::world::TILE_SIZE;
use crate::building::{Building, BuildingType};
use crate::daynight::DayNightCycle;

pub struct DeathPlugin;

impl Plugin for DeathPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnPoint::default())
            .insert_resource(DeathScreen::default())
            .insert_resource(DeathStats::default())
            .add_systems(Update, (
                check_player_death,
                update_death_screen,
                recover_death_marker,
                set_bed_spawn,
            ));
    }
}

#[derive(Resource)]
pub struct SpawnPoint {
    pub position: Vec3,
}

impl Default for SpawnPoint {
    fn default() -> Self {
        Self {
            position: Vec3::new(TILE_SIZE * 16.0, TILE_SIZE * 16.0, 10.0),
        }
    }
}

#[derive(Resource, Default)]
pub struct DeathScreen {
    pub active: bool,
    pub timer: f32,
    pub items_lost: u32,
    /// Stash death position so we can spawn the marker on respawn
    pub death_pos: Vec3,
    /// Items collected from inventory at time of death
    pub dropped_items: Vec<InventorySlot>,
}

#[derive(Resource, Default)]
pub struct DeathStats {
    pub total_kills: u32,
}

#[derive(Component)]
pub struct DeathMarker {
    pub items: Vec<InventorySlot>,
}

#[derive(Component)]
pub struct DeathScreenUI;

fn check_player_death(
    mut commands: Commands,
    mut player_query: Query<(&mut Health, &Transform), With<Player>>,
    mut inventory: ResMut<Inventory>,
    mut death_screen: ResMut<DeathScreen>,
    cycle: Res<DayNightCycle>,
    death_stats: Res<DeathStats>,
) {
    // Don't trigger again while death screen is active
    if death_screen.active {
        return;
    }

    let Ok((mut health, transform)) = player_query.get_single_mut() else {
        return;
    };

    if !health.is_dead() {
        return;
    }

    let death_pos = transform.translation;

    // Collect all items from inventory
    let mut dropped_items = Vec::new();
    let mut items_lost: u32 = 0;
    for slot in inventory.slots.iter_mut() {
        if let Some(s) = slot.take() {
            items_lost += s.count;
            dropped_items.push(s);
        }
    }

    // Activate death screen
    death_screen.active = true;
    death_screen.timer = 3.0;
    death_screen.items_lost = items_lost;
    death_screen.death_pos = death_pos;
    death_screen.dropped_items = dropped_items;

    // Freeze player health at 0 so is_dead() stays true until we respawn
    health.current = 0.0;

    // Spawn death screen UI
    commands.spawn((
        DeathScreenUI,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
        GlobalZIndex(100),
    )).with_children(|parent| {
        // "YOU DIED" title
        parent.spawn((
            Text::new("YOU DIED"),
            TextFont {
                font_size: 64.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.15, 0.15)),
            Node {
                margin: UiRect::bottom(Val::Px(30.0)),
                ..default()
            },
        ));

        // Day count
        parent.spawn((
            Text::new(format!("Day {}", cycle.day_count)),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
            Node {
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
        ));

        // Items lost
        parent.spawn((
            Text::new(format!("Items lost: {}", items_lost)),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
            Node {
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
        ));

        // Total kills
        parent.spawn((
            Text::new(format!("Total kills: {}", death_stats.total_kills)),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
            Node {
                margin: UiRect::bottom(Val::Px(30.0)),
                ..default()
            },
        ));

        // Press any key prompt
        parent.spawn((
            Text::new("Press any key..."),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ));
    });
}

fn update_death_screen(
    mut commands: Commands,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut death_screen: ResMut<DeathScreen>,
    mut player_query: Query<(&mut Health, &mut Hunger, &mut Transform), With<Player>>,
    existing_markers: Query<Entity, With<DeathMarker>>,
    ui_query: Query<Entity, With<DeathScreenUI>>,
    spawn_point: Res<SpawnPoint>,
) {
    if !death_screen.active {
        return;
    }

    // Tick timer
    death_screen.timer -= time.delta_secs();

    // Any key press skips the timer
    if keyboard.get_just_pressed().len() > 0 {
        death_screen.timer = 0.0;
    }

    // When timer expires, do the actual respawn
    if death_screen.timer <= 0.0 {
        let Ok((mut health, mut hunger, mut transform)) = player_query.get_single_mut() else {
            return;
        };

        let death_pos = death_screen.death_pos;
        let dropped_items = std::mem::take(&mut death_screen.dropped_items);

        // Remove old death marker (only one at a time)
        for entity in existing_markers.iter() {
            commands.entity(entity).despawn();
        }

        // Spawn new death marker at death position
        if !dropped_items.is_empty() {
            commands.spawn((
                DeathMarker { items: dropped_items },
                Sprite {
                    color: Color::srgb(1.0, 1.0, 0.2),
                    custom_size: Some(Vec2::new(10.0, 10.0)),
                    ..default()
                },
                Transform::from_xyz(death_pos.x, death_pos.y, 8.0),
            ));
        }

        // Respawn at spawn point (bed or default)
        transform.translation = spawn_point.position;
        health.current = health.max;
        hunger.current = hunger.max;
        hunger.starvation_timer = 0.0;

        // Despawn death screen UI
        for entity in ui_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        death_screen.active = false;
    }
}

fn recover_death_marker(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    marker_query: Query<(Entity, &Transform, &DeathMarker), Without<Player>>,
    mut inventory: ResMut<Inventory>,
) {
    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    for (entity, tf, marker) in marker_query.iter() {
        let dist = player_pos.distance(tf.translation.truncate());
        if dist <= 24.0 {
            // Recover all items
            for item in &marker.items {
                inventory.add_item(item.item, item.count);
            }
            commands.entity(entity).despawn();
        }
    }
}

fn set_bed_spawn(
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    bed_query: Query<(&Transform, &Building), Without<Player>>,
    mut spawn_point: ResMut<SpawnPoint>,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) { return; }
    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    for (bed_tf, building) in bed_query.iter() {
        if building.building_type != BuildingType::Bed { continue; }
        let dist = player_pos.distance(bed_tf.translation.truncate());
        if dist <= 32.0 {
            spawn_point.position = Vec3::new(bed_tf.translation.x, bed_tf.translation.y, 10.0);
            return;
        }
    }
}
