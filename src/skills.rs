use bevy::prelude::*;
use crate::hud::{not_paused, FloatingTextRequest};
use crate::player::Player;

pub struct SkillsPlugin;

impl Plugin for SkillsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SkillLevels::default())
            .add_event::<SkillXpEvent>()
            .add_systems(Update, (
                process_skill_xp.run_if(not_paused),
                toggle_skill_panel,
            ));
    }
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkillType {
    Gathering,
    Combat,
    Fishing,
    Farming,
    Crafting,
    Building,
}

impl SkillType {
    pub fn display_name(&self) -> &'static str {
        match self {
            SkillType::Gathering => "Gathering",
            SkillType::Combat => "Combat",
            SkillType::Fishing => "Fishing",
            SkillType::Farming => "Farming",
            SkillType::Crafting => "Crafting",
            SkillType::Building => "Building",
        }
    }

    pub fn save_key(&self) -> &'static str {
        match self {
            SkillType::Gathering => "gathering",
            SkillType::Combat => "combat",
            SkillType::Fishing => "fishing",
            SkillType::Farming => "farming",
            SkillType::Crafting => "crafting",
            SkillType::Building => "building",
        }
    }

    pub fn from_save_key(key: &str) -> Option<SkillType> {
        match key {
            "gathering" => Some(SkillType::Gathering),
            "combat" => Some(SkillType::Combat),
            "fishing" => Some(SkillType::Fishing),
            "farming" => Some(SkillType::Farming),
            "crafting" => Some(SkillType::Crafting),
            "building" => Some(SkillType::Building),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Skill data
// ---------------------------------------------------------------------------

pub struct SkillData {
    pub level: u32,
    pub xp: u32,
}

impl Default for SkillData {
    fn default() -> Self {
        Self { level: 1, xp: 0 }
    }
}

impl SkillData {
    /// XP required to reach the next level: `level * level * 50`.
    pub fn xp_for_next_level(&self) -> u32 {
        self.level * self.level * 50
    }

    /// Progress fraction toward the next level (0.0 .. 1.0).
    pub fn progress_fraction(&self) -> f32 {
        let needed = self.xp_for_next_level();
        if needed == 0 { return 1.0; }
        self.xp as f32 / needed as f32
    }
}

pub const MAX_SKILL_LEVEL: u32 = 20;

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

#[derive(Resource)]
pub struct SkillLevels {
    pub gathering: SkillData,
    pub combat: SkillData,
    pub fishing: SkillData,
    pub farming: SkillData,
    pub crafting: SkillData,
    pub building: SkillData,
    pub skills_open: bool,
}

impl Default for SkillLevels {
    fn default() -> Self {
        Self {
            gathering: SkillData::default(),
            combat: SkillData::default(),
            fishing: SkillData::default(),
            farming: SkillData::default(),
            crafting: SkillData::default(),
            building: SkillData::default(),
            skills_open: false,
        }
    }
}

impl SkillLevels {
    pub fn get(&self, skill: SkillType) -> &SkillData {
        match skill {
            SkillType::Gathering => &self.gathering,
            SkillType::Combat => &self.combat,
            SkillType::Fishing => &self.fishing,
            SkillType::Farming => &self.farming,
            SkillType::Crafting => &self.crafting,
            SkillType::Building => &self.building,
        }
    }

    pub fn get_mut(&mut self, skill: SkillType) -> &mut SkillData {
        match skill {
            SkillType::Gathering => &mut self.gathering,
            SkillType::Combat => &mut self.combat,
            SkillType::Fishing => &mut self.fishing,
            SkillType::Farming => &mut self.farming,
            SkillType::Crafting => &mut self.crafting,
            SkillType::Building => &mut self.building,
        }
    }

    /// Returns save data as Vec<(key, level, xp)>.
    pub fn to_save_data(&self) -> Vec<(String, u32, u32)> {
        let skills = [
            SkillType::Gathering,
            SkillType::Combat,
            SkillType::Fishing,
            SkillType::Farming,
            SkillType::Crafting,
            SkillType::Building,
        ];
        skills.iter().map(|s| {
            let data = self.get(*s);
            (s.save_key().to_string(), data.level, data.xp)
        }).collect()
    }

    /// Restores skill data from save format.
    pub fn restore_from_save_data(&mut self, data: &[(String, u32, u32)]) {
        for (key, level, xp) in data {
            if let Some(skill_type) = SkillType::from_save_key(key) {
                let skill = self.get_mut(skill_type);
                skill.level = (*level).clamp(1, MAX_SKILL_LEVEL);
                skill.xp = *xp;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Event
// ---------------------------------------------------------------------------

#[derive(Event)]
pub struct SkillXpEvent {
    pub skill: SkillType,
    pub amount: u32,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn process_skill_xp(
    mut events: EventReader<SkillXpEvent>,
    mut skill_levels: ResMut<SkillLevels>,
    mut floating_text_events: EventWriter<FloatingTextRequest>,
    player_query: Query<&Transform, With<Player>>,
) {
    for ev in events.read() {
        let data = skill_levels.get_mut(ev.skill);

        if data.level >= MAX_SKILL_LEVEL {
            continue;
        }

        data.xp = data.xp.saturating_add(ev.amount);

        // Check for level-ups (may skip multiple levels with large XP gains)
        loop {
            if data.level >= MAX_SKILL_LEVEL {
                data.xp = 0;
                break;
            }
            let needed = data.level * data.level * 50;
            if data.xp >= needed {
                data.xp -= needed;
                data.level += 1;

                // Send floating text on level-up
                if let Ok(player_tf) = player_query.get_single() {
                    floating_text_events.send(FloatingTextRequest {
                        text: format!("{} Lv {}!", ev.skill.display_name(), data.level),
                        position: player_tf.translation.truncate(),
                        color: Color::srgb(0.3, 1.0, 0.5),
                    });
                }

                info!("{} leveled up to {}!", ev.skill.display_name(), data.level);
            } else {
                break;
            }
        }
    }
}

fn toggle_skill_panel(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut skill_levels: ResMut<SkillLevels>,
) {
    if keyboard.just_pressed(KeyCode::KeyK) {
        skill_levels.skills_open = !skill_levels.skills_open;
    }
}
