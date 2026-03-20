use bevy::prelude::*;

use crate::hud::not_paused;
use crate::player::{Health, Player};
use crate::combat::Enemy;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StatusEffectType {
    Poison,
    Burn,
    Freeze,
    Bleed,
    Stun,
    Regen,
    WellFed,
}

impl StatusEffectType {
    /// Damage (positive) or healing (negative) applied each tick.
    /// Returns `None` for effects that don't deal periodic damage/healing.
    fn tick_damage(&self, stacks: u32) -> Option<f32> {
        match self {
            Self::Poison => Some(2.0),
            Self::Burn => Some(3.0),
            Self::Bleed => Some(1.0 * stacks as f32),
            Self::Regen => Some(-2.0),
            Self::WellFed => Some(-1.0),
            Self::Freeze | Self::Stun => None,
        }
    }

    /// Seconds between successive ticks.
    fn tick_interval(&self) -> f32 {
        match self {
            Self::WellFed => 2.0,
            _ => 1.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Components & supporting data
// ---------------------------------------------------------------------------

pub struct StatusInstance {
    pub effect_type: StatusEffectType,
    pub remaining_secs: f32,
    pub stacks: u32,
    pub tick_timer: f32,
}

impl StatusInstance {
    fn new(effect_type: StatusEffectType, duration: f32) -> Self {
        Self {
            effect_type,
            remaining_secs: duration,
            stacks: 1,
            tick_timer: effect_type.tick_interval(),
        }
    }
}

#[derive(Component)]
pub struct ActiveStatusEffects {
    pub effects: Vec<StatusInstance>,
}

impl ActiveStatusEffects {
    fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[derive(Event)]
pub struct ApplyStatusEvent {
    pub target: Entity,
    pub effect: StatusEffectType,
    pub duration: f32,
}

// ---------------------------------------------------------------------------
// Public helpers (called by movement / combat systems)
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn is_frozen(effects: &ActiveStatusEffects) -> bool {
    effects
        .effects
        .iter()
        .any(|e| e.effect_type == StatusEffectType::Freeze)
}

#[allow(dead_code)]
pub fn is_stunned(effects: &ActiveStatusEffects) -> bool {
    effects
        .effects
        .iter()
        .any(|e| e.effect_type == StatusEffectType::Stun)
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Reads [`ApplyStatusEvent`]s and inserts / refreshes effects on the target
/// entity. Bleed stacks up to 3; all other effects simply refresh their
/// duration when re-applied.
fn apply_status_effects(
    mut commands: Commands,
    mut events: EventReader<ApplyStatusEvent>,
    mut query: Query<&mut ActiveStatusEffects>,
) {
    for ev in events.read() {
        if let Ok(mut active) = query.get_mut(ev.target) {
            if let Some(existing) = active
                .effects
                .iter_mut()
                .find(|e| e.effect_type == ev.effect)
            {
                // Refresh duration.
                existing.remaining_secs = ev.duration;

                // Bleed stacks up to 3; other effects do not stack.
                if ev.effect == StatusEffectType::Bleed {
                    existing.stacks = (existing.stacks + 1).min(3);
                }
            } else {
                active
                    .effects
                    .push(StatusInstance::new(ev.effect, ev.duration));
            }
        } else {
            // Entity doesn't have the component yet -- insert one.
            let mut container = ActiveStatusEffects::new();
            container
                .effects
                .push(StatusInstance::new(ev.effect, ev.duration));
            commands.entity(ev.target).insert(container);
        }
    }
}

/// Ticks every active status effect, applies periodic damage / healing,
/// removes expired effects, and strips the component when the list is empty.
fn tick_status_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ActiveStatusEffects, &mut Health)>,
) {
    let dt = time.delta_secs();

    for (entity, mut active, mut health) in &mut query {
        for inst in active.effects.iter_mut() {
            inst.remaining_secs -= dt;
            inst.tick_timer -= dt;

            if inst.tick_timer <= 0.0 {
                if let Some(dmg) = inst.effect_type.tick_damage(inst.stacks) {
                    if dmg > 0.0 {
                        health.take_damage(dmg);
                    } else {
                        health.heal(-dmg);
                    }
                }
                inst.tick_timer += inst.effect_type.tick_interval();
            }
        }

        // Remove expired effects.
        active.effects.retain(|e| e.remaining_secs > 0.0);

        // Strip the component entirely when nothing is active.
        if active.effects.is_empty() {
            commands.entity(entity).remove::<ActiveStatusEffects>();
        }
    }
}

/// Ticks status effects on enemies. Enemies store health in `Enemy.health`
/// (not the `Health` component), so they need their own tick system.
fn tick_enemy_status_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ActiveStatusEffects, &mut Enemy)>,
) {
    let dt = time.delta_secs();

    for (entity, mut active, mut enemy) in &mut query {
        for inst in active.effects.iter_mut() {
            inst.remaining_secs -= dt;
            inst.tick_timer -= dt;

            if inst.tick_timer <= 0.0 {
                if let Some(dmg) = inst.effect_type.tick_damage(inst.stacks) {
                    if dmg > 0.0 {
                        enemy.health -= dmg;
                    }
                    // Negative dmg = healing; enemies don't regen from status effects
                }
                inst.tick_timer += inst.effect_type.tick_interval();
            }
        }

        active.effects.retain(|e| e.remaining_secs > 0.0);

        if active.effects.is_empty() {
            commands.entity(entity).remove::<ActiveStatusEffects>();
        }
    }
}

/// Applies a subtle colour tint to sprites that have active status effects.
///
/// The highest-priority effect determines the tint colour. The sprite colour
/// is lerped 30% towards that colour every frame. Player entities are skipped
/// -- their tint is handled by a dedicated system.
fn status_visual_tint(
    mut query: Query<(&ActiveStatusEffects, &mut Sprite), Without<Player>>,
) {
    // Priority order (highest first).
    const PRIORITY: &[(StatusEffectType, Color)] = &[
        (StatusEffectType::Stun, Color::srgb(1.0, 1.0, 0.0)),
        (StatusEffectType::Freeze, Color::srgb(0.5, 0.7, 1.0)),
        (StatusEffectType::Burn, Color::srgb(1.0, 0.3, 0.1)),
        (StatusEffectType::Poison, Color::srgb(0.3, 0.8, 0.2)),
        (StatusEffectType::Bleed, Color::srgb(0.7, 0.1, 0.4)),
    ];

    for (active, mut sprite) in query.iter_mut() {
        let tint = PRIORITY.iter().find_map(|(etype, color)| {
            if active.effects.iter().any(|e| e.effect_type == *etype) {
                Some(*color)
            } else {
                None
            }
        });

        if let Some(target) = tint {
            let base = sprite.color.to_srgba();
            let t = target.to_srgba();
            const BLEND: f32 = 0.3;
            sprite.color = Color::srgb(
                base.red + (t.red - base.red) * BLEND,
                base.green + (t.green - base.green) * BLEND,
                base.blue + (t.blue - base.blue) * BLEND,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct StatusEffectsPlugin;

impl Plugin for StatusEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ApplyStatusEvent>().add_systems(
            Update,
            (apply_status_effects, tick_status_effects, tick_enemy_status_effects, status_visual_tint).run_if(not_paused),
        );
    }
}
