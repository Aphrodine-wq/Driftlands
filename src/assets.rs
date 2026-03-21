use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_assets);
        app.add_systems(Update, build_animation_texture_atlases);
    }
}

#[derive(Resource)]
pub struct GameAssets {
    // ── Player ───────────────────────────────────────────────────────────────
    pub player: Handle<Image>,

    // ── Tiles (16x16, kept for backwards compat — chunk rendering uses biome_color directly) ──
    pub forest_grass: Handle<Image>,
    pub dirt: Handle<Image>,
    pub water: Handle<Image>,
    pub stone: Handle<Image>,
    pub sand: Handle<Image>,
    // New biome tiles
    pub swamp_mud: Handle<Image>,
    pub swamp_water: Handle<Image>,
    pub tundra_snow: Handle<Image>,
    pub tundra_ice: Handle<Image>,
    pub volcanic_ash: Handle<Image>,
    pub volcanic_basalt: Handle<Image>,
    pub crystal_ground: Handle<Image>,
    pub crystal_glow: Handle<Image>,
    pub fungal_mycelium: Handle<Image>,
    pub fungal_spore: Handle<Image>,
    pub coastal_sand: Handle<Image>,
    pub coastal_water: Handle<Image>,
    pub desert_cracked: Handle<Image>,
    pub desert_sand: Handle<Image>,
    pub mountain_gravel: Handle<Image>,
    pub mountain_stone: Handle<Image>,
    pub dungeon_stone: Handle<Image>,
    pub cave_floor: Handle<Image>,

    // ── Trees ────────────────────────────────────────────────────────────────
    pub oak_tree: Handle<Image>,
    pub pine_tree: Handle<Image>,
    // Seasonal tree variants
    pub oak_spring: Handle<Image>,
    pub oak_summer: Handle<Image>,
    pub oak_autumn: Handle<Image>,
    pub oak_winter: Handle<Image>,
    pub pine_spring: Handle<Image>,
    pub pine_summer: Handle<Image>,
    pub pine_autumn: Handle<Image>,
    pub pine_winter: Handle<Image>,

    // ── World Objects ────────────────────────────────────────────────────────
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
    // Objects (from objects/ directory)
    pub barrel: Handle<Image>,
    pub bone_pile: Handle<Image>,
    pub bush_berry: Handle<Image>,
    pub bush_green: Handle<Image>,
    pub copper_ore: Handle<Image>,
    pub coral: Handle<Image>,
    pub crate_obj: Handle<Image>,
    pub crystal_large: Handle<Image>,
    pub crystal_node: Handle<Image>,
    pub dead_tree: Handle<Image>,
    pub flower_blue: Handle<Image>,
    pub flower_red: Handle<Image>,
    pub gold_ore: Handle<Image>,
    pub hay_bale: Handle<Image>,
    pub iron_ore: Handle<Image>,
    pub mushroom_giant: Handle<Image>,
    pub mushroom_small: Handle<Image>,
    pub oak_tree_obj: Handle<Image>,
    pub palm_tree: Handle<Image>,
    pub pine_tree_obj: Handle<Image>,
    pub pumpkin: Handle<Image>,
    pub rock_large: Handle<Image>,
    pub rock_small: Handle<Image>,
    pub ruins_arch: Handle<Image>,
    pub ruins_pillar: Handle<Image>,
    pub seaweed: Handle<Image>,
    pub signpost: Handle<Image>,
    pub stalagmite: Handle<Image>,
    pub torch_wall: Handle<Image>,
    pub treasure_chest: Handle<Image>,
    pub vine_wall: Handle<Image>,
    pub wheat_crop: Handle<Image>,
    pub campfire_lit: Handle<Image>,
    // Objects extra (from objects_extra/ directory)
    pub alpine_flower: Handle<Image>,
    pub ancient_machinery: Handle<Image>,
    pub ancient_ruin_obj: Handle<Image>,
    pub bio_luminescent_gel: Handle<Image>,
    pub coal_deposit: Handle<Image>,
    pub crystal_cluster: Handle<Image>,
    pub driftwood: Handle<Image>,
    pub echo_stone_obj: Handle<Image>,
    pub fallen_log: Handle<Image>,
    pub frozen_ore_deposit: Handle<Image>,
    pub geyser: Handle<Image>,
    pub glowing_spore_obj: Handle<Image>,
    pub ice_crystal_obj: Handle<Image>,
    pub ice_formation: Handle<Image>,
    pub iron_vein_extra: Handle<Image>,
    pub oasis_palm: Handle<Image>,
    pub obsidian_node: Handle<Image>,
    pub reed_clump: Handle<Image>,
    pub sandstone_rock: Handle<Image>,
    pub seaweed_patch: Handle<Image>,
    pub shell_deposit: Handle<Image>,
    pub sulfur_deposit: Handle<Image>,
    pub sulfur_vent: Handle<Image>,
    pub supply_crate_extra: Handle<Image>,
    pub wildflower: Handle<Image>,

