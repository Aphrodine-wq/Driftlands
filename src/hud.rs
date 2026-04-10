use crate::assets::GameAssets;
use crate::audio::SoundEvent;
use crate::building::{BuildingState, ChestStorage, ChestUI, CraftingStation};
use crate::controls::ControlsOverlay;
use crate::crafting::{CraftingSystem, CraftingTier};
use crate::daynight::DayNightCycle;
use crate::experiment::{ExperimentMessage, ExperimentSlots};
use crate::fishing::{FishType, FishingPhase, FishingState};
use crate::gathering::dropped_item_color;
use crate::inventory::{Inventory, ItemType};
use crate::lore::{LoreMessage, LoreRegistry};
use crate::mainmenu::MainMenuActive;
use crate::npc::{HermitDialogueDisplay, NpcDialogueDisplay, TradeMenu, Trader};
use crate::pets::Pet;
use crate::player::{ActiveBuff, ArmorSlots, BuffType, Health, Hunger, Player};
use crate::quests::QuestLog;
use crate::saveload::SaveMessage;
use crate::season::SeasonCycle;
use crate::skills::{SkillLevels, SkillType};
use crate::status_effects::{ActiveStatusEffects, StatusEffectType};
use crate::techtree::TechTree;
use crate::theme::EtherealTheme;
use crate::weather::WeatherSystem;
use crate::world::chunk::Chunk;
use crate::world::generation::Biome;
use crate::world::CHUNK_WORLD_SIZE;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct PauseState {
    pub paused: bool,
}

/// Run condition: returns `true` when the game is NOT paused and the main menu is not active.
pub fn not_paused(pause: Res<PauseState>, menu: Res<MainMenuActive>) -> bool {
    !pause.paused && !menu.active
}

#[derive(Resource, Default)]
pub struct CurrentBiome {
    pub biome: Option<Biome>,
    pub display_timer: f32,
}

/// Biomes the player has entered at least once (for first-time discovery feedback).
#[derive(Resource, Default)]
pub struct ExploredBiomes {
    pub set: std::collections::HashSet<Biome>,
}

/// Smoothed display values for health/hunger bars (lerp over ~0.2s).
#[derive(Resource)]
pub struct BarDisplayState {
    pub health_frac: f32,
    pub hunger_frac: f32,
}

impl Default for BarDisplayState {
    fn default() -> Self {
        Self {
            health_frac: 1.0,
            hunger_frac: 1.0,
        }
    }
}

#[derive(Component)]
pub struct BiomeBannerText;

pub struct HudPlugin;

/// Tracks the "Caught a Fish!" flash timer for the fishing HUD.
#[derive(Resource, Default)]
pub struct FishingCatchFlash {
    pub fish_name: String,
    pub timer: f32,
}

// ── HUD Caches (skip text rebuild when nothing changed) ─────────────────────

#[derive(Resource, Default)]
struct StatusHudCache {
    last_hp_i: i32,
    last_max_hp_i: i32,
    last_hunger_i: i32,
    last_max_hunger_i: i32,
    last_armor: u32,
    last_atk_i: i32,
    last_buff: Option<(BuffType, i32, i32)>, // (type, magnitude_pct, remaining_secs)
    last_pet_happiness_i: i32,
    last_pet_exists: bool,
    last_save_text: String,
}

#[derive(Resource, Default)]
struct MainHudCache {
    last_day: u32,
    last_phase: String,
    last_season: String,
    last_weather: String,
    last_forecast: String,
    last_build_active: bool,
    last_build_name: String,
    last_paused: bool,
}

#[derive(Resource, Default)]
struct FishingHudCache {
    last_phase: String,
    last_reel_pct: u32,
    last_hook_window_i: i32,
    last_dots_idx: u32,
    last_catch_flash_active: bool,
}

#[derive(Resource, Default)]
struct QuestLogHudCache {
    last_open: bool,
    last_selected: usize,
    last_quest_fingerprint: u64,
}

#[derive(Resource, Default)]
struct StatusEffectsHudCache {
    last_count: usize,
    last_secs_fingerprint: Vec<(u8, u32, u32)>, // (effect_type_idx, stacks, remaining_whole_secs)
}

#[derive(Resource, Default)]
struct SkillHudCache {
    last_open: bool,
    last_fingerprint: Vec<(u32, u32)>, // (level, xp) per skill
}

#[derive(Resource, Default)]
struct NpcHudCache {
    last_chest_open: bool,
    last_chest_selected: usize,
    last_trade_open: bool,
    last_trade_selected: usize,
    last_experiment_open: bool,
    last_fingerprint: u64,
}

#[derive(Resource, Default)]
struct InventoryGridCache {
    last_slots_fingerprint: u64,
    last_selected: usize,
    last_open: bool,
}

/// Frame counter for grow_crops throttling in farming.rs.
#[derive(Resource, Default)]
pub struct FarmGrowthFrame(pub u32);

/// Timer for weather gameplay effects throttling.
#[derive(Resource)]
pub struct WeatherEffectsTimer(pub f32);

impl Default for WeatherEffectsTimer {
    fn default() -> Self {
        Self(0.0)
    }
}

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FloatingTextRequest>()
            .insert_resource(PauseState::default())
            .insert_resource(CurrentBiome::default())
            .insert_resource(ExploredBiomes::default())
            .insert_resource(BarDisplayState::default())
            .insert_resource(FloatingTextQueue::default())
            .insert_resource(FishingCatchFlash::default())
            .insert_resource(StatusHudCache::default())
            .insert_resource(MainHudCache::default())
            .insert_resource(FishingHudCache::default())
            .insert_resource(QuestLogHudCache::default())
            .insert_resource(StatusEffectsHudCache::default())
            .insert_resource(SkillHudCache::default())
            .insert_resource(NpcHudCache::default())
            .insert_resource(InventoryGridCache::default())
            .insert_resource(FarmGrowthFrame::default())
            .insert_resource(WeatherEffectsTimer::default())
            .insert_resource(PauseMenuState::default())
            .add_systems(Startup, spawn_hud)
            .add_systems(
                Update,
                (
                    toggle_pause,
                    pause_menu_navigation,
                    update_hud,
                    update_status_hud,
                    update_npc_hud,
                    update_feedback_hud,
                    update_inventory_grid,
                    inventory_navigation,
                    update_inventory_equip_panel,
                    update_graphical_hotbar,
                    track_player_biome,
                    update_biome_banner,
                    receive_floating_text_requests,
                    drain_floating_text_queue,
                    floating_text_system,
                    update_fishing_hud,
                    update_quest_log_hud,
                    update_status_effects_hud,
                    update_skill_hud,
                    toggle_panel_visibility,
                ),
            );
    }
}

#[derive(Component)]
pub struct HudText;

#[derive(Component)]
pub struct CraftingHudText;

/// Marker for the crafting panel's parent node (for visibility toggling).
#[derive(Component)]
pub struct CraftingPanelRoot;

#[derive(Component)]
pub struct StatusHudText;

#[derive(Component)]
pub struct NpcHudText;

/// Marker for the NPC panel's parent node (for visibility toggling).
#[derive(Component)]
pub struct NpcPanelRoot;

#[derive(Component)]
pub struct FeedbackHudText;

/// Marker for the feedback panel's parent node (for visibility toggling).
#[derive(Component)]
pub struct FeedbackPanelRoot;

#[derive(Component)]
pub struct InventoryPanelText;

#[derive(Component)]
pub struct HealthBarFill;

#[derive(Component)]
pub struct HungerBarFill;

// --- Pause Menu Components ---

/// Root container for the pause menu overlay.
#[derive(Component)]
pub struct PauseMenuPanel;

/// Individual selectable menu item in the pause menu.
#[derive(Component)]
pub struct PauseMenuItem {
    pub index: usize,
}

/// Tracks which pause menu item is selected.
#[derive(Resource, Default)]
pub struct PauseMenuState {
    pub selected: usize,
}

/// Marker for the fishing panel's parent node (for visibility toggling).
#[derive(Component)]
pub struct FishingPanelRoot;

/// Marker for the quest log panel's parent node (for visibility toggling).
#[derive(Component)]
pub struct QuestLogPanelRoot;

/// Marker for the skill panel's parent node (for visibility toggling).
#[derive(Component)]
pub struct SkillPanelRoot;

#[derive(Component)]
pub struct HotbarSlotUI {
    pub index: usize,
}

#[derive(Component)]
pub struct HotbarSlotColor;

#[derive(Component)]
pub struct HotbarSlotLabel;

#[derive(Component)]
pub struct HotbarTooltipText;

#[derive(Component)]
pub struct FishingHudText;

#[derive(Component)]
pub struct QuestLogHudText;

#[derive(Component)]
pub struct StatusEffectsHudText;

#[derive(Component)]
pub struct SkillHudText;

// --- Graphical Inventory Grid Components ---

/// Root container for the graphical inventory panel.
#[derive(Component)]
pub struct InventoryGrid;

/// Marks an individual inventory slot UI node. `index` is 0-35.
#[derive(Component)]
pub struct InventorySlotUI {
    pub index: usize,
}

/// The inner colored square representing the item type.
#[derive(Component)]
pub struct InventoryItemColor {
    pub index: usize,
}

/// Small text label at the bottom of a slot (abbreviated item name).
#[derive(Component)]
pub struct InventorySlotLabel {
    pub index: usize,
}

/// Count badge (top-right) showing "x5" for stackable items.
#[derive(Component)]
pub struct InventoryCountBadge {
    pub index: usize,
}

/// Thin durability bar at the bottom of a slot for tools.
#[derive(Component)]
pub struct InventoryDurabilityBar {
    pub index: usize,
}

/// Image child that displays the actual item sprite icon (if available).
#[derive(Component)]
pub struct InventoryItemIcon {
    pub index: usize,
}

/// Tooltip text below the grid showing details for the selected slot.
#[derive(Component)]
pub struct InventoryTooltip;

/// Footer text with controls help.
#[derive(Component)]
pub struct InventoryFooter;

/// Full-screen dimming overlay shown when inventory is open.
#[derive(Component)]
pub struct InventoryDimOverlay;

/// Marks one of the 3 equipment display slots (Helmet / Chest / Shield).
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum InventoryEquipSlotUI {
    Helmet,
    Chest,
    Shield,
}

/// The name-label inside an equipment display slot.
#[derive(Component)]
pub struct InventoryEquipLabel {
    pub slot: InventoryEquipSlotUI,
}

