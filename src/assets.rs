use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameAssets>()
            .add_systems(PreStartup, generate_assets);
    }
}

#[derive(Resource, Default)]
pub struct GameAssets {
    pub player: Handle<Image>,
    pub forest_grass: Handle<Image>,
    pub dirt: Handle<Image>,
    pub water: Handle<Image>,
    pub stone: Handle<Image>,
    pub sand: Handle<Image>,
    pub oak_tree: Handle<Image>,
    pub pine_tree: Handle<Image>,
    pub rock: Handle<Image>,
    // Audio handles could go here too
}

fn generate_assets(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let mut assets = GameAssets::default();

    // Generate Player Texture (16x16)
    assets.player = images.add(generate_noise_texture(16, 16, Color::srgb(0.2, 0.4, 0.9), 0.2));

    // Generate Tile Textures (16x16)
    assets.forest_grass = images.add(generate_noise_texture(16, 16, Color::srgb(0.2, 0.55, 0.2), 0.15));
    assets.dirt = images.add(generate_noise_texture(16, 16, Color::srgb(0.47, 0.35, 0.22), 0.1));
    assets.water = images.add(generate_noise_texture(16, 16, Color::srgb(0.25, 0.5, 0.8), 0.1));
    assets.stone = images.add(generate_noise_texture(16, 16, Color::srgb(0.5, 0.5, 0.5), 0.2));
    assets.sand = images.add(generate_noise_texture(16, 16, Color::srgb(0.8, 0.75, 0.5), 0.05));

    // Generate World Object Textures (32x32 for trees, 16x16 for rocks)
    assets.oak_tree = images.add(generate_tree_texture(false));
    assets.pine_tree = images.add(generate_tree_texture(true));
    assets.rock = images.add(generate_noise_texture(16, 16, Color::srgb(0.45, 0.45, 0.45), 0.3));

    commands.insert_resource(assets);
}

/// Generates a simple noisy texture for "pixel art" feel.
fn generate_noise_texture(width: u32, height: u32, base_color: Color, noise_amount: f32) -> Image {
    let mut data = vec![0u8; (width * height * 4) as usize];
    let base_rgba = base_color.to_srgba();
    
    for y in 0..height {
        for x in 0..width {
            let index = ((y * width + x) * 4) as usize;
            let noise = (rand::random::<f32>() - 0.5) * noise_amount;
            
            data[index] = ((base_rgba.red + noise).clamp(0.0, 1.0) * 255.0) as u8;
            data[index + 1] = ((base_rgba.green + noise).clamp(0.0, 1.0) * 255.0) as u8;
            data[index + 2] = ((base_rgba.blue + noise).clamp(0.0, 1.0) * 255.0) as u8;
            data[index + 3] = 255;
        }
    }

    Image::new(
        Extent3d { width, height, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

fn generate_tree_texture(is_pine: bool) -> Image {
    let width = 32;
    let height = 32;
    let mut data = vec![0u8; (width * height * 4) as usize];
    
    let trunk_color = [100, 70, 40, 255];
    let leaf_color = if is_pine { [30, 80, 40, 255] } else { [50, 120, 40, 255] };

    for y in 0..height {
        for x in 0..width {
            let index = ((y * width + x) * 4) as usize;
            
            // Simple geometric tree shape
            let is_trunk = x >= 14 && x <= 18 && y >= 20;
            let is_leaves = if is_pine {
                // Triangle
                let row_width = (y as f32 * 0.8) as i32;
                y < 24 && (x as i32 - 16).abs() < row_width
            } else {
                // Circle-ish
                let dx = x as f32 - 16.0;
                let dy = y as f32 - 12.0;
                (dx*dx + dy*dy).sqrt() < 10.0
            };

            if is_trunk {
                data[index..index+4].copy_from_slice(&trunk_color);
            } else if is_leaves {
                let noise = (rand::random::<f32>() - 0.5) * 20.0;
                data[index] = (leaf_color[0] as f32 + noise).clamp(0.0, 255.0) as u8;
                data[index+1] = (leaf_color[1] as f32 + noise).clamp(0.0, 255.0) as u8;
                data[index+2] = (leaf_color[2] as f32 + noise).clamp(0.0, 255.0) as u8;
                data[index+3] = 255;
            } else {
                data[index+3] = 0; // Transparent
            }
        }
    }

    Image::new(
        Extent3d { width, height, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}