    // ── Enemies ──────────────────────────────────────────────────────────────
    pub enemy_wolf: Handle<Image>,
    pub enemy_spider: Handle<Image>,
    pub enemy_crawler: Handle<Image>,
    pub enemy_zombie: Handle<Image>,
    pub enemy_elemental: Handle<Image>,
    pub enemy_wraith: Handle<Image>,
    pub enemy_scorpion: Handle<Image>,
    pub enemy_boss: Handle<Image>,

    // ── Elite Enemies ────────────────────────────────────────────────────────
    pub elite_alpha_wolf: Handle<Image>,
    pub elite_bog_lurker: Handle<Image>,
    pub elite_frost_lich: Handle<Image>,
    pub elite_magma_golem: Handle<Image>,
    pub elite_night_bat: Handle<Image>,
    pub elite_venom_scorpion: Handle<Image>,

    // ── Bosses ───────────────────────────────────────────────────────────────
    pub boss_forest_treant: Handle<Image>,
    pub boss_swamp_hydra: Handle<Image>,
    pub boss_desert_wyrm: Handle<Image>,
    pub boss_tundra_yeti: Handle<Image>,
    pub boss_volcanic_dragon: Handle<Image>,
    pub boss_fungal_overmind: Handle<Image>,
    pub boss_crystal_golem: Handle<Image>,
    pub boss_coastal_kraken: Handle<Image>,
    pub boss_mountain_titan: Handle<Image>,
    pub boss_stone_golem: Handle<Image>,

    // ── NPCs ─────────────────────────────────────────────────────────────────
    pub npc_blacksmith: Handle<Image>,
    pub npc_farmer: Handle<Image>,
    pub npc_hermit: Handle<Image>,
    pub npc_quest_giver: Handle<Image>,
    pub npc_wandering_trader: Handle<Image>,

    // ── Pets ─────────────────────────────────────────────────────────────────
    pub pet_bear: Handle<Image>,
    pub pet_cat: Handle<Image>,
    pub pet_hawk: Handle<Image>,
    pub pet_wolf: Handle<Image>,

    // ── Buildings ────────────────────────────────────────────────────────────
    pub wood_wall: Handle<Image>,
    pub wood_floor: Handle<Image>,
    pub wood_door: Handle<Image>,
    pub stone_wall: Handle<Image>,
    pub campfire: Handle<Image>,
    pub workbench: Handle<Image>,
    pub forge: Handle<Image>,
    pub chest_building: Handle<Image>,
    pub bed: Handle<Image>,
    pub roof_thatch: Handle<Image>,
    // Buildings extra
    pub advanced_forge: Handle<Image>,
    pub alarm_bell: Handle<Image>,
    pub alchemy_lab: Handle<Image>,
    pub ancient_workstation: Handle<Image>,
    pub auto_smelter: Handle<Image>,
    pub bookshelf: Handle<Image>,
    pub brick_wall: Handle<Image>,
    pub cooking_pot: Handle<Image>,
    pub crop_sprinkler: Handle<Image>,
    pub display_case: Handle<Image>,
    pub enchanting_table: Handle<Image>,
    pub fish_smoker: Handle<Image>,
    pub ladder: Handle<Image>,
    pub lantern: Handle<Image>,
    pub metal_door: Handle<Image>,
    pub metal_wall: Handle<Image>,
    pub pet_house: Handle<Image>,
    pub rain_collector: Handle<Image>,
    pub reinforced_wall: Handle<Image>,
    pub stone_door_building: Handle<Image>,
    pub stone_floor: Handle<Image>,
    pub stone_roof: Handle<Image>,
    pub stone_stairs: Handle<Image>,
    pub stone_wall_extra: Handle<Image>,
    pub trophy_mount: Handle<Image>,
    pub weapon_rack: Handle<Image>,
    pub wood_fence: Handle<Image>,
    pub wood_half_wall: Handle<Image>,
    pub wood_stairs: Handle<Image>,
    pub wood_wall_window: Handle<Image>,

    // ── Weapons (for future inventory icons) ─────────────────────────────────
    pub weapon_wood_sword: Handle<Image>,
    pub weapon_iron_sword: Handle<Image>,
    pub weapon_steel_sword: Handle<Image>,
    pub weapon_ancient_blade: Handle<Image>,
    pub weapon_flame_blade: Handle<Image>,
    pub weapon_frost_blade: Handle<Image>,
    pub weapon_venom_blade: Handle<Image>,
    pub weapon_lifesteal_blade: Handle<Image>,
    pub weapon_wood_bow: Handle<Image>,
    pub weapon_arrow: Handle<Image>,

