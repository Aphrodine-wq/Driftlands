use bevy::prelude::*;
use rand::Rng;
use crate::inventory::{Inventory, ItemType};
use crate::building::BuildingState;
use crate::audio::SoundEvent;
use crate::theme::EtherealTheme;

pub struct LorePlugin;

impl Plugin for LorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LoreRegistry::new())
            .insert_resource(LoreMessage::default())
            .insert_resource(LorePanel::default())
            .add_systems(Update, (
                use_journal_page,
                tick_lore_message,
                spawn_lore_panel,
                update_lore_panel,
            ));
    }
}

// ── Lore strings ──────────────────────────────────────────────────────────────

/// Original 20 lore entries (discovered via Journal Pages).
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

// ── Wave 7B: 30 new lore entries ─────────────────────────────────────────────

/// Biome lore (9 entries — one per biome).
const BIOME_LORE: [(&str, &str); 9] = [
    ("biome_forest", "The Forest of Whispers grows over an ancient Aethari orchard. Fruit trees here bear strange hybrid crops when tended carefully. Tip: Forest resources respawn faster than any other biome — harvest wood here for efficient stockpiling."),
    ("biome_coastal", "The Coastal Reaches were once the Aethari's main trade routes. Salt-encrusted ruins still line the shore. Tip: Coastal biomes provide unique shells and coral used in advanced crafting recipes."),
    ("biome_swamp", "The Verdant Mire formed when Aethari irrigation canals overflowed after the Collapse, drowning an entire province. Tip: Swamp herbs are essential for brewing antidotes — gather them before exploring poison-heavy dungeons."),
    ("biome_desert", "The Amber Wastes hide Aethari oases beneath sand dunes, fed by underground crystal-filtered springs. Tip: Desert heat drains hunger faster — carry extra food and water, and travel at night when enemies are weaker here."),
    ("biome_tundra", "The Frozen Reach preserves Aethari technology in ice. Crystal shards found here have double the resonance of those elsewhere. Tip: Winter gear and warm food prevent the Frostbite debuff — never explore the tundra unprepared."),
    ("biome_volcanic", "The Molten Core was the Aethari's primary forge. They channeled lava flows through crystal pipes to power their greatest works. Tip: Volcanic ore yields the strongest metals, but lava pools deal rapid damage — build bridges to cross safely."),
    ("biome_fungal", "The Sporemist Caverns were sealed by the Aethari to contain a failed biological experiment. The mushrooms evolved intelligence. Tip: Fungal enemies inflict poison — craft antidotes before entering, and watch for spore clouds that reduce visibility."),
    ("biome_crystalcave", "The Crystal Depths hum with residual Aethari energy. The caves grow and shift as crystals form new passages over time. Tip: Crystal Shards mined here are essential for enchanting and tech tree research — bring a good pickaxe."),
    ("biome_mountain", "The Pale Crown peaks were the Aethari seat of power. Wind-carved ruins at the summit still channel crystal energy. Tip: Mountain enemies hit hardest but are slow — kite them with a bow or use terrain to your advantage."),
];

