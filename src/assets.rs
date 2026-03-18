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
    pub bush: Handle<Image>,
    pub cactus: Handle<Image>,
    pub mushroom: Handle<Image>,
    pub giant_mushroom: Handle<Image>,
    pub crystal: Handle<Image>,
    pub iron_vein: Handle<Image>,
    pub berry_bush: Handle<Image>,
    pub supply_crate: Handle<Image>,
    pub dungeon_entrance: Handle<Image>,
    // Enemy textures
    pub enemy_wolf: Handle<Image>,
    pub enemy_spider: Handle<Image>,
    pub enemy_crawler: Handle<Image>,
    pub enemy_zombie: Handle<Image>,
    pub enemy_elemental: Handle<Image>,
    pub enemy_wraith: Handle<Image>,
    pub enemy_scorpion: Handle<Image>,
    pub enemy_boss: Handle<Image>,
    // Attack visual
    pub slash_arc: Handle<Image>,
    // Screen effects
    pub vignette: Handle<Image>,
    // Building textures
    pub wood_wall: Handle<Image>,
    pub wood_floor: Handle<Image>,
    pub wood_door: Handle<Image>,
    pub stone_wall: Handle<Image>,
    pub campfire: Handle<Image>,
    pub workbench: Handle<Image>,
    pub forge: Handle<Image>,
    pub chest_building: Handle<Image>,
    pub bed: Handle<Image>,
    /// Flat normal maps for lit sprites (same UV as color; use size that matches or is larger).
    pub flat_normal_16: Handle<Image>,
    pub flat_normal_32: Handle<Image>,
    /// Shaped normals for key sprites (for when player/enemies use LitSpriteMaterial).
    pub player_normal: Handle<Image>,
    pub enemy_wolf_normal: Handle<Image>,
    pub enemy_zombie_normal: Handle<Image>,
    /// 1x1 white pixel for color-only world objects (tinted by material color).
    pub white_pixel: Handle<Image>,
}

fn generate_assets(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let mut assets = GameAssets::default();

    // Player: blue tunic character (16x16)
    assets.player = images.add(generate_player_texture());

    // Tile Textures (16x16)
    assets.forest_grass = images.add(generate_noise_texture(16, 16, Color::srgb(0.2, 0.55, 0.2), 0.15));
    assets.dirt = images.add(generate_noise_texture(16, 16, Color::srgb(0.47, 0.35, 0.22), 0.1));
    assets.water = images.add(generate_noise_texture(16, 16, Color::srgb(0.25, 0.5, 0.8), 0.1));
    assets.stone = images.add(generate_noise_texture(16, 16, Color::srgb(0.5, 0.5, 0.5), 0.2));
    assets.sand = images.add(generate_noise_texture(16, 16, Color::srgb(0.8, 0.75, 0.5), 0.05));

    // Trees (32x32)
    assets.oak_tree = images.add(generate_tree_texture(false));
    assets.pine_tree = images.add(generate_tree_texture(true));

    // World Objects (16x16)
    assets.rock = images.add(generate_rock_texture());
    assets.bush = images.add(generate_bush_texture(Color::srgb(0.20, 0.50, 0.15)));
    assets.berry_bush = images.add(generate_berry_bush_texture());
    assets.cactus = images.add(generate_cactus_texture());
    assets.mushroom = images.add(generate_mushroom_texture(12, 12));
    assets.giant_mushroom = images.add(generate_mushroom_texture(24, 28));
    assets.crystal = images.add(generate_crystal_texture());
    assets.iron_vein = images.add(generate_ore_texture([90, 70, 55], [130, 100, 75]));
    assets.supply_crate = images.add(generate_crate_texture());
    assets.dungeon_entrance = images.add(generate_dungeon_entrance_texture());

    // Enemy textures
    assets.enemy_wolf = images.add(generate_wolf_texture());
    assets.enemy_spider = images.add(generate_spider_texture());
    assets.enemy_crawler = images.add(generate_crawler_texture());
    assets.enemy_zombie = images.add(generate_zombie_texture());
    assets.enemy_elemental = images.add(generate_elemental_texture());
    assets.enemy_wraith = images.add(generate_wraith_texture());
    assets.enemy_scorpion = images.add(generate_scorpion_texture());
    assets.enemy_boss = images.add(generate_boss_texture());

    // Combat visuals
    assets.slash_arc = images.add(generate_slash_arc_texture());

    // Screen vignette
    assets.vignette = images.add(generate_vignette_texture());

    // Building textures
    assets.wood_wall = images.add(generate_plank_texture(16, 24, [120, 80, 45], true));
    assets.wood_floor = images.add(generate_plank_texture(16, 16, [140, 95, 55], false));
    assets.wood_door = images.add(generate_door_texture());
    assets.stone_wall = images.add(generate_brick_texture(16, 24, [110, 110, 115]));
    assets.campfire = images.add(generate_campfire_texture());
    assets.workbench = images.add(generate_workbench_texture());
    assets.forge = images.add(generate_forge_texture());
    assets.chest_building = images.add(generate_chest_building_texture());
    assets.bed = images.add(generate_bed_texture());

    // Flat normals for lit 2D sprites (z-up in RGB encoding: 0.5, 0.5, 1.0)
    assets.flat_normal_16 = images.add(generate_flat_normal(16, 16));
    assets.flat_normal_32 = images.add(generate_flat_normal(32, 32));
    assets.player_normal = images.add(generate_player_normal());
    assets.enemy_wolf_normal = images.add(generate_wolf_normal());
    assets.enemy_zombie_normal = images.add(generate_zombie_normal());
    assets.white_pixel = images.add(generate_white_pixel());

    commands.insert_resource(assets);
}