    // ── Tools (for future inventory icons) ───────────────────────────────────
    pub tool_wood_axe: Handle<Image>,
    pub tool_stone_axe: Handle<Image>,
    pub tool_iron_axe: Handle<Image>,
    pub tool_steel_axe: Handle<Image>,
    pub tool_wood_pickaxe: Handle<Image>,
    pub tool_stone_pickaxe: Handle<Image>,
    pub tool_iron_pickaxe: Handle<Image>,
    pub tool_steel_pickaxe: Handle<Image>,
    pub tool_ancient_pickaxe: Handle<Image>,
    pub tool_hoe: Handle<Image>,
    pub tool_fishing_rod: Handle<Image>,
    pub tool_steel_fishing_rod: Handle<Image>,
    pub tool_fish_bait: Handle<Image>,

    // ── Animation Frames ──────────────────────────────────────────────────────
    pub wolf_walk_frames: Vec<Handle<Image>>,
    pub spider_walk_frames: Vec<Handle<Image>>,
    pub shadow_crawler_walk_frames: Vec<Handle<Image>>,
    pub campfire_anim_frames: Vec<Handle<Image>>,
    pub water_anim_frames: Vec<Handle<Image>>,
    pub torch_anim_frames: Vec<Handle<Image>>,

    // ── Animation Texture Atlases (built at runtime) ─────────────────────────
    pub wolf_walk_atlas_image: Option<Handle<Image>>,
    pub wolf_walk_atlas_layout: Option<Handle<TextureAtlasLayout>>,
    pub spider_walk_atlas_image: Option<Handle<Image>>,
    pub spider_walk_atlas_layout: Option<Handle<TextureAtlasLayout>>,
    pub shadow_crawler_walk_atlas_image: Option<Handle<Image>>,
    pub shadow_crawler_walk_atlas_layout: Option<Handle<TextureAtlasLayout>>,
    pub campfire_anim_atlas_image: Option<Handle<Image>>,
    pub campfire_anim_atlas_layout: Option<Handle<TextureAtlasLayout>>,

    // ── Procedural Utility Textures ──────────────────────────────────────────
    // Attack visual
    pub slash_arc: Handle<Image>,
    // Screen effects
    pub vignette: Handle<Image>,
    // Normal maps and utility
    pub flat_normal_16: Handle<Image>,
    pub flat_normal_32: Handle<Image>,
    pub player_normal: Handle<Image>,
    pub enemy_wolf_normal: Handle<Image>,
    pub enemy_zombie_normal: Handle<Image>,
    pub white_pixel: Handle<Image>,
}

fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    let assets = GameAssets {
        // ── Player ───────────────────────────────────────────────────────
        player: asset_server.load("player/player_idle_00001_.png"),

        // ── Tiles ────────────────────────────────────────────────────────
        forest_grass: asset_server.load("tiles/forest_grass_00001_.png"),
        dirt: asset_server.load("tiles/forest_dirt_00001_.png"),
        water: asset_server.load("tiles/coastal_water_00001_.png"),
        stone: asset_server.load("tiles/mountain_stone_00001_.png"),
        sand: asset_server.load("tiles/desert_sand_00001_.png"),
        swamp_mud: asset_server.load("tiles/swamp_mud_00001_.png"),
        swamp_water: asset_server.load("tiles/swamp_water_00001_.png"),
        tundra_snow: asset_server.load("tiles/tundra_snow_00001_.png"),
        tundra_ice: asset_server.load("tiles/tundra_ice_00001_.png"),
        volcanic_ash: asset_server.load("tiles/volcanic_ash_00001_.png"),
        volcanic_basalt: asset_server.load("tiles/volcanic_basalt_00001_.png"),
        crystal_ground: asset_server.load("tiles/crystal_ground_00001_.png"),
        crystal_glow: asset_server.load("tiles/crystal_glow_00001_.png"),
        fungal_mycelium: asset_server.load("tiles/fungal_mycelium_00001_.png"),
        fungal_spore: asset_server.load("tiles/fungal_spore_00001_.png"),
        coastal_sand: asset_server.load("tiles/coastal_sand_00001_.png"),
        coastal_water: asset_server.load("tiles/coastal_water_00001_.png"),
        desert_cracked: asset_server.load("tiles/desert_cracked_00001_.png"),
        desert_sand: asset_server.load("tiles/desert_sand_00001_.png"),
        mountain_gravel: asset_server.load("tiles/mountain_gravel_00001_.png"),
        mountain_stone: asset_server.load("tiles/mountain_stone_00001_.png"),
        dungeon_stone: asset_server.load("tiles/dungeon_stone_00001_.png"),
        cave_floor: asset_server.load("tiles/cave_floor_00001_.png"),

        // ── Trees ────────────────────────────────────────────────────────
        oak_tree: asset_server.load("objects/oak_tree_00001_.png"),
        pine_tree: asset_server.load("objects/pine_tree_00001_.png"),
        oak_spring: asset_server.load("trees_seasonal/oak_spring_00001_.png"),
        oak_summer: asset_server.load("trees_seasonal/oak_summer_00001_.png"),
        oak_autumn: asset_server.load("trees_seasonal/oak_autumn_00001_.png"),
        oak_winter: asset_server.load("trees_seasonal/oak_winter_00001_.png"),
        pine_spring: asset_server.load("trees_seasonal/pine_spring_00001_.png"),
        pine_summer: asset_server.load("trees_seasonal/pine_summer_00001_.png"),
        pine_autumn: asset_server.load("trees_seasonal/pine_autumn_00001_.png"),
        pine_winter: asset_server.load("trees_seasonal/pine_winter_00001_.png"),

        // ── World Objects ────────────────────────────────────────────────
        rock: asset_server.load("objects/rock_large_00001_.png"),
        bush: asset_server.load("objects/bush_green_00001_.png"),
        cactus: asset_server.load("objects/cactus_00001_.png"),
        mushroom: asset_server.load("objects/mushroom_small_00001_.png"),
        giant_mushroom: asset_server.load("objects/mushroom_giant_00001_.png"),
        crystal: asset_server.load("objects/crystal_large_00001_.png"),
        iron_vein: asset_server.load("objects/iron_ore_00001_.png"),
        berry_bush: asset_server.load("objects/bush_berry_00001_.png"),
        supply_crate: asset_server.load("objects_extra/supply_crate_00001_.png"),
        dungeon_entrance: asset_server.load("objects/ruins_arch_00001_.png"),
        // Objects
        barrel: asset_server.load("objects/barrel_00001_.png"),
        bone_pile: asset_server.load("objects/bone_pile_00001_.png"),
        bush_berry: asset_server.load("objects/bush_berry_00001_.png"),
        bush_green: asset_server.load("objects/bush_green_00001_.png"),
        copper_ore: asset_server.load("objects/copper_ore_00001_.png"),
        coral: asset_server.load("objects/coral_00001_.png"),
        crate_obj: asset_server.load("objects/crate_00001_.png"),
        crystal_large: asset_server.load("objects/crystal_large_00001_.png"),
        crystal_node: asset_server.load("objects/crystal_node_00001_.png"),
        dead_tree: asset_server.load("objects/dead_tree_00001_.png"),
        flower_blue: asset_server.load("objects/flower_blue_00001_.png"),
        flower_red: asset_server.load("objects/flower_red_00001_.png"),
        gold_ore: asset_server.load("objects/gold_ore_00001_.png"),
        hay_bale: asset_server.load("objects/hay_bale_00001_.png"),
        iron_ore: asset_server.load("objects/iron_ore_00001_.png"),
        mushroom_giant: asset_server.load("objects/mushroom_giant_00001_.png"),
        mushroom_small: asset_server.load("objects/mushroom_small_00001_.png"),
        oak_tree_obj: asset_server.load("objects/oak_tree_00001_.png"),
        palm_tree: asset_server.load("objects/palm_tree_00001_.png"),
        pine_tree_obj: asset_server.load("objects/pine_tree_00001_.png"),
        pumpkin: asset_server.load("objects/pumpkin_00001_.png"),
        rock_large: asset_server.load("objects/rock_large_00001_.png"),
        rock_small: asset_server.load("objects/rock_small_00001_.png"),
        ruins_arch: asset_server.load("objects/ruins_arch_00001_.png"),
        ruins_pillar: asset_server.load("objects/ruins_pillar_00001_.png"),
        seaweed: asset_server.load("objects/seaweed_00001_.png"),
        signpost: asset_server.load("objects/signpost_00001_.png"),
        stalagmite: asset_server.load("objects/stalagmite_00001_.png"),
        torch_wall: asset_server.load("objects/torch_wall_00001_.png"),
        treasure_chest: asset_server.load("objects/treasure_chest_00001_.png"),
        vine_wall: asset_server.load("objects/vine_wall_00001_.png"),
        wheat_crop: asset_server.load("objects/wheat_crop_00001_.png"),
        campfire_lit: asset_server.load("objects/campfire_lit_00001_.png"),
        // Objects extra
        alpine_flower: asset_server.load("objects_extra/alpine_flower_00001_.png"),
        ancient_machinery: asset_server.load("objects_extra/ancient_machinery_00001_.png"),
        ancient_ruin_obj: asset_server.load("objects_extra/ancient_ruin_obj_00001_.png"),
        bio_luminescent_gel: asset_server.load("objects_extra/bio_luminescent_gel_00001_.png"),
        coal_deposit: asset_server.load("objects_extra/coal_deposit_00001_.png"),
        crystal_cluster: asset_server.load("objects_extra/crystal_cluster_00001_.png"),
        driftwood: asset_server.load("objects_extra/driftwood_00001_.png"),
        echo_stone_obj: asset_server.load("objects_extra/echo_stone_obj_00001_.png"),
        fallen_log: asset_server.load("objects_extra/fallen_log_00001_.png"),
        frozen_ore_deposit: asset_server.load("objects_extra/frozen_ore_deposit_00001_.png"),
        geyser: asset_server.load("objects_extra/geyser_00001_.png"),
        glowing_spore_obj: asset_server.load("objects_extra/glowing_spore_obj_00001_.png"),
        ice_crystal_obj: asset_server.load("objects_extra/ice_crystal_obj_00001_.png"),
        ice_formation: asset_server.load("objects_extra/ice_formation_00001_.png"),
        iron_vein_extra: asset_server.load("objects_extra/iron_vein_00001_.png"),
        oasis_palm: asset_server.load("objects_extra/oasis_palm_00001_.png"),
        obsidian_node: asset_server.load("objects_extra/obsidian_node_00001_.png"),
        reed_clump: asset_server.load("objects_extra/reed_clump_00001_.png"),
        sandstone_rock: asset_server.load("objects_extra/sandstone_rock_00001_.png"),
        seaweed_patch: asset_server.load("objects_extra/seaweed_patch_00001_.png"),
        shell_deposit: asset_server.load("objects_extra/shell_deposit_00001_.png"),
        sulfur_deposit: asset_server.load("objects_extra/sulfur_deposit_00001_.png"),
        sulfur_vent: asset_server.load("objects_extra/sulfur_vent_00001_.png"),
        supply_crate_extra: asset_server.load("objects_extra/supply_crate_00001_.png"),
        wildflower: asset_server.load("objects_extra/wildflower_00001_.png"),

        // ── Enemies ──────────────────────────────────────────────────────
        enemy_wolf: asset_server.load("enemies/wolf_00001_.png"),
        enemy_spider: asset_server.load("enemies/spider_00001_.png"),
        enemy_crawler: asset_server.load("enemies/shadow_crawler_00001_.png"),
        enemy_zombie: asset_server.load("enemies/fungal_zombie_00001_.png"),
        enemy_elemental: asset_server.load("enemies/lava_elemental_00001_.png"),
        enemy_wraith: asset_server.load("enemies/ice_wraith_00001_.png"),
        enemy_scorpion: asset_server.load("enemies/sand_scorpion_00001_.png"),
        enemy_boss: asset_server.load("bosses/stone_golem_00001_.png"),

        // ── Elite Enemies ────────────────────────────────────────────────
        elite_alpha_wolf: asset_server.load("elite_enemies/alpha_wolf_00001_.png"),
        elite_bog_lurker: asset_server.load("elite_enemies/bog_lurker_00001_.png"),
        elite_frost_lich: asset_server.load("elite_enemies/frost_lich_00001_.png"),
        elite_magma_golem: asset_server.load("elite_enemies/magma_golem_00001_.png"),
        elite_night_bat: asset_server.load("elite_enemies/night_bat_00001_.png"),
        elite_venom_scorpion: asset_server.load("elite_enemies/venom_scorpion_00001_.png"),

        // ── Bosses ───────────────────────────────────────────────────────
        boss_forest_treant: asset_server.load("bosses/forest_treant_00001_.png"),
        boss_swamp_hydra: asset_server.load("bosses/swamp_hydra_00001_.png"),
        boss_desert_wyrm: asset_server.load("bosses/desert_wyrm_00001_.png"),
        boss_tundra_yeti: asset_server.load("bosses/tundra_yeti_00001_.png"),
        boss_volcanic_dragon: asset_server.load("bosses/volcanic_dragon_00001_.png"),
        boss_fungal_overmind: asset_server.load("bosses/fungal_overmind_00001_.png"),
        boss_crystal_golem: asset_server.load("bosses/crystal_golem_00001_.png"),
        boss_coastal_kraken: asset_server.load("bosses/coastal_kraken_00001_.png"),
        boss_mountain_titan: asset_server.load("bosses/mountain_titan_00001_.png"),
        boss_stone_golem: asset_server.load("bosses/stone_golem_00001_.png"),

        // ── NPCs ─────────────────────────────────────────────────────────
        npc_blacksmith: asset_server.load("npcs/blacksmith_00001_.png"),
        npc_farmer: asset_server.load("npcs/farmer_npc_00001_.png"),
        npc_hermit: asset_server.load("npcs/hermit_00001_.png"),
        npc_quest_giver: asset_server.load("npcs/quest_giver_00001_.png"),
        npc_wandering_trader: asset_server.load("npcs/wandering_trader_00001_.png"),

        // ── Pets ─────────────────────────────────────────────────────────
        pet_bear: asset_server.load("pets/pet_bear_00001_.png"),
        pet_cat: asset_server.load("pets/pet_cat_00001_.png"),
        pet_hawk: asset_server.load("pets/pet_hawk_00001_.png"),
        pet_wolf: asset_server.load("pets/pet_wolf_00001_.png"),

        // ── Buildings ────────────────────────────────────────────────────
        wood_wall: asset_server.load("buildings/wall_wood_00001_.png"),
        wood_floor: asset_server.load("buildings/floor_wood_00001_.png"),
        wood_door: asset_server.load("buildings/door_wood_00001_.png"),
        stone_wall: asset_server.load("buildings/wall_stone_00001_.png"),
        campfire: asset_server.load("objects/campfire_lit_00001_.png"),
        workbench: asset_server.load("buildings/workbench_00001_.png"),
        forge: asset_server.load("buildings/forge_00001_.png"),
        chest_building: asset_server.load("buildings/storage_chest_00001_.png"),
        bed: asset_server.load("buildings/bed_simple_00001_.png"),
        roof_thatch: asset_server.load("buildings/roof_thatch_00001_.png"),
        // Buildings extra
        advanced_forge: asset_server.load("buildings_extra/advanced_forge_00001_.png"),
        alarm_bell: asset_server.load("buildings_extra/alarm_bell_00001_.png"),
        alchemy_lab: asset_server.load("buildings_extra/alchemy_lab_00001_.png"),
        ancient_workstation: asset_server.load("buildings_extra/ancient_workstation_00001_.png"),
        auto_smelter: asset_server.load("buildings_extra/auto_smelter_00001_.png"),
        bookshelf: asset_server.load("buildings_extra/bookshelf_00001_.png"),
        brick_wall: asset_server.load("buildings_extra/brick_wall_00001_.png"),
        cooking_pot: asset_server.load("buildings_extra/cooking_pot_00001_.png"),
        crop_sprinkler: asset_server.load("buildings_extra/crop_sprinkler_00001_.png"),
        display_case: asset_server.load("buildings_extra/display_case_00001_.png"),
        enchanting_table: asset_server.load("buildings_extra/enchanting_table_00001_.png"),
        fish_smoker: asset_server.load("buildings_extra/fish_smoker_00001_.png"),
        ladder: asset_server.load("buildings_extra/ladder_00001_.png"),
        lantern: asset_server.load("buildings_extra/lantern_00001_.png"),
        metal_door: asset_server.load("buildings_extra/metal_door_00001_.png"),
        metal_wall: asset_server.load("buildings_extra/metal_wall_00001_.png"),
        pet_house: asset_server.load("buildings_extra/pet_house_00001_.png"),
        rain_collector: asset_server.load("buildings_extra/rain_collector_00001_.png"),
        reinforced_wall: asset_server.load("buildings_extra/reinforced_wall_00001_.png"),
        stone_door_building: asset_server.load("buildings_extra/stone_door_00001_.png"),
        stone_floor: asset_server.load("buildings_extra/stone_floor_00001_.png"),
        stone_roof: asset_server.load("buildings_extra/stone_roof_00001_.png"),
        stone_stairs: asset_server.load("buildings_extra/stone_stairs_00001_.png"),
        stone_wall_extra: asset_server.load("buildings_extra/stone_wall_00001_.png"),
        trophy_mount: asset_server.load("buildings_extra/trophy_mount_00001_.png"),
        weapon_rack: asset_server.load("buildings_extra/weapon_rack_00001_.png"),
        wood_fence: asset_server.load("buildings_extra/wood_fence_00001_.png"),
        wood_half_wall: asset_server.load("buildings_extra/wood_half_wall_00001_.png"),
        wood_stairs: asset_server.load("buildings_extra/wood_stairs_00001_.png"),
        wood_wall_window: asset_server.load("buildings_extra/wood_wall_window_00001_.png"),

        // ── Weapons ──────────────────────────────────────────────────────
        weapon_wood_sword: asset_server.load("weapons/wood_sword_00001_.png"),
        weapon_iron_sword: asset_server.load("weapons/iron_sword_00001_.png"),
        weapon_steel_sword: asset_server.load("weapons/steel_sword_00001_.png"),
        weapon_ancient_blade: asset_server.load("weapons/ancient_blade_00001_.png"),
        weapon_flame_blade: asset_server.load("weapons/flame_blade_00001_.png"),
        weapon_frost_blade: asset_server.load("weapons/frost_blade_00001_.png"),
        weapon_venom_blade: asset_server.load("weapons/venom_blade_00001_.png"),
        weapon_lifesteal_blade: asset_server.load("weapons/lifesteal_blade_00001_.png"),
        weapon_wood_bow: asset_server.load("weapons/wood_bow_00001_.png"),
        weapon_arrow: asset_server.load("weapons/arrow_00001_.png"),

        // ── Tools ────────────────────────────────────────────────────────
        tool_wood_axe: asset_server.load("tools/wood_axe_00001_.png"),
        tool_stone_axe: asset_server.load("tools/stone_axe_00001_.png"),
        tool_iron_axe: asset_server.load("tools/iron_axe_00001_.png"),
        tool_steel_axe: asset_server.load("tools/steel_axe_00001_.png"),
        tool_wood_pickaxe: asset_server.load("tools/wood_pickaxe_00001_.png"),
        tool_stone_pickaxe: asset_server.load("tools/stone_pickaxe_00001_.png"),
        tool_iron_pickaxe: asset_server.load("tools/iron_pickaxe_00001_.png"),
        tool_steel_pickaxe: asset_server.load("tools/steel_pickaxe_00001_.png"),
        tool_ancient_pickaxe: asset_server.load("tools/ancient_pickaxe_00001_.png"),
        tool_hoe: asset_server.load("tools/hoe_00001_.png"),
        tool_fishing_rod: asset_server.load("tools/fishing_rod_00001_.png"),
        tool_steel_fishing_rod: asset_server.load("tools/steel_fishing_rod_00001_.png"),
        tool_fish_bait: asset_server.load("tools/fish_bait_00001_.png"),

        // ── Animation Frames ─────────────────────────────────────────────
        wolf_walk_frames: vec![
            asset_server.load("animations/enemies/wolf_walk/frame_01_00001_.png"),
            asset_server.load("animations/enemies/wolf_walk/frame_02_00001_.png"),
            asset_server.load("animations/enemies/wolf_walk/frame_03_00001_.png"),
            asset_server.load("animations/enemies/wolf_walk/frame_04_00001_.png"),
        ],
        spider_walk_frames: vec![
            asset_server.load("animations/enemies/spider_walk/frame_01_00001_.png"),
            asset_server.load("animations/enemies/spider_walk/frame_02_00001_.png"),
            asset_server.load("animations/enemies/spider_walk/frame_03_00001_.png"),
            asset_server.load("animations/enemies/spider_walk/frame_04_00001_.png"),
        ],
        shadow_crawler_walk_frames: vec![
            asset_server.load("animations/enemies/shadow_crawler_walk/frame_01_00001_.png"),
            asset_server.load("animations/enemies/shadow_crawler_walk/frame_02_00001_.png"),
            asset_server.load("animations/enemies/shadow_crawler_walk/frame_03_00001_.png"),
            asset_server.load("animations/enemies/shadow_crawler_walk/frame_04_00001_.png"),
        ],
        campfire_anim_frames: vec![
            asset_server.load("animations/environment/campfire_anim/frame_01_00001_.png"),
            asset_server.load("animations/environment/campfire_anim/frame_02_00001_.png"),
            asset_server.load("animations/environment/campfire_anim/frame_03_00001_.png"),
            asset_server.load("animations/environment/campfire_anim/frame_04_00001_.png"),
        ],
        water_anim_frames: vec![
            asset_server.load("animations/environment/water_anim/frame_01_00001_.png"),
            asset_server.load("animations/environment/water_anim/frame_02_00001_.png"),
            asset_server.load("animations/environment/water_anim/frame_03_00001_.png"),
            asset_server.load("animations/environment/water_anim/frame_04_00001_.png"),
        ],
        torch_anim_frames: vec![
            asset_server.load("animations/environment/torch_anim/frame_01_00001_.png"),
            asset_server.load("animations/environment/torch_anim/frame_02_00001_.png"),
            asset_server.load("animations/environment/torch_anim/frame_03_00001_.png"),
            asset_server.load("animations/environment/torch_anim/frame_04_00001_.png"),
        ],

        // Texture atlases are built once the frame images have loaded.
        wolf_walk_atlas_image: None,
        wolf_walk_atlas_layout: None,
        spider_walk_atlas_image: None,
        spider_walk_atlas_layout: None,
        shadow_crawler_walk_atlas_image: None,
        shadow_crawler_walk_atlas_layout: None,
        campfire_anim_atlas_image: None,
        campfire_anim_atlas_layout: None,

        // ── Procedural Utility Textures ──────────────────────────────────
        slash_arc: images.add(generate_slash_arc_texture()),
        vignette: images.add(generate_vignette_texture()),
        flat_normal_16: images.add(generate_flat_normal(16, 16)),
        flat_normal_32: images.add(generate_flat_normal(32, 32)),
        player_normal: images.add(generate_player_normal()),
        enemy_wolf_normal: images.add(generate_wolf_normal()),
        enemy_zombie_normal: images.add(generate_zombie_normal()),
        white_pixel: images.add(generate_white_pixel()),
    };

    commands.insert_resource(assets);
}

