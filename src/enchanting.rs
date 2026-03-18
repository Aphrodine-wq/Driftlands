//! Enchanting system — makes enchanted weapons apply status effects on hit.
//!
//! The actual crafting recipes (FlameBlade = SteelSword + FireEssence, etc.)
//! live in `crafting.rs`. This plugin handles the combat integration side:
//! public helper functions that `combat.rs` calls during damage resolution.

use bevy::prelude::*;
use crate::inventory::ItemType;
use crate::status_effects::StatusEffectType;

// ---------------------------------------------------------------------------
// Player weapon helpers
// ---------------------------------------------------------------------------

/// Returns the status effect and duration that this weapon applies on hit, if any.
#[allow(dead_code)]
pub fn weapon_on_hit_effect(weapon: ItemType) -> Option<(StatusEffectType, f32)> {
    match weapon {
        ItemType::FlameBlade => Some((StatusEffectType::Burn, 4.0)),
        ItemType::FrostBlade => Some((StatusEffectType::Freeze, 3.0)), // 30% chance checked by caller
        ItemType::VenomBlade => Some((StatusEffectType::Poison, 5.0)),
        ItemType::LifestealBlade => None, // Lifesteal is handled differently (heal, not status effect)
        _ => None,
    }
}

/// Returns the fraction of damage healed for lifesteal weapons.
#[allow(dead_code)]
pub fn weapon_lifesteal_fraction(weapon: ItemType) -> f32 {
    match weapon {
        ItemType::LifestealBlade => 0.15,
        _ => 0.0,
    }
}

// ---------------------------------------------------------------------------
// Enemy attack helpers
// ---------------------------------------------------------------------------

/// Returns the status effect, duration, and chance (0.0-1.0) for an enemy's melee attack.
/// Called from `combat.rs` `enemy_attack_player` system.
pub fn enemy_on_hit_effect(enemy_type: crate::combat::EnemyType) -> Option<(StatusEffectType, f32, f32)> {
    use crate::combat::EnemyType;
    match enemy_type {
        EnemyType::FungalZombie => Some((StatusEffectType::Poison, 5.0, 0.3)),  // 30% chance
        EnemyType::LavaElemental => Some((StatusEffectType::Burn, 4.0, 1.0)),   // always
        EnemyType::IceWraith => Some((StatusEffectType::Freeze, 3.0, 0.5)),     // 50% chance
        EnemyType::BogLurker => Some((StatusEffectType::Poison, 3.0, 0.2)),     // 20% chance
        // Elite enemies (added by expansion)
        EnemyType::SandScorpion => Some((StatusEffectType::Poison, 4.0, 0.25)), // 25% (scorpion sting)
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct EnchantingPlugin;

impl Plugin for EnchantingPlugin {
    fn build(&self, _app: &mut App) {
        // Enchanting effects are applied via public functions called from combat.rs.
        // No systems needed — all integration happens through function calls.
    }
}
