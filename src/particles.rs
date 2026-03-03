use bevy::prelude::*;
use rand::Rng;

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnParticlesEvent>()
            .add_systems(Update, (
                spawn_particles_from_events,
                update_particles,
            ));
    }
}

#[derive(Component)]
pub struct Particle {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

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
    mut query: Query<(Entity, &mut Particle, &mut Transform, &mut Sprite)>,
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
