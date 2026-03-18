use bevy::prelude::*;

#[derive(Resource)]
pub struct EtherealTheme {
    /// Foundation: Near-black obsidian (#020206)
    #[allow(dead_code)]
    pub background: Color,
    /// Markers: Spectral slate (#9999B3)
    pub accent_slate: Color,
    /// Pulses: Amber gold (#E6BF4D)
    pub accent_gold: Color,
    /// Critical: Spectral red
    #[allow(dead_code)]
    pub critical: Color,
    /// Healing: Spectral green
    pub healing: Color,
}

impl Default for EtherealTheme {
    fn default() -> Self {
        Self {
            background: Color::srgb(0.008, 0.008, 0.024), // #020206 approx
            accent_slate: Color::srgb(0.6, 0.6, 0.7),    // #9999B3 approx
            accent_gold: Color::srgb(0.9, 0.75, 0.3),    // #E6BF4D approx
            critical: Color::srgb(0.8, 0.2, 0.2),
            healing: Color::srgb(0.2, 0.8, 0.4),
        }
    }
}

impl EtherealTheme {
    pub fn panel_bg(&self) -> Color {
        Color::srgba(0.02, 0.02, 0.06, 0.85)
    }

    pub fn panel_border(&self, highlighted: bool) -> Color {
        if highlighted {
            Color::srgba(0.9, 0.75, 0.3, 0.9)
        } else {
            Color::srgba(0.3, 0.3, 0.4, 0.6)
        }
    }

    pub fn hud_label_color(&self) -> Color {
        self.accent_slate
    }

    pub fn hud_primary_text(&self) -> Color {
        self.accent_gold
    }
}

pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EtherealTheme::default());
    }
}