/// Returns the sprite handle for an item type, if one exists in GameAssets.
fn item_sprite(item: &ItemType, assets: &GameAssets) -> Option<Handle<Image>> {
    match item {
        // Weapons
        ItemType::WoodSword => Some(assets.weapon_wood_sword.clone()),
        ItemType::IronSword => Some(assets.weapon_iron_sword.clone()),
        ItemType::SteelSword => Some(assets.weapon_steel_sword.clone()),
        ItemType::AncientBlade => Some(assets.weapon_ancient_blade.clone()),
        ItemType::FlameBlade => Some(assets.weapon_flame_blade.clone()),
        ItemType::FrostBlade => Some(assets.weapon_frost_blade.clone()),
        ItemType::VenomBlade => Some(assets.weapon_venom_blade.clone()),
        ItemType::LifestealBlade => Some(assets.weapon_lifesteal_blade.clone()),
        ItemType::WoodBow => Some(assets.weapon_wood_bow.clone()),
        ItemType::Arrow => Some(assets.weapon_arrow.clone()),
        // Tools
        ItemType::WoodAxe => Some(assets.tool_wood_axe.clone()),
        ItemType::StoneAxe => Some(assets.tool_stone_axe.clone()),
        ItemType::IronAxe => Some(assets.tool_iron_axe.clone()),
        ItemType::SteelAxe => Some(assets.tool_steel_axe.clone()),
        ItemType::WoodPickaxe => Some(assets.tool_wood_pickaxe.clone()),
        ItemType::StonePickaxe => Some(assets.tool_stone_pickaxe.clone()),
        ItemType::IronPickaxe => Some(assets.tool_iron_pickaxe.clone()),
        ItemType::SteelPickaxe => Some(assets.tool_steel_pickaxe.clone()),
        ItemType::AncientPickaxe => Some(assets.tool_ancient_pickaxe.clone()),
        ItemType::Hoe => Some(assets.tool_hoe.clone()),
        ItemType::FishingRod => Some(assets.tool_fishing_rod.clone()),
        ItemType::SteelFishingRod => Some(assets.tool_steel_fishing_rod.clone()),
        ItemType::FishBait => Some(assets.tool_fish_bait.clone()),
        // Buildings / placeables
        ItemType::Campfire => Some(assets.campfire.clone()),
        ItemType::WoodFloor => Some(assets.wood_floor.clone()),
        ItemType::Workbench => Some(assets.workbench.clone()),
        ItemType::WoodWall => Some(assets.wood_wall.clone()),
        ItemType::WoodDoor => Some(assets.wood_door.clone()),
        ItemType::WoodRoof => Some(assets.roof_thatch.clone()),
        ItemType::WoodFence => Some(assets.wood_fence.clone()),
        ItemType::Chest => Some(assets.chest_building.clone()),
        ItemType::Torch => Some(assets.torch_wall.clone()),
        ItemType::StoneWall => Some(assets.stone_wall.clone()),
        ItemType::Forge => Some(assets.forge.clone()),
        ItemType::Anvil => Some(assets.forge.clone()),
        ItemType::Bed => Some(assets.bed.clone()),
        ItemType::StoneFloor => Some(assets.stone_floor.clone()),
        ItemType::StoneDoor => Some(assets.stone_door_building.clone()),
        ItemType::StoneRoof => Some(assets.stone_roof.clone()),
        ItemType::MetalWall => Some(assets.metal_wall.clone()),
        ItemType::MetalDoor => Some(assets.metal_door.clone()),
        ItemType::WoodStairs => Some(assets.wood_stairs.clone()),
        ItemType::StoneStairs => Some(assets.stone_stairs.clone()),
        ItemType::Ladder => Some(assets.ladder.clone()),
        ItemType::BrickWall => Some(assets.brick_wall.clone()),
        ItemType::ReinforcedStoneWall => Some(assets.reinforced_wall.clone()),
        ItemType::WoodHalfWall => Some(assets.wood_half_wall.clone()),
        ItemType::WoodWallWindow => Some(assets.wood_wall_window.clone()),
        ItemType::AdvancedForge => Some(assets.advanced_forge.clone()),
        ItemType::AlchemyLab => Some(assets.alchemy_lab.clone()),
        ItemType::AncientWorkstation => Some(assets.ancient_workstation.clone()),
        ItemType::EnchantingTable => Some(assets.enchanting_table.clone()),
        ItemType::FishSmoker => Some(assets.fish_smoker.clone()),
        ItemType::PetHouse => Some(assets.pet_house.clone()),
        ItemType::DisplayCase => Some(assets.display_case.clone()),
        ItemType::Lantern => Some(assets.lantern.clone()),
        ItemType::Bookshelf => Some(assets.bookshelf.clone()),
        ItemType::WeaponRack => Some(assets.weapon_rack.clone()),
        ItemType::CookingPot => Some(assets.cooking_pot.clone()),
        ItemType::RainCollector => Some(assets.rain_collector.clone()),
        ItemType::TrophyMount => Some(assets.trophy_mount.clone()),
        ItemType::AutoSmelterItem => Some(assets.auto_smelter.clone()),
        ItemType::CropSprinklerItem => Some(assets.crop_sprinkler.clone()),
        ItemType::AlarmBellItem => Some(assets.alarm_bell.clone()),
        // Armor
        ItemType::IronHelmet => Some(assets.armor_iron_helmet.clone()),
        ItemType::IronChestplate => Some(assets.armor_iron_chestplate.clone()),
        ItemType::SteelArmor => Some(assets.armor_steel.clone()),
        ItemType::AncientArmor => Some(assets.armor_ancient.clone()),
        ItemType::IronShield => Some(assets.armor_iron_shield.clone()),
        ItemType::WoodShield => Some(assets.armor_wood_shield.clone()),
        // Raw materials
        ItemType::Wood => Some(assets.item_wood.clone()),
        ItemType::Stone => Some(assets.item_stone.clone()),
        ItemType::PlantFiber => Some(assets.item_plant_fiber.clone()),
        ItemType::Stick => Some(assets.item_stick.clone()),
        ItemType::Flint => Some(assets.item_flint.clone()),
        ItemType::WoodPlank => Some(assets.item_wood_plank.clone()),
        ItemType::Rope => Some(assets.item_rope.clone()),
        ItemType::Coal => Some(assets.item_coal.clone()),
        ItemType::IronOre => Some(assets.item_iron_ore.clone()),
        ItemType::IronIngot => Some(assets.item_iron_ingot.clone()),
        ItemType::SteelAlloy => Some(assets.item_steel_alloy.clone()),
        ItemType::StoneBlock => Some(assets.item_stone_block.clone()),
        ItemType::AncientCore => Some(assets.item_ancient_core.clone()),
        ItemType::Gemstone => Some(assets.item_gemstone.clone()),
        ItemType::RareHerb => Some(assets.item_rare_herb.clone()),
        ItemType::Brick => Some(assets.item_brick.clone()),
        ItemType::ReinforcedStoneBlock => Some(assets.item_reinforced_stone_block.clone()),
        ItemType::CrystalShard => Some(assets.item_crystal_shard.clone()),
        // Seeds
        ItemType::WheatSeed => Some(assets.seed_wheat.clone()),
        ItemType::CarrotSeed => Some(assets.seed_carrot.clone()),
        ItemType::TomatoSeed => Some(assets.seed_tomato.clone()),
        ItemType::PumpkinSeed => Some(assets.seed_pumpkin.clone()),
        ItemType::CornSeed => Some(assets.seed_corn.clone()),
        ItemType::PotatoSeed => Some(assets.seed_potato.clone()),
        ItemType::MelonSeed => Some(assets.seed_melon.clone()),
        ItemType::RiceSeed => Some(assets.seed_rice.clone()),
        ItemType::PepperSeed => Some(assets.seed_pepper.clone()),
        ItemType::OnionSeed => Some(assets.seed_onion.clone()),
        ItemType::FlaxSeed => Some(assets.seed_flax.clone()),
        ItemType::SugarcaneSeed => Some(assets.seed_sugarcane.clone()),
        // Raw crops
        ItemType::Wheat => Some(assets.crop_wheat.clone()),
        ItemType::Carrot => Some(assets.crop_carrot.clone()),
        ItemType::Tomato => Some(assets.crop_tomato.clone()),
        ItemType::Pumpkin => Some(assets.pumpkin.clone()),
        ItemType::Corn => Some(assets.crop_corn.clone()),
        ItemType::Potato => Some(assets.crop_potato.clone()),
        ItemType::Melon => Some(assets.crop_melon.clone()),
        ItemType::Rice => Some(assets.crop_rice.clone()),
        ItemType::Pepper => Some(assets.crop_pepper.clone()),
        ItemType::Onion => Some(assets.crop_onion.clone()),
        ItemType::Flax => Some(assets.crop_flax.clone()),
        ItemType::Sugarcane => Some(assets.crop_sugarcane.clone()),
        // Cooked food & processed
        ItemType::Berry => Some(assets.bush_berry.clone()),
        ItemType::CookedBerry => Some(assets.food_cooked_berry.clone()),
        ItemType::BakedWheat => Some(assets.food_baked_wheat.clone()),
        ItemType::CookedCarrot => Some(assets.food_cooked_carrot.clone()),
        ItemType::CookedTomato => Some(assets.food_cooked_tomato.clone()),
        ItemType::BakedPumpkin => Some(assets.food_baked_pumpkin.clone()),
        ItemType::RoastedCorn => Some(assets.food_roasted_corn.clone()),
        ItemType::BakedPotato => Some(assets.food_baked_potato.clone()),
        ItemType::MelonSlice => Some(assets.food_melon_slice.clone()),
        ItemType::CookedRice => Some(assets.food_cooked_rice.clone()),
        ItemType::RoastedPepper => Some(assets.food_roasted_pepper.clone()),
        ItemType::CookedOnion => Some(assets.food_cooked_onion.clone()),
        ItemType::LinenCloth => Some(assets.food_linen_cloth.clone()),
        ItemType::Sugar => Some(assets.food_sugar.clone()),
        // Biome items
        ItemType::CactusFiber => Some(assets.biome_cactus_fiber.clone()),
        ItemType::IceShard => Some(assets.biome_ice_shard.clone()),
        ItemType::MushroomCap => Some(assets.biome_mushroom_cap.clone()),
        ItemType::Spore => Some(assets.biome_spore.clone()),
        ItemType::Reed => Some(assets.biome_reed.clone()),
        ItemType::Sulfur => Some(assets.biome_sulfur.clone()),
        ItemType::AlpineHerb => Some(assets.biome_alpine_herb.clone()),
        ItemType::Peat => Some(assets.biome_peat.clone()),
        ItemType::ObsidianShard => Some(assets.biome_obsidian_shard.clone()),
        ItemType::SandstoneChip => Some(assets.biome_sandstone_chip.clone()),
        ItemType::Shell => Some(assets.biome_shell.clone()),
        ItemType::Seaweed => Some(assets.biome_seaweed.clone()),
        ItemType::BioGel => Some(assets.biome_bio_gel.clone()),
        ItemType::EchoStoneFragment => Some(assets.biome_echo_stone.clone()),
        ItemType::FrozenOre => Some(assets.biome_frozen_ore.clone()),
        ItemType::CaveSlime => Some(assets.biome_cave_slime.clone()),
        ItemType::SpiderSilk => Some(assets.biome_spider_silk.clone()),
        // Potions
        ItemType::HealthPotion => Some(assets.potion_health.clone()),
        ItemType::SpeedPotion => Some(assets.potion_speed.clone()),
        ItemType::StrengthPotion => Some(assets.potion_strength.clone()),
        // Essences
        ItemType::FireEssence => Some(assets.essence_fire.clone()),
        ItemType::IceEssence => Some(assets.essence_ice.clone()),
        ItemType::VenomEssence => Some(assets.essence_venom.clone()),
        ItemType::LifeEssence => Some(assets.essence_life.clone()),
        // Fish
        ItemType::RawTrout => Some(assets.fish_raw_trout.clone()),
        ItemType::RawSalmon => Some(assets.fish_raw_salmon.clone()),
        ItemType::RawCatfish => Some(assets.fish_raw_catfish.clone()),
        ItemType::RawPufferfish => Some(assets.fish_raw_pufferfish.clone()),
        ItemType::RawEel => Some(assets.fish_raw_eel.clone()),
        ItemType::RawCrab => Some(assets.fish_raw_crab.clone()),
        ItemType::CookedTrout => Some(assets.fish_cooked_trout.clone()),
        ItemType::CookedSalmon => Some(assets.fish_cooked_salmon.clone()),
        ItemType::CookedCatfish => Some(assets.fish_cooked_catfish.clone()),
        ItemType::CookedEel => Some(assets.fish_cooked_eel.clone()),
        ItemType::CrabMeat => Some(assets.fish_crab_meat.clone()),
        // Quest / boss drop items
        ItemType::Blueprint => Some(assets.quest_blueprint.clone()),
        ItemType::GuardianHeart => Some(assets.quest_guardian_heart.clone()),
        ItemType::SwampEssence => Some(assets.quest_swamp_essence.clone()),
        ItemType::WyrmScale => Some(assets.quest_wyrm_scale.clone()),
        ItemType::FrostGem => Some(assets.quest_frost_gem.clone()),
        ItemType::MagmaCore => Some(assets.quest_magma_core.clone()),
        ItemType::FungalSporeEssence => Some(assets.quest_fungal_spore_essence.clone()),
        ItemType::CrystalHeart => Some(assets.quest_crystal_heart.clone()),
        ItemType::JournalPage => Some(assets.quest_journal_page.clone()),
        ItemType::CoralEssence => Some(assets.quest_coral_essence.clone()),
        ItemType::TitanBone => Some(assets.quest_titan_bone.clone()),
        ItemType::PetCollar => Some(assets.quest_pet_collar.clone()),
        ItemType::PetFood => Some(assets.quest_pet_food.clone()),
    }
}