/// 1x1 white pixel for color-only sprites (tinted by material).
fn generate_white_pixel() -> Image {
    Image::new(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![255, 255, 255, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// Flat normal (0, 0, 1) for 2D lit sprites; RGB = (128, 128, 255).
fn generate_flat_normal(width: u32, height: u32) -> Image {
    let mut data = vec![0u8; (width * height * 4) as usize];
    for i in (0..data.len()).step_by(4) {
        data[i] = 128;
        data[i + 1] = 128;
        data[i + 2] = 255;
        data[i + 3] = 255;
    }
    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// Encodes normal (nx, ny, nz) to RGB: (0.5 + 0.5*nx, 0.5 + 0.5*ny, 0.5 + 0.5*nz), 255 alpha.
fn encode_normal(r: &mut [u8], i: usize, nx: f32, ny: f32, nz: f32) {
    let nz = nz.clamp(0.0, 1.0);
    r[i] = ((nx * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;
    r[i + 1] = ((ny * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;
    r[i + 2] = ((nz * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;
    r[i + 3] = 255;
}

/// Shaped normal for player (16x16): head sphere, body/legs mostly flat.
fn generate_player_normal() -> Image {
    let w = 16u32;
    let h = 16u32;
    let mut data = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let cx = x as f32 - 8.0;
            let cy = y as f32 - 8.0;
            let head_dy = cy + 4.0;
            let in_head = cx * cx + head_dy * head_dy < 11.0;
            let in_body = cx.abs() < 3.0 && cy > -1.0 && cy < 8.0;
            if in_head {
                let len = (cx * cx + head_dy * head_dy + 4.0).sqrt();
                let nx = cx / len;
                let ny = head_dy / len;
                let nz = 2.0 / len;
                encode_normal(&mut data, i, nx, ny, nz);
            } else if in_body {
                let tilt = cx * 0.04;
                let nz = (1.0_f32 - tilt * tilt).sqrt().max(0.3);
                encode_normal(&mut data, i, tilt, 0.0, nz);
            } else {
                encode_normal(&mut data, i, 0.0, 0.0, 1.0);
            }
        }
    }
    make_image(w, h, data)
}

/// Shaped normal for wolf (16x14): rounded body and head.
fn generate_wolf_normal() -> Image {
    let w = 16u32;
    let h = 14u32;
    let mut data = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let cx = x as f32;
            let cy = y as f32;
            let hx = cx - 12.0;
            let hy = cy - 4.0;
            let in_head = hx * hx + hy * hy < 11.0;
            let bx = cx - 8.0;
            let by = cy - 5.0;
            let in_body = (bx * bx / 25.0 + by * by / 9.0) < 1.0;
            if in_head {
                let len = (hx * hx + hy * hy + 3.0).sqrt();
                encode_normal(&mut data, i, hx / len, hy / len, 1.5_f32 / len);
            } else if in_body {
                let len = (bx * bx / 25.0 + by * by / 9.0 + 0.5).sqrt();
                let nx = (bx / 25.0) / len;
                let ny = (by / 9.0) / len;
                let nz = (0.5_f32 / len).max(0.4);
                encode_normal(&mut data, i, nx, ny, nz);
            } else {
                encode_normal(&mut data, i, 0.0, 0.0, 1.0);
            }
        }
    }
    make_image(w, h, data)
}

/// Shaped normal for zombie (14x18): rounded head, flat body.
fn generate_zombie_normal() -> Image {
    let w = 14u32;
    let h = 18u32;
    let mut data = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let cx = x as f32 - 7.0;
            let cy = y as f32 - 9.0;
            let in_head = cx * cx + (cy + 6.0) * (cy + 6.0) < 7.5;
            let in_body = cx.abs() < 3.5 && cy > -4.0 && cy < 4.0;
            if in_head {
                let len = (cx * cx + (cy + 6.0) * (cy + 6.0) + 4.0).sqrt();
                let nx = cx / len;
                let ny = (cy + 6.0) / len;
                let nz = 2.0 / len;
                encode_normal(&mut data, i, nx, ny, nz);
            } else if in_body {
                encode_normal(&mut data, i, 0.0, 0.0, 1.0);
            } else {
                encode_normal(&mut data, i, 0.0, 0.0, 1.0);
            }
        }
    }
    make_image(w, h, data)
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

    make_image(width, height, data)
}

fn make_image(width: u32, height: u32, data: Vec<u8>) -> Image {
    Image::new(
        Extent3d { width, height, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

fn generate_player_texture() -> Image {
    let w: u32 = 16;
    let h: u32 = 16;
    let mut data = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let cx = x as f32 - 8.0;
            let cy = y as f32 - 8.0;

            // Head (top circle) — slightly tighter for clearer silhouette
            let head_dy = cy + 4.0;
            let in_head = cx * cx + head_dy * head_dy < 11.0;
            let head_edge = cx * cx + head_dy * head_dy < 12.5 && !in_head;

            // Body (rectangle with slight taper)
            let in_body = cx.abs() < 3.0 && cy > -1.0 && cy < 5.0;
            let body_edge = cx.abs() < 3.5 && cy > -1.5 && cy < 5.5 && !in_body && !in_head;

            // Legs
            let in_legs = (cx.abs() > 0.5 && cx.abs() < 2.5) && cy >= 5.0 && cy < 8.0;
            let leg_edge = (cx.abs() < 3.0 && cy >= 4.5 && cy < 8.5) && !in_legs && !in_body;

            let n = ((simple_hash(x, y, 11) % 12) as i32 - 6) as f32;

            if head_edge || body_edge || leg_edge {
                // Dark outline for readable silhouette
                data[i] = 15;
                data[i + 1] = 12;
                data[i + 2] = 22;
                data[i + 3] = 255;
            } else if in_head {
                // Skin tone (deterministic variation)
                data[i] = (200.0 + n).clamp(0.0, 255.0) as u8;
                data[i + 1] = (160.0 + n).clamp(0.0, 255.0) as u8;
                data[i + 2] = (120.0 + n).clamp(0.0, 255.0) as u8;
                data[i + 3] = 255;
                // Eyes
                if (cx - 1.5).abs() < 0.8 && (head_dy - 0.5).abs() < 0.8 {
                    data[i] = 20; data[i + 1] = 20; data[i + 2] = 40; data[i + 3] = 255;
                }
                if (cx + 1.5).abs() < 0.8 && (head_dy - 0.5).abs() < 0.8 {
                    data[i] = 20; data[i + 1] = 20; data[i + 2] = 40; data[i + 3] = 255;
                }
                // Nose highlight
                if cx.abs() < 0.5 && head_dy > 0.5 && head_dy < 1.5 {
                    data[i] = (data[i] as f32 * 0.9) as u8;
                    data[i + 1] = (data[i + 1] as f32 * 0.9) as u8;
                    data[i + 2] = (data[i + 2] as f32 * 0.85) as u8;
                }
            } else if in_body {
                // Blue tunic with subtle highlight (top-left light)
                let highlight = if cx < 0.0 && cy > 2.0 { 18 } else { 0 };
                data[i] = (40.0 + n + highlight as f32).clamp(0.0, 255.0) as u8;
                data[i + 1] = (80.0 + n + highlight as f32 * 0.8).clamp(0.0, 255.0) as u8;
                data[i + 2] = (180.0 + n + highlight as f32 * 0.5).clamp(0.0, 255.0) as u8;
                data[i + 3] = 255;
            } else if in_legs {
                // Brown pants
                data[i] = (80.0 + n).clamp(0.0, 255.0) as u8;
                data[i + 1] = (55.0 + n).clamp(0.0, 255.0) as u8;
                data[i + 2] = (35.0 + n).clamp(0.0, 255.0) as u8;
                data[i + 3] = 255;
            }
        }
    }

    make_image(w, h, data)
}

fn generate_tree_texture(is_pine: bool) -> Image {
    let width: u32 = 32;
    let height: u32 = 32;
    let mut data = vec![0u8; (width * height * 4) as usize];

    for y in 0..height {
        for x in 0..width {
            let index = ((y * width + x) * 4) as usize;
            let n = ((simple_hash(x, y, if is_pine { 31 } else { 30 }) % 18) as i32 - 9) as f32;

            let is_trunk = x >= 14 && x <= 18 && y >= 20;
            // Bark lines (vertical streaks)
            let bark_line = is_trunk && (x == 15 || x == 17) && (simple_hash(x, y, 60) % 3 == 0);
            let bark_dark = is_trunk && (simple_hash(x, y, 61) % 5 == 0);

            let is_leaves = if is_pine {
                let row_width = (y as f32 * 0.75) as i32;
                y < 24 && (x as i32 - 16).abs() < row_width.max(2)
            } else {
                let dx = x as f32 - 16.0;
                let dy = y as f32 - 12.0;
                (dx * dx + dy * dy).sqrt() < 10.5
            };
            // Leaf edge for outline
            let leaf_edge = if is_pine {
                let row_width = (y as f32 * 0.75) as i32;
                y < 24 && (x as i32 - 16).abs() == row_width.max(2)
            } else {
                let dx = x as f32 - 16.0;
                let dy = y as f32 - 12.0;
                let d = (dx * dx + dy * dy).sqrt();
                d >= 9.5 && d < 11.0
            };

            if is_trunk {
                if bark_line {
                    data[index] = (70 + n as i32).clamp(0, 255) as u8;
                    data[index + 1] = (48 + n as i32).clamp(0, 255) as u8;
                    data[index + 2] = (28 + n as i32).clamp(0, 255) as u8;
                } else if bark_dark {
                    data[index] = (85 + n as i32).clamp(0, 255) as u8;
                    data[index + 1] = (58 + n as i32).clamp(0, 255) as u8;
                    data[index + 2] = (32 + n as i32).clamp(0, 255) as u8;
                } else {
                    data[index] = (100 + n as i32).clamp(0, 255) as u8;
                    data[index + 1] = (70 + n as i32).clamp(0, 255) as u8;
                    data[index + 2] = (40 + n as i32).clamp(0, 255) as u8;
                }
                data[index + 3] = 255;
            } else if leaf_edge {
                data[index] = 12;
                data[index + 1] = 22;
                data[index + 2] = 10;
                data[index + 3] = 255;
            } else if is_leaves {
                let (r, g, b) = if is_pine {
                    (30.0 + n, 80.0 + n, 40.0 + n)
                } else {
                    (50.0 + n, 120.0 + n, 40.0 + n)
                };
                // Highlight on upper-left quadrant for depth
                let highlight = if !is_pine && (x as f32 - 16.0) < 0.0 && (y as f32 - 12.0) < 0.0 {
                    (simple_hash(x, y, 62) % 4) as f32 * 8.0
                } else if is_pine && y < 8 {
                    (simple_hash(x, y, 62) % 3) as f32 * 6.0
                } else {
                    0.0
                };
                data[index] = (r + highlight).clamp(0.0, 255.0) as u8;
                data[index + 1] = (g + highlight * 0.9).clamp(0.0, 255.0) as u8;
                data[index + 2] = (b + highlight * 0.5).clamp(0.0, 255.0) as u8;
                data[index + 3] = 255;
            }
        }
    }

    make_image(width, height, data)
}

fn generate_rock_texture() -> Image {
    let w: u32 = 16;
    let h: u32 = 16;
    let mut data = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let cx = x as f32 - 8.0;
            let cy = y as f32 - 9.0;
            // Slightly squashed ellipse
            let dist = (cx * cx / 49.0 + cy * cy / 36.0).sqrt();
            if dist < 1.0 {
                let n = rand::random::<f32>() * 30.0;
                let shade = 100.0 + n + (1.0 - dist) * 30.0; // lighter center
                data[i] = shade.clamp(0.0, 255.0) as u8;
                data[i + 1] = shade.clamp(0.0, 255.0) as u8;
                data[i + 2] = (shade + 5.0).clamp(0.0, 255.0) as u8;
                data[i + 3] = 255;
            }
        }
    }

    make_image(w, h, data)
}

fn generate_bush_texture(base: Color) -> Image {
    let w: u32 = 16;
    let h: u32 = 14;
    let mut data = vec![0u8; (w * h * 4) as usize];
    let c = base.to_srgba();

    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let cx = x as f32 - 8.0;
            let cy = y as f32 - 7.0;
            let dist = (cx * cx / 50.0 + cy * cy / 36.0).sqrt();
            if dist < 1.0 {
                let n = (rand::random::<f32>() - 0.5) * 0.08;
                data[i] = ((c.red + n).clamp(0.0, 1.0) * 255.0) as u8;
                data[i + 1] = ((c.green + n).clamp(0.0, 1.0) * 255.0) as u8;
                data[i + 2] = ((c.blue + n).clamp(0.0, 1.0) * 255.0) as u8;
                data[i + 3] = 255;
            }
        }
    }

    make_image(w, h, data)
}

fn generate_berry_bush_texture() -> Image {
    let w: u32 = 16;
    let h: u32 = 14;
    let mut data = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let cx = x as f32 - 8.0;
            let cy = y as f32 - 7.0;
            let dist = (cx * cx / 50.0 + cy * cy / 36.0).sqrt();
            if dist < 1.0 {
                let n = (rand::random::<f32>() - 0.5) * 15.0;
                // Green bush base
                data[i] = (50.0 + n).clamp(0.0, 255.0) as u8;
                data[i + 1] = (110.0 + n).clamp(0.0, 255.0) as u8;
                data[i + 2] = (40.0 + n).clamp(0.0, 255.0) as u8;
                data[i + 3] = 255;

                // Scatter red berries
                let bh = simple_hash(x, y, 42);
                if bh % 7 == 0 {
                    data[i] = 200;
                    data[i + 1] = 40;
                    data[i + 2] = 50;
                }
            }
        }
    }

    make_image(w, h, data)
}

