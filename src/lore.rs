use bevy::prelude::*;
use rand::Rng;
use crate::inventory::{Inventory, ItemType};
use crate::building::BuildingState;

pub struct LorePlugin;

impl Plugin for LorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LoreRegistry::new())
            .insert_resource(LoreMessage::default())
            .add_systems(Update, (use_journal_page, tick_lore_message));
    }
}

// ── Lore strings ──────────────────────────────────────────────────────────────

const ALL_LORE: [&str; 20] = [
    "The ancient civilization called themselves the Aethari. They built their first city on the mountain known today as the Pale Crown.",
    "Aethari scribes recorded that their empire stretched across nine biomes, each governed by a crystal pillar that regulated the weather.",
    "The Driftlands were named by survivors of the Collapse; they said the very ground 'drifted' as if adrift from the world.",
    "Aethari engineers pioneered crystal-lattice construction, fusing obsidian and crystal shards with volcanic heat to create Ancient Cores.",
    "The last Aethari emperor vanished into the Crystal Cave network, supposedly searching for a way to reverse the Collapse.",
    "Archaeologists have found Aethari tools made of an alloy unknown to modern smiths — stronger than steel yet lighter than iron.",
    "The crystal pillars did not merely control weather; they acted as a distributed memory store for the entire civilization's knowledge.",
    "Aethari farmers cultivated herbs that could not grow naturally in any biome — some believe the pillars altered local climate for crops.",
    "A fragment of an Aethari tablet reads: 'When the last pillar falls, the land shall breathe again, though we shall not breathe with it.'",
    "The swamp regions expanded dramatically after the Collapse, swallowing several Aethari coastal settlements within a generation.",
    "Wandering traders tell of a sealed vault deep beneath the volcanic biome, said to contain the Aethari's greatest invention.",
    "The Aethari used a calendar based on crystal resonance cycles rather than solar cycles; their 'year' was approximately 400 days.",
    "Hermits who live near the ruins claim to hear faint harmonic tones at night — remnants of the pillars' resonance, still echoing.",
    "Aethari soldiers carried blades forged from ancient cores. These 'Ancient Blades' could cut stone as easily as flesh.",
    "The Fungal biome was once an Aethari pleasure garden. The giant mushrooms are mutations caused by leaking crystal energy.",
    "No two Aethari ruins have exactly the same architectural style, suggesting a decentralized empire of many city-states rather than one capital.",
    "The Aethari word for 'home' and 'world' is the same: 'Vaethar'. Scholars debate whether this implies they viewed the whole world as home.",
    "Some ice crystals in the tundra biome contain preserved Aethari organic matter — seeds, insects, and once, what appeared to be a hand.",
    "Aethari records mention a class of citizens called 'Driftwalkers' who could navigate between biomes using crystal-attuned compasses.",
    "The final Aethari log ever found simply reads: 'We gave it everything. May whoever comes next use it more wisely than we did.'",
];

// ── Resource ──────────────────────────────────────────────────────────────────

#[derive(Resource)]
pub struct LoreRegistry {
    pub collected_entries: Vec<String>,
    pub total_entries: u32,
}

impl LoreRegistry {
    pub fn new() -> Self {
        Self {
            collected_entries: Vec::new(),
            total_entries: 20,
        }
    }

    #[allow(dead_code)]
    pub fn is_complete(&self) -> bool {
        self.collected_entries.len() as u32 >= self.total_entries
    }

    /// Add a random entry that has not yet been collected.
    /// Returns the new entry text, or None if all are collected.
    pub fn add_random_entry(&mut self) -> Option<String> {
        let mut rng = rand::thread_rng();
        let uncollected: Vec<&str> = ALL_LORE
            .iter()
            .copied()
            .filter(|&e| !self.collected_entries.iter().any(|c| c == e))
            .collect();

        if uncollected.is_empty() {
            return None;
        }

        let chosen = uncollected[rng.gen_range(0..uncollected.len())];
        self.collected_entries.push(chosen.to_string());
        Some(chosen.to_string())
    }
}

/// Brief HUD feedback message shown after reading a journal page.
#[derive(Resource, Default)]
pub struct LoreMessage {
    pub text: String,
    pub timer: f32,
}

// ── Systems ───────────────────────────────────────────────────────────────────

fn use_journal_page(
    mouse: Res<ButtonInput<MouseButton>>,
    building_state: Res<BuildingState>,
    mut inventory: ResMut<Inventory>,
    mut lore_registry: ResMut<LoreRegistry>,
    mut lore_message: ResMut<LoreMessage>,
) {
    // Right-click to use, skip when build mode or trade menus would consume click
    if !mouse.just_pressed(MouseButton::Right) || building_state.active {
        return;
    }

    let Some(slot) = inventory.selected_item() else { return };
    if slot.item != ItemType::JournalPage {
        return;
    }

    // Consume the journal page
    if !inventory.remove_items(ItemType::JournalPage, 1) {
        return;
    }

    match lore_registry.add_random_entry() {
        Some(entry) => {
            let n = lore_registry.collected_entries.len();
            let total = lore_registry.total_entries;
            lore_message.text = format!(
                "Lore ({}/{}): {}",
                n, total, entry
            );
            lore_message.timer = 8.0;
        }
        None => {
            lore_message.text = "You have collected all lore entries!".to_string();
            lore_message.timer = 4.0;
        }
    }
}

fn tick_lore_message(
    time: Res<Time>,
    mut lore_message: ResMut<LoreMessage>,
) {
    if lore_message.timer > 0.0 {
        lore_message.timer -= time.delta_secs();
        if lore_message.timer <= 0.0 {
            lore_message.text.clear();
        }
    }
}
