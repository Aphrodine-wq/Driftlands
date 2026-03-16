use bevy::prelude::*;

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameAudio::default())
            .add_event::<SoundEvent>()
            .add_systems(Startup, load_audio)
            .add_systems(Update, handle_sound_events);
    }
}

#[derive(Resource, Default)]
pub struct GameAudio {
    pub master_volume: f32,
    pub sfx_volume: f32,
    pub music_volume: f32,
    
    // Audio handles
    pub hit: Handle<AudioSource>,
    pub gather: Handle<AudioSource>,
    pub build: Handle<AudioSource>,
    pub craft: Handle<AudioSource>,
    pub pickup: Handle<AudioSource>,
}

fn load_audio(
    mut game_audio: ResMut<GameAudio>,
    asset_server: Res<AssetServer>,
) {
    // These will look in assets/audio/
    game_audio.hit = asset_server.load("audio/hit.ogg");
    game_audio.gather = asset_server.load("audio/gather.ogg");
    game_audio.build = asset_server.load("audio/build.ogg");
    game_audio.craft = asset_server.load("audio/craft.ogg");
    game_audio.pickup = asset_server.load("audio/pickup.ogg");
    
    game_audio.master_volume = 1.0;
    game_audio.sfx_volume = 0.8;
    game_audio.music_volume = 0.5;
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
    game_audio: Res<GameAudio>,
    mut commands: Commands,
) {
    for event in events.read() {
        let handle = match event {
            SoundEvent::Hit => &game_audio.hit,
            SoundEvent::Gather => &game_audio.gather,
            SoundEvent::Build => &game_audio.build,
            SoundEvent::Craft => &game_audio.craft,
            SoundEvent::Pickup => &game_audio.pickup,
            _ => continue,
        };

        // Play the sound using Bevy's built-in audio
        commands.spawn((
            AudioPlayer(handle.clone()),
            PlaybackSettings {
                volume: bevy::audio::Volume::new(game_audio.sfx_volume * game_audio.master_volume),
                ..default()
            }
        ));
        
        info!("Sound played: {:?}", event);
    }
}
