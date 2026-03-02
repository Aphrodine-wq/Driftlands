use bevy::prelude::*;
use rand::Rng;
use crate::inventory::{Inventory, ItemType};
use crate::player::Player;
use crate::daynight::DayNightCycle;
use crate::building::BuildingState;

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TradeMenu::default())
            .insert_resource(HermitDialogueDisplay::default())
            .add_systems(Update, (
                spawn_trader,
                despawn_trader,
                trader_interaction,
                execute_trade,
                hermit_interaction,
            ));
    }
}

// ── Invulnerable marker ───────────────────────────────────────────────────────

/// Entities with this component cannot be attacked by the player.
#[derive(Component)]
pub struct Invulnerable;

// ── Trader ────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct TradeOffer {
    pub item_for_sale: ItemType,
    pub cost_item: ItemType,
    pub cost_count: u32,
    pub sold: bool,
}

#[derive(Component)]
pub struct Trader {
    pub offers: Vec<TradeOffer>,
    pub despawn_day: u32,
}

/// Resource tracking the trade UI state.
#[derive(Resource, Default)]
pub struct TradeMenu {
    pub is_open: bool,
    pub selected_offer: usize,
    pub trader_entity: Option<Entity>,
}

/// Resource tracking the last day on which a trader was spawned (used to
/// enforce the 3-5 day gap between visits).
#[derive(Resource, Default)]
struct TraderSpawnState {
    last_spawn_day: u32,
    next_interval: u32,
}

/// All possible offers a wandering trader may bring.
const OFFER_POOL: [(ItemType, ItemType, u32); 6] = [
    (ItemType::WheatSeed,  ItemType::Stone,    3),
    (ItemType::CarrotSeed, ItemType::Stone,    5),
    (ItemType::Blueprint,  ItemType::IronIngot, 10),
    (ItemType::IronOre,    ItemType::Stone,    5),
    (ItemType::RareHerb,   ItemType::IceShard, 8),
    (ItemType::HealthPotion, ItemType::MushroomCap, 3),
];

fn generate_offers(rng: &mut impl Rng) -> Vec<TradeOffer> {
    let count = rng.gen_range(3..=5_usize);
    // Shuffle pool by picking random indices without repeating
    let mut indices: Vec<usize> = (0..OFFER_POOL.len()).collect();
    // Partial Fisher-Yates
    for i in 0..count.min(indices.len()) {
        let j = rng.gen_range(i..indices.len());
        indices.swap(i, j);
    }
    indices[..count.min(OFFER_POOL.len())]
        .iter()
        .map(|&i| {
            let (item, cost_item, cost_count) = OFFER_POOL[i];
            TradeOffer { item_for_sale: item, cost_item, cost_count, sold: false }
        })
        .collect()
}

fn spawn_trader(
    mut commands: Commands,
    cycle: Res<DayNightCycle>,
    player_query: Query<&Transform, With<Player>>,
    trader_query: Query<&Trader>,
    mut spawn_state: Local<TraderSpawnState>,
) {
    // Initialise interval lazily on first call
    if spawn_state.next_interval == 0 {
        let mut rng = rand::thread_rng();
        spawn_state.next_interval = rng.gen_range(3..=5);
    }

    // Don't spawn if there is already a trader in the world
    if trader_query.iter().next().is_some() {
        return;
    }

    // Wait for enough days to have passed since last spawn
    let days_since = cycle.day_count.saturating_sub(spawn_state.last_spawn_day);
    if days_since < spawn_state.next_interval {
        return;
    }

    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    let mut rng = rand::thread_rng();
    let angle = rng.gen::<f32>() * std::f32::consts::TAU;
    let dist = rng.gen_range(200.0_f32..=400.0);
    let spawn_pos = player_pos + Vec2::new(angle.cos(), angle.sin()) * dist;

    let offers = generate_offers(&mut rng);
    let despawn_day = cycle.day_count + 2;

    commands.spawn((
        Trader { offers, despawn_day },
        Sprite {
            color: Color::srgb(0.1, 0.75, 0.2),
            custom_size: Some(Vec2::new(12.0, 12.0)),
            ..default()
        },
        Transform::from_xyz(spawn_pos.x, spawn_pos.y, 6.0),
    ));

    spawn_state.last_spawn_day = cycle.day_count;
    spawn_state.next_interval = rng.gen_range(3..=5);
}

fn despawn_trader(
    mut commands: Commands,
    cycle: Res<DayNightCycle>,
    trader_query: Query<(Entity, &Trader)>,
    mut trade_menu: ResMut<TradeMenu>,
) {
    for (entity, trader) in trader_query.iter() {
        if cycle.day_count >= trader.despawn_day {
            commands.entity(entity).despawn();
            if trade_menu.trader_entity == Some(entity) {
                trade_menu.is_open = false;
                trade_menu.trader_entity = None;
            }
        }
    }
}

fn trader_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    building_state: Res<BuildingState>,
    crafting: Res<crate::crafting::CraftingSystem>,
    player_query: Query<&Transform, With<Player>>,
    trader_query: Query<(Entity, &Transform), With<Trader>>,
    mut trade_menu: ResMut<TradeMenu>,
) {
    // Don't open trade if other menus are active
    if building_state.active || crafting.is_open {
        return;
    }

    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    // Find nearest trader within 32px
    let mut nearest: Option<(Entity, f32)> = None;
    for (entity, tf) in trader_query.iter() {
        let dist = player_pos.distance(tf.translation.truncate());
        if dist <= 32.0 {
            if nearest.is_none() || dist < nearest.unwrap().1 {
                nearest = Some((entity, dist));
            }
        }
    }

    if let Some((entity, _)) = nearest {
        if trade_menu.trader_entity == Some(entity) {
            // Toggle
            trade_menu.is_open = !trade_menu.is_open;
        } else {
            trade_menu.is_open = true;
            trade_menu.trader_entity = Some(entity);
            trade_menu.selected_offer = 0;
        }
    }
}