fn spawn_hud(mut commands: Commands, theme: Res<EtherealTheme>) {
    // Root UI container
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        })
        .with_children(|parent| {
            // Status area: Top-Left (HP/Hunger bars + stats)
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(10.0),
                        left: Val::Px(10.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(theme.panel_bg()),
                    BorderColor(theme.panel_border(false)),
                ))
                .with_children(|status_root| {
                    // HP Bar
                    status_root
                        .spawn((
                            Node {
                                width: Val::Px(160.0),
                                height: Val::Px(12.0),
                                margin: UiRect::bottom(Val::Px(4.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.3, 0.05, 0.05, 0.6)),
                            BorderColor(Color::srgba(0.5, 0.2, 0.2, 0.5)),
                        ))
                        .with_children(|bar| {
                            bar.spawn((
                                HealthBarFill,
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(theme.healing),
                            ));
                        });

                    // Hunger Bar
                    status_root
                        .spawn((
                            Node {
                                width: Val::Px(160.0),
                                height: Val::Px(12.0),
                                margin: UiRect::bottom(Val::Px(6.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.2, 0.15, 0.02, 0.6)),
                            BorderColor(Color::srgba(0.4, 0.35, 0.1, 0.5)),
                        ))
                        .with_children(|bar| {
                            bar.spawn((
                                HungerBarFill,
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(theme.accent_gold),
                            ));
                        });

                    // Status Text
                    status_root.spawn((
                        StatusHudText,
                        Text::new(""),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(theme.hud_label_color()),
                    ));
                });

            // Main HUD text (day info, build mode) - small, top-left under status
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(104.0),
                        left: Val::Px(14.0),
                        padding: UiRect::all(Val::Px(6.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(theme.panel_bg()),
                    BorderColor(theme.panel_border(false)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        HudText,
                        Text::new(""),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(theme.hud_label_color()),
                    ));
                });

            // Graphical Hotbar: Bottom-Center — 9 colored slots
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(14.0),
                        left: Val::Percent(50.0),
                        margin: UiRect::left(Val::Px(-210.0)), // Center: 9 slots * 42px + gaps / 2
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(6.0),
                        padding: UiRect::all(Val::Px(5.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(theme.panel_bg()),
                    BorderColor(theme.panel_border(false)),
                ))
                .with_children(|hotbar| {
                    for i in 0..9 {
                        hotbar
                            .spawn((
                                HotbarSlotUI { index: i },
                                Node {
                                    width: Val::Px(40.0),
                                    height: Val::Px(40.0),
                                    border: UiRect::all(Val::Px(1.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::End,
                                    padding: UiRect::all(Val::Px(2.0)),
                                    flex_direction: FlexDirection::Column,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.8)),
                                BorderColor(Color::srgba(0.25, 0.25, 0.35, 0.5)),
                            ))
                            .with_children(|slot| {
                                // Colored item indicator
                                slot.spawn((
                                    HotbarSlotColor,
                                    Node {
                                        width: Val::Px(30.0),
                                        height: Val::Px(24.0),
                                        margin: UiRect::bottom(Val::Px(2.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                                ));
                                // Label (item name / count)
                                slot.spawn((
                                    HotbarSlotLabel,
                                    Text::new(""),
                                    TextFont {
                                        font_size: 10.0,
                                        ..default()
                                    },
                                    TextColor(theme.hud_label_color()),
                                ));
                            });
                    }
                });

            // Hotbar tooltip: selected item name (below hotbar)
            parent.spawn((
                HotbarTooltipText,
                Text::new(""),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(theme.hud_label_color()),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(58.0),
                    left: Val::Percent(50.0),
                    margin: UiRect::left(Val::Px(-100.0)),
                    max_width: Val::Px(200.0),
                    ..default()
                },
            ));

            // Crafting Menu: Right
            parent
                .spawn((
                    CraftingPanelRoot,
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(12.0),
                        right: Val::Px(12.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        max_width: Val::Px(320.0),
                        ..default()
                    },
                    BackgroundColor(theme.panel_bg()),
                    BorderColor(theme.panel_border(false)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        CraftingHudText,
                        Text::new(""),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(theme.hud_label_color()),
                    ));
                });

            // NPC / Experiment Panel: Far-Right
            parent
                .spawn((
                    NpcPanelRoot,
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(12.0),
                        right: Val::Px(346.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(theme.panel_bg()),
                    BorderColor(theme.panel_border(false)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        NpcHudText,
                        Text::new(""),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(theme.hud_primary_text()),
                    ));
                });

            // Feedback: Bottom-Left
            parent
                .spawn((
                    FeedbackPanelRoot,
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(52.0),
                        left: Val::Px(12.0),
                        padding: UiRect::all(Val::Px(8.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(theme.panel_bg()),
                    BorderColor(theme.panel_border(false)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        FeedbackHudText,
                        Text::new(""),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(theme.hud_primary_text()),
                    ));
                });

            // Inventory dim overlay: full-screen dark veil (behind the panel)
            parent.spawn((
                InventoryDimOverlay,
                Node {
                    display: Display::None,
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            ));

            // Inventory Panel: Center — Graphical Grid (9x4 = 36 slots)
            // Layout math: 9 slots * 48px + 8 gaps * 4px = 464px grid
            // + 32px padding = 496px wide. Half = 248px offset.
            // Height: equip(76) + title(32) + grid(4*52) + tooltip(28) + footer(22) + padding(32) ≈ 398px. Half ≈ 200px.
            parent
                .spawn((
                    InventoryGrid,
                    InventoryPanelText, // kept for Without<> filter compat
                    Node {
                        display: Display::None,
                        position_type: PositionType::Absolute,
                        top: Val::Percent(50.0),
                        left: Val::Percent(50.0),
                        margin: UiRect {
                            left: Val::Px(-256.0),
                            top: Val::Px(-220.0),
                            ..default()
                        },
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(16.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        row_gap: Val::Px(6.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.03, 0.02, 0.07, 0.97)),
                    BorderColor(theme.panel_border(true)),
                ))
                .with_children(|panel| {
                    // Title
                    panel.spawn((
                        Text::new("INVENTORY"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(theme.accent_gold),
                        Node {
                            margin: UiRect::bottom(Val::Px(4.0)),
                            ..default()
                        },
                    ));

                    // Equipment slots row: Helmet / Chest / Shield
                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(10.0),
                            margin: UiRect::bottom(Val::Px(6.0)),
                            ..default()
                        })
                        .with_children(|equip_row| {
                            for (slot_type, label) in [
                                (InventoryEquipSlotUI::Helmet, "Helmet"),
                                (InventoryEquipSlotUI::Chest, "Chest"),
                                (InventoryEquipSlotUI::Shield, "Shield"),
                            ] {
                                equip_row
                                    .spawn(Node {
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::Center,
                                        row_gap: Val::Px(2.0),
                                        ..default()
                                    })
                                    .with_children(|col| {
                                        // Slot box
                                        col.spawn((
                                            slot_type,
                                            Node {
                                                width: Val::Px(52.0),
                                                height: Val::Px(52.0),
                                                border: UiRect::all(Val::Px(1.0)),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgba(0.06, 0.04, 0.12, 0.9)),
                                            BorderColor(Color::srgba(0.5, 0.4, 0.15, 0.7)),
                                        ))
                                        .with_children(
                                            |slot_inner| {
                                                slot_inner.spawn((
                                                    InventoryEquipLabel { slot: slot_type },
                                                    Text::new("—"),
                                                    TextFont {
                                                        font_size: 9.0,
                                                        ..default()
                                                    },
                                                    TextColor(theme.accent_slate),
                                                ));
                                            },
                                        );
                                        // Slot type label below the box
                                        col.spawn((
                                            Text::new(label),
                                            TextFont {
                                                font_size: 9.0,
                                                ..default()
                                            },
                                            TextColor(theme.accent_slate),
                                        ));
                                    });
                            }
                        });

                    // Thin gold separator line between equipment and grid
                    panel.spawn((
                        Node {
                            width: Val::Px(464.0),
                            height: Val::Px(1.0),
                            margin: UiRect::bottom(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.7, 0.6, 0.2, 0.4)),
                    ));

                    // Grid container: 9 columns x 4 rows
                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            ..default()
                        })
                        .with_children(|grid| {
                            for row in 0..4 {
                                // Extra gap between hotbar row (row 0) and main inventory
                                if row == 1 {
                                    grid.spawn((
                                        Node {
                                            width: Val::Px(464.0),
                                            height: Val::Px(1.0),
                                            margin: UiRect::vertical(Val::Px(2.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgba(0.7, 0.6, 0.2, 0.35)),
                                    ));
                                }
                                grid.spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(4.0),
                                    ..default()
                                })
                                .with_children(|row_node| {
                                    for col in 0..9 {
                                        let idx = row * 9 + col;
                                        // Hotbar row gets a slightly warmer tint
                                        let slot_bg = if row == 0 {
                                            Color::srgba(0.07, 0.06, 0.14, 0.85)
                                        } else {
                                            Color::srgba(0.05, 0.04, 0.10, 0.85)
                                        };
                                        let slot_border = if row == 0 {
                                            Color::srgba(0.4, 0.35, 0.2, 0.5)
                                        } else {
                                            Color::srgba(0.28, 0.28, 0.38, 0.55)
                                        };
                                        // Slot container
                                        row_node
                                            .spawn((
                                                InventorySlotUI { index: idx },
                                                Node {
                                                    width: Val::Px(48.0),
                                                    height: Val::Px(48.0),
                                                    border: UiRect::all(Val::Px(1.0)),
                                                    flex_direction: FlexDirection::Column,
                                                    align_items: AlignItems::Center,
                                                    justify_content: JustifyContent::Center,
                                                    ..default()
                                                },
                                                BackgroundColor(slot_bg),
                                                BorderColor(slot_border),
                                            ))
                                            .with_children(|slot| {
                                                // Inner colored square (item indicator — hidden when sprite is available)
                                                slot.spawn((
                                                    InventoryItemColor { index: idx },
                                                    Node {
                                                        width: Val::Px(34.0),
                                                        height: Val::Px(34.0),
                                                        position_type: PositionType::Absolute,
                                                        top: Val::Px(5.0),
                                                        left: Val::Px(7.0),
                                                        ..default()
                                                    },
                                                    BackgroundColor(Color::NONE),
                                                ));
                                                // Sprite icon overlay (shown when item has a sprite)
                                                slot.spawn((
                                                    InventoryItemIcon { index: idx },
                                                    ImageNode::default(),
                                                    Node {
                                                        width: Val::Px(36.0),
                                                        height: Val::Px(36.0),
                                                        position_type: PositionType::Absolute,
                                                        top: Val::Px(4.0),
                                                        left: Val::Px(6.0),
                                                        ..default()
                                                    },
                                                ));
                                                // Count badge (top-right)
                                                slot.spawn((
                                                    InventoryCountBadge { index: idx },
                                                    Text::new(""),
                                                    TextFont {
                                                        font_size: 10.0,
                                                        ..default()
                                                    },
                                                    TextColor(Color::WHITE),
                                                    Node {
                                                        position_type: PositionType::Absolute,
                                                        top: Val::Px(1.0),
                                                        right: Val::Px(2.0),
                                                        ..default()
                                                    },
                                                ));
                                                // Item name label (bottom)
                                                slot.spawn((
                                                    InventorySlotLabel { index: idx },
                                                    Text::new(""),
                                                    TextFont {
                                                        font_size: 9.0,
                                                        ..default()
                                                    },
                                                    TextColor(theme.accent_slate),
                                                    Node {
                                                        position_type: PositionType::Absolute,
                                                        bottom: Val::Px(2.0),
                                                        ..default()
                                                    },
                                                ));
                                                // Durability bar (thin bar at very bottom)
                                                slot.spawn((
                                                    InventoryDurabilityBar { index: idx },
                                                    Node {
                                                        position_type: PositionType::Absolute,
                                                        bottom: Val::Px(0.0),
                                                        left: Val::Px(2.0),
                                                        width: Val::Px(0.0),
                                                        height: Val::Px(3.0),
                                                        ..default()
                                                    },
                                                    BackgroundColor(Color::NONE),
                                                ));
                                            });
                                    }
                                });
                            }
                        });

                    // Tooltip (selected slot details)
                    panel.spawn((
                        InventoryTooltip,
                        Text::new(""),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(theme.accent_gold),
                        Node {
                            margin: UiRect::top(Val::Px(6.0)),
                            max_width: Val::Px(480.0),
                            ..default()
                        },
                    ));

                    // Footer (controls)
                    panel.spawn((
                        InventoryFooter,
                        Text::new("[I] Close  [1-9] Hotbar  [Arrows] Navigate  [R] Equip"),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(theme.accent_slate),
                        Node {
                            margin: UiRect::top(Val::Px(2.0)),
                            ..default()
                        },
                    ));
                });

            // Biome Banner: Center-Top (no panel — just floating text)
            parent.spawn((
                BiomeBannerText,
                Text::new(""),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(theme.hud_primary_text()),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(20.0),
                    left: Val::Percent(45.0),
                    ..default()
                },
            ));

            // Fishing HUD: Bottom-center above hotbar
            parent
                .spawn((
                    FishingPanelRoot,
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(68.0),
                        left: Val::Percent(50.0),
                        margin: UiRect::left(Val::Px(-160.0)),
                        max_width: Val::Px(320.0),
                        padding: UiRect::all(Val::Px(6.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(theme.panel_bg()),
                    BorderColor(theme.panel_border(false)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        FishingHudText,
                        Text::new(""),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(theme.hud_primary_text()),
                    ));
                });

            // Quest Log HUD: Center panel (like inventory, toggled by J)
            parent
                .spawn((
                    QuestLogPanelRoot,
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(80.0),
                        left: Val::Percent(25.0),
                        padding: UiRect::all(Val::Px(12.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        max_width: Val::Px(400.0),
                        ..default()
                    },
                    BackgroundColor(theme.panel_bg()),
                    BorderColor(theme.panel_border(false)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        QuestLogHudText,
                        Text::new(""),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(theme.hud_label_color()),
                    ));
                });

            // Status Effects HUD: Top-left below status panel
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(160.0),
                        left: Val::Px(14.0),
                        padding: UiRect::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        StatusEffectsHudText,
                        Text::new(""),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(theme.hud_primary_text()),
                    ));
                });

            // Skill Panel: Center-Left (toggled by K)
            parent
                .spawn((
                    SkillPanelRoot,
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(160.0),
                        left: Val::Percent(5.0),
                        padding: UiRect::all(Val::Px(12.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        max_width: Val::Px(340.0),
                        ..default()
                    },
                    BackgroundColor(theme.panel_bg()),
                    BorderColor(theme.panel_border(false)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        SkillHudText,
                        Text::new(""),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(theme.hud_label_color()),
                    ));
                });

            // Pause Menu: Centered overlay (hidden by default)
            parent
                .spawn((
                    PauseMenuPanel,
                    Node {
                        display: Display::None,
                        position_type: PositionType::Absolute,
                        top: Val::Percent(50.0),
                        left: Val::Percent(50.0),
                        margin: UiRect {
                            left: Val::Px(-160.0),
                            top: Val::Px(-180.0),
                            ..default()
                        },
                        width: Val::Px(320.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(24.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.02, 0.02, 0.06, 0.92)),
                    BorderColor(Color::srgba(0.4, 0.35, 0.2, 0.8)),
                ))
                .with_children(|menu| {
                    // Title
                    menu.spawn((
                        Text::new("PAUSED"),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(theme.accent_gold),
                        Node {
                            margin: UiRect::bottom(Val::Px(16.0)),
                            ..default()
                        },
                    ));

                    let items = [
                        "Resume Game",
                        "Save Game",
                        "Load Game",
                        "Settings",
                        "Controls",
                        "Quit to Menu",
                    ];

                    for (i, label) in items.iter().enumerate() {
                        menu.spawn((
                            PauseMenuItem { index: i },
                            Node {
                                width: Val::Px(240.0),
                                height: Val::Px(36.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::vertical(Val::Px(2.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.08, 0.08, 0.14, 0.8)),
                            BorderColor(Color::srgba(0.25, 0.25, 0.35, 0.5)),
                        ))
                        .with_children(|item_node| {
                            item_node.spawn((
                                Text::new(label.to_string()),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(theme.hud_label_color()),
                            ));
                        });
                    }

                    // Volume control hint
                    menu.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(theme.accent_slate),
                        Node {
                            margin: UiRect::top(Val::Px(12.0)),
                            ..default()
                        },
                    ));
                });
        });
}

fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pause_state: ResMut<PauseState>,
    mut cycle: ResMut<crate::daynight::DayNightCycle>,
    chest_ui: Res<ChestUI>,
    trade_menu: Res<TradeMenu>,
    menu: Res<MainMenuActive>,
    controls_overlay: Res<ControlsOverlay>,
    mut pause_menu_state: ResMut<PauseMenuState>,
    mut pause_panel_query: Query<&mut Node, With<PauseMenuPanel>>,
    settings_state: Res<crate::settings::SettingsMenuState>,
    save_slot_browser_state: Res<crate::saveslots::SaveSlotBrowserState>,
) {
    if menu.active {
        return;
    }
    if save_slot_browser_state.open {
        return;
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        if chest_ui.is_open || trade_menu.is_open || controls_overlay.is_visible {
            return;
        }
        // If settings menu is open, let settings handle its own ESC
        if settings_state.is_open {
            return;
        }
        pause_state.paused = !pause_state.paused;
        cycle.paused = pause_state.paused;

        // Show/hide pause menu panel
        if let Ok(mut node) = pause_panel_query.get_single_mut() {
            node.display = if pause_state.paused {
                Display::Flex
            } else {
                Display::None
            };
        }
        // Reset selection when opening
        if pause_state.paused {
            pause_menu_state.selected = 0;
        }
    }
}

const BAR_LERP_SPEED: f32 = 5.0; // ~0.2s to catch up

fn update_status_hud(
    time: Res<Time>,
    player_query: Query<(&Health, &Hunger, Option<&ActiveBuff>), With<Player>>,
    mut status_query: Query<&mut Text, With<StatusHudText>>,
    mut health_fill_query: Query<&mut Node, (With<HealthBarFill>, Without<HungerBarFill>)>,
    mut hunger_fill_query: Query<&mut Node, (With<HungerBarFill>, Without<HealthBarFill>)>,
    mut bar_display: ResMut<BarDisplayState>,
    save_msg: Res<SaveMessage>,
    armor: Res<ArmorSlots>,
    inventory: Res<Inventory>,
    pet_query: Query<&Pet>,
    _theme: Res<EtherealTheme>,
    mut cache: ResMut<StatusHudCache>,
) {
    let Ok((health, hunger, active_buff)) = player_query.get_single() else {
        return;
    };

    // Bars still lerp every frame (cheap, no allocations)
    let target_health = (health.current / health.max).clamp(0.0, 1.0);
    let target_hunger = (hunger.current / hunger.max).clamp(0.0, 1.0);
    let dt = time.delta_secs();
    bar_display.health_frac +=
        (target_health - bar_display.health_frac) * (BAR_LERP_SPEED * dt).min(1.0);
    bar_display.hunger_frac +=
        (target_hunger - bar_display.hunger_frac) * (BAR_LERP_SPEED * dt).min(1.0);

    if let Ok(mut node) = health_fill_query.get_single_mut() {
        node.width = Val::Percent(bar_display.health_frac * 100.0);
    }
    if let Ok(mut node) = hunger_fill_query.get_single_mut() {
        node.width = Val::Percent(bar_display.hunger_frac * 100.0);
    }

    // Fingerprint current values (integers — avoids float epsilon issues)
    let hp_i = health.current as i32;
    let max_hp_i = health.max as i32;
    let hunger_i = hunger.current as i32;
    let max_hunger_i = hunger.max as i32;
    let armor_val = armor.total_armor();
    let atk = inventory
        .selected_item()
        .and_then(|s| s.item.weapon_damage())
        .unwrap_or(5.0);
    let atk_i = atk as i32;

    let buff_key = active_buff.map(|b| {
        (
            b.buff_type,
            ((b.magnitude - 1.0) * 100.0) as i32,
            b.remaining as i32,
        )
    });

    let pet_exists = pet_query.get_single().is_ok();
    let pet_happiness_i = pet_query
        .get_single()
        .map(|p| p.happiness as i32)
        .unwrap_or(-1);

    let save_text = &save_msg.text;

    // Compare with cache — skip rebuild if nothing changed
    if hp_i == cache.last_hp_i
        && max_hp_i == cache.last_max_hp_i
        && hunger_i == cache.last_hunger_i
        && max_hunger_i == cache.last_max_hunger_i
        && armor_val == cache.last_armor
        && atk_i == cache.last_atk_i
        && buff_key == cache.last_buff
        && pet_exists == cache.last_pet_exists
        && pet_happiness_i == cache.last_pet_happiness_i
        && *save_text == cache.last_save_text
    {
        return;
    }

    // Update cache
    cache.last_hp_i = hp_i;
    cache.last_max_hp_i = max_hp_i;
    cache.last_hunger_i = hunger_i;
    cache.last_max_hunger_i = max_hunger_i;
    cache.last_armor = armor_val;
    cache.last_atk_i = atk_i;
    cache.last_buff = buff_key;
    cache.last_pet_exists = pet_exists;
    cache.last_pet_happiness_i = pet_happiness_i;
    cache.last_save_text = save_text.clone();

    let Ok(mut text) = status_query.get_single_mut() else {
        return;
    };

    let mut lines = vec![
        format!(
            "{:.0}/{:.0} HP | {:.0}/{:.0} FOOD",
            health.current, health.max, hunger.current, hunger.max
        ),
        format!("ARMOR: {} | ATK: {:.0}", armor_val, atk),
    ];

    if let Some(buff) = active_buff {
        let buff_name = match buff.buff_type {
            BuffType::Speed => "Speed",
            BuffType::Strength => "Strength",
            BuffType::Regen => "Regen",
        };
        lines.push(format!(
            "[{}] +{:.0}% ({:.0}s)",
            buff_name,
            (buff.magnitude - 1.0) * 100.0,
            buff.remaining
        ));
    }

    if let Ok(pet) = pet_query.get_single() {
        let max_h = pet.pet_type.max_happiness();
        let frac = (pet.happiness / max_h).clamp(0.0, 1.0);
        let bar_len = 8;
        let filled = (frac * bar_len as f32).round() as usize;
        let bar: String = "=".repeat(filled) + &"-".repeat(bar_len - filled);
        let warning = if pet.happiness < 30.0 {
            " !!UNHAPPY!!"
        } else {
            ""
        };
        lines.push(format!(
            "Pet: {} [{}] {:.0}%{}",
            pet.pet_type.display_name(),
            bar,
            frac * 100.0,
            warning
        ));
    }

    if !save_msg.text.is_empty() {
        lines.push(save_msg.text.clone());
    }

    **text = lines.join("\n");
}

