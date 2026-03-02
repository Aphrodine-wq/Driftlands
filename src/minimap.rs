use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use crate::player::Player;
use crate::camera::GameCamera;
use crate::world::chunk::{Chunk, CHUNK_SIZE};
use crate::world::{TILE_SIZE, CHUNK_WORLD_SIZE};

pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_minimap)
            .add_systems(Update, update_minimap);
    }
}

const MINIMAP_SIZE: usize = 120;
const MINIMAP_SCALE: f32 = 4.0; // Each minimap pixel = 4 world pixels

#[derive(Component)]
pub struct Minimap;

fn spawn_minimap(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let image = create_minimap_image();
    let image_handle = images.add(image);

    commands.spawn((
        Minimap,
        Sprite {
            image: image_handle,
            custom_size: Some(Vec2::new(MINIMAP_SIZE as f32, MINIMAP_SIZE as f32)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 100.0),
    ));
}

fn create_minimap_image() -> Image {
    let size = Extent3d {
        width: MINIMAP_SIZE as u32,
        height: MINIMAP_SIZE as u32,
        depth_or_array_layers: 1,
    };
    let data = vec![30u8; MINIMAP_SIZE * MINIMAP_SIZE * 4]; // Dark background
    Image::new(
        size,
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

fn update_minimap(
    player_query: Query<&Transform, With<Player>>,
    camera_query: Query<&Transform, (With<GameCamera>, Without<Player>, Without<Minimap>)>,
    chunk_query: Query<&Chunk>,
    mut minimap_query: Query<(&Minimap, &Sprite, &mut Transform), (Without<Player>, Without<GameCamera>)>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok(player_tf) = player_query.get_single() else { return };
    let Ok(cam_tf) = camera_query.get_single() else { return };
    let Ok((_, sprite, mut minimap_tf)) = minimap_query.get_single_mut() else { return };

    // Position minimap in top-right of screen (offset from camera)
    minimap_tf.translation.x = cam_tf.translation.x + 570.0;
    minimap_tf.translation.y = cam_tf.translation.y + 290.0;

    // Update the minimap image
    let image_handle = &sprite.image;
    let Some(image) = images.get_mut(image_handle) else { return };

    let player_world_x = player_tf.translation.x;
    let player_world_y = player_tf.translation.y;

    // Clear image
    for pixel in image.data.chunks_exact_mut(4) {
        pixel[0] = 20;
        pixel[1] = 20;
        pixel[2] = 30;
        pixel[3] = 255;
    }

    // For each pixel in the minimap, determine the world tile and get its color
    let half = MINIMAP_SIZE as f32 / 2.0;

    for my in 0..MINIMAP_SIZE {
        for mx in 0..MINIMAP_SIZE {
            // Map minimap pixel to world position
            let world_x = player_world_x + (mx as f32 - half) * MINIMAP_SCALE;
            let world_y = player_world_y + (half - my as f32) * MINIMAP_SCALE; // Flip Y

            // Find which chunk and tile this corresponds to
            let chunk_x = (world_x / CHUNK_WORLD_SIZE).floor() as i32;
            let chunk_y = (world_y / CHUNK_WORLD_SIZE).floor() as i32;

            let tile_x = ((world_x / TILE_SIZE).floor() as i32 - chunk_x * CHUNK_SIZE as i32).clamp(0, CHUNK_SIZE as i32 - 1) as usize;
            let tile_y = ((world_y / TILE_SIZE).floor() as i32 - chunk_y * CHUNK_SIZE as i32).clamp(0, CHUNK_SIZE as i32 - 1) as usize;

            // Find the chunk
            for chunk in chunk_query.iter() {
                if chunk.position.x == chunk_x && chunk.position.y == chunk_y {
                    let tile = chunk.get_tile(tile_x, tile_y);
                    let color = tile.color();
                    let idx = (my * MINIMAP_SIZE + mx) * 4;
                    if idx + 3 < image.data.len() {
                        image.data[idx] = color[0];
                        image.data[idx + 1] = color[1];
                        image.data[idx + 2] = color[2];
                        image.data[idx + 3] = 255;
                    }
                    break;
                }
            }
        }
    }

    // Draw player dot (white, 3x3 pixels at center)
    let center = MINIMAP_SIZE / 2;
    for dy in 0..3_usize {
        for dx in 0..3_usize {
            let px = center - 1 + dx;
            let py = center - 1 + dy;
            let idx = (py * MINIMAP_SIZE + px) * 4;
            if idx + 3 < image.data.len() {
                image.data[idx] = 255;
                image.data[idx + 1] = 255;
                image.data[idx + 2] = 255;
                image.data[idx + 3] = 255;
            }
        }
    }
}