fn generate_cactus_texture() -> Image {
    let w: u32 = 12;
    let h: u32 = 20;
    let mut data = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let cx = (x as f32 - 6.0).abs();

            // Main trunk
            let in_trunk = cx < 2.0 && y > 3;
            // Arms
            let in_left_arm = x >= 1 && x <= 4 && y >= 8 && y <= 12;
            let in_right_arm = x >= 8 && x <= 11 && y >= 6 && y <= 10;

            if in_trunk || in_left_arm || in_right_arm {
                let n = rand::random::<f32>() * 15.0;
                data[i] = (60.0 + n) as u8;
                data[i + 1] = (130.0 + n) as u8;
                data[i + 2] = (50.0 + n) as u8;
                data[i + 3] = 255;

                // Spines
                let sh = simple_hash(x, y, 77);
                if sh % 5 == 0 {
                    data[i] = 200;
                    data[i + 1] = 200;
                    data[i + 2] = 180;
                }
            }
        }
    }

    make_image(w, h, data)
}

fn generate_mushroom_texture(w: u32, h: u32) -> Image {
    let mut data = vec![0u8; (w * h * 4) as usize];
    let cap_cy = h as f32 * 0.3;
    let cap_r = w as f32 * 0.4;

    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let cx = x as f32 - w as f32 / 2.0;
            let cy = y as f32 - cap_cy;

            // Stem
            let stem_half = w as f32 * 0.12;
            let in_stem = cx.abs() < stem_half && y as f32 > cap_cy && y < h;

            // Cap (top half of ellipse)
            let in_cap = (cx * cx / (cap_r * cap_r) + cy * cy / (cap_r * 0.6 * cap_r * 0.6)) < 1.0
                && y as f32 <= cap_cy + 2.0;

            if in_cap {
                let n = rand::random::<f32>() * 20.0;
                data[i] = (160.0 + n) as u8;
                data[i + 1] = (50.0 + n) as u8;
                data[i + 2] = (40.0 + n) as u8;
                data[i + 3] = 255;

                // Spots
                let sh = simple_hash(x, y, 33);
                if sh % 8 == 0 {
                    data[i] = 230;
                    data[i + 1] = 220;
                    data[i + 2] = 200;
                }
            } else if in_stem {
                let n = rand::random::<f32>() * 10.0;
                data[i] = (200.0 + n) as u8;
                data[i + 1] = (190.0 + n) as u8;
                data[i + 2] = (170.0 + n) as u8;
                data[i + 3] = 255;
            }
        }
    }

    make_image(w, h, data)
}