fn update_hud(
    inventory: Res<Inventory>,
    crafting: Res<CraftingSystem>,
    cycle: Res<DayNightCycle>,
    building_state: Res<BuildingState>,
    season: Res<SeasonCycle>,
    weather: Res<WeatherSystem>,
    _lore_registry: Res<LoreRegistry>,
    pause_state: Res<PauseState>,
    tech_tree: Res<TechTree>,
    mut hud_query: Query<
        &mut Text,
        (
            With<HudText>,
            Without<CraftingHudText>,
            Without<StatusHudText>,
            Without<NpcHudText>,
            Without<FeedbackHudText>,
            Without<InventoryPanelText>,
        ),
    >,
    mut craft_hud_query: Query<
        &mut Text,
        (
            With<CraftingHudText>,
            Without<HudText>,
            Without<StatusHudText>,
            Without<NpcHudText>,
            Without<FeedbackHudText>,
            Without<InventoryPanelText>,
        ),
    >,
    station_query: Query<(&CraftingStation, &Transform), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
    mut cache: ResMut<MainHudCache>,
) {
    if let Ok(mut text) = hud_query.get_single_mut() {
        if pause_state.paused {
            if !cache.last_paused {
                cache.last_paused = true;
                **text = String::new();
            }
            return;
        }

        let weather_str = match weather.current {
            crate::weather::Weather::Clear => "",
            crate::weather::Weather::Rain => " Rain",
            crate::weather::Weather::Snow => " Snow",
            crate::weather::Weather::Storm => " STORM",
            crate::weather::Weather::Fog => " Fog",
            crate::weather::Weather::Blizzard => " BLIZZARD",
        };

        let forecast_str = match weather.next_weather {
            Some(next) => format!(" -> {}", next.name()),
            None => String::new(),
        };

        let phase_name = cycle.phase_name().to_string();
        let season_name = season.current.name().to_string();
        let build_name = if building_state.active {
            building_state.selected_type.name().to_string()
        } else {
            String::new()
        };

        // Compare with cache
        if !cache.last_paused
            && cycle.day_count == cache.last_day
            && phase_name == cache.last_phase
            && season_name == cache.last_season
            && weather_str == cache.last_weather
            && forecast_str == cache.last_forecast
            && building_state.active == cache.last_build_active
            && build_name == cache.last_build_name
        {
            return;
        }

        cache.last_paused = false;
        cache.last_day = cycle.day_count;
        cache.last_phase = phase_name;
        cache.last_season = season_name;
        cache.last_weather = weather_str.to_string();
        cache.last_forecast = forecast_str.clone();
        cache.last_build_active = building_state.active;
        cache.last_build_name = build_name.clone();

        let mut lines = Vec::new();

        if building_state.active {
            lines.push(format!("BUILD: {} | Q Cycle | RClick Place", build_name));
        }

        lines.push(format!(
            "Day {} {} | {}{}{}",
            cycle.day_count, cache.last_phase, cache.last_season, weather_str, forecast_str,
        ));

        **text = lines.join("\n");
    }

    // Crafting HUD (US-013 — improved with ingredient availability)
    if let Ok(mut text) = craft_hud_query.get_single_mut() {
        if !crafting.is_open {
            **text = String::new();
            return;
        }

        let mut near_workbench = false;
        let mut near_forge = false;
        let mut near_campfire = false;
        let mut near_advanced_forge = false;
        let mut near_ancient = false;
        if let Ok(player_tf) = player_query.get_single() {
            let player_pos = player_tf.translation.truncate();
            for (station, tf) in station_query.iter() {
                let dist = player_pos.distance(tf.translation.truncate());
                if dist <= 64.0 {
                    match station.tier {
                        CraftingTier::Workbench => near_workbench = true,
                        CraftingTier::Forge => near_forge = true,
                        CraftingTier::Campfire => near_campfire = true,
                        CraftingTier::AdvancedForge => near_advanced_forge = true,
                        CraftingTier::Ancient => near_ancient = true,
                        CraftingTier::Hand => {}
                    }
                }
            }
        }
        let visible = crafting.recipes_visible_at_stations(
            near_workbench,
            near_forge,
            near_campfire,
            near_advanced_forge,
            near_ancient,
            &tech_tree,
        );

        let mut lines = vec!["=== CRAFTING (C to close) ===".to_string()];
        lines.push(format!(
            "RP: {}  [U] Unlock (when locked recipe shown)",
            tech_tree.research_points
        ));
        {
            let mut stations = vec!["Hand"];
            if near_workbench {
                stations.push("Workbench");
            }
            if near_campfire {
                stations.push("Campfire");
            }
            if near_forge {
                stations.push("Forge");
            }
            if near_advanced_forge {
                stations.push("AdvForge");
            }
            if near_ancient {
                stations.push("Ancient");
            }
            lines.push(format!("Stations: {}", stations.join(", ")));
        }
        lines.push(String::new());

        for (display_idx, (recipe_idx, locked)) in visible.iter().enumerate() {
            let recipe = &crafting.recipes[*recipe_idx];
            let is_selected = display_idx == crafting.selected_recipe;
            let sel_marker = if is_selected { "> " } else { "  " };

            if *locked {
                let hint = tech_tree.unlock_hint(recipe.tech_key);
                lines.push(format!(
                    "{}[LOCKED] {}  Unlock: {}",
                    sel_marker, recipe.name, hint
                ));
                // Show prerequisite chain if selected and has unmet prerequisites
                if is_selected {
                    if let Some(key) = recipe.tech_key {
                        let prereq_hint = tech_tree.prerequisite_hint(key);
                        if !prereq_hint.is_empty() {
                            lines.push(format!("    {}", prereq_hint));
                        }
                    }
                }
            } else if is_selected && !*locked {
                let craftable = crafting.can_craft(*recipe_idx, &inventory);
                let craft_tag = if craftable { " [READY]" } else { "" };
                lines.push(format!(
                    "{}{}{} [SELECTED]",
                    sel_marker, recipe.name, craft_tag
                ));
            } else if !*locked {
                let can_craft = crafting.can_craft(*recipe_idx, &inventory);
                let status = if can_craft { "" } else { " [missing]" };
                lines.push(format!(
                    "{}{} {}{}",
                    sel_marker,
                    recipe.tier.label(),
                    recipe.name,
                    status
                ));
            }

            if is_selected {
                for (item, count) in &recipe.inputs {
                    let have = inventory.count_items(*item);
                    let has_enough = have >= *count;
                    let mark = if has_enough { "\u{2713}" } else { "\u{2717}" };
                    lines.push(format!(
                        "    {} {} x{} (have {})",
                        mark,
                        item.display_name(),
                        count,
                        have
                    ));
                }
                let (out_item, out_count) = recipe.output;
                if out_count > 1 {
                    lines.push(format!("    -> {} x{}", out_item.display_name(), out_count));
                } else {
                    lines.push(format!("    -> {}", out_item.display_name()));
                }
            }
        }

        if visible.is_empty() {
            lines.push("  (no recipes at current stations)".to_string());
        }

        lines.push(String::new());
        lines.push("[Up/Down] Select  [Enter] Craft  [U] Unlock (locked)  [C] Close".into());

        **text = lines.join("\n");
    }
}

/// Renders the chest UI, trader trade menu, or experiment UI on the secondary right-side panel.
fn update_npc_hud(
    trade_menu: Res<TradeMenu>,
    experiment_slots: Res<ExperimentSlots>,
    chest_ui: Res<ChestUI>,
    inventory: Res<Inventory>,
    trader_query: Query<&Trader>,
    chest_query: Query<&ChestStorage>,
    mut npc_hud_query: Query<&mut Text, With<NpcHudText>>,
    mut cache: ResMut<NpcHudCache>,
) {
    let Ok(mut text) = npc_hud_query.get_single_mut() else {
        return;
    };

    // Build a cheap fingerprint of current NPC UI state
    let mut fp: u64 = 0;

    if chest_ui.is_open {
        fp = fp.wrapping_mul(31).wrapping_add(1);
        fp = fp
            .wrapping_mul(31)
            .wrapping_add(chest_ui.selected_slot as u64);
        if let Some(entity) = chest_ui.target_entity {
            if let Ok(chest) = chest_query.get(entity) {
                for slot in &chest.slots {
                    match slot {
                        Some(s) => {
                            fp = fp.wrapping_mul(31).wrapping_add(s.count as u64);
                            fp = fp
                                .wrapping_mul(31)
                                .wrapping_add(s.durability.unwrap_or(0) as u64);
                        }
                        None => {
                            fp = fp.wrapping_mul(31).wrapping_add(9999);
                        }
                    }
                }
            }
        }
    } else if experiment_slots.is_open {
        fp = fp.wrapping_mul(31).wrapping_add(2);
        fp = fp
            .wrapping_mul(31)
            .wrapping_add(experiment_slots.slot_a.map(|_| 1u64).unwrap_or(0));
        fp = fp
            .wrapping_mul(31)
            .wrapping_add(experiment_slots.slot_b.map(|_| 1u64).unwrap_or(0));
    } else if trade_menu.is_open {
        fp = fp.wrapping_mul(31).wrapping_add(3);
        fp = fp
            .wrapping_mul(31)
            .wrapping_add(trade_menu.selected_offer as u64);
    }

    if chest_ui.is_open == cache.last_chest_open
        && chest_ui.selected_slot == cache.last_chest_selected
        && trade_menu.is_open == cache.last_trade_open
        && trade_menu.selected_offer == cache.last_trade_selected
        && experiment_slots.is_open == cache.last_experiment_open
        && fp == cache.last_fingerprint
    {
        return;
    }

    cache.last_chest_open = chest_ui.is_open;
    cache.last_chest_selected = chest_ui.selected_slot;
    cache.last_trade_open = trade_menu.is_open;
    cache.last_trade_selected = trade_menu.selected_offer;
    cache.last_experiment_open = experiment_slots.is_open;
    cache.last_fingerprint = fp;

    // Chest UI takes highest priority if open
    if chest_ui.is_open {
        if let Some(entity) = chest_ui.target_entity {
            if let Ok(chest) = chest_query.get(entity) {
                let mut lines = vec!["=== CHEST ===".to_string(), String::new()];

                for (i, slot) in chest.slots.iter().enumerate() {
                    let marker = if i == chest_ui.selected_slot {
                        "> "
                    } else {
                        "  "
                    };
                    let slot_text = match slot {
                        Some(s) => {
                            if let Some(dur) = s.durability {
                                let max_dur = s.item.max_durability().unwrap_or(dur);
                                format!(
                                    "{}{:2}. {} ({}/{})",
                                    marker,
                                    i + 1,
                                    s.item.display_name(),
                                    dur,
                                    max_dur
                                )
                            } else {
                                format!(
                                    "{}{:2}. {} x{}",
                                    marker,
                                    i + 1,
                                    s.item.display_name(),
                                    s.count
                                )
                            }
                        }
                        None => format!("{}{:2}. (empty)", marker, i + 1),
                    };
                    lines.push(slot_text);
                }

                lines.push(String::new());
                lines.push("1-9: Store hotbar item | Up/Down+Enter: Take | E: Close".to_string());
                **text = lines.join("\n");
                return;
            }
        }
    }

    // Experiment UI takes priority if open
    if experiment_slots.is_open {
        let slot_a_name = experiment_slots
            .slot_a
            .map(|i| i.display_name().to_string())
            .unwrap_or_else(|| "---".to_string());
        let slot_b_name = experiment_slots
            .slot_b
            .map(|i| i.display_name().to_string())
            .unwrap_or_else(|| "---".to_string());

        let lines = vec![
            "== EXPERIMENT TABLE ==".to_string(),
            String::new(),
            format!("Slot A: {}", slot_a_name),
            format!("Slot B: {}", slot_b_name),
            String::new(),
            "[1] Assign selected item to Slot A".to_string(),
            "[2] Assign selected item to Slot B".to_string(),
            "[Enter] Attempt combination".to_string(),
            "[X] Close".to_string(),
        ];
        **text = lines.join("\n");
        return;
    }

    // Trade menu
    if trade_menu.is_open {
        if let Some(entity) = trade_menu.trader_entity {
            if let Ok(trader) = trader_query.get(entity) {
                let mut lines = vec!["== WANDERING TRADER ==".to_string(), String::new()];

                for (i, offer) in trader.offers.iter().enumerate() {
                    let marker = if i == trade_menu.selected_offer {
                        "> "
                    } else {
                        "  "
                    };
                    let status = if offer.sold {
                        " [SOLD]".to_string()
                    } else {
                        let can_afford = inventory.has_items(offer.cost_item, offer.cost_count);
                        if can_afford {
                            String::new()
                        } else {
                            " [need more]".to_string()
                        }
                    };
                    lines.push(format!(
                        "{}{}  for {} x{}{}",
                        marker,
                        offer.item_for_sale.display_name(),
                        offer.cost_item.display_name(),
                        offer.cost_count,
                        status,
                    ));
                }

                lines.push(String::new());
                lines.push("[Up/Down] Select  [Enter] Buy  [Esc] Close".to_string());
                **text = lines.join("\n");
                return;
            }
        }
    }

    **text = String::new();
}

/// Shows ephemeral feedback messages: lore discoveries, hermit dialogue, NPC dialogue, experiment results.
fn update_feedback_hud(
    lore_msg: Res<LoreMessage>,
    hermit_display: Res<HermitDialogueDisplay>,
    npc_display: Res<NpcDialogueDisplay>,
    experiment_msg: Res<ExperimentMessage>,
    mut feedback_query: Query<&mut Text, With<FeedbackHudText>>,
) {
    // Only update when any of the source resources changed
    if !experiment_msg.is_changed()
        && !lore_msg.is_changed()
        && !hermit_display.is_changed()
        && !npc_display.is_changed()
    {
        return;
    }

    let Ok(mut text) = feedback_query.get_single_mut() else {
        return;
    };

    // Priority order: experiment > lore > hermit > npc
    if !experiment_msg.text.is_empty() {
        **text = experiment_msg.text.clone();
    } else if !lore_msg.text.is_empty() {
        **text = lore_msg.text.clone();
    } else if !hermit_display.text.is_empty() {
        **text = hermit_display.text.clone();
    } else if !npc_display.text.is_empty() {
        **text = npc_display.text.clone();
    } else {
        **text = String::new();
    }
}

