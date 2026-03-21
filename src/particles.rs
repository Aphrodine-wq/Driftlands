use bevy::prelude::*;
use rand::Rng;
use crate::hud::CurrentBiome;
use crate::player::Player;
use crate::weather::{WeatherSystem, Weather};
use crate::world::generation::Biome;

/// Hard cap on total particles (ambient + effect) to protect frame budget.
const MAX_PARTICLES: usize = 200;

pub struct ParticlePlugin;

/// Tracks ambient particle count without iterating the query every frame.
#[derive(Resource, Default)]
pub struct ParticleCount {
    pub ambient: usize,
}

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnParticlesEvent>()
            .insert_resource(AmbientParticleTimer(Timer::from_seconds(0.08, TimerMode::Repeating)))
            .insert_resource(ParticleCount::default())
            .add_systems(Update, (
                spawn_particles_from_events,
                update_particles,
                update_ambient_particles,
                spawn_ambient_particles,
            ));
    }
}

#[derive(Component)]
pub struct Particle {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

/// Ambient particles don't shrink — they just drift and fade.
#[derive(Component)]
pub struct AmbientParticle;

#[derive(Resource)]
struct AmbientParticleTimer(Timer);

#[derive(Event)]
pub struct SpawnParticlesEvent {
    pub position: Vec2,
    pub color: Color,
    pub count: usize,
}

fn spawn_particles_from_events(
    mut commands: Commands,
    mut events: EventReader<SpawnParticlesEvent>,
) {
    let mut rng = rand::thread_rng();
    for event in events.read() {
        for _ in 0..event.count {
            let lifetime = rng.gen_range(0.3..0.5);
            let velocity = Vec2::new(
                rng.gen_range(-40.0..40.0),
                rng.gen_range(-40.0..40.0),
            );
            commands.spawn((
                Particle {
                    velocity,
                    lifetime,
                    max_lifetime: lifetime,
                },
                Sprite {
                    color: event.color,
                    custom_size: Some(Vec2::new(4.0, 4.0)),
                    ..default()
                },
                Transform::from_xyz(event.position.x, event.position.y, 8.0),
            ));
        }
    }
}

fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Particle, &mut Transform, &mut Sprite), Without<AmbientParticle>>,
) {
    let dt = time.delta_secs();
    for (entity, mut particle, mut transform, mut sprite) in query.iter_mut() {
        particle.lifetime -= dt;
        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        // Move
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;
        // Shrink
        let ratio = particle.lifetime / particle.max_lifetime;
        let size = 4.0 * ratio;
        sprite.custom_size = Some(Vec2::new(size, size));
        // Fade
        let c = sprite.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, ratio);
    }
}