/// Boss lore (9 entries — unlocked on first kill).
const BOSS_LORE: [(&str, &str); 9] = [
    ("boss_forest_guardian", "The Forest Guardian is a living tree animated by an Aethari crystal heart buried in its trunk. It protects the ancient orchard from all outsiders. Weakness: Fire damage burns its bark — enchant a weapon with flame, or use torches."),
    ("boss_swamp_beast", "The Swamp Beast is a massive amphibian mutated by centuries of exposure to corrupted Aethari irrigation crystals. It lurks in the deepest mire. Weakness: It is slow on dry land — lure it out of the water and strike while it is sluggish."),
    ("boss_desert_wyrm", "The Desert Wyrm burrows through sand using a crystalline horn that once served as an Aethari drilling tool. It surfaces to feed. Weakness: When it charges, dodge sideways — it cannot turn quickly. Attack its exposed belly after a missed charge."),
    ("boss_frost_giant", "The Frost Giant was once an Aethari golem designed to patrol the tundra borders. Its crystal core froze over, driving it berserk. Weakness: Fire and blunt weapons crack its icy armor — avoid its ground-slam attack by staying mobile."),
    ("boss_magma_king", "The Magma King dwells in the deepest forge chamber, a being of living lava shaped by the Aethari to fuel their smelters forever. Weakness: Water and ice attacks cool its molten shell temporarily — strike fast during the hardened phase."),
    ("boss_fungal_overlord", "The Fungal Overlord is the largest organism in the Driftlands — a network of mycelium that grew a mobile body to hunt. Weakness: It regenerates unless you destroy its spore nodes first. Clear the small mushroom minions before focusing the boss."),
    ("boss_crystal_sentinel", "The Crystal Sentinel guards the deepest mine shaft, a geometric construct of pure Aethari crystal lattice. It shatters and reforms. Weakness: Heavy weapons stagger it before it can reform — use a hammer or axe for maximum effect."),
    ("boss_tidal_serpent", "The Tidal Serpent patrols the coastal depths, an eel-like creature whose scales are fused with Aethari naval armor plating. Weakness: It must surface to breathe — attack during its surface phase and dodge when it dives."),
    ("boss_mountain_titan", "The Mountain Titan is a stone colossus carved by the Aethari as a last line of defense. It still follows its ancient orders. Weakness: Its attacks are slow and telegraphed — dodge-roll through its slam and counter-attack from behind."),
];

/// Structure lore (5 entries).
const STRUCTURE_LORE: [(&str, &str); 5] = [
    ("struct_abandoned_village", "Abandoned villages dot the landscape, remnants of post-Collapse settlements that failed to survive. Scavengers sometimes find intact tools and supplies in their ruins. Tip: Search every building — some contain hidden caches behind breakable walls."),
    ("struct_mine_shaft", "Mine shafts were dug by Aethari miners who followed crystal veins deep underground. The tunnels branch unpredictably and many dead-end in sealed Aethari vaults. Tip: Bring torches and a weapon — cave spiders nest near crystal deposits."),
    ("struct_trader_outpost", "Trader outposts are built by the few remaining merchants who brave the wilds. They offer rare materials and blueprints in exchange for gathered resources. Tip: Save your Gemstones and Crystal Shards — traders often stock enchanting materials."),
    ("struct_watchtower", "Watchtowers were built by early Driftlands settlers to spot enemy raids. Climbing one reveals a wide area on the minimap and may contain a supply chest. Tip: Rest at a watchtower to get a temporary vision buff that extends your minimap range."),
    ("struct_fishing_dock", "Fishing docks were constructed near Aethari-engineered fish nurseries. The waters nearby still teem with unusual species bred for both food and alchemy. Tip: Fishing at a dock increases your catch rate — use a Steel Fishing Rod for the best results."),
];

/// Gameplay hint lore (7 entries).
const GAMEPLAY_LORE: [(&str, &str); 7] = [
    ("hint_enchanting", "The Art of Enchanting: The Aethari discovered that crystal shards can store intent. By placing a weapon on an enchanting table with gems, the crystal infuses the metal with elemental power. Tip: Combine Crystal Shards with Gemstones to create Fire, Ice, or Poison enchantments. Higher-tier gems yield stronger effects."),
    ("hint_fishing", "Secrets of the Deep: Aethari fishers catalogued over forty species, some of which still swim the Driftlands waters. Rare fish appear only during certain seasons and weather. Tip: Fish during rain for increased rare catch rates. Use a Steel Fishing Rod and fish near structures for the best hauls."),
    ("hint_pet_care", "Taming the Wild: Aethari beast-keepers used crystal-attuned collars to bond with wild creatures. A weakened animal is receptive to the collar's calming resonance. Tip: Weaken a tameable creature below 25% HP, then use a Pet Collar. Feed your pet regularly to keep its loyalty — a hungry pet may flee during combat."),
    ("hint_combat_tactics", "Combat Doctrine: Aethari warriors trained in a discipline they called 'the dance of edges' — constant movement and timed strikes. Tip: Perfect-block enemy attacks within 0.2 seconds for full damage negation. Dodge-roll through boss attacks to avoid damage entirely. Combo hits deal bonus damage on the third consecutive strike."),
    ("hint_farming", "The Cultivator's Almanac: Each season affects crop growth differently. Spring accelerates growth, Summer maximizes yield, Autumn slows maturation, and Winter nearly halts it. Tip: Plant crops at the start of Spring for maximum harvests. Use Aethari fertilizer (crafted from bone meal and crystal dust) to ignore seasonal penalties."),
    ("hint_building", "Master Builder's Notes: Walls placed in a complete enclosure create a shelter zone that slows hunger drain and boosts health regeneration. Doors block enemy pathfinding. Tip: Build a 3x3 shelter with a door before your first nightfall. Place a bed inside to set your spawn point — dying without a bed means a long walk back."),
    ("hint_survival", "Survivor's Wisdom: The Driftlands punish the unprepared. Every expedition should include food, a weapon, building materials, and torches. Tip: Craft a backpack to expand your inventory before long trips. Mark your base location on the map, and always carry materials to build an emergency shelter if night catches you away from home."),
];