/// Toggles the inventory grid visibility and updates all slot visuals.
fn update_inventory_grid(
    inventory: Res<Inventory>,
    game_assets: Res<GameAssets>,
    mut grid_query: Query<&mut Node, With<InventoryGrid>>,
    mut dim_query: Query<&mut Node, (With<InventoryDimOverlay>, Without<InventoryGrid>)>,
    mut slot_query: Query<
        (&InventorySlotUI, &mut BackgroundColor, &mut BorderColor),
        Without<InventoryGrid>,
    >,
    mut item_color_query: Query<
        (&InventoryItemColor, &mut BackgroundColor),
        (
            Without<InventorySlotUI>,
            Without<InventoryDurabilityBar>,
            Without<InventoryItemIcon>,
        ),
    >,
    mut icon_query: Query<
        (&InventoryItemIcon, &mut ImageNode, &mut Node),
        (
            Without<InventoryGrid>,
            Without<InventorySlotUI>,
            Without<InventoryItemColor>,
            Without<InventoryDurabilityBar>,
        ),
    >,
    mut durability_query: Query<
        (&InventoryDurabilityBar, &mut Node, &mut BackgroundColor),
        (
            Without<InventoryGrid>,
            Without<InventorySlotUI>,
            Without<InventoryItemColor>,
            Without<InventoryItemIcon>,
        ),
    >,
    mut text_queries: ParamSet<(
        Query<(&InventorySlotLabel, &mut Text)>,
        Query<(&InventoryCountBadge, &mut Text)>,
        Query<&mut Text, With<InventoryTooltip>>,
    )>,
    mut inv_cache: ResMut<InventoryGridCache>,
) {
    // Toggle visibility
    let vis = if inventory.is_open {
        Display::Flex
    } else {
        Display::None
    };
    if let Ok(mut grid_node) = grid_query.get_single_mut() {
        grid_node.display = vis;
    }
    if let Ok(mut dim_node) = dim_query.get_single_mut() {
        dim_node.display = vis;
    }

    if !inventory.is_open {
        inv_cache.last_open = false;
        return;
    }

    // Build cheap fingerprint of inventory state
    let mut fp: u64 = 0;
    for slot in &inventory.slots {
        match slot {
            Some(s) => {
                fp = fp.wrapping_mul(31).wrapping_add(s.count as u64);
                fp = fp
                    .wrapping_mul(31)
                    .wrapping_add(s.durability.unwrap_or(0) as u64);
                fp = fp.wrapping_mul(31).wrapping_add(1); // occupied marker
            }
            None => {
                fp = fp.wrapping_mul(31).wrapping_add(9999);
            }
        }
    }

    if inv_cache.last_open
        && inventory.selected_slot == inv_cache.last_selected
        && fp == inv_cache.last_slots_fingerprint
    {
        return;
    }
    inv_cache.last_open = true;
    inv_cache.last_selected = inventory.selected_slot;
    inv_cache.last_slots_fingerprint = fp;

    let selected = inventory.selected_slot;

    // Update slot backgrounds and borders
    for (slot_ui, mut bg, mut border) in slot_query.iter_mut() {
        let idx = slot_ui.index;
        let is_selected = idx == selected;
        let is_occupied = inventory.slots[idx].is_some();
        let is_hotbar = idx < 9;

        *bg = BackgroundColor(if is_selected {
            Color::srgba(0.18, 0.14, 0.28, 0.97)
        } else if is_occupied && is_hotbar {
            Color::srgba(0.10, 0.08, 0.18, 0.92)
        } else if is_occupied {
            Color::srgba(0.09, 0.07, 0.16, 0.92)
        } else if is_hotbar {
            Color::srgba(0.07, 0.06, 0.14, 0.85)
        } else {
            Color::srgba(0.05, 0.04, 0.10, 0.85)
        });

        *border = BorderColor(if is_selected {
            Color::srgba(0.95, 0.80, 0.25, 1.0)
        } else if is_hotbar {
            Color::srgba(0.4, 0.35, 0.2, 0.55)
        } else {
            Color::srgba(0.28, 0.28, 0.38, 0.55)
        });
    }

    // Update item sprite icons
    for (icon, mut image_node, mut icon_node) in icon_query.iter_mut() {
        let idx = icon.index;
        if let Some(slot) = &inventory.slots[idx] {
            if let Some(sprite_handle) = item_sprite(&slot.item, &game_assets) {
                image_node.image = sprite_handle;
                icon_node.display = Display::Flex;
            } else {
                image_node.image = Handle::default();
                icon_node.display = Display::None;
            }
        } else {
            image_node.image = Handle::default();
            icon_node.display = Display::None;
        }
    }

    // Update item color indicators (hidden when a sprite icon is available)
    for (item_color, mut bg) in item_color_query.iter_mut() {
        let idx = item_color.index;
        if let Some(slot) = &inventory.slots[idx] {
            if item_sprite(&slot.item, &game_assets).is_some() {
                // Sprite icon handles this slot — hide the color swatch
                *bg = BackgroundColor(Color::NONE);
            } else {
                // No sprite available — show the colored rectangle fallback
                let c = dropped_item_color(slot.item).to_srgba();
                *bg = BackgroundColor(Color::srgba(c.red, c.green, c.blue, 0.85));
            }
        } else {
            *bg = BackgroundColor(Color::NONE);
        }
    }

    // Update durability bars
    for (dur_bar, mut node, mut bg) in durability_query.iter_mut() {
        let idx = dur_bar.index;
        if let Some(slot) = &inventory.slots[idx] {
            if let Some(dur) = slot.durability {
                let max_dur = slot.item.max_durability().unwrap_or(1);
                let frac = dur as f32 / max_dur as f32;
                node.width = Val::Px(44.0 * frac);
                let bar_color = if frac > 0.5 {
                    Color::srgb(0.2, 0.8, 0.3)
                } else if frac > 0.25 {
                    Color::srgb(0.9, 0.8, 0.2)
                } else {
                    Color::srgb(0.9, 0.2, 0.2)
                };
                *bg = BackgroundColor(bar_color);
            } else {
                node.width = Val::Px(0.0);
                *bg = BackgroundColor(Color::NONE);
            }
        } else {
            node.width = Val::Px(0.0);
            *bg = BackgroundColor(Color::NONE);
        }
    }

    // Update slot labels (p0)
    {
        let mut label_query = text_queries.p0();
        for (label, mut text) in label_query.iter_mut() {
            let idx = label.index;
            if let Some(slot) = &inventory.slots[idx] {
                let name: String = slot.item.display_name().chars().take(6).collect();
                **text = name;
            } else {
                **text = String::new();
            }
        }
    }

    // Update count badges (p1)
    {
        let mut badge_query = text_queries.p1();
        for (badge, mut text) in badge_query.iter_mut() {
            let idx = badge.index;
            if let Some(slot) = &inventory.slots[idx] {
                if slot.count > 1 {
                    **text = format!("x{}", slot.count);
                } else {
                    **text = String::new();
                }
            } else {
                **text = String::new();
            }
        }
    }

    // Update tooltip (p2)
    {
        let mut tooltip_query = text_queries.p2();
        if let Ok(mut text) = tooltip_query.get_single_mut() {
            if let Some(slot) = &inventory.slots[selected] {
                let name = slot.item.display_name();
                let mut detail = name.to_string();
                if slot.count > 1 {
                    detail.push_str(&format!("  (x{})", slot.count));
                }
                if let Some(dur) = slot.durability {
                    let max_dur = slot.item.max_durability().unwrap_or(dur);
                    detail.push_str(&format!("  [{}/{}]", dur, max_dur));
                }
                if let Some(dmg) = slot.item.weapon_damage() {
                    detail.push_str(&format!("  ATK: {:.0}", dmg));
                }
                let armor = slot.item.armor_value();
                if armor > 0 {
                    detail.push_str(&format!("  DEF: {}", armor));
                }
                let shield = slot.item.shield_value();
                if shield > 0 {
                    detail.push_str(&format!("  BLOCK: {}", shield));
                }
                if let Some(tier) = match slot.item.tool_tier() {
                    0 => None,
                    t => Some(t),
                } {
                    if slot.item.weapon_damage().is_none() {
                        detail.push_str(&format!("  Tier {}", tier));
                    }
                }
                **text = detail;
            } else {
                **text = format!("Slot {} - Empty", selected + 1);
            }
        }
    }
}

/// Arrow-key navigation for inventory grid when open.
fn inventory_navigation(keyboard: Res<ButtonInput<KeyCode>>, mut inventory: ResMut<Inventory>) {
    if !inventory.is_open {
        return;
    }

    let cols: usize = 9;
    let total: usize = 36;
    let cur = inventory.selected_slot;

    if keyboard.just_pressed(KeyCode::ArrowRight) {
        let new = if cur + 1 < total { cur + 1 } else { cur };
        inventory.selected_slot = new;
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        let new = if cur > 0 { cur - 1 } else { cur };
        inventory.selected_slot = new;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        let new = cur + cols;
        if new < total {
            inventory.selected_slot = new;
        }
    }
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        if cur >= cols {
            inventory.selected_slot = cur - cols;
        }
    }
}

/// Updates the equipment slot display panel inside the inventory.
fn update_inventory_equip_panel(
    inventory: Res<Inventory>,
    armor: Res<crate::player::ArmorSlots>,
    mut label_query: Query<(&InventoryEquipLabel, &mut Text)>,
    mut slot_query: Query<(&InventoryEquipSlotUI, &mut BorderColor)>,
) {
    if !inventory.is_open {
        return;
    }
    for (label, mut text) in label_query.iter_mut() {
        let item_opt: Option<crate::inventory::ItemType> = match label.slot {
            InventoryEquipSlotUI::Helmet => armor.helmet,
            InventoryEquipSlotUI::Chest => armor.chest,
            InventoryEquipSlotUI::Shield => armor.shield,
        };
        **text = match item_opt {
            Some(item) => {
                let name = item.display_name();
                let short: String = name.chars().take(7).collect();
                short
            }
            None => "—".to_string(),
        };
    }
    for (slot_type, mut border) in slot_query.iter_mut() {
        let equipped = match slot_type {
            InventoryEquipSlotUI::Helmet => armor.helmet.is_some(),
            InventoryEquipSlotUI::Chest => armor.chest.is_some(),
            InventoryEquipSlotUI::Shield => armor.shield.is_some(),
        };
        *border = BorderColor(if equipped {
            Color::srgba(0.9, 0.75, 0.3, 0.85)
        } else {
            Color::srgba(0.5, 0.4, 0.15, 0.7)
        });
    }
}

/// Updates the graphical hotbar slots: highlights selected, shows item color and label; updates tooltip.
fn update_graphical_hotbar(
    inventory: Res<Inventory>,
    mut slot_query: Query<(&HotbarSlotUI, &mut BorderColor, &Children)>,
    mut color_query: Query<&mut BackgroundColor, With<HotbarSlotColor>>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<HotbarSlotLabel>>,
        Query<&mut Text, With<HotbarTooltipText>>,
    )>,
) {
    // Update tooltip first (uses p1)
    {
        let mut tooltip_query = text_queries.p1();
        if let Ok(mut tooltip) = tooltip_query.get_single_mut() {
            let idx = inventory.selected_slot;
            **tooltip = inventory
                .slots
                .get(idx)
                .and_then(|s| s.as_ref())
                .map(|s| s.item.display_name().to_string())
                .unwrap_or_else(|| format!("Slot {}", idx + 1));
        }
    }

    // Collect slot data to avoid borrow conflicts
    let slot_data: Vec<(usize, bool, Vec<Entity>)> = slot_query
        .iter_mut()
        .map(|(slot_ui, _, children)| {
            (
                slot_ui.index,
                slot_ui.index == inventory.selected_slot,
                children.iter().copied().collect(),
            )
        })
        .collect();

    // Update borders
    for (slot_ui, mut border, _) in slot_query.iter_mut() {
        if slot_ui.index == inventory.selected_slot {
            *border = BorderColor(Color::srgba(0.9, 0.75, 0.3, 0.9));
        } else {
            *border = BorderColor(Color::srgba(0.25, 0.25, 0.35, 0.5));
        }
    }

    // Update child color blocks
    for (i, _, ref children) in &slot_data {
        for child in children {
            if let Ok(mut bg) = color_query.get_mut(*child) {
                if let Some(slot) = &inventory.slots[*i] {
                    let item_color = dropped_item_color(slot.item);
                    let c = item_color.to_srgba();
                    *bg = BackgroundColor(Color::srgba(c.red, c.green, c.blue, 0.7));
                } else {
                    *bg = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0));
                }
            }
        }
    }

    // Update labels (uses p0)
    {
        let mut label_query = text_queries.p0();
        for (i, _, ref children) in &slot_data {
            for child in children {
                if let Ok(mut text) = label_query.get_mut(*child) {
                    if let Some(slot) = &inventory.slots[*i] {
                        let name: String = slot.item.display_name().chars().take(6).collect();
                        if slot.count > 1 {
                            **text = format!("{} x{}", name, slot.count);
                        } else {
                            **text = name;
                        }
                    } else {
                        **text = format!("{}", *i + 1);
                    }
                }
            }
        }
    }
}

/// Returns a human-readable name for a biome.
fn biome_display_name(biome: Biome) -> &'static str {
    match biome {
        Biome::Forest => "Forest",
        Biome::Coastal => "Coastal",
        Biome::Swamp => "Swamp",
        Biome::Desert => "Desert",
        Biome::Tundra => "Tundra",
        Biome::Volcanic => "Volcanic Wastes",
        Biome::Fungal => "Fungal Groves",
        Biome::CrystalCave => "Crystal Caverns",
        Biome::Mountain => "Mountains",
    }
}

/// Determines which biome the player is currently standing in and starts
/// the banner timer whenever the biome changes. First-time discovery plays a sound and longer banner.
fn track_player_biome(
    player_query: Query<&Transform, With<Player>>,
    chunk_query: Query<&Chunk>,
    mut current_biome: ResMut<CurrentBiome>,
    mut explored_biomes: ResMut<ExploredBiomes>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    let Ok(player_tf) = player_query.get_single() else {
        return;
    };

    let chunk_x = (player_tf.translation.x / CHUNK_WORLD_SIZE).floor() as i32;
    let chunk_y = (player_tf.translation.y / CHUNK_WORLD_SIZE).floor() as i32;

    for chunk in chunk_query.iter() {
        if chunk.position.x == chunk_x && chunk.position.y == chunk_y {
            let new_biome = chunk.biome;
            if current_biome.biome != Some(new_biome) {
                current_biome.biome = Some(new_biome);
                let first_time = !explored_biomes.set.contains(&new_biome);
                if first_time {
                    explored_biomes.set.insert(new_biome);
                    sound_events.send(SoundEvent::Discovery);
                    current_biome.display_timer = 4.5; // Longer banner for first discovery
                } else {
                    current_biome.display_timer = 3.0;
                }
            }
            return;
        }
    }
}

