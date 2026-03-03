use bevy::prelude::*;

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameAudio::default())
            .add_event::<SoundEvent>()
            .add_systems(Update, handle_sound_events);
    }
}

#[derive(Resource)]
pub struct GameAudio {
    pub master_volume: f32,
    pub sfx_volume: f32,
    pub music_volume: f32,
}

impl Default for GameAudio {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            sfx_volume: 0.8,
            music_volume: 0.5,
        }
    }
}

#[derive(Event, Clone, Debug)]
pub enum SoundEvent {
    Hit,
    Gather,
    Build,
    Craft,
    Pickup,
    MenuOpen,
    MenuClose,
    Death,
    BossRoar,
}

fn handle_sound_events(
    mut events: EventReader<SoundEvent>,
) {
    for event in events.read() {
        // TODO: Play actual audio when assets are available
        // For now, just log the event for debugging
        info!("Sound: {:?}", event);
    }
}