fn build_horizontal_animation_atlas(
    frames: &[Handle<Image>],
    images: &mut ResMut<Assets<Image>>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
) -> Option<(Handle<Image>, Handle<TextureAtlasLayout>)> {
    if frames.is_empty() {
        return None;
    }

    // Wait until all frame images have finished loading.
    let first = images.get(&frames[0])?;
    let frame_w = first.size().x.max(1);
    let frame_h = first.size().y.max(1);
    let frame_w_usize = frame_w as usize;
    let frame_h_usize = frame_h as usize;

    // Verify all frames have matching dimensions.
    for h in frames {
        let img = images.get(h)?;
        if img.size().x != frame_w || img.size().y != frame_h {
            return None;
        }
    }

    let atlas_w = frame_w * frames.len() as u32;
    let atlas_h = frame_h;
    let atlas_w_usize = atlas_w as usize;
    let atlas_h_usize = atlas_h as usize;

    // RGBA8 atlas.
    let mut atlas_data = vec![0u8; atlas_w_usize * atlas_h_usize * 4];

    for (frame_i, h) in frames.iter().enumerate() {
        let img = images.get(h)?;
        let x_offset = frame_i * frame_w_usize;
        for y in 0..frame_h_usize {
            let src_row_start = (y * frame_w_usize) * 4;
            let dst_row_start = (y * atlas_w_usize + x_offset) * 4;
            let src_row_end = src_row_start + frame_w_usize * 4;
            atlas_data[dst_row_start..dst_row_start + frame_w_usize * 4]
                .copy_from_slice(&img.data[src_row_start..src_row_end]);
        }
    }

    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(frame_w, frame_h),
        frames.len() as u32,
        1,
        None,
        None,
    );
    let layout_handle = texture_atlas_layouts.add(layout);

    let atlas_image = Image::new(
        Extent3d {
            width: atlas_w,
            height: atlas_h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        atlas_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );

    let atlas_handle = images.add(atlas_image);

    Some((atlas_handle, layout_handle))
}

