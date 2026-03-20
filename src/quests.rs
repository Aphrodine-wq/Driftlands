use bevy::prelude::*;
use crate::hud::not_paused;
use crate::inventory::{Inventory, ItemType};
use crate::combat::ResearchPointEvent;
use crate::daynight::DayNightCycle;
use crate::npc::QuestGiver;
use crate::player::Player;
use crate::death::DeathStats;

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[derive(Event)]
pub struct QuestProgressEvent {
    pub quest_type: QuestType,
    pub amount: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum QuestType {
    KillEnemy,
    SurviveNight,
    CraftItem,
    PlaceBuilding,
    CraftIronTool,
    CraftItems25,
    KillBoss,
    VisitBiome,
    PlantCrop,
    CatchFish,
    TamePet,
    CraftEnchanted,
    CompleteDungeon,
    GatherResource,
    CookMeal,
    CraftSteelSet,
    CraftAncient,
}

// ---------------------------------------------------------------------------
// Quest definitions
// ---------------------------------------------------------------------------

pub struct QuestDef {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub quest_type: QuestType,
    pub target: u32,
    pub reward: Vec<(ItemType, u32)>,
    pub rp_reward: u32,
}

pub fn quest_definitions() -> Vec<QuestDef> {
    vec![
        QuestDef {
            id: "first_blood",
            name: "First Blood",
            description: "Slay your first enemy.",
            quest_type: QuestType::KillEnemy,
            target: 1,
            reward: vec![],
            rp_reward: 5,
        },
        QuestDef {
            id: "night_survivor",
            name: "Night Survivor",
            description: "Survive 3 nights in the Driftlands.",
            quest_type: QuestType::SurviveNight,
            target: 3,
            reward: vec![(ItemType::Torch, 5)],
            rp_reward: 0,
        },
        QuestDef {
            id: "tool_maker",
            name: "Tool Maker",
            description: "Craft your first item at a workbench.",
            quest_type: QuestType::CraftItem,
            target: 1,
            reward: vec![(ItemType::StoneAxe, 1)],
            rp_reward: 0,
        },
        QuestDef {
            id: "home_builder",
            name: "Home Builder",
            description: "Place 5 buildings to establish a base.",
            quest_type: QuestType::PlaceBuilding,
            target: 5,
            reward: vec![(ItemType::WoodPlank, 20)],
            rp_reward: 0,
        },
        QuestDef {
            id: "iron_age",
            name: "Iron Age",
            description: "Craft an iron tool at the forge.",
            quest_type: QuestType::CraftIronTool,
            target: 1,
            reward: vec![(ItemType::IronIngot, 5)],
            rp_reward: 0,
        },
        QuestDef {
            id: "master_crafter",
            name: "Master Crafter",
            description: "Craft 25 items total.",
            quest_type: QuestType::CraftItems25,
            target: 25,
            reward: vec![(ItemType::Blueprint, 1)],
            rp_reward: 0,
        },
        QuestDef {
            id: "monster_slayer",
            name: "Monster Slayer",
            description: "Defeat 25 enemies.",
            quest_type: QuestType::KillEnemy,
            target: 25,
            reward: vec![(ItemType::HealthPotion, 3)],
            rp_reward: 0,
        },
        QuestDef {
            id: "boss_hunter",
            name: "Boss Hunter",
            description: "Defeat a biome boss.",
            quest_type: QuestType::KillBoss,
            target: 1,
            reward: vec![(ItemType::AncientCore, 1)],
            rp_reward: 0,
        },
        QuestDef {
            id: "explorer",
            name: "Explorer",
            description: "Discover 5 different biomes.",
            quest_type: QuestType::VisitBiome,
            target: 5,
            reward: vec![(ItemType::SpeedPotion, 2)],
            rp_reward: 0,
        },
        QuestDef {
            id: "green_thumb",
            name: "Green Thumb",
            description: "Plant 10 crops.",
            quest_type: QuestType::PlantCrop,
            target: 10,
            reward: vec![(ItemType::CornSeed, 3), (ItemType::PotatoSeed, 3)],
            rp_reward: 0,
        },
        QuestDef {
            id: "master_angler",
            name: "Master Angler",
            description: "Catch 10 fish.",
            quest_type: QuestType::CatchFish,
            target: 10,
            reward: vec![(ItemType::SteelFishingRod, 1)],
            rp_reward: 0,
        },
        QuestDef {
            id: "beast_friend",
            name: "Beast Friend",
            description: "Tame your first pet.",
            quest_type: QuestType::TamePet,
            target: 1,
            reward: vec![(ItemType::PetFood, 5)],
            rp_reward: 0,
        },
        QuestDef {
            id: "enchanter",
            name: "Enchanter",
            description: "Craft an enchanted weapon.",
            quest_type: QuestType::CraftEnchanted,
            target: 1,
            reward: vec![(ItemType::Gemstone, 2)],
            rp_reward: 0,
        },
        QuestDef {
            id: "dungeon_delver",
            name: "Dungeon Delver",
            description: "Complete 3 dungeons.",
            quest_type: QuestType::CompleteDungeon,
            target: 3,
            reward: vec![(ItemType::AncientBlade, 1)],
            rp_reward: 0,
        },
        QuestDef {
            id: "hoarder",
            name: "Hoarder",
            description: "Gather 500 resources.",
            quest_type: QuestType::GatherResource,
            target: 500,
            reward: vec![(ItemType::Chest, 3)],
            rp_reward: 0,
        },
        QuestDef {
            id: "chef",
            name: "Chef",
            description: "Cook 10 meals.",
            quest_type: QuestType::CookMeal,
            target: 10,
            reward: vec![(ItemType::BakedPotato, 5)],
            rp_reward: 0,
        },
        QuestDef {
            id: "night_terror",
            name: "Night Terror",
            description: "Defeat 50 enemies.",
            quest_type: QuestType::KillEnemy,
            target: 50,
            reward: vec![(ItemType::StrengthPotion, 3)],
            rp_reward: 0,
        },
        QuestDef {
            id: "steel_yourself",
            name: "Steel Yourself",
            description: "Craft a full set of steel equipment.",
            quest_type: QuestType::CraftSteelSet,
            target: 1,
            reward: vec![(ItemType::Blueprint, 2)],
            rp_reward: 0,
        },
        QuestDef {
            id: "ancient_power",
            name: "Ancient Power",
            description: "Craft an ancient artifact.",
            quest_type: QuestType::CraftAncient,
            target: 1,
            reward: vec![(ItemType::AncientCore, 2)],
            rp_reward: 0,
        },
        QuestDef {
            id: "driftlander",
            name: "Driftlander",
            description: "Complete every quest and master the Driftlands.",
            quest_type: QuestType::KillEnemy, // placeholder; checked via special logic
            target: 1,
            reward: vec![],
            rp_reward: 50,
        },
    ]
}

const DRIFTLANDER_INDEX: usize = 19;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

#[derive(Resource)]
pub struct QuestLog {
    pub quests: Vec<QuestState>,
    pub is_open: bool,
    pub selected: usize,
}

pub struct QuestState {
    pub def_index: usize,
    pub progress: u32,
    pub completed: bool,
    pub claimed: bool,
}

impl QuestLog {
    pub fn new() -> Self {
        let defs = quest_definitions();
        let quests = defs
            .iter()
            .enumerate()
            .map(|(i, _)| QuestState {
                def_index: i,
                progress: 0,
                completed: false,
                claimed: false,
            })
            .collect();
        Self {
            quests,
            is_open: false,
            selected: 0,
        }
    }