// ── Resource ──────────────────────────────────────────────────────────────────

#[derive(Resource)]
pub struct LoreRegistry {
    pub collected_entries: Vec<String>,
    pub total_entries: u32,
    /// Wave 7B: IDs of expansion lore entries that have been unlocked.
    pub unlocked_expansion_lore: std::collections::HashSet<String>,
    /// Wave 7B: All expansion lore definitions (id, text) for biome/boss/structure/gameplay.
    expansion_entries: Vec<(&'static str, &'static str)>,
}

impl LoreRegistry {
    pub fn new() -> Self {
        // Build the expansion entries list from all four categories.
        let mut expansion = Vec::with_capacity(30);
        for &(id, text) in &BIOME_LORE {
            expansion.push((id, text));
        }
        for &(id, text) in &BOSS_LORE {
            expansion.push((id, text));
        }
        for &(id, text) in &STRUCTURE_LORE {
            expansion.push((id, text));
        }
        for &(id, text) in &GAMEPLAY_LORE {
            expansion.push((id, text));
        }

        Self {
            collected_entries: Vec::new(),
            total_entries: 50, // 20 original + 30 expansion
            unlocked_expansion_lore: std::collections::HashSet::new(),
            expansion_entries: expansion,
        }
    }

    #[allow(dead_code)]
    pub fn is_complete(&self) -> bool {
        self.collected_entries.len() as u32 >= self.total_entries
    }

    /// Add a random entry that has not yet been collected (from original 20).
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

    /// Wave 7B: Unlock an expansion lore entry by ID. Returns the text if newly unlocked.
    pub fn unlock_expansion(&mut self, lore_id: &str) -> Option<String> {
        if self.unlocked_expansion_lore.contains(lore_id) {
            return None;
        }
        let entry = self.expansion_entries.iter().find(|&&(id, _)| id == lore_id);
        if let Some(&(id, text)) = entry {
            self.unlocked_expansion_lore.insert(id.to_string());
            self.collected_entries.push(text.to_string());
            Some(text.to_string())
        } else {
            None
        }
    }

    /// Wave 7B: Get expansion lore text by ID (regardless of unlock state).
    #[allow(dead_code)]
    pub fn get_expansion_text(&self, lore_id: &str) -> Option<&'static str> {
        self.expansion_entries.iter()
            .find(|&&(id, _)| id == lore_id)
            .map(|&(_, text)| text)
    }

    /// Wave 7B: Check if an expansion entry has been unlocked.
    #[allow(dead_code)]
    pub fn is_expansion_unlocked(&self, lore_id: &str) -> bool {
        self.unlocked_expansion_lore.contains(lore_id)
    }
}

/// Brief HUD feedback message shown after reading a journal page (fallback when panel closed).
#[derive(Resource, Default)]
pub struct LoreMessage {
    pub text: String,
    pub timer: f32,
}

