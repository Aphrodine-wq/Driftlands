use bevy::prelude::*;

#[derive(Resource)]
pub struct EtherealTheme {
    /// Foundation: Near-black obsidian (#020206)
    pub background: Color,
    /// Markers: Spectral slate (#9999B3)
    pub accent_slate: Color,
    /// Pulses: Amber gold (#E6BF4D)
    pub accent_gold: Color,
    /// Critical: Spectral red
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

pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EtherealTheme::default());
    }
}