/// Fades the biome banner text over 3 seconds and hides it when done.
fn update_biome_banner(
    time: Res<Time>,
    mut current_biome: ResMut<CurrentBiome>,
    mut banner_query: Query<(&mut Text, &mut TextColor), With<BiomeBannerText>>,
) {
    let Ok((mut text, mut color)) = banner_query.get_single_mut() else {
        return;
    };

    if current_biome.display_timer > 0.0 {
        // Set text to biome name
        if let Some(biome) = current_biome.biome {
            **text = biome_display_name(biome).to_string();
        }

        // Fade: full opacity for first 2 seconds, then fade out over the last 1 second
        let alpha = if current_biome.display_timer > 1.0 {
            1.0
        } else {
            current_biome.display_timer
        };
        let mut c = color.0.to_srgba();
        c.alpha = alpha;
        *color = TextColor(Color::Srgba(c));

        current_biome.display_timer -= time.delta_secs();
    } else {
        // Hide banner
        let mut c = color.0.to_srgba();
        c.alpha = 0.0;
        *color = TextColor(Color::Srgba(c));
        **text = String::new();
    }
}

// --- US-028: Floating Text ---

/// Request to show floating text; queued and shown with delay to avoid overlap.
#[derive(Event)]
pub struct FloatingTextRequest {
    pub text: String,
    pub position: Vec2,
    pub color: Color,
}

/// Queue for floating text so multiple requests in one frame don't overlap illegibly.
#[derive(Resource, Default)]
pub struct FloatingTextQueue {
    pub pending: Vec<(String, Vec2, Color)>,
    pub cooldown: f32,
}

/// World-space floating text that drifts upward and fades out.
/// Used for damage numbers, item pickup notifications, etc.
#[derive(Component)]
pub struct FloatingText {
    pub timer: f32,
    pub max_timer: f32,
    pub velocity: Vec2,
}

const FLOATING_TEXT_QUEUE_INTERVAL: f32 = 0.35;

/// Spawns a floating text entity in world space at the given position.
pub fn spawn_floating_text(commands: &mut Commands, text: &str, position: Vec2, color: Color) {
    commands.spawn((
        FloatingText {
            timer: 1.5,
            max_timer: 1.5,
            velocity: Vec2::new(0.0, 30.0),
        },
        Text2d::new(text.to_string()),
        TextFont {
            font_size: choose_floating_font_size(color),
            ..default()
        },
        TextColor(color),
        Transform::from_xyz(position.x, position.y + 8.0, 100.0),
    ));
}

fn receive_floating_text_requests(
    mut events: EventReader<FloatingTextRequest>,
    mut queue: ResMut<FloatingTextQueue>,
) {
    for ev in events.read() {
        queue.pending.push((ev.text.clone(), ev.position, ev.color));
    }
}

fn drain_floating_text_queue(
    time: Res<Time>,
    mut commands: Commands,
    mut queue: ResMut<FloatingTextQueue>,
) {
    queue.cooldown -= time.delta_secs();
    if queue.cooldown <= 0.0 {
        if let Some((text, position, color)) = queue.pending.first() {
            spawn_floating_text(&mut commands, text.as_str(), *position, *color);
            queue.pending.remove(0);
            queue.cooldown = FLOATING_TEXT_QUEUE_INTERVAL;
        }
    }
}

fn choose_floating_font_size(color: Color) -> f32 {
    let c = color.to_srgba();
    // Damage/heal numbers: larger
    if c.red > 0.8 && c.green < 0.6 {
        18.0
    // Rare/important pickups: bright gold/white
    } else if c.red > 0.85 && c.green > 0.75 {
        17.0
    } else {
        14.0
    }
}

// ---------------------------------------------------------------------------
// 1A: Fishing HUD
// ---------------------------------------------------------------------------

fn update_fishing_hud(
    time: Res<Time>,
    fishing: Res<FishingState>,
    mut catch_flash: ResMut<FishingCatchFlash>,
    mut fishing_hud_query: Query<&mut Text, With<FishingHudText>>,
    player_query: Query<&Transform, With<Player>>,
    mut floating_text_events: EventWriter<FloatingTextRequest>,
    mut cache: ResMut<FishingHudCache>,
) {
    let Ok(mut text) = fishing_hud_query.get_single_mut() else {
        return;
    };

    // Tick catch flash timer
    if catch_flash.timer > 0.0 {
        catch_flash.timer -= time.delta_secs();
    }

    // Build fingerprint of current phase
    let phase_str = match fishing.phase {
        FishingPhase::Idle => "idle",
        FishingPhase::Casting => "casting",
        FishingPhase::Waiting => "waiting",
        FishingPhase::Hooked => "hooked",
        FishingPhase::Reeling => "reeling",
        FishingPhase::Caught => "caught",
    };
    let reel_pct = (fishing.reel_progress * 100.0) as u32;
    let hook_window_i = (fishing.hook_window * 10.0) as i32;
    let dots_idx = ((time.elapsed_secs() * 2.0) as u32) % 4;
    let flash_active = catch_flash.timer > 0.0;

    if phase_str == cache.last_phase
        && reel_pct == cache.last_reel_pct
        && hook_window_i == cache.last_hook_window_i
        && dots_idx == cache.last_dots_idx
        && flash_active == cache.last_catch_flash_active
    {
        return;
    }

    cache.last_phase = phase_str.to_string();
    cache.last_reel_pct = reel_pct;
    cache.last_hook_window_i = hook_window_i;
    cache.last_dots_idx = dots_idx;
    cache.last_catch_flash_active = flash_active;

    match fishing.phase {
        FishingPhase::Idle => {
            if catch_flash.timer > 0.0 {
                **text = format!("Caught a {}!", catch_flash.fish_name);
            } else {
                **text = String::new();
            }
        }
        FishingPhase::Casting => {
            **text = "Casting...".to_string();
        }
        FishingPhase::Waiting => {
            let dots = match dots_idx {
                0 => "",
                1 => ".",
                2 => "..",
                _ => "...",
            };
            **text = format!("Waiting for a bite{}", dots);
        }
        FishingPhase::Hooked => {
            **text = format!("FISH ON! Press [E]! ({:.1}s)", fishing.hook_window);
        }
        FishingPhase::Reeling => {
            let bar_len = 20;
            let filled = (fishing.reel_progress * bar_len as f32).round() as usize;
            let bar: String = "=".repeat(filled) + &"-".repeat(bar_len - filled);
            let fish_name = fishing
                .target_fish
                .map(|f| fish_type_name(f))
                .unwrap_or("???");
            **text = format!("Reeling: {} [{}] {}%", fish_name, bar, reel_pct);
        }
        FishingPhase::Caught => {
            let fish_name = fishing
                .target_fish
                .map(|f| fish_type_name(f))
                .unwrap_or("Fish");
            catch_flash.fish_name = fish_name.to_string();
            catch_flash.timer = 2.0;
            **text = format!("Caught a {}!", fish_name);

            // Floating text on catch
            if let Ok(player_tf) = player_query.get_single() {
                floating_text_events.send(FloatingTextRequest {
                    text: format!("Caught {}!", fish_name),
                    position: player_tf.translation.truncate(),
                    color: Color::srgb(0.95, 0.85, 0.3),
                });
            }
        }
    }
}

fn fish_type_name(fish: FishType) -> &'static str {
    match fish {
        FishType::Trout => "Trout",
        FishType::Salmon => "Salmon",
        FishType::Catfish => "Catfish",
        FishType::Pufferfish => "Pufferfish",
        FishType::Eel => "Eel",
        FishType::Crab => "Crab",
    }
}

// ---------------------------------------------------------------------------
// 1B: Quest Log HUD
// ---------------------------------------------------------------------------

fn update_quest_log_hud(
    quest_log: Res<QuestLog>,
    dynamic_log: Res<crate::quests::DynamicQuestLog>,
    mut quest_hud_query: Query<&mut Text, With<QuestLogHudText>>,
    mut cache: ResMut<QuestLogHudCache>,
) {
    let Ok(mut text) = quest_hud_query.get_single_mut() else {
        return;
    };

    if !quest_log.is_open {
        if cache.last_open {
            cache.last_open = false;
            **text = String::new();
        }
        return;
    }

    // Cheap fingerprint: hash selected + all progress/completed/claimed values
    let mut fp: u64 = quest_log.selected as u64;
    for q in &quest_log.quests {
        fp = fp.wrapping_mul(31).wrapping_add(q.progress as u64);
        fp = fp
            .wrapping_mul(31)
            .wrapping_add(if q.completed { 1 } else { 0 });
        fp = fp
            .wrapping_mul(31)
            .wrapping_add(if q.claimed { 1 } else { 0 });
    }
    for dq in &dynamic_log.quests {
        fp = fp.wrapping_mul(31).wrapping_add(dq.progress as u64);
        fp = fp
            .wrapping_mul(31)
            .wrapping_add(if dq.completed { 1 } else { 0 });
        fp = fp
            .wrapping_mul(31)
            .wrapping_add(if dq.claimed { 1 } else { 0 });
    }
    fp = fp
        .wrapping_mul(31)
        .wrapping_add(dynamic_log.quests.len() as u64);

    if cache.last_open
        && quest_log.selected == cache.last_selected
        && fp == cache.last_quest_fingerprint
    {
        return;
    }

    cache.last_open = true;
    cache.last_selected = quest_log.selected;
    cache.last_quest_fingerprint = fp;

    let defs = crate::quests::quest_definitions();
    let mut lines = vec!["=== QUEST LOG [J] ===".to_string(), String::new()];

    let static_count = quest_log.quests.len();

    for (i, quest) in quest_log.quests.iter().enumerate() {
        let def = &defs[quest.def_index];
        let sel = if i == quest_log.selected { "> " } else { "  " };

        let status = if quest.claimed {
            format!(
                "[X] {}  {}/{}  CLAIMED",
                def.name, quest.progress, def.target
            )
        } else if quest.completed {
            format!("[!] {}  {}/{}  CLAIM", def.name, quest.progress, def.target)
        } else {
            format!("[ ] {}  {}/{}", def.name, quest.progress, def.target)
        };

        lines.push(format!("{}{}", sel, status));

        if i == quest_log.selected {
            lines.push(format!("    {}", def.description));
            if !def.reward.is_empty() {
                let rewards: Vec<String> = def
                    .reward
                    .iter()
                    .map(|(item, count)| {
                        if *count > 1 {
                            format!("{} x{}", item.display_name(), count)
                        } else {
                            item.display_name().to_string()
                        }
                    })
                    .collect();
                lines.push(format!("    Reward: {}", rewards.join(", ")));
            }
            if def.rp_reward > 0 {
                lines.push(format!("    +{} Research Points", def.rp_reward));
            }
        }
    }

    if !dynamic_log.quests.is_empty() {
        lines.push(String::new());
        lines.push("--- DYNAMIC QUESTS ---".to_string());

        for (di, dq) in dynamic_log.quests.iter().enumerate() {
            let global_idx = static_count + di;
            let sel = if global_idx == quest_log.selected {
                "> "
            } else {
                "  "
            };

            let status = if dq.claimed {
                format!(
                    "[DYNAMIC] [X]  {}/{}  CLAIMED",
                    dq.progress, dq.target_count
                )
            } else if dq.completed {
                format!("[DYNAMIC] [!]  {}/{}  CLAIM", dq.progress, dq.target_count)
            } else {
                format!("[DYNAMIC] [ ]  {}/{}", dq.progress, dq.target_count)
            };

            lines.push(format!("{}{}", sel, status));

            if global_idx == quest_log.selected {
                lines.push(format!("    {}", dq.description));
                lines.push(format!("    Expires: Day {}", dq.expiry_day));
                if !dq.reward_items.is_empty() {
                    let rewards: Vec<String> = dq
                        .reward_items
                        .iter()
                        .map(|(item, count)| {
                            if *count > 1 {
                                format!("{} x{}", item.display_name(), count)
                            } else {
                                item.display_name().to_string()
                            }
                        })
                        .collect();
                    lines.push(format!("    Reward: {}", rewards.join(", ")));
                }
            }
        }
    }

    lines.push(String::new());
    lines.push("[Up/Down] Select  [Enter] Claim  [J] Close".to_string());

    **text = lines.join("\n");
}

// ---------------------------------------------------------------------------
// 1D: Status Effects HUD
// ---------------------------------------------------------------------------

