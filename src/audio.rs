use bevy::prelude::*;
use bevy::audio::{AudioPlayer, PlaybackSettings, Volume};
use crate::daynight::{DayNightCycle, DayPhase};
use crate::weather::{Weather, WeatherSystem};

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameAudio::default())
            .add_event::<SoundEvent>()
            .add_systems(Update, (drive_ambient_sound, handle_sound_events));
    }
}

#[derive(Resource)]
#[allow(dead_code)]
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
    TreeFall,
    OreMine,
    Build,
    BuildBreak,
    Craft,
    Pickup,
    MenuOpen,
    #[allow(dead_code)]
    MenuClose,
    Death,
    BossRoar,
    PlaceInvalid,
    Trade,
    Discovery,
    LoreComplete,
    AmbientDay,
    AmbientNight,
    AmbientRain,
    AmbientStorm,
}

fn sound_event_path(event: &SoundEvent) -> Option<&'static str> {
    match event {
        SoundEvent::Hit => Some("audio/sfx/hit.ogg"),
        SoundEvent::Gather => Some("audio/sfx/gather.ogg"),
        SoundEvent::TreeFall => Some("audio/sfx/gather.ogg"),
        SoundEvent::OreMine => Some("audio/sfx/gather.ogg"),
        SoundEvent::Build => Some("audio/sfx/build.ogg"),
        SoundEvent::BuildBreak => Some("audio/sfx/build.ogg"),
        SoundEvent::Craft => Some("audio/sfx/craft.ogg"),
        SoundEvent::Pickup => Some("audio/sfx/pickup.ogg"),
        SoundEvent::MenuOpen => Some("audio/sfx/menu_open.ogg"),
        SoundEvent::MenuClose => Some("audio/sfx/menu_open.ogg"),
        SoundEvent::Death => Some("audio/sfx/death.ogg"),
        SoundEvent::BossRoar => Some("audio/sfx/boss_roar.ogg"),
        SoundEvent::PlaceInvalid => Some("audio/sfx/place_invalid.ogg"),
        SoundEvent::Trade => Some("audio/sfx/trade.ogg"),
        SoundEvent::Discovery => Some("audio/sfx/discovery.ogg"),
        SoundEvent::LoreComplete => Some("audio/sfx/lore_complete.ogg"),
        SoundEvent::AmbientDay
        | SoundEvent::AmbientNight
        | SoundEvent::AmbientRain
        | SoundEvent::AmbientStorm => None,
    }
}

fn handle_sound_events(
    mut commands: Commands,
    mut events: EventReader<SoundEvent>,
    asset_server: Res<AssetServer>,
    game_audio: Res<GameAudio>,
) {
    let vol = game_audio.master_volume * game_audio.sfx_volume;
    let volume = Volume::new(vol);

    for event in events.read() {
        let Some(path) = sound_event_path(event) else {
            continue;
        };
        let source = asset_server.load(path);
        commands.spawn((
            AudioPlayer::new(source),
            PlaybackSettings::DESPAWN.with_volume(volume),
        ));
    }
}

fn drive_ambient_sound(
    cycle: Res<DayNightCycle>,
    season: Res<crate::season::SeasonCycle>,
    weather: Res<WeatherSystem>,
    mut timer: Local<f32>,
    mut events: EventWriter<SoundEvent>,
) {
    // Simple heartbeat to request ambience every few seconds.
    *timer -= 1.0 / 60.0;
    if *timer > 0.0 {
        return;
    }
    *timer = 8.0;

    // Weather ambience takes priority.
    match weather.current {
        Weather::Storm => {
            events.send(SoundEvent::AmbientStorm);
            return;
        }
        Weather::Rain | Weather::Snow | Weather::Fog => {
            events.send(SoundEvent::AmbientRain);
            return;
        }
        Weather::Blizzard => {
            events.send(SoundEvent::AmbientStorm);
            return;
        }
        Weather::Clear => {}
    }

    let phase = cycle.phase_with_season(season.current);
    match phase {
        DayPhase::Night => {
            events.send(SoundEvent::AmbientNight);
        }
        _ => {
            events.send(SoundEvent::AmbientDay);
        }
    }
}