    /// Serialize quest progress for save files.
    pub fn to_save_data(&self) -> Vec<(String, u32, bool, bool)> {
        let defs = quest_definitions();
        self.quests
            .iter()
            .map(|q| {
                let id = defs[q.def_index].id.to_owned();
                (id, q.progress, q.completed, q.claimed)
            })
            .collect()
    }

    /// Restore quest progress from save data.
    pub fn from_save_data(data: &[(String, u32, bool, bool)]) -> Self {
        let defs = quest_definitions();
        let mut quests: Vec<QuestState> = defs
            .iter()
            .enumerate()
            .map(|(i, _)| QuestState {
                def_index: i,
                progress: 0,
                completed: false,
                claimed: false,
            })
            .collect();

        for (saved_id, progress, completed, claimed) in data {
            if let Some(idx) = defs.iter().position(|d| d.id == saved_id.as_str()) {
                quests[idx].progress = *progress;
                quests[idx].completed = *completed;
                quests[idx].claimed = *claimed;
            }
        }

        Self {
            quests,
            is_open: false,
            selected: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn track_quest_progress(
    mut quest_log: ResMut<QuestLog>,
    mut events: EventReader<QuestProgressEvent>,
) {
    let defs = quest_definitions();

    for event in events.read() {
        for quest in quest_log.quests.iter_mut() {
            if quest.completed || quest.def_index == DRIFTLANDER_INDEX {
                continue;
            }
            let def = &defs[quest.def_index];
            if def.quest_type == event.quest_type {
                quest.progress = (quest.progress + event.amount).min(def.target);
                if quest.progress >= def.target {
                    quest.completed = true;
                }
            }
        }
    }

    // Special handling: "Driftlander" completes when every other quest is done.
    let all_others_complete = quest_log
        .quests
        .iter()
        .enumerate()
        .all(|(i, q)| i == DRIFTLANDER_INDEX || q.completed);

    if all_others_complete && !quest_log.quests[DRIFTLANDER_INDEX].completed {
        quest_log.quests[DRIFTLANDER_INDEX].progress = 1;
        quest_log.quests[DRIFTLANDER_INDEX].completed = true;
    }
}

fn toggle_quest_log(
    keyboard: Res<ButtonInput<KeyCode>>,
    game_settings: Res<crate::settings::GameSettings>,
    mut quest_log: ResMut<QuestLog>,
) {
    if keyboard.just_pressed(game_settings.keybinds.journal) {
        quest_log.is_open = !quest_log.is_open;
    }
}

fn quest_log_navigation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut quest_log: ResMut<QuestLog>,
    dynamic_log: Res<DynamicQuestLog>,
) {
    if !quest_log.is_open {
        return;
    }

    // Total entries = static quests + dynamic quests
    let count = quest_log.quests.len() + dynamic_log.quests.len();
    if count == 0 {
        return;
    }

    if keyboard.just_pressed(KeyCode::ArrowUp) {
        quest_log.selected = quest_log.selected.saturating_sub(1);
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        quest_log.selected = (quest_log.selected + 1).min(count - 1);
    }
}

fn claim_quest_rewards(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut quest_log: ResMut<QuestLog>,
    mut inventory: ResMut<Inventory>,
    mut rp_events: EventWriter<ResearchPointEvent>,
) {
    if !quest_log.is_open || !keyboard.just_pressed(KeyCode::Enter) {
        return;
    }

    let selected = quest_log.selected;
    let quest = &quest_log.quests[selected];

    if !quest.completed || quest.claimed {
        return;
    }

    let defs = quest_definitions();
    let def = &defs[selected];

    // Grant item rewards.
    for &(item, amount) in &def.reward {
        inventory.add_item(item, amount);
    }

    // Grant research point rewards.
    if def.rp_reward > 0 {
        rp_events.send(ResearchPointEvent {
            amount: def.rp_reward,
        });
    }

    quest_log.quests[selected].claimed = true;
}

// ---------------------------------------------------------------------------
// Dynamic Quests
// ---------------------------------------------------------------------------

/// A dynamic quest generated at runtime by Quest Giver NPCs.
pub struct DynamicQuest {
    pub quest_type: QuestType,
    pub description: String,
    pub target_count: u32,
    pub progress: u32,
    pub expiry_day: u32,
    pub reward_items: Vec<(ItemType, u32)>,
    pub completed: bool,
    pub claimed: bool,
}

/// Resource holding up to 3 active dynamic quests.
#[derive(Resource, Default)]
pub struct DynamicQuestLog {
    pub quests: Vec<DynamicQuest>,
}

impl DynamicQuestLog {
    /// Maximum number of active dynamic quests.
    pub const MAX_QUESTS: usize = 3;
}

/// Templates for dynamic quest generation (quest_type, description_template, min_target, max_target, reward_pool).
const DYNAMIC_QUEST_TEMPLATES: &[(QuestType, &str, u32, u32)] = &[
    (QuestType::KillEnemy,       "Slay {} enemies lurking nearby.",        3, 15),
    (QuestType::GatherResource,  "Gather {} resources from the wilds.",   10, 50),
    (QuestType::CraftItem,       "Craft {} items at a workbench.",         2,  8),
    (QuestType::PlantCrop,       "Plant {} crops in the fertile soil.",    3, 10),
    (QuestType::CatchFish,       "Catch {} fish from the waters.",         2,  8),
    (QuestType::CookMeal,        "Cook {} meals for the settlement.",      2,  6),
    (QuestType::PlaceBuilding,   "Place {} buildings to strengthen camp.", 2,  5),
];

/// Reward pools for dynamic quests (selected by hash).
const DYNAMIC_REWARDS: &[&[(ItemType, u32)]] = &[
    &[(ItemType::HealthPotion, 2)],
    &[(ItemType::IronIngot, 3)],
    &[(ItemType::Arrow, 20)],
    &[(ItemType::Torch, 5), (ItemType::Stone, 10)],
    &[(ItemType::SpeedPotion, 1)],
    &[(ItemType::StrengthPotion, 1)],
    &[(ItemType::RareHerb, 2)],
    &[(ItemType::Gemstone, 1)],
];

/// System: when the player talks to a Quest Giver NPC, offer a dynamic quest
/// if the log isn't full. Uses interaction proximity check.
fn generate_dynamic_quests(
    keyboard: Res<ButtonInput<KeyCode>>,
    game_settings: Res<crate::settings::GameSettings>,
    cycle: Res<DayNightCycle>,
    player_query: Query<&Transform, With<Player>>,
    quest_giver_query: Query<&Transform, With<QuestGiver>>,
    mut dynamic_log: ResMut<DynamicQuestLog>,
) {
    // Only generate on interaction key
    if !keyboard.just_pressed(game_settings.keybinds.interact) {
        return;
    }

    if dynamic_log.quests.len() >= DynamicQuestLog::MAX_QUESTS {
        return;
    }

    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    // Check if any quest giver is within range
    let mut near_quest_giver = false;
    for tf in quest_giver_query.iter() {
        if player_pos.distance(tf.translation.truncate()) <= 32.0 {
            near_quest_giver = true;
            break;
        }
    }

    if !near_quest_giver {
        return;
    }

    // Generate a quest deterministically from day count + position
    let seed_val = cycle.day_count
        .wrapping_mul(374761393)
        .wrapping_add(dynamic_log.quests.len() as u32)
        .wrapping_mul(668265263);

    let template_idx = (seed_val as usize) % DYNAMIC_QUEST_TEMPLATES.len();
    let (quest_type, desc_template, min_target, max_target) = DYNAMIC_QUEST_TEMPLATES[template_idx];

    let target_range = max_target.saturating_sub(min_target).max(1);
    let target = min_target + (seed_val.wrapping_mul(2654435761) % target_range);

    let description = desc_template.replace("{}", &target.to_string());

    let reward_idx = (seed_val.wrapping_mul(1274126177) >> 16) as usize % DYNAMIC_REWARDS.len();
    let reward_items: Vec<(ItemType, u32)> = DYNAMIC_REWARDS[reward_idx].to_vec();

    let expiry_day = cycle.day_count + 5; // 5-day deadline

    dynamic_log.quests.push(DynamicQuest {
        quest_type,
        description,
        target_count: target,
        progress: 0,
        expiry_day,
        reward_items,
        completed: false,
        claimed: false,
    });
}

/// Track progress on dynamic quests from the same QuestProgressEvent stream.
fn track_dynamic_quest_progress(
    mut dynamic_log: ResMut<DynamicQuestLog>,
    mut events: EventReader<QuestProgressEvent>,
) {
    for event in events.read() {
        for quest in dynamic_log.quests.iter_mut() {
            if quest.completed {
                continue;
            }
            if quest.quest_type == event.quest_type {
                quest.progress = (quest.progress + event.amount).min(quest.target_count);
                if quest.progress >= quest.target_count {
                    quest.completed = true;
                }
            }
        }
    }
}

/// Expire dynamic quests that have passed their deadline.
fn expire_dynamic_quests(
    cycle: Res<DayNightCycle>,
    mut dynamic_log: ResMut<DynamicQuestLog>,
) {
    dynamic_log.quests.retain(|q| {
        // Keep completed-but-unclaimed quests even past expiry so player can claim
        q.completed || cycle.day_count < q.expiry_day
    });
}

/// Claim rewards for completed dynamic quests (when selected in quest log).
fn claim_dynamic_quest_rewards(
    keyboard: Res<ButtonInput<KeyCode>>,
    quest_log: Res<QuestLog>,
    mut dynamic_log: ResMut<DynamicQuestLog>,
    mut inventory: ResMut<Inventory>,
) {
    if !quest_log.is_open || !keyboard.just_pressed(KeyCode::Enter) {
        return;
    }

    let static_count = quest_log.quests.len();
    let selected = quest_log.selected;

    // Only handle if selection is in the dynamic range
    if selected < static_count {
        return;
    }

    let dynamic_idx = selected - static_count;
    if dynamic_idx >= dynamic_log.quests.len() {
        return;
    }

    let quest = &dynamic_log.quests[dynamic_idx];
    if !quest.completed || quest.claimed {
        return;
    }

    // Grant rewards
    for &(item, amount) in &quest.reward_items {
        inventory.add_item(item, amount);
    }

    dynamic_log.quests[dynamic_idx].claimed = true;
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Tracks previously seen kill/day counts to generate delta-based QuestProgressEvents.
#[derive(Resource, Default)]
struct QuestActivityTracker {
    last_total_kills: u32,
    last_day_count: u32,
}

/// Send KillEnemy events based on the delta in DeathStats.total_kills.
/// This avoids adding an EventWriter to player_attack (already at param limit).
fn dispatch_kill_quest_events(
    death_stats: Res<DeathStats>,
    mut tracker: ResMut<QuestActivityTracker>,
    mut quest_events: EventWriter<QuestProgressEvent>,
) {
    let kills = death_stats.total_kills;
    if kills > tracker.last_total_kills {
        let delta = kills - tracker.last_total_kills;
        quest_events.send(QuestProgressEvent { quest_type: QuestType::KillEnemy, amount: delta });
        tracker.last_total_kills = kills;
    }
}

/// Send SurviveNight events when the in-game day count increases.
fn dispatch_survive_night_events(
    cycle: Res<DayNightCycle>,
    mut tracker: ResMut<QuestActivityTracker>,
    mut quest_events: EventWriter<QuestProgressEvent>,
) {
    let day = cycle.day_count;
    if day > tracker.last_day_count {
        let delta = day - tracker.last_day_count;
        quest_events.send(QuestProgressEvent { quest_type: QuestType::SurviveNight, amount: delta });
        tracker.last_day_count = day;
    }
}

pub struct QuestPlugin;

impl Plugin for QuestPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<QuestProgressEvent>()
            .insert_resource(QuestLog::new())
            .insert_resource(DynamicQuestLog::default())
            .insert_resource(QuestActivityTracker::default())
            .add_systems(
                Update,
                (
                    dispatch_kill_quest_events,
                    dispatch_survive_night_events,
                    track_quest_progress,
                    toggle_quest_log,
                    claim_quest_rewards,
                    quest_log_navigation,
                    track_dynamic_quest_progress,
                    expire_dynamic_quests,
                    generate_dynamic_quests,
                    claim_dynamic_quest_rewards,
                )
                    .run_if(not_paused),
            );
    }
}