fn build_animation_texture_atlases(
    mut game_assets: ResMut<GameAssets>,
    mut images: ResMut<Assets<Image>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut perf: ResMut<crate::debug_perf::DebugPerfTiming>,
) {
    let start = std::time::Instant::now();
    let mut built_count: u32 = 0;

    // Wolf walk
    if game_assets.wolf_walk_atlas_image.is_none() && !game_assets.wolf_walk_frames.is_empty() {
        if let Some((atlas_image, atlas_layout)) = build_horizontal_animation_atlas(
            &game_assets.wolf_walk_frames,
            &mut images,
            &mut texture_atlas_layouts,
        ) {
            game_assets.wolf_walk_atlas_image = Some(atlas_image);
            game_assets.wolf_walk_atlas_layout = Some(atlas_layout);
            built_count += 1;
        }
    }

    // Spider walk
    if game_assets.spider_walk_atlas_image.is_none() && !game_assets.spider_walk_frames.is_empty() {
        if let Some((atlas_image, atlas_layout)) = build_horizontal_animation_atlas(
            &game_assets.spider_walk_frames,
            &mut images,
            &mut texture_atlas_layouts,
        ) {
            game_assets.spider_walk_atlas_image = Some(atlas_image);
            game_assets.spider_walk_atlas_layout = Some(atlas_layout);
            built_count += 1;
        }
    }

    // Shadow crawler walk
    if game_assets.shadow_crawler_walk_atlas_image.is_none()
        && !game_assets.shadow_crawler_walk_frames.is_empty()
    {
        if let Some((atlas_image, atlas_layout)) = build_horizontal_animation_atlas(
            &game_assets.shadow_crawler_walk_frames,
            &mut images,
            &mut texture_atlas_layouts,
        ) {
            game_assets.shadow_crawler_walk_atlas_image = Some(atlas_image);
            game_assets.shadow_crawler_walk_atlas_layout = Some(atlas_layout);
            built_count += 1;
        }
    }

    // Campfire
    if game_assets.campfire_anim_atlas_image.is_none() && !game_assets.campfire_anim_frames.is_empty() {
        if let Some((atlas_image, atlas_layout)) = build_horizontal_animation_atlas(
            &game_assets.campfire_anim_frames,
            &mut images,
            &mut texture_atlas_layouts,
        ) {
            game_assets.campfire_anim_atlas_image = Some(atlas_image);
            game_assets.campfire_anim_atlas_layout = Some(atlas_layout);
            built_count += 1;
        }
    }

    if built_count > 0 {
        perf.atlas_build_ms = start.elapsed().as_secs_f32() * 1000.0;
        perf.atlases_built_this_session += built_count;
    }
}

// ============================================================
// Procedural Utility Texture Generators (kept)
// ============================================================

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

fn make_image(width: u32, height: u32, data: Vec<u8>) -> Image {
    Image::new(
        Extent3d { width, height, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
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
            let alpha = if dist < 0.5 {
                0.0
            } else {
                ((dist - 0.5) / 0.7).clamp(0.0, 1.0) * 0.6
            };
            data[i] = 2;
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
            let dist = (cx * cx + cy * cy).sqrt();
            if dist > 5.0 && dist < 9.0 && cy < 2.0 {
                let t = ((dist - 5.0) / 4.0).clamp(0.0, 1.0);
                let edge_fade = 1.0 - (t - 0.5).abs() * 2.0;
                let alpha = (edge_fade * 220.0) as u8;
                let i = ((y * w + x) * 4) as usize;
                data[i] = 255;
                data[i + 1] = 255;
                data[i + 2] = 240;
                data[i + 3] = alpha;
            }
        }
    }
    make_image(w, h, data)
}