fn execute_trade(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut trade_menu: ResMut<TradeMenu>,
    mut trader_query: Query<&mut Trader>,
    mut inventory: ResMut<Inventory>,
) {
    if !trade_menu.is_open {
        return;
    }

    let Some(trader_entity) = trade_menu.trader_entity else { return };
    let Ok(mut trader) = trader_query.get_mut(trader_entity) else {
        trade_menu.is_open = false;
        trade_menu.trader_entity = None;
        return;
    };

    let offer_count = trader.offers.len();
    if offer_count == 0 {
        return;
    }

    // Navigate offers with arrows
    if keyboard.just_pressed(KeyCode::ArrowUp) && trade_menu.selected_offer > 0 {
        trade_menu.selected_offer -= 1;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown)
        && trade_menu.selected_offer < offer_count - 1
    {
        trade_menu.selected_offer += 1;
    }

    // Close with Escape
    if keyboard.just_pressed(KeyCode::Escape) {
        trade_menu.is_open = false;
        return;
    }

    // Execute trade on Enter
    if keyboard.just_pressed(KeyCode::Enter) {
        let idx = trade_menu.selected_offer;
        if idx < offer_count {
            let offer = &trader.offers[idx];
            if !offer.sold && inventory.has_items(offer.cost_item, offer.cost_count) {
                let item_to_buy = offer.item_for_sale;
                let cost_item = offer.cost_item;
                let cost_count = offer.cost_count;
                inventory.remove_items(cost_item, cost_count);
                inventory.add_item(item_to_buy, 1);
                trader.offers[idx].sold = true;
            }
        }
    }
}

// ── Hermit ────────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct Hermit {
    pub dialogue: Vec<String>,
    pub dialogue_index: usize,
    pub has_interacted: bool,
}

impl Hermit {
    pub fn new(lines: Vec<&'static str>) -> Self {
        Self {
            dialogue: lines.into_iter().map(|s| s.to_string()).collect(),
            dialogue_index: 0,
            has_interacted: false,
        }
    }
}

/// Resource holding the current hermit dialogue line displayed in the HUD.
#[derive(Resource, Default)]
pub struct HermitDialogueDisplay {
    pub text: String,
    pub timer: f32,
}

/// Public helper to spawn a hermit at a world position.
/// Called from `world/mod.rs` (wired up externally as noted in the task).
pub fn spawn_hermit(commands: &mut Commands, x: f32, y: f32) {
    let lines = vec![
        "The old ones built their towers to touch the sky...",
        "I have wandered these lands for thirty years.",
        "Beware the crystal caves at night.",
        "They say the volcanic region holds ancient secrets.",
        "Trade wisely — the wanderers don't stay long.",
        "The Driftlands were not always so wild.",
        "I found an ancient tablet near the swamp once.",
        "Sometimes I hear voices from the ruins underground.",
    ];

    commands.spawn((
        Hermit::new(lines),
        Invulnerable,
        Sprite {
            color: Color::srgb(0.7, 0.55, 0.3),
            custom_size: Some(Vec2::new(10.0, 14.0)),
            ..default()
        },
        Transform::from_xyz(x, y, 6.0),
    ));
}

fn hermit_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    building_state: Res<BuildingState>,
    crafting: Res<crate::crafting::CraftingSystem>,
    trade_menu: Res<TradeMenu>,
    player_query: Query<&Transform, With<Player>>,
    mut hermit_query: Query<(&Transform, &mut Hermit), Without<Player>>,
    mut dialogue_display: ResMut<HermitDialogueDisplay>,
    time: Res<Time>,
) {
    // Tick down any active dialogue timer
    if dialogue_display.timer > 0.0 {
        dialogue_display.timer -= time.delta_secs();
        if dialogue_display.timer <= 0.0 {
            dialogue_display.text.clear();
        }
    }

    // Don't open hermit dialogue if other menus are active
    if building_state.active || crafting.is_open || trade_menu.is_open {
        return;
    }

    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    // Find nearest hermit within 32px
    let mut best: Option<(f32, usize)> = None;
    for (idx, (tf, _)) in hermit_query.iter().enumerate() {
        let dist = player_pos.distance(tf.translation.truncate());
        if dist <= 32.0 {
            if best.is_none() || dist < best.unwrap().0 {
                best = Some((dist, idx));
            }
        }
    }

    let Some((_, target_idx)) = best else { return };

    for (i, (_, mut hermit)) in hermit_query.iter_mut().enumerate() {
        if i == target_idx {
            hermit.has_interacted = true;
            let line = hermit.dialogue[hermit.dialogue_index].clone();
            dialogue_display.text = format!("Hermit: \"{}\"", line);
            dialogue_display.timer = 5.0;
            // Advance to next line (cycles)
            hermit.dialogue_index = (hermit.dialogue_index + 1) % hermit.dialogue.len();
            break;
        }
    }
}