fn spawn_ambient_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<AmbientParticleTimer>,
    current_biome: Res<CurrentBiome>,
    player_query: Query<&Transform, With<Player>>,
    weather: Res<WeatherSystem>,
    mut particle_count: ResMut<ParticleCount>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() { return; }

    // Use tracked count instead of iterating the query every frame
    if particle_count.ambient > 60 { return; }

    // Global hard cap — skip ALL spawning if too many particles exist
    if particle_count.ambient >= MAX_PARTICLES { return; }

    let Ok(player_tf) = player_query.get_single() else { return };
    let center = player_tf.translation.truncate();
    let mut rng = rand::thread_rng();

    // Weather particles (reduced spawn counts)
    match weather.current {
        Weather::Rain => {
            for _ in 0..2 {
                let pos = center + Vec2::new(rng.gen_range(-200.0..200.0), rng.gen_range(80.0..120.0));
                spawn_ambient_tracked(&mut commands, &mut particle_count, pos,
                    Vec2::new(rng.gen_range(-10.0..10.0), -120.0),
                    Color::srgba(0.5, 0.6, 0.8, 0.4), 2.0, Vec2::new(1.0, 3.0));
            }
        }
        Weather::Snow => {
            if rng.gen::<f32>() < 0.7 {
                let pos = center + Vec2::new(rng.gen_range(-200.0..200.0), rng.gen_range(80.0..120.0));
                spawn_ambient_tracked(&mut commands, &mut particle_count, pos,
                    Vec2::new(rng.gen_range(-15.0..15.0), rng.gen_range(-20.0..-10.0)),
                    Color::srgba(0.9, 0.92, 0.95, 0.6), 4.0, Vec2::new(2.0, 2.0));
            }
        }
        Weather::Storm => {
            for _ in 0..2 {
                let pos = center + Vec2::new(rng.gen_range(-200.0..200.0), rng.gen_range(80.0..120.0));
                spawn_ambient_tracked(&mut commands, &mut particle_count, pos,
                    Vec2::new(rng.gen_range(-30.0..30.0), -180.0),
                    Color::srgba(0.4, 0.5, 0.7, 0.5), 1.5, Vec2::new(1.0, 4.0));
            }
        }
        Weather::Clear => {}
        Weather::Fog => {
            // Fog: spawn drifting low-opacity particles
            if rng.gen::<f32>() < 0.7 {
                let pos = center + Vec2::new(rng.gen_range(-200.0..200.0), rng.gen_range(-40.0..40.0));
                spawn_ambient_tracked(&mut commands, &mut particle_count, pos,
                    Vec2::new(rng.gen_range(-8.0..8.0), rng.gen_range(-2.0..2.0)),
                    Color::srgba(0.7, 0.7, 0.7, 0.25), 5.0, Vec2::new(6.0, 4.0));
            }
        }
        Weather::Blizzard => {
            for _ in 0..3 {
                let pos = center + Vec2::new(rng.gen_range(-200.0..200.0), rng.gen_range(80.0..120.0));
                spawn_ambient_tracked(&mut commands, &mut particle_count, pos,
                    Vec2::new(rng.gen_range(-60.0..-20.0), rng.gen_range(-80.0..-40.0)),
                    Color::srgba(0.85, 0.9, 1.0, 0.7), 2.5, Vec2::new(3.0, 3.0));
            }
        }
    }

    // Biome-specific ambient particles (reduced spawn thresholds)
    let Some(biome) = current_biome.biome else { return };
    match biome {
        Biome::Forest => {
            // Falling leaves
            if rng.gen::<f32>() < 0.2 {
                let pos = center + Vec2::new(rng.gen_range(-150.0..150.0), rng.gen_range(60.0..100.0));
                let color = if rng.gen::<bool>() {
                    Color::srgba(0.4, 0.55, 0.15, 0.5)
                } else {
                    Color::srgba(0.6, 0.4, 0.1, 0.5)
                };
                spawn_ambient_tracked(&mut commands, &mut particle_count, pos,
                    Vec2::new(rng.gen_range(-20.0..20.0), rng.gen_range(-15.0..-5.0)),
                    color, 5.0, Vec2::new(2.0, 2.0));
            }
        }
        Biome::Desert => {
            // Sand/dust motes
            if rng.gen::<f32>() < 0.25 {
                let pos = center + Vec2::new(rng.gen_range(-150.0..150.0), rng.gen_range(-50.0..50.0));
                spawn_ambient_tracked(&mut commands, &mut particle_count, pos,
                    Vec2::new(rng.gen_range(10.0..40.0), rng.gen_range(-5.0..5.0)),
                    Color::srgba(0.7, 0.6, 0.4, 0.3), 3.0, Vec2::new(1.5, 1.5));
            }
        }
        Biome::Tundra => {
            // Drifting snowflakes (always, even in clear weather)
            if rng.gen::<f32>() < 0.25 {
                let pos = center + Vec2::new(rng.gen_range(-150.0..150.0), rng.gen_range(60.0..100.0));
                spawn_ambient_tracked(&mut commands, &mut particle_count, pos,
                    Vec2::new(rng.gen_range(-10.0..10.0), rng.gen_range(-12.0..-4.0)),
                    Color::srgba(0.85, 0.88, 0.92, 0.5), 6.0, Vec2::new(2.0, 2.0));
            }
        }
        Biome::Volcanic => {
            // Embers and ash
            if rng.gen::<f32>() < 0.3 {
                let pos = center + Vec2::new(rng.gen_range(-120.0..120.0), rng.gen_range(-60.0..20.0));
                let is_ember = rng.gen::<f32>() < 0.3;
                let color = if is_ember {
                    Color::srgba(1.0, 0.5, 0.1, 0.7)
                } else {
                    Color::srgba(0.3, 0.3, 0.3, 0.3)
                };
                spawn_ambient_tracked(&mut commands, &mut particle_count, pos,
                    Vec2::new(rng.gen_range(-5.0..5.0), rng.gen_range(8.0..25.0)),
                    color, 3.0, Vec2::new(if is_ember { 1.5 } else { 2.5 }, if is_ember { 1.5 } else { 2.5 }));
            }
        }
        Biome::Fungal => {
            // Floating spores
            if rng.gen::<f32>() < 0.25 {
                let pos = center + Vec2::new(rng.gen_range(-120.0..120.0), rng.gen_range(-60.0..60.0));
                spawn_ambient_tracked(&mut commands, &mut particle_count, pos,
                    Vec2::new(rng.gen_range(-8.0..8.0), rng.gen_range(3.0..10.0)),
                    Color::srgba(0.5, 0.8, 0.4, 0.4), 4.0, Vec2::new(1.5, 1.5));
            }
        }
        Biome::CrystalCave => {
            // Sparkles
            if rng.gen::<f32>() < 0.2 {
                let pos = center + Vec2::new(rng.gen_range(-120.0..120.0), rng.gen_range(-60.0..60.0));
                spawn_ambient_tracked(&mut commands, &mut particle_count, pos,
                    Vec2::new(rng.gen_range(-3.0..3.0), rng.gen_range(-3.0..3.0)),
                    Color::srgba(0.7, 0.6, 1.0, 0.6), 2.0, Vec2::new(1.0, 1.0));
            }
        }
        Biome::Swamp => {
            // Mist wisps
            if rng.gen::<f32>() < 0.15 {
                let pos = center + Vec2::new(rng.gen_range(-150.0..150.0), rng.gen_range(-40.0..20.0));
                spawn_ambient_tracked(&mut commands, &mut particle_count, pos,
                    Vec2::new(rng.gen_range(-5.0..5.0), rng.gen_range(1.0..4.0)),
                    Color::srgba(0.4, 0.5, 0.35, 0.2), 5.0, Vec2::new(4.0, 3.0));
            }
        }
        Biome::Coastal => {
            // Sea spray
            if rng.gen::<f32>() < 0.15 {
                let pos = center + Vec2::new(rng.gen_range(-150.0..150.0), rng.gen_range(-20.0..40.0));
                spawn_ambient_tracked(&mut commands, &mut particle_count, pos,
                    Vec2::new(rng.gen_range(5.0..20.0), rng.gen_range(2.0..8.0)),
                    Color::srgba(0.7, 0.8, 0.9, 0.3), 3.0, Vec2::new(2.0, 1.5));
            }
        }
        Biome::Mountain => {
            // Wind-blown dust
            if rng.gen::<f32>() < 0.15 {
                let pos = center + Vec2::new(rng.gen_range(-150.0..150.0), rng.gen_range(40.0..80.0));
                spawn_ambient_tracked(&mut commands, &mut particle_count, pos,
                    Vec2::new(rng.gen_range(15.0..35.0), rng.gen_range(-3.0..3.0)),
                    Color::srgba(0.6, 0.58, 0.55, 0.25), 3.0, Vec2::new(2.0, 1.5));
            }
        }
    }
}

