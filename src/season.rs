use bevy::prelude::*;
use crate::daynight::DayNightCycle;
use crate::hud::not_paused;

pub struct SeasonPlugin;

impl Plugin for SeasonPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SeasonCycle::default())
            .add_systems(Update, advance_season.run_if(not_paused));
    }
}

/// How many in-game days each season lasts.
pub const DAYS_PER_SEASON: u32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub fn name(&self) -> &str {
        match self {
            Season::Spring => "Spring",
            Season::Summer => "Summer",
            Season::Autumn => "Autumn",
            Season::Winter => "Winter",
        }
    }

    /// Returns the next season in the cycle.
    pub fn next(self) -> Self {
        match self {
            Season::Spring => Season::Summer,
            Season::Summer => Season::Autumn,
            Season::Autumn => Season::Winter,
            Season::Winter => Season::Spring,
        }
    }

    /// Derive the season from an absolute day count (1-based).
    pub fn from_day(day: u32) -> Self {
        let cycle_pos = ((day.saturating_sub(1)) / DAYS_PER_SEASON) % 4;
        match cycle_pos {
            0 => Season::Spring,
            1 => Season::Summer,
            2 => Season::Autumn,
            _ => Season::Winter,
        }
    }

    /// Growth speed multiplier applied to farm plots.
    pub fn growth_multiplier(&self) -> f32 {
        match self {
            Season::Spring => 1.2,
            Season::Summer => 1.5,
            Season::Autumn => 0.8,
            Season::Winter => 0.3,
        }
    }
}

#[derive(Resource)]
pub struct SeasonCycle {
    pub current: Season,
    /// Tracks the last day_count value so we only run advance logic on day change.
    last_day: u32,
    /// True for one frame after a season transition.
    pub just_changed: bool,
}

impl Default for SeasonCycle {
    fn default() -> Self {
        Self {
            current: Season::Spring,
            last_day: 1,
            just_changed: false,
        }
    }
}

impl Season {
    /// Grass/ground color tint multiplier for this season.
    pub fn grass_color(&self) -> Color {
        match self {
            Season::Spring => Color::srgb(0.3, 0.75, 0.3),
            Season::Summer => Color::srgb(0.25, 0.6, 0.2),
            Season::Autumn => Color::srgb(0.7, 0.55, 0.2),
            Season::Winter => Color::srgb(0.8, 0.85, 0.9),
        }
    }

    /// Tree color tint for this season.
    pub fn tree_color(&self) -> Color {
        match self {
            Season::Spring => Color::srgb(0.2, 0.7, 0.25),
            Season::Summer => Color::srgb(0.15, 0.55, 0.15),
            Season::Autumn => Color::srgb(0.7, 0.35, 0.1),
            Season::Winter => Color::srgb(0.45, 0.3, 0.15),
        }
    }

    /// Water color variation per season.
    pub fn water_color(&self) -> Color {
        match self {
            Season::Spring => Color::srgb(0.15, 0.35, 0.7),
            Season::Summer => Color::srgb(0.1, 0.4, 0.75),
            Season::Autumn => Color::srgb(0.2, 0.3, 0.55),
            Season::Winter => Color::srgb(0.5, 0.6, 0.75),
        }
    }

    /// ClearColor background shift for each season.
    pub fn clear_color(&self) -> Color {
        match self {
            Season::Spring => Color::srgb(0.08, 0.12, 0.08),
            Season::Summer => Color::srgb(0.1, 0.1, 0.06),
            Season::Autumn => Color::srgb(0.1, 0.08, 0.05),
            Season::Winter => Color::srgb(0.08, 0.1, 0.14),
        }
    }
}

fn advance_season(
    mut season: ResMut<SeasonCycle>,
    cycle: Res<DayNightCycle>,
) {
    if cycle.day_count == season.last_day {
        return;
    }
    let old = season.current;
    season.last_day = cycle.day_count;
    season.current = Season::from_day(cycle.day_count);
    if season.current != old {
        season.just_changed = true;
    }
}