fn update_status_effects_hud(
    player_query: Query<Option<&ActiveStatusEffects>, With<Player>>,
    mut effects_hud_query: Query<&mut Text, With<StatusEffectsHudText>>,
    mut cache: ResMut<StatusEffectsHudCache>,
) {
    let Ok(mut text) = effects_hud_query.get_single_mut() else {
        return;
    };
    let Ok(maybe_effects) = player_query.get_single() else {
        if cache.last_count != 0 {
            cache.last_count = 0;
            cache.last_secs_fingerprint.clear();
            **text = String::new();
        }
        return;
    };

    let Some(active) = maybe_effects else {
        if cache.last_count != 0 {
            cache.last_count = 0;
            cache.last_secs_fingerprint.clear();
            **text = String::new();
        }
        return;
    };

    if active.effects.is_empty() {
        if cache.last_count != 0 {
            cache.last_count = 0;
            cache.last_secs_fingerprint.clear();
            **text = String::new();
        }
        return;
    }

    // Build fingerprint: (effect_type as u8, stacks, remaining whole seconds)
    let current_fp: Vec<(u8, u32, u32)> = active
        .effects
        .iter()
        .map(|e| {
            let type_idx = match e.effect_type {
                StatusEffectType::Poison => 0,
                StatusEffectType::Burn => 1,
                StatusEffectType::Freeze => 2,
                StatusEffectType::Bleed => 3,
                StatusEffectType::Stun => 4,
                StatusEffectType::Regen => 5,
                StatusEffectType::WellFed => 6,
            };
            (type_idx, e.stacks, e.remaining_secs as u32)
        })
        .collect();

    if current_fp == cache.last_secs_fingerprint {
        return;
    }
    cache.last_count = active.effects.len();
    cache.last_secs_fingerprint = current_fp;

    let parts: Vec<String> = active
        .effects
        .iter()
        .map(|e| {
            let name = match e.effect_type {
                StatusEffectType::Poison => "POISON",
                StatusEffectType::Burn => "BURN",
                StatusEffectType::Freeze => "FREEZE",
                StatusEffectType::Bleed => "BLEED",
                StatusEffectType::Stun => "STUN",
                StatusEffectType::Regen => "REGEN",
                StatusEffectType::WellFed => "WELL FED",
            };
            if e.stacks > 1 {
                format!("[{} x{} {:.0}s]", name, e.stacks, e.remaining_secs)
            } else {
                format!("[{} {:.0}s]", name, e.remaining_secs)
            }
        })
        .collect();

    **text = parts.join(" ");
}

// ---------------------------------------------------------------------------
// 4A: Skills HUD
// ---------------------------------------------------------------------------

fn update_skill_hud(
    skill_levels: Res<SkillLevels>,
    mut skill_hud_query: Query<&mut Text, With<SkillHudText>>,
    mut cache: ResMut<SkillHudCache>,
) {
    let Ok(mut text) = skill_hud_query.get_single_mut() else {
        return;
    };

    if !skill_levels.skills_open {
        if cache.last_open {
            cache.last_open = false;
            **text = String::new();
        }
        return;
    }

    let skills = [
        SkillType::Gathering,
        SkillType::Combat,
        SkillType::Fishing,
        SkillType::Farming,
        SkillType::Crafting,
        SkillType::Building,
    ];

    let current_fp: Vec<(u32, u32)> = skills
        .iter()
        .map(|s| {
            let d = skill_levels.get(*s);
            (d.level, d.xp)
        })
        .collect();

    if cache.last_open && current_fp == cache.last_fingerprint {
        return;
    }

    cache.last_open = true;
    cache.last_fingerprint = current_fp;

    let mut lines = vec!["=== SKILLS [K] ===".to_string(), String::new()];

    for skill in &skills {
        let data = skill_levels.get(*skill);
        let frac = data.progress_fraction();
        let pct = (frac * 100.0) as u32;
        let filled = (frac * 8.0) as usize;
        let bar: String = "=".repeat(filled) + &"-".repeat(8 - filled);
        lines.push(format!(
            "{:<10} Lv {:>2}  [{}] {:>3}%",
            skill.display_name(),
            data.level,
            bar,
            pct,
        ));
    }

    lines.push(String::new());
    lines.push("[K] Close".to_string());

    **text = lines.join("\n");
}

/// Moves floating text upward, fades alpha, and despawns when expired.
fn floating_text_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut FloatingText, &mut Transform, &mut TextColor)>,
) {
    let dt = time.delta_secs();
    for (entity, mut ft, mut tf, mut color) in query.iter_mut() {
        // Move upward
        tf.translation.x += ft.velocity.x * dt;
        tf.translation.y += ft.velocity.y * dt;

        // Decrease timer
        ft.timer -= dt;

        // Fade alpha based on remaining time (slower at start, faster at end)
        let t = (ft.timer / ft.max_timer).clamp(0.0, 1.0);
        let alpha = t * t;
        let c = color.0.to_srgba();
        color.0 = Color::srgba(c.red, c.green, c.blue, alpha);

        // Despawn when done
        if ft.timer <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

// ---------------------------------------------------------------------------
// Pause Menu Navigation
// ---------------------------------------------------------------------------

const PAUSE_MENU_ITEMS: usize = 6;

fn pause_menu_navigation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pause_menu_state: ResMut<PauseMenuState>,
    mut menu_item_query: Query<(&PauseMenuItem, &mut BackgroundColor, &mut BorderColor)>,
    mut pause_panel_query: Query<
        &mut Node,
        (
            With<PauseMenuPanel>,
            Without<crate::settings::SettingsPanel>,
        ),
    >,
    mut controls_overlay: ResMut<ControlsOverlay>,
    mut main_menu: ResMut<MainMenuActive>,
    active_slot: Res<crate::saveload::ActiveSaveSlot>,
    mut save_slot_browser_state: ResMut<crate::saveslots::SaveSlotBrowserState>,
    _save_msg: ResMut<SaveMessage>,
    mut cycle: ResMut<crate::daynight::DayNightCycle>,
    mut pause_state: ResMut<PauseState>,
    mut settings_state: ResMut<crate::settings::SettingsMenuState>,
    mut settings_panel_query: Query<
        &mut Node,
        (
            With<crate::settings::SettingsPanel>,
            Without<PauseMenuPanel>,
        ),
    >,
) {
    if !pause_state.paused {
        return;
    }
    // Don't process pause menu navigation while settings is open
    if settings_state.is_open {
        return;
    }

    // Don't process pause menu navigation while save-slot browser is open.
    if save_slot_browser_state.open {
        return;
    }

    // Navigate up/down
    if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
        if pause_menu_state.selected > 0 {
            pause_menu_state.selected -= 1;
        }
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        if pause_menu_state.selected < PAUSE_MENU_ITEMS - 1 {
            pause_menu_state.selected += 1;
        }
    }

    // Update visual highlights
    for (item, mut bg, mut border) in menu_item_query.iter_mut() {
        if item.index == pause_menu_state.selected {
            *bg = BackgroundColor(Color::srgba(0.15, 0.13, 0.08, 0.95));
            *border = BorderColor(Color::srgba(0.9, 0.75, 0.3, 0.9));
        } else {
            *bg = BackgroundColor(Color::srgba(0.08, 0.08, 0.14, 0.8));
            *border = BorderColor(Color::srgba(0.25, 0.25, 0.35, 0.5));
        }
    }

    // Handle selection
    if keyboard.just_pressed(KeyCode::Enter) {
        match pause_menu_state.selected {
            0 => {
                // Resume Game
                pause_state.paused = false;
                cycle.paused = false;
                if let Ok(mut node) = pause_panel_query.get_single_mut() {
                    node.display = Display::None;
                }
            }
            1 => {
                // Save Game -> open save slot browser
                save_slot_browser_state.open = true;
                save_slot_browser_state.context =
                    crate::saveslots::SaveSlotBrowserContext::PauseSave;
                save_slot_browser_state.selected_focus = active_slot.index;
                save_slot_browser_state.confirm_delete = false;
                save_slot_browser_state.delete_target_slot = active_slot.index;
            }
            2 => {
                // Load Game -> open save slot browser
                save_slot_browser_state.open = true;
                save_slot_browser_state.context =
                    crate::saveslots::SaveSlotBrowserContext::PauseLoad;
                save_slot_browser_state.selected_focus = active_slot.index;
                save_slot_browser_state.confirm_delete = false;
                save_slot_browser_state.delete_target_slot = active_slot.index;
            }
            3 => {
                // Settings — open settings panel, hide pause menu
                settings_state.is_open = true;
                settings_state.selected = 0;
                if let Ok(mut node) = pause_panel_query.get_single_mut() {
                    node.display = Display::None;
                }
                if let Ok(mut node) = settings_panel_query.get_single_mut() {
                    node.display = Display::Flex;
                }
            }
            4 => {
                // Controls
                controls_overlay.is_visible = !controls_overlay.is_visible;
            }
            5 => {
                // Quit to Menu
                main_menu.active = true;
                pause_state.paused = false;
                cycle.paused = false;
                if let Ok(mut node) = pause_panel_query.get_single_mut() {
                    node.display = Display::None;
                }
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Panel Visibility Toggle: hide empty panels
// ---------------------------------------------------------------------------

fn toggle_panel_visibility(
    feedback_text_query: Query<&Text, With<FeedbackHudText>>,
    npc_text_query: Query<&Text, (With<NpcHudText>, Without<FeedbackHudText>)>,
    crafting_text_query: Query<
        &Text,
        (
            With<CraftingHudText>,
            Without<FeedbackHudText>,
            Without<NpcHudText>,
        ),
    >,
    fishing_text_query: Query<
        &Text,
        (
            With<FishingHudText>,
            Without<FeedbackHudText>,
            Without<NpcHudText>,
            Without<CraftingHudText>,
        ),
    >,
    quest_text_query: Query<
        &Text,
        (
            With<QuestLogHudText>,
            Without<FeedbackHudText>,
            Without<NpcHudText>,
            Without<CraftingHudText>,
            Without<FishingHudText>,
        ),
    >,
    skill_text_query: Query<
        &Text,
        (
            With<SkillHudText>,
            Without<FeedbackHudText>,
            Without<NpcHudText>,
            Without<CraftingHudText>,
            Without<FishingHudText>,
            Without<QuestLogHudText>,
        ),
    >,
    mut panel_queries: ParamSet<(
        Query<&mut Node, With<FeedbackPanelRoot>>,
        Query<&mut Node, With<NpcPanelRoot>>,
        Query<&mut Node, With<CraftingPanelRoot>>,
        Query<&mut Node, With<FishingPanelRoot>>,
        Query<&mut Node, With<QuestLogPanelRoot>>,
        Query<&mut Node, With<SkillPanelRoot>>,
    )>,
) {
    // Feedback panel
    let feedback_empty = feedback_text_query
        .get_single()
        .map(|t| t.as_str().is_empty())
        .unwrap_or(true);
    if let Ok(mut node) = panel_queries.p0().get_single_mut() {
        node.display = if feedback_empty {
            Display::None
        } else {
            Display::Flex
        };
    }

    // NPC panel
    let npc_empty = npc_text_query
        .get_single()
        .map(|t| t.as_str().is_empty())
        .unwrap_or(true);
    if let Ok(mut node) = panel_queries.p1().get_single_mut() {
        node.display = if npc_empty {
            Display::None
        } else {
            Display::Flex
        };
    }

    // Crafting panel
    let crafting_empty = crafting_text_query
        .get_single()
        .map(|t| t.as_str().is_empty())
        .unwrap_or(true);
    if let Ok(mut node) = panel_queries.p2().get_single_mut() {
        node.display = if crafting_empty {
            Display::None
        } else {
            Display::Flex
        };
    }

    // Fishing panel
    let fishing_empty = fishing_text_query
        .get_single()
        .map(|t| t.as_str().is_empty())
        .unwrap_or(true);
    if let Ok(mut node) = panel_queries.p3().get_single_mut() {
        node.display = if fishing_empty {
            Display::None
        } else {
            Display::Flex
        };
    }

    // Quest log panel
    let quest_empty = quest_text_query
        .get_single()
        .map(|t| t.as_str().is_empty())
        .unwrap_or(true);
    if let Ok(mut node) = panel_queries.p4().get_single_mut() {
        node.display = if quest_empty {
            Display::None
        } else {
            Display::Flex
        };
    }

    // Skill panel
    let skill_empty = skill_text_query
        .get_single()
        .map(|t| t.as_str().is_empty())
        .unwrap_or(true);
    if let Ok(mut node) = panel_queries.p5().get_single_mut() {
        node.display = if skill_empty {
            Display::None
        } else {
            Display::Flex
        };
    }
}