fn generate_crystal_texture() -> Image {
    let w: u32 = 14;
    let h: u32 = 18;
    let mut data = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let cx = x as f32 - 7.0;
            // Diamond / crystal shape: narrower at top and bottom
            let t = y as f32 / h as f32;
            let half_width = if t < 0.5 {
                t * 8.0
            } else {
                (1.0 - t) * 8.0
            };

            if cx.abs() < half_width {
                let n = rand::random::<f32>() * 20.0;
                let brightness = 0.5 + (1.0 - (cx.abs() / half_width)) * 0.4;
                data[i] = (100.0 * brightness + n) as u8;
                data[i + 1] = (80.0 * brightness + n) as u8;
                data[i + 2] = (180.0 * brightness + n) as u8;
                data[i + 3] = 255;
            }
        }
    }

    make_image(w, h, data)
}

fn generate_ore_texture(dark: [u8; 3], light: [u8; 3]) -> Image {
    let w: u32 = 16;
    let h: u32 = 12;
    let mut data = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let cx = x as f32 - 8.0;
            let cy = y as f32 - 6.0;
            let dist = (cx * cx / 50.0 + cy * cy / 25.0).sqrt();
            if dist < 1.0 {
                let n = rand::random::<f32>();
                // Mix dark and light based on noise
                let r = dark[0] as f32 + (light[0] as f32 - dark[0] as f32) * n;
                let g = dark[1] as f32 + (light[1] as f32 - dark[1] as f32) * n;
                let b = dark[2] as f32 + (light[2] as f32 - dark[2] as f32) * n;
                data[i] = r as u8;
                data[i + 1] = g as u8;
                data[i + 2] = b as u8;
                data[i + 3] = 255;

                // Ore glint spots
                let sh = simple_hash(x, y, 99);
                if sh % 11 == 0 {
                    data[i] = (data[i] as f32 * 1.4).min(255.0) as u8;
                    data[i + 1] = (data[i + 1] as f32 * 1.3).min(255.0) as u8;
                    data[i + 2] = (data[i + 2] as f32 * 1.2).min(255.0) as u8;
                }
            }
        }
    }

    make_image(w, h, data)
}

fn generate_crate_texture() -> Image {
    let w: u32 = 12;
    let h: u32 = 10;
    let mut data = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            // Simple box with wood planks
            if x > 0 && x < w - 1 && y > 0 && y < h - 1 {
                let n = rand::random::<f32>() * 15.0;
                // Cross plank pattern
                let is_plank = x == w / 2 || y == h / 2;
                if is_plank {
                    data[i] = (100.0 + n) as u8;
                    data[i + 1] = (70.0 + n) as u8;
                    data[i + 2] = (35.0 + n) as u8;
                } else {
                    data[i] = (130.0 + n) as u8;
                    data[i + 1] = (95.0 + n) as u8;
                    data[i + 2] = (50.0 + n) as u8;
                }
                data[i + 3] = 255;
            } else {
                // Border
                data[i] = 70;
                data[i + 1] = 50;
                data[i + 2] = 25;
                data[i + 3] = 255;
            }
        }
    }

    make_image(w, h, data)
}

fn generate_dungeon_entrance_texture() -> Image {
    let w: u32 = 20;
    let h: u32 = 20;
    let mut data = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let cx = x as f32 - 10.0;
            let cy = y as f32 - 10.0;
            let dist = (cx * cx + cy * cy).sqrt();

            if dist < 9.0 {
                if dist < 5.0 {
                    // Dark pit center
                    let d = (dist / 5.0) * 20.0;
                    data[i] = d as u8;
                    data[i + 1] = d as u8;
                    data[i + 2] = (d + 5.0) as u8;
                    data[i + 3] = 255;
                } else {
                    // Stone rim
                    let n = rand::random::<f32>() * 20.0;
                    data[i] = (80.0 + n) as u8;
                    data[i + 1] = (75.0 + n) as u8;
                    data[i + 2] = (85.0 + n) as u8;
                    data[i + 3] = 255;
                }
            }
        }
    }

    make_image(w, h, data)
}

// ============================================================
// Building Textures
// ============================================================

/// Wood planks — horizontal lines with grain noise
fn generate_plank_texture(w: u32, h: u32, base: [u8; 3], vertical_grain: bool) -> Image {
    let mut data = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let n = (simple_hash(x, y, 12) % 20) as i32 - 10;
            // Plank lines every 4-5 pixels
            let plank_line = if vertical_grain { x % 5 == 0 } else { y % 5 == 0 };
            let darken: i32 = if plank_line { -20 } else { 0 };
            data[i] = (base[0] as i32 + n + darken).clamp(0, 255) as u8;
            data[i+1] = (base[1] as i32 + n + darken).clamp(0, 255) as u8;
            data[i+2] = (base[2] as i32 + n + darken).clamp(0, 255) as u8;
            data[i+3] = 255;
        }
    }
    make_image(w, h, data)
}

