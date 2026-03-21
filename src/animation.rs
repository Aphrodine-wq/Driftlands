use bevy::prelude::*;

use crate::assets::GameAssets;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpriteAnimationKind {
    WolfWalk,
    SpiderWalk,
    ShadowCrawlerWalk,
    Campfire,
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate_sprites);
    }
}

/// Generic frame-based sprite animation component.
/// Attach to any entity with a `Sprite` to cycle through frames.
#[derive(Component)]
pub struct SpriteAnimation {
    pub kind: SpriteAnimationKind,
    pub frames: Vec<Handle<Image>>,
    pub current_frame: usize,
    pub timer: f32,
    pub frame_duration: f32,
    pub looping: bool,
}

impl SpriteAnimation {
    pub fn new(
        kind: SpriteAnimationKind,
        frames: Vec<Handle<Image>>,
        frame_duration: f32,
        looping: bool,
    ) -> Self {
        Self {
            kind,
            frames,
            current_frame: 0,
            timer: 0.0,
            frame_duration,
            looping,
        }
    }
}

fn animate_sprites(
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut perf: ResMut<crate::debug_perf::DebugPerfTiming>,
    mut query: Query<(&mut SpriteAnimation, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (mut anim, mut sprite) in query.iter_mut() {
        if anim.frames.is_empty() {
            continue;
        }
        anim.timer += dt;
        if anim.timer >= anim.frame_duration {
            anim.timer -= anim.frame_duration;
            anim.current_frame += 1;
            if anim.current_frame >= anim.frames.len() {
                if anim.looping {
                    anim.current_frame = 0;
                } else {
                    anim.current_frame = anim.frames.len() - 1;
                }
            }
            // If an atlas is available and this sprite isn't already using one,
            // upgrade it once (then animate by index without swapping handles).
            if sprite.texture_atlas.is_none() {
                let (atlas_image, atlas_layout) = match anim.kind {
                    SpriteAnimationKind::WolfWalk => (
                        assets.wolf_walk_atlas_image.clone(),
                        assets.wolf_walk_atlas_layout.clone(),
                    ),
                    SpriteAnimationKind::SpiderWalk => (
                        assets.spider_walk_atlas_image.clone(),
                        assets.spider_walk_atlas_layout.clone(),
                    ),
                    SpriteAnimationKind::ShadowCrawlerWalk => (
                        assets.shadow_crawler_walk_atlas_image.clone(),
                        assets.shadow_crawler_walk_atlas_layout.clone(),
                    ),
                    SpriteAnimationKind::Campfire => (
                        assets.campfire_anim_atlas_image.clone(),
                        assets.campfire_anim_atlas_layout.clone(),
                    ),
                };

                if let (Some(atlas_image), Some(atlas_layout)) = (atlas_image, atlas_layout) {
                    sprite.image = atlas_image;
                    sprite.texture_atlas = Some(TextureAtlas {
                        layout: atlas_layout,
                        index: anim.current_frame,
                    });
                    perf.atlas_upgrades += 1;
                }
            }

            // Prefer atlas index updates if we're using an atlas; otherwise,
            // fall back to swapping sprite.image (should only happen before the
            // atlas is ready).
            if let Some(atlas) = sprite.texture_atlas.as_mut() {
                atlas.index = anim.current_frame;
            } else {
                sprite.image = anim.frames[anim.current_frame].clone();
            }
        }
    }
}
