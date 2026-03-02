use bevy::prelude::*;
use rand::Rng;
use crate::player::{Player, Health};
use crate::daynight::{DayNightCycle, DayPhase};
use crate::inventory::{Inventory, ItemType};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            spawn_night_enemies,
            despawn_enemies_at_sunrise,
            enemy_ai,
            player_attack,
            enemy_attack_player,
            update_hit_flash,
        ));
    }
}

// --- Components ---

#[derive(Component)]
pub struct Enemy {
    pub enemy_type: EnemyType,
    pub health: f32,
    pub max_health: f32,
    pub damage: f32,
    pub speed: f32,
    pub aggro_range: f32,
    pub state: EnemyState,
    pub patrol_target: Vec2,
    pub attack_cooldown: Timer,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnemyType {
    ShadowCrawler,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnemyState {
    Idle,
    Patrol,
    Chase,
}

#[derive(Component)]
pub struct HitFlash {
    pub timer: Timer,
    pub original_color: Color,
}

#[derive(Component)]
pub struct PlayerAttackCooldown {
    pub timer: Timer,
}

// --- Systems ---

fn spawn_night_enemies(
    mut commands: Commands,
    cycle: Res<DayNightCycle>,
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<&Enemy>,
) {
    // Only spawn during Night phase
    if cycle.phase() != DayPhase::Night {
        return;
    }

    // Max 5 enemies
    if enemy_query.iter().count() >= 5 {
        return;
    }

    // Spawn chance per frame (roughly 1 enemy per 2-3 seconds)
    let mut rng = rand::thread_rng();
    if rng.gen::<f32>() > 0.01 {
        return;
    }

    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    // Random position 300-500px from player
    let angle = rng.gen::<f32>() * std::f32::consts::TAU;
    let dist = rng.gen_range(300.0..500.0);
    let spawn_pos = player_pos + Vec2::new(angle.cos(), angle.sin()) * dist;

    commands.spawn((
        Enemy {
            enemy_type: EnemyType::ShadowCrawler,
            health: 30.0,
            max_health: 30.0,
            damage: 5.0,
            speed: 80.0,
            aggro_range: 150.0,
            state: EnemyState::Idle,
            patrol_target: spawn_pos,
            attack_cooldown: Timer::from_seconds(1.0, TimerMode::Once),
        },
        Sprite {
            color: Color::srgb(0.4, 0.1, 0.5),
            custom_size: Some(Vec2::new(10.0, 10.0)),
            ..default()
        },
        Transform::from_xyz(spawn_pos.x, spawn_pos.y, 5.0),
    ));
}

fn despawn_enemies_at_sunrise(
    mut commands: Commands,
    cycle: Res<DayNightCycle>,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    // Despawn at Sunrise (time 0.2-0.3)
    if cycle.phase() != DayPhase::Sunrise {
        return;
    }

    for entity in enemy_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn enemy_ai(
    mut enemy_query: Query<(&mut Enemy, &mut Transform), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
    time: Res<Time>,
) {
    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    let mut rng = rand::thread_rng();

    for (mut enemy, mut tf) in enemy_query.iter_mut() {
        let enemy_pos = tf.translation.truncate();
        let dist_to_player = enemy_pos.distance(player_pos);

        // State transitions
        match enemy.state {
            EnemyState::Idle => {
                // Pick a patrol target
                let offset = Vec2::new(
                    rng.gen_range(-100.0..100.0),
                    rng.gen_range(-100.0..100.0),
                );
                enemy.patrol_target = enemy_pos + offset;
                enemy.state = EnemyState::Patrol;
            }
            EnemyState::Patrol => {
                if dist_to_player <= enemy.aggro_range {
                    enemy.state = EnemyState::Chase;
                } else {
                    // Move toward patrol target at half speed
                    let dir = (enemy.patrol_target - enemy_pos).normalize_or_zero();
                    let move_delta = dir * enemy.speed * 0.5 * time.delta_secs();
                    tf.translation.x += move_delta.x;
                    tf.translation.y += move_delta.y;

                    // If close to target, go idle again
                    if enemy_pos.distance(enemy.patrol_target) < 10.0 {
                        enemy.state = EnemyState::Idle;
                    }
                }
            }
            EnemyState::Chase => {
                if dist_to_player > 250.0 {
                    enemy.state = EnemyState::Patrol;
                } else {
                    // Move toward player at full speed
                    let dir = (player_pos - enemy_pos).normalize_or_zero();
                    let move_delta = dir * enemy.speed * time.delta_secs();
                    tf.translation.x += move_delta.x;
                    tf.translation.y += move_delta.y;
                }
            }
        }
    }
}

fn player_attack(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    building_state: Res<crate::building::BuildingState>,
    mut player_query: Query<(Entity, &Transform), With<Player>>,
    mut enemy_query: Query<(Entity, &Transform, &mut Enemy, &mut Sprite), Without<Player>>,
    mut cooldown_query: Query<&mut PlayerAttackCooldown>,
    mut inventory: ResMut<Inventory>,
) {
    // Don't attack in build mode
    if building_state.active {
        return;
    }

    // Handle cooldown
    if let Ok(mut cd) = cooldown_query.get_single_mut() {
        cd.timer.tick(time.delta());
        if !cd.timer.finished() {
            return;
        }
    }

    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok((player_entity, player_tf)) = player_query.get_single_mut() else { return };
    let player_pos = player_tf.translation.truncate();

    // Find nearest enemy within 40px
    let mut nearest: Option<(Entity, f32)> = None;
    for (entity, tf, _, _) in enemy_query.iter() {
        let dist = player_pos.distance(tf.translation.truncate());
        if dist <= 40.0 {
            if nearest.is_none() || dist < nearest.unwrap().1 {
                nearest = Some((entity, dist));
            }
        }
    }

    let Some((target_entity, _)) = nearest else { return };

    // Deal damage
    let mut killed = false;
    if let Ok((_, _, mut enemy, mut sprite)) = enemy_query.get_mut(target_entity) {
        enemy.health -= 10.0;

        // Flash white on hit
        let original_color = sprite.color;
        sprite.color = Color::WHITE;
        commands.entity(target_entity).insert(HitFlash {
            timer: Timer::from_seconds(0.1, TimerMode::Once),
            original_color,
        });

        if enemy.health <= 0.0 {
            killed = true;
        }
    }

    if killed {
        // Drop random item
        let mut rng = rand::thread_rng();
        let drop_item = if rng.gen_bool(0.5) { ItemType::Stone } else { ItemType::PlantFiber };
        inventory.add_item(drop_item, 1);
        commands.entity(target_entity).despawn();
    }

    // Set/reset cooldown
    if cooldown_query.get_single_mut().is_ok() {
        if let Ok(mut cd) = cooldown_query.get_single_mut() {
            cd.timer.reset();
        }
    } else {
        commands.entity(player_entity).insert(PlayerAttackCooldown {
            timer: Timer::from_seconds(0.5, TimerMode::Once),
        });
    }
}

fn enemy_attack_player(
    time: Res<Time>,
    mut enemy_query: Query<(&mut Enemy, &Transform), Without<Player>>,
    mut player_query: Query<(&Transform, &mut Health, &mut Sprite), With<Player>>,
    mut commands: Commands,
    player_entity_query: Query<Entity, With<Player>>,
) {
    let Ok((player_tf, mut health, mut sprite)) = player_query.get_single_mut() else { return };
    let player_pos = player_tf.translation.truncate();

    let mut took_damage = false;

    for (mut enemy, tf) in enemy_query.iter_mut() {
        if enemy.state != EnemyState::Chase {
            continue;
        }

        enemy.attack_cooldown.tick(time.delta());

        let dist = player_pos.distance(tf.translation.truncate());
        if dist <= 20.0 && enemy.attack_cooldown.finished() {
            health.take_damage(enemy.damage);
            enemy.attack_cooldown.reset();
            took_damage = true;
        }
    }

    // Flash player red when hit
    if took_damage {
        let original_color = sprite.color;
        sprite.color = Color::srgb(1.0, 0.2, 0.2);
        if let Ok(entity) = player_entity_query.get_single() {
            commands.entity(entity).insert(HitFlash {
                timer: Timer::from_seconds(0.1, TimerMode::Once),
                original_color,
            });
        }
    }
}

fn update_hit_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut HitFlash, &mut Sprite)>,
) {
    for (entity, mut flash, mut sprite) in query.iter_mut() {
        flash.timer.tick(time.delta());
        if flash.timer.finished() {
            sprite.color = flash.original_color;
            commands.entity(entity).remove::<HitFlash>();
        }
    }
}