/// Stone/brick texture — grid pattern with noise
fn generate_brick_texture(w: u32, h: u32, base: [u8; 3]) -> Image {
    let mut data = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let n = (simple_hash(x, y, 22) % 25) as i32 - 12;
            // Mortar lines
            let row = y / 4;
            let offset = if row % 2 == 0 { 0 } else { 3 };
            let is_mortar = y % 4 == 0 || (x + offset) % 6 == 0;
            if is_mortar {
                data[i] = (base[0] as i32 - 30 + n).clamp(0, 255) as u8;
                data[i+1] = (base[1] as i32 - 30 + n).clamp(0, 255) as u8;
                data[i+2] = (base[2] as i32 - 25 + n).clamp(0, 255) as u8;
            } else {
                data[i] = (base[0] as i32 + n).clamp(0, 255) as u8;
                data[i+1] = (base[1] as i32 + n).clamp(0, 255) as u8;
                data[i+2] = (base[2] as i32 + n).clamp(0, 255) as u8;
            }
            data[i+3] = 255;
        }
    }
    make_image(w, h, data)
}

fn generate_door_texture() -> Image {
    let w: u32 = 10; let h: u32 = 20;
    let mut data = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let n = (simple_hash(x, y, 32) % 15) as i32 - 7;
            // Door frame (border)
            let is_frame = x == 0 || x == w-1 || y == 0 || y == h-1;
            // Handle
            let is_handle = x == 7 && (y == 10 || y == 11);
            if is_handle {
                data[i] = 180; data[i+1] = 160; data[i+2] = 60; data[i+3] = 255;
            } else if is_frame {
                data[i] = (90 + n).clamp(0, 255) as u8;
                data[i+1] = (60 + n).clamp(0, 255) as u8;
                data[i+2] = (30 + n).clamp(0, 255) as u8;
                data[i+3] = 255;
            } else {
                // Wood panels with horizontal grain
                let plank_dark = if y % 6 == 0 { -15i32 } else { 0 };
                data[i] = (130 + n + plank_dark).clamp(0, 255) as u8;
                data[i+1] = (85 + n + plank_dark).clamp(0, 255) as u8;
                data[i+2] = (45 + n + plank_dark).clamp(0, 255) as u8;
                data[i+3] = 255;
            }
        }
    }
    make_image(w, h, data)
}

fn generate_campfire_texture() -> Image {
    let w: u32 = 12; let h: u32 = 12;
    let mut data = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let cx = x as f32 - 6.0; let cy = y as f32 - 6.0;
            let in_logs = (cy > 2.0 && cx.abs() < 4.0 && cy < 5.0) ||
                          (cy > 1.0 && cy < 6.0 && cx.abs() < 1.5);
            let flame_r = 3.5 - cy * 0.4;
            let in_flame = cx.abs() < flame_r.max(0.0) && cy < 3.0 && cy > -4.0;
            let log_end = in_logs && (cy > 3.5 && cy < 4.5 && (cx - 3.0).abs() < 1.0 || (cx + 3.0).abs() < 1.0);

            if in_flame {
                let t = ((cy + 4.0) / 7.0).clamp(0.0, 1.0);
                let n = (simple_hash(x, y, 55) % 30) as f32;
                if t < 0.25 {
                    data[i] = 255; data[i+1] = 252; data[i+2] = 220;
                } else if t < 0.35 {
                    data[i] = 255; data[i+1] = (240.0 + n * 0.5) as u8; data[i+2] = (180.0 + n) as u8;
                } else if t < 0.6 {
                    data[i] = 255; data[i+1] = (150.0 + n) as u8; data[i+2] = (30.0 + n) as u8;
                } else {
                    data[i] = (200.0 + n) as u8; data[i+1] = (50.0 + n) as u8; data[i+2] = 20;
                }
                data[i+3] = 255;
            } else if log_end {
                let n = (simple_hash(x, y, 44) % 12) as i32 - 6;
                data[i] = (65 + n).clamp(0, 255) as u8;
                data[i+1] = (40 + n).clamp(0, 255) as u8;
                data[i+2] = (18 + n).clamp(0, 255) as u8;
                data[i+3] = 255;
            } else if in_logs {
                let n = (simple_hash(x, y, 44) % 15) as i32 - 7;
                data[i] = (80 + n).clamp(0, 255) as u8;
                data[i+1] = (50 + n).clamp(0, 255) as u8;
                data[i+2] = (25 + n).clamp(0, 255) as u8;
                data[i+3] = 255;
            }
            let stone_dist = (cx*cx + (cy - 3.0) * (cy - 3.0)).sqrt();
            if stone_dist > 4.0 && stone_dist < 5.5 && cy > 0.0 {
                let n = (simple_hash(x, y, 66) % 20) as u8;
                data[i] = 100 + n; data[i+1] = 95 + n; data[i+2] = 90 + n; data[i+3] = 255;
            }
        }
    }
    make_image(w, h, data)
}

fn generate_workbench_texture() -> Image {
    let w: u32 = 16; let h: u32 = 16;
    let mut data = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let n = (simple_hash(x, y, 77) % 15) as i32 - 7;
            // Table top (upper 60%)
            if y < 10 {
                let plank = if y % 4 == 0 { -15i32 } else { 0 };
                data[i] = (110 + n + plank).clamp(0, 255) as u8;
                data[i+1] = (75 + n + plank).clamp(0, 255) as u8;
                data[i+2] = (40 + n + plank).clamp(0, 255) as u8;
                data[i+3] = 255;
                // Tool marks
                if simple_hash(x, y, 88) % 12 == 0 {
                    data[i] = (data[i] as i32 - 15).clamp(0, 255) as u8;
                    data[i+1] = (data[i+1] as i32 - 10).clamp(0, 255) as u8;
                }
            }
            // Legs
            else if (x >= 1 && x <= 3) || (x >= 12 && x <= 14) {
                data[i] = (90 + n).clamp(0, 255) as u8;
                data[i+1] = (60 + n).clamp(0, 255) as u8;
                data[i+2] = (30 + n).clamp(0, 255) as u8;
                data[i+3] = 255;
            }
        }
    }
    make_image(w, h, data)
}

