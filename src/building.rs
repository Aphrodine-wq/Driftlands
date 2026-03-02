use bevy::prelude::*;
use crate::player::Player;
use crate::inventory::{Inventory, ItemType};
use crate::world::TILE_SIZE;

pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BuildingState::default())
            .add_systems(Update, (
                toggle_build_mode,
                cycle_building_type,
                place_building,
                update_build_preview,
                door_interaction,
                roof_transparency,
                destroy_building,
            ));
    }
}

#[derive(Resource)]
pub struct BuildingState {
    pub active: bool,
    pub selected_type: BuildingType,
}

impl Default for BuildingState {
    fn default() -> Self {
        Self {
            active: false,
            selected_type: BuildingType::WoodFloor,
        }
    }
}

#[derive(Component)]
pub struct Building {
    pub building_type: BuildingType,
}

#[derive(Component)]
pub struct Door {
    pub is_open: bool,
}

#[derive(Component)]
pub struct Roof;

#[derive(Component)]
pub struct BuildPreview;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BuildingType {
    WoodFloor,
    WoodWall,
    WoodDoor,
    WoodRoof,
}

impl BuildingType {
    pub fn next(self) -> Self {
        match self {
            BuildingType::WoodFloor => BuildingType::WoodWall,
            BuildingType::WoodWall => BuildingType::WoodDoor,
            BuildingType::WoodDoor => BuildingType::WoodRoof,
            BuildingType::WoodRoof => BuildingType::WoodFloor,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            BuildingType::WoodFloor => "Wood Floor",
            BuildingType::WoodWall => "Wood Wall",
            BuildingType::WoodDoor => "Wood Door",
            BuildingType::WoodRoof => "Wood Roof",
        }
    }

    pub fn required_item(&self) -> ItemType {
        match self {
            BuildingType::WoodFloor => ItemType::WoodFloor,
            BuildingType::WoodWall => ItemType::WoodWall,
            BuildingType::WoodDoor => ItemType::WoodDoor,
            BuildingType::WoodRoof => ItemType::WoodRoof,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            BuildingType::WoodFloor => Color::srgb(0.6, 0.4, 0.2),
            BuildingType::WoodWall => Color::srgb(0.5, 0.3, 0.15),
            BuildingType::WoodDoor => Color::srgb(0.55, 0.35, 0.2),
            BuildingType::WoodRoof => Color::srgb(0.35, 0.2, 0.1),
        }
    }

    pub fn size(&self) -> Vec2 {
        match self {
            BuildingType::WoodFloor => Vec2::new(TILE_SIZE, TILE_SIZE),
            BuildingType::WoodWall => Vec2::new(TILE_SIZE, 24.0),
            BuildingType::WoodDoor => Vec2::new(10.0, 20.0),
            BuildingType::WoodRoof => Vec2::new(TILE_SIZE, TILE_SIZE),
        }
    }

    pub fn z_depth(&self) -> f32 {
        match self {
            BuildingType::WoodFloor => 1.0,
            BuildingType::WoodWall => 3.0,
            BuildingType::WoodDoor => 3.0,
            BuildingType::WoodRoof => 15.0,
        }
    }

    /// Returns materials returned when destroyed (50% of recipe, min 1)
    pub fn salvage(&self) -> Vec<(ItemType, u32)> {
        match self {
            BuildingType::WoodFloor => vec![(ItemType::WoodPlank, 2)],
            BuildingType::WoodWall => vec![(ItemType::WoodPlank, 2), (ItemType::Stick, 1)],
            BuildingType::WoodDoor => vec![(ItemType::WoodPlank, 3)],
            BuildingType::WoodRoof => vec![(ItemType::WoodPlank, 3), (ItemType::Stick, 2)],
        }
    }
}

fn toggle_build_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut building_state: ResMut<BuildingState>,
) {
    if keyboard.just_pressed(KeyCode::KeyB) {
        building_state.active = !building_state.active;
    }
}

fn cycle_building_type(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut building_state: ResMut<BuildingState>,
) {
    if !building_state.active {
        return;
    }
    if keyboard.just_pressed(KeyCode::KeyQ) {
        building_state.selected_type = building_state.selected_type.next();
    }
}

fn update_build_preview(
    mut commands: Commands,
    building_state: Res<BuildingState>,
    player_query: Query<&Transform, With<Player>>,
    preview_query: Query<Entity, With<BuildPreview>>,
) {
    for entity in preview_query.iter() {
        commands.entity(entity).despawn();
    }

    if !building_state.active {
        return;
    }

    let Ok(player_tf) = player_query.get_single() else { return };

    let snapped_x = (player_tf.translation.x / TILE_SIZE).round() * TILE_SIZE;
    let snapped_y = (player_tf.translation.y / TILE_SIZE).round() * TILE_SIZE;

    let bt = building_state.selected_type;
    let mut color = bt.color();
    color = color.with_alpha(0.5);

    commands.spawn((
        BuildPreview,
        Sprite {
            color,
            custom_size: Some(bt.size()),
            ..default()
        },
        Transform::from_xyz(snapped_x, snapped_y, bt.z_depth() + 0.1),
    ));
}

