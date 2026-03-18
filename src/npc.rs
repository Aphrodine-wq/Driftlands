use bevy::prelude::*;
use rand::Rng;
use serde::{Serialize, Deserialize};
use crate::hud::not_paused;
use crate::inventory::{Inventory, ItemType};
use crate::player::Player;
use crate::daynight::{DayNightCycle, DayPhase};
use crate::audio::SoundEvent;
use crate::building::BuildingState;
use crate::world::ChunkObject;
use crate::world::generation::WorldGenerator;

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TradeMenu::default())
            .insert_resource(HermitDialogueDisplay::default())
            .insert_resource(NpcDialogueDisplay::default())
            .add_systems(Update, (
                spawn_trader,
                despawn_trader,
                trader_interaction,
                hermit_interaction,
                npc_schedule_behavior,
                npc_interaction,
            ).run_if(not_paused))
            .add_systems(Update, execute_trade);
    }
}

// ── NPC Type Enum ────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NpcType {
    WanderingTrader,
    Hermit,
    Blacksmith,
    QuestGiver,
    Farmer,
}

// ── Schedule / Wander Component ──────────────────────────────────────────────

/// NPCs with this component wander near their spawn point during the day
/// and stand still at night.
#[derive(Component)]
pub struct NpcSchedule {
    pub npc_type: NpcType,
    pub home_pos: Vec2,
    /// Timer for wander direction changes.
    pub wander_timer: f32,
    /// Current wander velocity.
    pub wander_dir: Vec2,
}

/// Resource for NPC dialogue display (non-hermit NPCs).
#[derive(Resource, Default)]
pub struct NpcDialogueDisplay {
    pub text: String,
    pub timer: f32,
}

// ── Blacksmith ───────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct Blacksmith {
    pub dialogue: Vec<String>,
    pub dialogue_index: usize,
}

impl Blacksmith {
    pub fn new() -> Self {
        Self {
            dialogue: vec![
                "I can repair your equipment... for a price.".to_string(),
                "Bring me iron and I'll restore your gear.".to_string(),
                "The mountain ores make the finest blades.".to_string(),
                "I've been forging in these mountains for decades.".to_string(),
            ],
            dialogue_index: 0,
        }
    }
}

// ── Quest Giver ──────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct QuestGiver {
    pub dialogue: Vec<String>,
    pub dialogue_index: usize,
}

impl QuestGiver {
    pub fn new() -> Self {
        Self {
            dialogue: vec![
                "I have work for a capable drifter...".to_string(),
                "The creatures grow bolder each night.".to_string(),
                "There are resources to gather if you're willing.".to_string(),
                "Prove your worth and I'll share what I know.".to_string(),
            ],
            dialogue_index: 0,
        }
    }
}

// ── Farmer ───────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct Farmer {
    pub dialogue: Vec<String>,
    pub dialogue_index: usize,
}

impl Farmer {
    pub fn new() -> Self {
        Self {
            dialogue: vec![
                "Fresh seeds for sale! Rare varieties too.".to_string(),
                "I'll buy your crops at a fair price.".to_string(),
                "The soil here is rich after the rains.".to_string(),
                "Farming keeps the Driftlands fed.".to_string(),
            ],
            dialogue_index: 0,
        }
    }
}

// ── Intelligence Layer ────────────────────────────────────────────────────────

#[derive(Component, Default, Serialize, Deserialize)]
pub struct Knowledge {
    /// Mapping of player actions to affinity scores (-100 to 100).
    pub player_affinity: i32,
    /// Memory of recent events: (event_name, timestamp)
    pub memories: Vec<(String, f64)>,
}

impl Knowledge {
    pub fn add_memory(&mut self, event: &str, time: f64) {
        self.memories.push((event.to_string(), time));
        if self.memories.len() > 10 {
            self.memories.remove(0);
        }
    }

