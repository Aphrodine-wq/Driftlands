use bevy::prelude::*;
use bevy::audio::{AudioPlayer, PlaybackSettings, Volume};
use crate::daynight::{DayNightCycle, DayPhase};
use crate::weather::{Weather, WeatherSystem};
use std::path::Path;

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameAudio::default())
            .add_event::<SoundEvent>()
            .add_systems(PreStartup, generate_placeholder_sfx)
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
    Eat,
    PlayerHurt,
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
        SoundEvent::Hit => Some("audio/sfx/hit.wav"),
        SoundEvent::Gather => Some("audio/sfx/gather.wav"),
        SoundEvent::TreeFall => Some("audio/sfx/tree_fall.wav"),
        SoundEvent::OreMine => Some("audio/sfx/ore_mine.wav"),
        SoundEvent::Build => Some("audio/sfx/build.wav"),
        SoundEvent::BuildBreak => Some("audio/sfx/build_break.wav"),
        SoundEvent::Craft => Some("audio/sfx/craft.wav"),
        SoundEvent::Pickup => Some("audio/sfx/pickup.wav"),
        SoundEvent::MenuOpen => Some("audio/sfx/menu_open.wav"),
        SoundEvent::MenuClose => Some("audio/sfx/menu_open.wav"),
        SoundEvent::Eat => Some("audio/sfx/eat.wav"),
        SoundEvent::PlayerHurt => Some("audio/sfx/player_hurt.wav"),
        SoundEvent::Death => Some("audio/sfx/death.wav"),
        SoundEvent::BossRoar => Some("audio/sfx/boss_roar.wav"),
        SoundEvent::PlaceInvalid => Some("audio/sfx/place_invalid.wav"),
        SoundEvent::Trade => Some("audio/sfx/trade.wav"),
        SoundEvent::Discovery => Some("audio/sfx/discovery.wav"),
        SoundEvent::LoreComplete => Some("audio/sfx/lore_complete.wav"),
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
    *timer -= 1.0 / 60.0;
    if *timer > 0.0 {
        return;
    }
    *timer = 8.0;

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

// ============================================================
// Procedural SFX Generator — creates WAV files on first launch
// ============================================================

const SAMPLE_RATE: u32 = 44100;

fn encode_wav(samples: &[i16]) -> Vec<u8> {
    let data_size = (samples.len() * 2) as u32;
    let file_size = 36 + data_size;
    let mut buf = Vec::with_capacity(44 + data_size as usize);

    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&file_size.to_le_bytes());
    buf.extend_from_slice(b"WAVE");
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes()); // PCM
    buf.extend_from_slice(&1u16.to_le_bytes()); // mono
    buf.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    buf.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes()); // block align
    buf.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_size.to_le_bytes());
    for &s in samples {
        buf.extend_from_slice(&s.to_le_bytes());
    }
    buf
}

/// Deterministic noise from integer seed (no rand dependency).
fn noise(seed: u32) -> f32 {
    let mut h = seed;
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^= h >> 15;
    (h as f32 / u32::MAX as f32) * 2.0 - 1.0
}

/// Core synthesizer: sine wave with optional sweep, noise mix, harmonic, decay envelope.
fn synth(
    duration_ms: u32,
    freq_start: f32,
    freq_end: f32,
    decay: f32,
    noise_mix: f32,
    harm_ratio: f32,
    harm_amp: f32,
) -> Vec<i16> {
    let n = (SAMPLE_RATE * duration_ms / 1000) as usize;
    let mut out = Vec::with_capacity(n);
    let pi2 = std::f32::consts::PI * 2.0;
    let mut phase = 0.0f32;
    let mut harm_phase = 0.0f32;

    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let progress = i as f32 / n as f32;
        let freq = freq_start + (freq_end - freq_start) * progress;
        let envelope = (-decay * t).exp();

        // Accumulate phase for smooth frequency sweeps
        phase += freq / SAMPLE_RATE as f32;
        harm_phase += freq * harm_ratio / SAMPLE_RATE as f32;

        let sine = (pi2 * phase).sin();
        let harm = (pi2 * harm_phase).sin() * harm_amp;
        let n_val = noise(i as u32 ^ 0xDEAD) * noise_mix;
        let tone_mix = 1.0 - noise_mix;

        let sample = (sine * tone_mix + n_val + harm) * envelope * 0.7;
        out.push((sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16);
    }
    out
}

/// Multi-note sequence: plays notes back-to-back.
fn synth_notes(notes: &[(u32, f32, f32, f32)]) -> Vec<i16> {
    let mut out = Vec::new();
    for &(ms, freq, decay, harm_ratio) in notes {
        out.extend(synth(ms, freq, freq, decay, 0.0, harm_ratio, 0.3));
    }
    out
}