fn place_building(
    mut commands: Commands,
    building_state: Res<BuildingState>,
    mut inventory: ResMut<Inventory>,
    player_query: Query<&Transform, With<Player>>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if !building_state.active || !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    let Ok(player_tf) = player_query.get_single() else { return };
    let bt = building_state.selected_type;

    if !inventory.has_items(bt.required_item(), 1) {
        return;
    }

    let snapped_x = (player_tf.translation.x / TILE_SIZE).round() * TILE_SIZE;
    let snapped_y = (player_tf.translation.y / TILE_SIZE).round() * TILE_SIZE;

    inventory.remove_items(bt.required_item(), 1);

    let mut entity_commands = commands.spawn((
        Building { building_type: bt },
        Sprite {
            color: bt.color(),
            custom_size: Some(bt.size()),
            ..default()
        },
        Transform::from_xyz(snapped_x, snapped_y, bt.z_depth()),
    ));

    if bt == BuildingType::WoodDoor {
        entity_commands.insert(Door { is_open: false });
    }
    if bt == BuildingType::WoodRoof {
        entity_commands.insert(Roof);
    }
}

fn door_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    mut door_query: Query<(&Transform, &mut Door, &mut Sprite), Without<Player>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    let mut nearest: Option<(f32, Entity)> = None;
    for (tf, _, _) in door_query.iter() {
        let dist = player_pos.distance(tf.translation.truncate());
        if dist <= 32.0 {
            if nearest.is_none() || dist < nearest.unwrap().0 {
                // We can't store Entity from the query easily, so we'll just toggle all in range
                nearest = Some((dist, Entity::PLACEHOLDER));
            }
        }
    }

    if nearest.is_none() {
        return;
    }

    // Toggle nearest door
    let mut best_dist = f32::MAX;
    let mut best_idx = None;
    for (i, (tf, _, _)) in door_query.iter().enumerate() {
        let dist = player_pos.distance(tf.translation.truncate());
        if dist <= 32.0 && dist < best_dist {
            best_dist = dist;
            best_idx = Some(i);
        }
    }

    if let Some(idx) = best_idx {
        for (i, (_, mut door, mut sprite)) in door_query.iter_mut().enumerate() {
            if i == idx {
                door.is_open = !door.is_open;
                if door.is_open {
                    sprite.color = Color::srgba(0.55, 0.35, 0.2, 0.4);
                    sprite.custom_size = Some(Vec2::new(4.0, 20.0));
                } else {
                    sprite.color = Color::srgb(0.55, 0.35, 0.2);
                    sprite.custom_size = Some(Vec2::new(10.0, 20.0));
                }
                break;
            }
        }
    }
}

fn roof_transparency(
    player_query: Query<&Transform, With<Player>>,
    mut roof_query: Query<(&Transform, &mut Sprite), (With<Roof>, Without<Player>)>,
) {
    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    for (tf, mut sprite) in roof_query.iter_mut() {
        let roof_pos = tf.translation.truncate();
        let dist = player_pos.distance(roof_pos);
        if dist < TILE_SIZE * 0.8 {
            sprite.color = Color::srgba(0.35, 0.2, 0.1, 0.3);
        } else {
            sprite.color = Color::srgb(0.35, 0.2, 0.1);
        }
    }
}

fn destroy_building(
    mut commands: Commands,
    building_state: Res<BuildingState>,
    mouse: Res<ButtonInput<MouseButton>>,
    player_query: Query<&Transform, With<Player>>,
    building_query: Query<(Entity, &Transform, &Building), Without<Player>>,
    mut inventory: ResMut<Inventory>,
) {
    if !building_state.active || !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    // Find nearest building within 32px
    let mut nearest: Option<(Entity, f32, BuildingType)> = None;
    for (entity, tf, building) in building_query.iter() {
        let dist = player_pos.distance(tf.translation.truncate());
        if dist <= 32.0 {
            if nearest.is_none() || dist < nearest.as_ref().unwrap().1 {
                nearest = Some((entity, dist, building.building_type));
            }
        }
    }

    if let Some((entity, _, bt)) = nearest {
        // Return 50% materials
        for (item, count) in bt.salvage() {
            inventory.add_item(item, count);
        }
        commands.entity(entity).despawn();
    }
}