/// Dedicated journal read panel: full text, typewriter, optional completion chime.
#[derive(Resource, Default)]
pub struct LorePanel {
    pub active: bool,
    pub full_text: String,
    pub typewriter_pos: f32,
    pub display_timer: f32,
    pub play_completion_chime: bool,
}

// ── Systems ───────────────────────────────────────────────────────────────────

fn use_journal_page(
    mouse: Res<ButtonInput<MouseButton>>,
    building_state: Res<BuildingState>,
    mut inventory: ResMut<Inventory>,
    mut lore_registry: ResMut<LoreRegistry>,
    mut lore_message: ResMut<LoreMessage>,
    mut lore_panel: ResMut<LorePanel>,
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
            lore_message.text = format!("Lore ({}/{}): entry recorded.", n, total);
            lore_message.timer = 4.0;
            lore_panel.active = true;
            lore_panel.full_text = format!("[{}/{}]\n\n{}", n, total, entry);
            lore_panel.typewriter_pos = 0.0;
            lore_panel.display_timer = 15.0;
            lore_panel.play_completion_chime = lore_registry.is_complete();
        }
        None => {
            lore_message.text = "You have collected all lore entries!".to_string();
            lore_message.timer = 4.0;
            lore_panel.active = true;
            lore_panel.full_text = "You have collected all lore entries.".to_string();
            lore_panel.typewriter_pos = f32::MAX;
            lore_panel.display_timer = 5.0;
            lore_panel.play_completion_chime = false;
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

#[derive(Component)]
struct LorePanelUI;

#[derive(Component)]
struct LorePanelText;

const TYPEWRITER_CHARS_PER_SEC: f32 = 35.0;

fn spawn_lore_panel(
    mut commands: Commands,
    lore_panel: Res<LorePanel>,
    theme: Res<EtherealTheme>,
    panel_query: Query<Entity, With<LorePanelUI>>,
) {
    if !lore_panel.active {
        for entity in panel_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        return;
    }
    if panel_query.iter().next().is_some() {
        return;
    }

    let max_chars = (lore_panel.typewriter_pos as usize).min(lore_panel.full_text.len());
    let visible = &lore_panel.full_text[..max_chars];

    commands.spawn((
        LorePanelUI,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            padding: UiRect::all(Val::Px(40.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.02, 0.06, 0.92)),
        GlobalZIndex(90),
    )).with_children(|parent| {
        parent.spawn((
            LorePanelText,
            Text::new(visible),
            TextFont { font_size: 18.0, ..default() },
            TextColor(theme.hud_primary_text()),
            Node {
                max_width: Val::Px(700.0),
                ..default()
            },
        ));
        parent.spawn((
            Text::new("Enter to close"),
            TextFont { font_size: 14.0, ..default() },
            TextColor(theme.hud_label_color()),
            Node { margin: UiRect::top(Val::Px(24.0)), ..default() },
        ));
    });
}

fn update_lore_panel(
    mut commands: Commands,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut lore_panel: ResMut<LorePanel>,
    mut text_query: Query<&mut Text, With<LorePanelText>>,
    panel_query: Query<Entity, With<LorePanelUI>>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    if !lore_panel.active {
        return;
    }

    lore_panel.typewriter_pos += TYPEWRITER_CHARS_PER_SEC * time.delta_secs();
    let len = lore_panel.full_text.len() as f32;
    if lore_panel.typewriter_pos >= len && lore_panel.play_completion_chime {
        sound_events.send(SoundEvent::LoreComplete);
        lore_panel.play_completion_chime = false;
    }

    lore_panel.display_timer -= time.delta_secs();
    let close = keyboard.just_pressed(KeyCode::Enter)
        || keyboard.just_pressed(KeyCode::Escape)
        || lore_panel.display_timer <= 0.0;

    if close {
        lore_panel.active = false;
        lore_panel.full_text.clear();
        lore_panel.typewriter_pos = 0.0;
        for entity in panel_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        return;
    }

    let max_chars = (lore_panel.typewriter_pos as usize).min(lore_panel.full_text.len());
    let visible = lore_panel.full_text[..max_chars].to_string();
    for mut text in text_query.iter_mut() {
        **text = visible.clone();
    }
}