fn generate_forge_texture() -> Image {
    let w: u32 = 16; let h: u32 = 16;
    let mut data = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let n = (simple_hash(x, y, 99) % 20) as i32 - 10;
            let cx = x as f32 - 8.0; let cy = y as f32 - 8.0;
            let in_stone = cx.abs() < 6.0 && cy > -4.0;
            let in_fire = cx.abs() < 3.0 && cy > 0.0 && cy < 4.0;
            let in_chimney = cx.abs() < 2.0 && cy < -4.0 && cy > -7.0;
            let chimney_highlight = in_chimney && (cx < 0.0 && cy > -5.5);

            if in_stone && !in_fire {
                data[i] = (80 + n).clamp(0, 255) as u8;
                data[i+1] = (75 + n).clamp(0, 255) as u8;
                data[i+2] = (80 + n).clamp(0, 255) as u8;
                data[i+3] = 255;
            }
            if in_fire {
                let t = (cy / 4.0).clamp(0.0, 1.0);
                let core = (1.0 - (cx.abs() / 3.0) * 0.3) * (1.0 - t * 0.5);
                data[i] = (200.0 + n as f32 + t * 55.0 + core * 55.0).clamp(0.0, 255.0) as u8;
                data[i+1] = (100.0 + n as f32 * 0.5 + core * 80.0).clamp(0.0, 255.0) as u8;
                data[i+2] = (30.0 + core * 60.0).clamp(0.0, 255.0) as u8;
                data[i+3] = 255;
            }
            if in_chimney {
                if chimney_highlight {
                    data[i] = (95 + n).clamp(0, 255) as u8;
                    data[i+1] = (90 + n).clamp(0, 255) as u8;
                    data[i+2] = (95 + n).clamp(0, 255) as u8;
                } else {
                    data[i] = (70 + n).clamp(0, 255) as u8;
                    data[i+1] = (65 + n).clamp(0, 255) as u8;
                    data[i+2] = (70 + n).clamp(0, 255) as u8;
                }
                data[i+3] = 255;
            }
        }
    }
    make_image(w, h, data)
}

fn generate_chest_building_texture() -> Image {
    let w: u32 = 12; let h: u32 = 12;
    let mut data = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            if x >= 1 && x < w-1 && y >= 2 && y < h-1 {
                let n = (simple_hash(x, y, 11) % 15) as i32 - 7;
                // Chest body
                let is_lid = y < 5;
                let lid_dark = if is_lid { 10i32 } else { 0 };
                data[i] = (135 + n - lid_dark).clamp(0, 255) as u8;
                data[i+1] = (95 + n - lid_dark).clamp(0, 255) as u8;
                data[i+2] = (50 + n - lid_dark).clamp(0, 255) as u8;
                data[i+3] = 255;
                // Metal clasp
                if x == 5 && (y == 5 || y == 6) {
                    data[i] = 180; data[i+1] = 170; data[i+2] = 80;
                }
                // Metal bands
                if y == 4 || y == 8 {
                    data[i] = (data[i] as i32 - 20).clamp(0, 255) as u8;
                    data[i+1] = (data[i+1] as i32 - 15).clamp(0, 255) as u8;
                }
            }
            // Border
            else if x == 0 || x == w-1 || y == h-1 {
                data[i] = 60; data[i+1] = 40; data[i+2] = 20; data[i+3] = 255;
            }
        }
    }
    make_image(w, h, data)
}

fn generate_bed_texture() -> Image {
    let w: u32 = 16; let h: u32 = 16;
    let mut data = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let n = (simple_hash(x, y, 33) % 12) as i32 - 6;
            // Bed frame (wood border)
            let is_frame = x == 0 || x == w-1 || y == 0 || y == h-1;
            // Pillow (top area)
            let is_pillow = x >= 3 && x <= 12 && y >= 2 && y <= 5;
            // Blanket
            let is_blanket = x >= 1 && x <= 14 && y >= 6 && y <= 14;

            if is_pillow {
                data[i] = (220 + n).clamp(0, 255) as u8;
                data[i+1] = (215 + n).clamp(0, 255) as u8;
                data[i+2] = (200 + n).clamp(0, 255) as u8;
                data[i+3] = 255;
            } else if is_blanket {
                data[i] = (140 + n).clamp(0, 255) as u8;
                data[i+1] = (55 + n).clamp(0, 255) as u8;
                data[i+2] = (55 + n).clamp(0, 255) as u8;
                data[i+3] = 255;
            } else if is_frame {
                data[i] = (90 + n).clamp(0, 255) as u8;
                data[i+1] = (60 + n).clamp(0, 255) as u8;
                data[i+2] = (30 + n).clamp(0, 255) as u8;
                data[i+3] = 255;
            }
        }
    }
    make_image(w, h, data)
}

/// Simple deterministic hash for pixel patterns.
fn simple_hash(x: u32, y: u32, seed: u32) -> u32 {
    let mut h = seed;
    h = h.wrapping_mul(374761393);
    h = h.wrapping_add(x).wrapping_mul(668265263);
    h = h.wrapping_add(y).wrapping_mul(2654435761);
    h ^= h >> 13;
    h
}

// ============================================================
// Enemy Textures
// ============================================================

fn set_pixel(data: &mut [u8], w: u32, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
    let i = ((y * w + x) * 4) as usize;
    if i + 3 < data.len() {
        data[i] = r; data[i+1] = g; data[i+2] = b; data[i+3] = a;
    }
}

fn set_pixel_noisy(data: &mut [u8], w: u32, x: u32, y: u32, r: u8, g: u8, b: u8) {
    let n = ((simple_hash(x, y, 55) % 20) as i32 - 10) as f32;
    set_pixel(data, w, x, y,
        (r as f32 + n).clamp(0.0, 255.0) as u8,
        (g as f32 + n).clamp(0.0, 255.0) as u8,
        (b as f32 + n).clamp(0.0, 255.0) as u8,
        255);
}

/// Wolf: four-legged quadruped silhouette (16x14) with outline and fur detail
fn generate_wolf_texture() -> Image {
    let w: u32 = 16; let h: u32 = 14;
    let mut data = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let cx = x as f32; let cy = y as f32;
            let hx = cx - 12.0; let hy = cy - 4.0;
            let in_head = hx*hx + hy*hy < 11.0;
            let head_edge = hx*hx + hy*hy < 12.5 && !in_head;
            let bx = cx - 8.0; let by = cy - 5.0;
            let in_body = (bx*bx / 25.0 + by*by / 9.0) < 1.0;
            let body_edge = (bx*bx / 28.0 + by*by / 10.0) < 1.0 && !in_body && !in_head;
            let in_legs = (cy >= 9.0 && cy <= 13.0) &&
                ((cx >= 3.0 && cx <= 5.0) || (cx >= 6.0 && cx <= 8.0) ||
                 (cx >= 10.0 && cx <= 12.0) || (cx >= 13.0 && cx <= 15.0));
            let in_tail = cx <= 3.0 && cy >= 3.0 && cy <= 5.0;
            let tail_edge = cx <= 3.5 && cy >= 2.5 && cy <= 5.5 && !in_tail && !in_body;

            if head_edge || body_edge || tail_edge {
                set_pixel(&mut data, w, x, y, 25, 22, 30, 255);
            } else if in_head || in_body || in_legs || in_tail {
                set_pixel_noisy(&mut data, w, x, y, 120, 115, 110);
                // Fur highlight on snout/ear
                if in_head && (hx > 2.0 || hy < -1.5) {
                    let light = (simple_hash(x, y, 70) % 12) as u8;
                    set_pixel(&mut data, w, x, y,
                        (120 + light).min(255),
                        (115 + light).min(255),
                        (110 + light).min(255),
                        255);
                }
                if in_head && ((cx - 13.0).abs() < 0.8 && (cy - 3.0).abs() < 0.8) {
                    set_pixel(&mut data, w, x, y, 220, 180, 50, 255);
                }
                if in_head && ((cx - 11.0).abs() < 0.8 && (cy - 3.0).abs() < 0.8) {
                    set_pixel(&mut data, w, x, y, 220, 180, 50, 255);
                }
            }
        }
    }
    make_image(w, h, data)
}