fn update_ambient_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Particle, &mut Transform, &mut Sprite), With<AmbientParticle>>,
    mut particle_count: ResMut<ParticleCount>,
) {
    let dt = time.delta_secs();
    for (entity, mut particle, mut transform, mut sprite) in query.iter_mut() {
        particle.lifetime -= dt;
        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            particle_count.ambient = particle_count.ambient.saturating_sub(1);
            continue;
        }
        // Move (drift)
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;
        // Fade out in the last 30% of lifetime
        let ratio = (particle.lifetime / particle.max_lifetime).clamp(0.0, 1.0);
        let alpha = if ratio < 0.3 { ratio / 0.3 } else { 1.0 };
        let c = sprite.color.to_srgba();
        // Preserve original alpha intention by scaling it
        sprite.color = Color::srgba(c.red, c.green, c.blue, c.alpha.min(alpha));
    }
}

/// Spawn an ambient particle and increment the tracked count.
fn spawn_ambient_tracked(
    commands: &mut Commands,
    particle_count: &mut ResMut<ParticleCount>,
    pos: Vec2, vel: Vec2, color: Color, lifetime: f32, size: Vec2,
) {
    // Respect global cap
    if particle_count.ambient >= MAX_PARTICLES { return; }
    commands.spawn((
        AmbientParticle,
        Particle { velocity: vel, lifetime, max_lifetime: lifetime },
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 45.0), // Above world, below UI overlay
    ));
    particle_count.ambient += 1;
}