    pub fn update_affinity(&mut self, delta: i32) {
        self.player_affinity = (self.player_affinity + delta).clamp(-100, 100);
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
const OFFER_POOL: [(ItemType, ItemType, u32); 16] = [
    (ItemType::WheatSeed,     ItemType::Stone,       3),
    (ItemType::CarrotSeed,    ItemType::Stone,       5),
    (ItemType::Blueprint,     ItemType::IronIngot,   10),
    (ItemType::IronOre,       ItemType::Stone,       5),
    (ItemType::RareHerb,      ItemType::IceShard,    8),
    (ItemType::HealthPotion,  ItemType::MushroomCap, 3),
    // Expansion: new crop seeds
    (ItemType::CornSeed,      ItemType::Stone,       3),
    (ItemType::PotatoSeed,    ItemType::Stone,       3),
    (ItemType::PepperSeed,    ItemType::Stone,       4),
    (ItemType::OnionSeed,     ItemType::Stone,       3),
    (ItemType::RiceSeed,      ItemType::Reed,        2),
    (ItemType::MelonSeed,     ItemType::Stone,       5),
    (ItemType::FlaxSeed,      ItemType::PlantFiber,  4),
    (ItemType::SugarcaneSeed, ItemType::Stone,       4),
    // Expansion: fishing & pet supplies
    (ItemType::FishingRod,    ItemType::IronIngot,   3),
    (ItemType::PetCollar,     ItemType::IronIngot,   5),
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
        Knowledge::default(),
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
    mut trader_query: Query<(&mut Trader, &mut Knowledge)>,
    mut sound_events: EventWriter<SoundEvent>,
    mut inventory: ResMut<Inventory>,
    time: Res<Time>,
) {
    if !trade_menu.is_open {
        return;
    }

    let Some(trader_entity) = trade_menu.trader_entity else { return };
    let Ok((mut trader, mut knowledge)) = trader_query.get_mut(trader_entity) else {
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
                sound_events.send(SoundEvent::Trade);

                // Intelligence Layer: Record trade
                knowledge.update_affinity(10);
                knowledge.add_memory("traded_with_player", time.elapsed_secs_f64());
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
/// Called from `world/mod.rs` during chunk generation.
pub fn spawn_hermit(commands: &mut Commands, x: f32, y: f32, chunk_pos: IVec2) {
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
        Knowledge::default(),
        Invulnerable,
        ChunkObject { chunk_pos },
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
    mut hermit_query: Query<(&Transform, &mut Hermit, &mut Knowledge), Without<Player>>,
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
    for (idx, (tf, _, _)) in hermit_query.iter().enumerate() {
        let dist = player_pos.distance(tf.translation.truncate());
        if dist <= 32.0 {
            if best.is_none() || dist < best.unwrap().0 {
                best = Some((dist, idx));
            }
        }
    }

    let Some((_, target_idx)) = best else { return };

    for (i, (_, mut hermit, mut knowledge)) in hermit_query.iter_mut().enumerate() {
        if i == target_idx {
            hermit.has_interacted = true;
            let line = hermit.dialogue[hermit.dialogue_index].clone();
            dialogue_display.text = format!("Hermit: \"{}\"", line);
            dialogue_display.timer = 5.0;
            // Advance to next line (cycles)
            hermit.dialogue_index = (hermit.dialogue_index + 1) % hermit.dialogue.len();

            // Intelligence Layer: Record interaction
            knowledge.update_affinity(5);
            knowledge.add_memory("talked_to_player", time.elapsed_secs_f64());
            break;
        }
    }
}

// ── NPC Schedule Behavior ────────────────────────────────────────────────────

/// NPCs wander near their spawn point during the day and stand still at night.
fn npc_schedule_behavior(
    time: Res<Time>,
    cycle: Res<DayNightCycle>,
    mut npc_query: Query<(&mut Transform, &mut NpcSchedule)>,
) {
    let is_night = matches!(cycle.phase(), DayPhase::Night);
    let dt = time.delta_secs();

    for (mut transform, mut schedule) in npc_query.iter_mut() {
        if is_night {
            // At night, NPCs stand still — no movement
            schedule.wander_dir = Vec2::ZERO;
            continue;
        }

        // During the day, wander near home
        schedule.wander_timer -= dt;
        if schedule.wander_timer <= 0.0 {
            // Pick a new random direction (deterministic from position)
            let pos = transform.translation.truncate();
            let hash = WorldGenerator::position_hash(
                (pos.x * 100.0) as i32,
                (pos.y * 100.0 + schedule.wander_timer * 1000.0) as i32,
                42,
            );
            let angle = (hash % 628) as f32 / 100.0; // ~0..TAU
            schedule.wander_dir = Vec2::new(angle.cos(), angle.sin()) * 8.0; // 8 px/s
            schedule.wander_timer = 2.0 + (hash % 300) as f32 / 100.0; // 2-5 seconds
        }

        // Move
        let new_pos = transform.translation.truncate() + schedule.wander_dir * dt;

        // Leash: stay within 24px of home
        let to_home = schedule.home_pos - new_pos;
        if to_home.length() > 24.0 {
            // Reverse direction toward home
            schedule.wander_dir = to_home.normalize() * 8.0;
        }

        let final_pos = transform.translation.truncate() + schedule.wander_dir * dt;
        transform.translation.x = final_pos.x;
        transform.translation.y = final_pos.y;
    }
}

// ── NPC Interaction (Blacksmith / QuestGiver / Farmer) ───────────────────────

fn npc_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    building_state: Res<BuildingState>,
    crafting: Res<crate::crafting::CraftingSystem>,
    trade_menu: Res<TradeMenu>,
    player_query: Query<&Transform, With<Player>>,
    mut blacksmith_query: Query<(&Transform, &mut Blacksmith, &mut Knowledge), Without<Player>>,
    mut quest_giver_query: Query<(&Transform, &mut QuestGiver, &mut Knowledge), (Without<Player>, Without<Blacksmith>)>,
    mut farmer_query: Query<(&Transform, &mut Farmer, &mut Knowledge), (Without<Player>, Without<Blacksmith>, Without<QuestGiver>)>,
    mut npc_display: ResMut<NpcDialogueDisplay>,
    time: Res<Time>,
) {
    // Tick down dialogue timer
    if npc_display.timer > 0.0 {
        npc_display.timer -= time.delta_secs();
        if npc_display.timer <= 0.0 {
            npc_display.text.clear();
        }
    }

    if building_state.active || crafting.is_open || trade_menu.is_open {
        return;
    }

    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    // Check blacksmiths
    for (tf, mut bs, mut knowledge) in blacksmith_query.iter_mut() {
        let dist = player_pos.distance(tf.translation.truncate());
        if dist <= 32.0 {
            let line = bs.dialogue[bs.dialogue_index].clone();
            npc_display.text = format!("Blacksmith: \"{}\"", line);
            npc_display.timer = 5.0;
            bs.dialogue_index = (bs.dialogue_index + 1) % bs.dialogue.len();
            knowledge.update_affinity(5);
            knowledge.add_memory("talked_to_player", time.elapsed_secs_f64());
            return;
        }
    }

    // Check quest givers
    for (tf, mut qg, mut knowledge) in quest_giver_query.iter_mut() {
        let dist = player_pos.distance(tf.translation.truncate());
        if dist <= 32.0 {
            let line = qg.dialogue[qg.dialogue_index].clone();
            npc_display.text = format!("Quest Giver: \"{}\"", line);
            npc_display.timer = 5.0;
            qg.dialogue_index = (qg.dialogue_index + 1) % qg.dialogue.len();
            knowledge.update_affinity(5);
            knowledge.add_memory("talked_to_player", time.elapsed_secs_f64());
            return;
        }
    }

    // Check farmers
    for (tf, mut farmer, mut knowledge) in farmer_query.iter_mut() {
        let dist = player_pos.distance(tf.translation.truncate());
        if dist <= 32.0 {
            let line = farmer.dialogue[farmer.dialogue_index].clone();
            npc_display.text = format!("Farmer: \"{}\"", line);
            npc_display.timer = 5.0;
            farmer.dialogue_index = (farmer.dialogue_index + 1) % farmer.dialogue.len();
            knowledge.update_affinity(5);
            knowledge.add_memory("talked_to_player", time.elapsed_secs_f64());
            return;
        }
    }
}

// ── Public NPC Spawn Helpers ─────────────────────────────────────────────────

/// Spawn a Blacksmith NPC at a world position (used by structures module).
pub fn spawn_blacksmith(commands: &mut Commands, x: f32, y: f32, chunk_pos: IVec2) {
    commands.spawn((
        Blacksmith::new(),
        Knowledge::default(),
        Invulnerable,
        NpcSchedule {
            npc_type: NpcType::Blacksmith,
            home_pos: Vec2::new(x, y),
            wander_timer: 0.0,
            wander_dir: Vec2::ZERO,
        },
        ChunkObject { chunk_pos },
        Sprite {
            color: Color::srgb(0.55, 0.35, 0.25),
            custom_size: Some(Vec2::new(11.0, 14.0)),
            ..default()
        },
        Transform::from_xyz(x, y, 6.0),
    ));
}

/// Spawn a Quest Giver NPC at a world position (used by structures module).
pub fn spawn_quest_giver(commands: &mut Commands, x: f32, y: f32, chunk_pos: IVec2) {
    commands.spawn((
        QuestGiver::new(),
        Knowledge::default(),
        Invulnerable,
        NpcSchedule {
            npc_type: NpcType::QuestGiver,
            home_pos: Vec2::new(x, y),
            wander_timer: 0.0,
            wander_dir: Vec2::ZERO,
        },
        ChunkObject { chunk_pos },
        Sprite {
            color: Color::srgb(0.65, 0.55, 0.15),
            custom_size: Some(Vec2::new(10.0, 14.0)),
            ..default()
        },
        Transform::from_xyz(x, y, 6.0),
    ));
}

/// Spawn a Farmer NPC at a world position (used by structures module).
pub fn spawn_farmer(commands: &mut Commands, x: f32, y: f32, chunk_pos: IVec2) {
    commands.spawn((
        Farmer::new(),
        Knowledge::default(),
        Invulnerable,
        NpcSchedule {
            npc_type: NpcType::Farmer,
            home_pos: Vec2::new(x, y),
            wander_timer: 0.0,
            wander_dir: Vec2::ZERO,
        },
        ChunkObject { chunk_pos },
        Sprite {
            color: Color::srgb(0.35, 0.60, 0.20),
            custom_size: Some(Vec2::new(10.0, 14.0)),
            ..default()
        },
        Transform::from_xyz(x, y, 6.0),
    ));
}