/// Spider: body with 8 legs radiating out (12x12)
fn generate_spider_texture() -> Image {
    let w: u32 = 12; let h: u32 = 12;
    let mut data = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let cx = x as f32 - 6.0; let cy = y as f32 - 6.0;
            // Body: small circle
            let in_body = cx*cx + cy*cy < 6.0;
            // Head: smaller circle in front
            let hx = cx - 2.5; let hy = cy;
            let in_head = hx*hx + hy*hy < 3.0;
            // Legs: 8 diagonal lines
            let leg_hit = {
                let ax = cx.abs(); let ay = cy.abs();
                let d1 = (ax - ay).abs(); // 45 degree
                let d2 = (ax - ay * 0.4).abs(); // shallow
                let d3 = (ax * 0.4 - ay).abs(); // steep
                (d1 < 0.8 || d2 < 0.8 || d3 < 0.8) && ax + ay > 2.0 && ax + ay < 6.5
            };

            if in_body || in_head {
                set_pixel_noisy(&mut data, w, x, y, 70, 50, 40);
                // Eyes: two red dots
                if (cx - 3.5).abs() < 0.7 && (cy - 0.7).abs() < 0.7 {
                    set_pixel(&mut data, w, x, y, 200, 30, 30, 255);
                }
                if (cx - 3.5).abs() < 0.7 && (cy + 0.7).abs() < 0.7 {
                    set_pixel(&mut data, w, x, y, 200, 30, 30, 255);
                }
            } else if leg_hit {
                set_pixel_noisy(&mut data, w, x, y, 60, 40, 30);
            }
        }
    }
    make_image(w, h, data)
}

/// Shadow crawler: low hunched creature with glowing eyes (14x12)
fn generate_crawler_texture() -> Image {
    let w: u32 = 14; let h: u32 = 12;
    let mut data = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let cx = x as f32 - 7.0; let cy = y as f32 - 7.0;
            // Low, wide body ellipse
            let in_body = (cx*cx / 36.0 + cy*cy / 12.0) < 1.0 && cy > -2.0;
            // Two small feet
            let in_feet = cy >= 4.0 && ((cx + 3.0).abs() < 1.5 || (cx - 3.0).abs() < 1.5);

            if in_body || in_feet {
                set_pixel_noisy(&mut data, w, x, y, 80, 30, 100);
                // Glowing purple eyes
                if cy > -2.0 && cy < 0.0 {
                    if (cx - 2.0).abs() < 1.0 { set_pixel(&mut data, w, x, y, 200, 100, 255, 255); }
                    if (cx + 2.0).abs() < 1.0 { set_pixel(&mut data, w, x, y, 200, 100, 255, 255); }
                }
            }
        }
    }
    make_image(w, h, data)
}

/// Fungal zombie: shambling humanoid shape, greenish (14x18) with outline and tattered detail
fn generate_zombie_texture() -> Image {
    let w: u32 = 14; let h: u32 = 18;
    let mut data = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let cx = x as f32 - 7.0; let cy = y as f32 - 9.0;
            let in_head = cx*cx + (cy + 6.0) * (cy + 6.0) < 7.5;
            let head_edge = cx*cx + (cy + 6.0) * (cy + 6.0) < 9.0 && !in_head;
            let in_body = cx.abs() < 3.0 && cy > -4.0 && cy < 3.0;
            let body_edge = cx.abs() < 3.5 && cy > -4.5 && cy < 3.5 && !in_body && !in_head;
            let in_larm = (cx + 3.0 + cy * 0.3).abs() < 1.2 && cy > -3.0 && cy < 2.0 && cx < -2.0;
            let in_rarm = (cx - 3.0 - cy * 0.3).abs() < 1.2 && cy > -3.0 && cy < 2.0 && cx > 2.0;
            let in_legs = cy >= 3.0 && ((cx + 1.5).abs() < 1.5 || (cx - 1.5).abs() < 1.5);
            let leg_edge = cy >= 2.5 && cy < 4.0 && (cx + 2.0).abs() < 2.0 && !in_legs && !in_body;

            if head_edge || body_edge || leg_edge {
                set_pixel(&mut data, w, x, y, 18, 28, 15, 255);
            } else if in_head || in_body || in_larm || in_rarm || in_legs {
                set_pixel_noisy(&mut data, w, x, y, 70, 110, 55);
                let sh = simple_hash(x, y, 88);
                if sh % 9 == 0 && in_body { set_pixel(&mut data, w, x, y, 120, 60, 90, 255); }
                // Tattered cloth hint (darker patches)
                if (in_body || in_larm || in_rarm) && sh % 11 == 1 {
                    set_pixel(&mut data, w, x, y, 55, 90, 45, 255);
                }
                if in_head && (cy + 6.0).abs() < 0.8 {
                    if (cx - 1.2).abs() < 0.7 { set_pixel(&mut data, w, x, y, 180, 200, 50, 255); }
                    if (cx + 1.2).abs() < 0.7 { set_pixel(&mut data, w, x, y, 180, 200, 50, 255); }
                }
            }
        }
    }
    make_image(w, h, data)
}

/// Lava elemental: fiery blob with bright core (16x16)
fn generate_elemental_texture() -> Image {
    let w: u32 = 16; let h: u32 = 16;
    let mut data = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let cx = x as f32 - 8.0; let cy = y as f32 - 8.0;
            let dist = (cx*cx + cy*cy).sqrt();
            // Irregular blob: use hash to wobble the radius
            let angle_hash = simple_hash(x, y, 44) % 100;
            let wobble = 1.0 + (angle_hash as f32 / 100.0 - 0.5) * 0.3;
            let r = 6.5 * wobble;

            if dist < r {
                let t = dist / r;
                if t < 0.35 {
                    // Bright core: white-yellow
                    set_pixel(&mut data, w, x, y, 255, 240, 180, 255);
                } else if t < 0.65 {
                    // Orange mid
                    let n = rand::random::<f32>() * 20.0;
                    set_pixel(&mut data, w, x, y, (240.0 + n) as u8, (140.0 + n) as u8, 40, 255);
                } else {
                    // Dark red edge
                    let n = rand::random::<f32>() * 15.0;
                    set_pixel(&mut data, w, x, y, (180.0 + n) as u8, (50.0 + n) as u8, 20, 255);
                }
                // Eyes
                if (cy + 1.0).abs() < 1.0 {
                    if (cx - 2.0).abs() < 0.8 { set_pixel(&mut data, w, x, y, 20, 10, 5, 255); }
                    if (cx + 2.0).abs() < 0.8 { set_pixel(&mut data, w, x, y, 20, 10, 5, 255); }
                }
            }
        }
    }
    make_image(w, h, data)
}