/// Major chord: multiple frequencies played simultaneously.
fn synth_chord(duration_ms: u32, freqs: &[f32], decay: f32) -> Vec<i16> {
    let n = (SAMPLE_RATE * duration_ms / 1000) as usize;
    let pi2 = std::f32::consts::PI * 2.0;
    let amp = 0.7 / freqs.len() as f32;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let env = (-decay * t).exp();
        let mut sample = 0.0f32;
        for &f in freqs {
            sample += (pi2 * f * t).sin();
        }
        out.push(((sample * amp * env).clamp(-1.0, 1.0) * i16::MAX as f32) as i16);
    }
    out
}

fn write_if_missing(dir: &Path, name: &str, samples: Vec<i16>) {
    let path = dir.join(name);
    if path.exists() {
        return;
    }
    let _ = std::fs::write(&path, encode_wav(&samples));
}

fn generate_placeholder_sfx() {
    let dir = Path::new("assets/audio/sfx");
    if let Err(e) = std::fs::create_dir_all(dir) {
        warn!("Failed to create audio/sfx dir: {e}");
        return;
    }

    // Hit: sharp noise burst, 50ms
    write_if_missing(dir, "hit.wav",
        synth(50, 200.0, 100.0, 40.0, 0.8, 3.0, 0.2));

    // Gather: two quick taps, 80ms
    write_if_missing(dir, "gather.wav",
        synth(80, 800.0, 600.0, 30.0, 0.3, 2.0, 0.15));

    // Tree fall: low crunch + thump, 200ms
    write_if_missing(dir, "tree_fall.wav",
        synth(200, 180.0, 60.0, 8.0, 0.6, 1.5, 0.3));

    // Ore mine: metallic ring, 120ms
    write_if_missing(dir, "ore_mine.wav",
        synth(120, 2200.0, 2000.0, 18.0, 0.15, 1.5, 0.5));

    // Build: solid thump, 60ms
    write_if_missing(dir, "build.wav",
        synth(60, 150.0, 100.0, 35.0, 0.25, 2.0, 0.1));

    // Build break: crumble, 150ms
    write_if_missing(dir, "build_break.wav",
        synth(150, 120.0, 60.0, 10.0, 0.7, 1.5, 0.2));

    // Craft: anvil ding, 120ms
    write_if_missing(dir, "craft.wav",
        synth(120, 1400.0, 1400.0, 12.0, 0.05, 1.5, 0.5));

    // Pickup: rising sweep, 120ms
    write_if_missing(dir, "pickup.wav",
        synth(120, 400.0, 1200.0, 10.0, 0.0, 2.0, 0.2));

    // Menu open: soft click, 30ms
    write_if_missing(dir, "menu_open.wav",
        synth(30, 1000.0, 1000.0, 60.0, 0.0, 2.0, 0.1));

    // Eat: soft crunch, 100ms
    write_if_missing(dir, "eat.wav",
        synth(100, 300.0, 200.0, 25.0, 0.5, 3.0, 0.15));

    // Player hurt: sharp descending sting, 120ms
    write_if_missing(dir, "player_hurt.wav",
        synth(120, 600.0, 200.0, 15.0, 0.3, 1.5, 0.25));

    // Death: low descending groan, 500ms
    write_if_missing(dir, "death.wav",
        synth(500, 300.0, 60.0, 3.0, 0.35, 1.5, 0.3));

    // Boss roar: deep rumble, 350ms
    write_if_missing(dir, "boss_roar.wav",
        synth(350, 80.0, 50.0, 4.0, 0.6, 1.5, 0.4));

    // Place invalid: dissonant buzz, 80ms
    write_if_missing(dir, "place_invalid.wav",
        synth(80, 200.0, 200.0, 25.0, 0.4, 1.07, 0.9));

    // Trade: coin clink, 80ms
    write_if_missing(dir, "trade.wav",
        synth(80, 2500.0, 2500.0, 20.0, 0.0, 1.5, 0.6));

    // Discovery: ascending three-note chime (C5 → E5 → G5)
    write_if_missing(dir, "discovery.wav",
        synth_notes(&[
            (100, 523.0, 10.0, 2.0),
            (100, 659.0, 10.0, 2.0),
            (200, 784.0, 5.0, 2.0),
        ]));

    // Lore complete: triumphant C major chord
    write_if_missing(dir, "lore_complete.wav",
        synth_chord(500, &[523.0, 659.0, 784.0], 3.0));

    info!("Procedural SFX ready (assets/audio/sfx/)");
}