/// Ice wraith: ghostly floating figure (12x16)
fn generate_wraith_texture() -> Image {
    let w: u32 = 12; let h: u32 = 16;
    let mut data = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let cx = x as f32 - 6.0; let cy = y as f32 - 6.0;
            // Head (top circle)
            let in_head = cx*cx + (cy + 3.0) * (cy + 3.0) < 8.0;
            // Wispy body: wider at bottom, tapers at top
            let t = (y as f32 / h as f32).clamp(0.0, 1.0);
            let body_half = 1.5 + t * 3.5;
            let in_body = cx.abs() < body_half && cy > -1.0;
            // Ragged bottom edge
            let ragged = cy > 6.0 && simple_hash(x, y, 77) % 3 == 0;

            if (in_head || in_body) && !ragged {
                // Semi-transparent icy blue
                let alpha = if in_head { 220 } else { (160.0 - cy * 8.0).clamp(80.0, 200.0) as u8 };
                let n = (simple_hash(x, y, 33) % 20) as u8;
                set_pixel(&mut data, w, x, y, 170 + n, 200 + n.min(55), 240, alpha);
                // Glowing eyes
                if in_head && (cy + 3.0).abs() < 0.8 {
                    if (cx - 1.5).abs() < 0.7 { set_pixel(&mut data, w, x, y, 150, 220, 255, 255); }
                    if (cx + 1.5).abs() < 0.7 { set_pixel(&mut data, w, x, y, 150, 220, 255, 255); }
                }
            }
        }
    }
    make_image(w, h, data)
}

/// Scorpion: body with claws and curved tail (14x10)
fn generate_scorpion_texture() -> Image {
    let w: u32 = 14; let h: u32 = 10;
    let mut data = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let cx = x as f32 - 7.0; let cy = y as f32 - 5.0;
            // Body: oval
            let in_body = (cx*cx / 12.0 + cy*cy / 6.0) < 1.0;
            // Claws: two circles at front
            let lc = (cx + 4.5) * (cx + 4.5) + (cy + 2.5) * (cy + 2.5);
            let rc = (cx - 4.5) * (cx - 4.5) + (cy + 2.5) * (cy + 2.5);
            let in_claws = lc < 4.0 || rc < 4.0;
            // Tail: curves up from back
            let in_tail = cx.abs() < 1.0 && cy > 1.0 && cy < 5.0;
            // Stinger
            let in_stinger = (cx*cx + (cy - 4.5) * (cy - 4.5)) < 2.0;
            // Legs
            let in_legs = cy.abs() < 0.6 && (cx.abs() > 2.0 && cx.abs() < 6.0) &&
                simple_hash(x, y, 11) % 2 == 0;

            if in_body || in_claws || in_tail || in_stinger || in_legs {
                set_pixel_noisy(&mut data, w, x, y, 160, 120, 70);
                if in_stinger { set_pixel(&mut data, w, x, y, 200, 60, 40, 255); }
                // Eyes
                if (cy + 1.5).abs() < 0.6 {
                    if (cx - 1.0).abs() < 0.6 { set_pixel(&mut data, w, x, y, 20, 20, 20, 255); }
                    if (cx + 1.0).abs() < 0.6 { set_pixel(&mut data, w, x, y, 20, 20, 20, 255); }
                }
            }
        }
    }
    make_image(w, h, data)
}

/// Boss: large menacing silhouette with crown-like horns (24x24)
fn generate_boss_texture() -> Image {
    let w: u32 = 24; let h: u32 = 24;
    let mut data = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let cx = x as f32 - 12.0; let cy = y as f32 - 12.0;
            // Large body
            let in_body = (cx*cx / 80.0 + cy*cy / 64.0) < 1.0;
            // Horns/crown: two triangular protrusions on top
            let in_lhorn = cx < -2.0 && cx > -6.0 && cy < -5.0 && (cy + 5.0 + (cx + 4.0) * 0.8).abs() < 1.5;
            let in_rhorn = cx > 2.0 && cx < 6.0 && cy < -5.0 && (cy + 5.0 - (cx - 4.0) * 0.8).abs() < 1.5;
            // Arms
            let in_arms = cy.abs() < 2.5 && cx.abs() > 6.0 && cx.abs() < 11.0;

            if in_body || in_lhorn || in_rhorn || in_arms {
                let n = (simple_hash(x, y, 66) % 25) as f32;
                set_pixel(&mut data, w, x, y, (100.0 + n) as u8, (40.0 + n) as u8, (50.0 + n) as u8, 255);
                // Glowing red eyes
                if (cy + 1.0).abs() < 1.2 {
                    if (cx - 3.0).abs() < 1.2 { set_pixel(&mut data, w, x, y, 255, 50, 30, 255); }
                    if (cx + 3.0).abs() < 1.2 { set_pixel(&mut data, w, x, y, 255, 50, 30, 255); }
                }
            }
        }
    }
    make_image(w, h, data)
}

/// Vignette: radial darkening from transparent center to dark edges (256x256)
fn generate_vignette_texture() -> Image {
    let w: u32 = 256; let h: u32 = 256;
    let mut data = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let cx = (x as f32 / w as f32) * 2.0 - 1.0;
            let cy = (y as f32 / h as f32) * 2.0 - 1.0;
            let dist = (cx * cx + cy * cy).sqrt();
            // Vignette: transparent center, dark edges
            let alpha = if dist < 0.5 {
                0.0
            } else {
                ((dist - 0.5) / 0.7).clamp(0.0, 1.0) * 0.6
            };
            data[i] = 2;     // near-black
            data[i + 1] = 2;
            data[i + 2] = 6;
            data[i + 3] = (alpha * 255.0) as u8;
        }
    }
    make_image(w, h, data)
}

/// Slash arc: white crescent shape for attack visual (20x20)
fn generate_slash_arc_texture() -> Image {
    let w: u32 = 20; let h: u32 = 20;
    let mut data = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let cx = x as f32 - 10.0; let cy = y as f32 - 10.0;
            let dist = (cx*cx + cy*cy).sqrt();
            // Arc: ring between radius 6 and 9, only top half
            if dist > 5.0 && dist < 9.0 && cy < 2.0 {
                let t = ((dist - 5.0) / 4.0).clamp(0.0, 1.0);
                let edge_fade = 1.0 - (t - 0.5).abs() * 2.0;
                let alpha = (edge_fade * 220.0) as u8;
                set_pixel(&mut data, w, x, y, 255, 255, 240, alpha);
            }
        }
    }
    make_image(w, h, data)
}
