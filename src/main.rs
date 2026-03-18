mod assets;
mod camera;
mod player;
mod world;
mod inventory;
mod gathering;
mod crafting;
mod building;
mod daynight;
mod hud;
mod combat;
mod death;
mod saveload;
mod minimap;
mod dungeon;
mod season;
mod weather;
mod farming;
mod techtree;
mod npc;
mod lore;
mod experiment;
mod particles;
mod audio;
mod mainmenu;
mod tutorial;
mod controls;
mod theme;
mod lighting;
mod lit_materials;
mod status_effects;
mod fishing;
mod enchanting;
mod pets;
mod quests;
mod structures;
mod skills;
mod automation;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Driftlands".into(),
                        resolution: (1280.0, 720.0).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(theme::ThemePlugin)
        .add_plugins(lighting::LightingPlugin)
        .add_plugins(lit_materials::LitMaterialsPlugin)
        .insert_resource(ClearColor(Color::srgb(0.008, 0.008, 0.024))) // Deep Nocturne
        .add_plugins(assets::AssetsPlugin)
        .add_plugins((
            world::WorldPlugin,
            player::PlayerPlugin,
            camera::CameraPlugin,
            inventory::InventoryPlugin,
            gathering::GatheringPlugin,
            crafting::CraftingPlugin,
            building::BuildingPlugin,
            daynight::DayNightPlugin,
            hud::HudPlugin,
            combat::CombatPlugin,
            death::DeathPlugin,
            saveload::SaveLoadPlugin,
        ))
        .add_plugins((
            minimap::MinimapPlugin,
            dungeon::DungeonPlugin,
            season::SeasonPlugin,
            weather::WeatherPlugin,
            farming::FarmingPlugin,
            techtree::TechTreePlugin,
            npc::NpcPlugin,
            lore::LorePlugin,
            experiment::ExperimentPlugin,
            particles::ParticlePlugin,
            audio::GameAudioPlugin,
            mainmenu::MainMenuPlugin,
            tutorial::TutorialPlugin,
            controls::ControlsPlugin,
        ))
        .add_plugins((
            status_effects::StatusEffectsPlugin,
            fishing::FishingPlugin,
            enchanting::EnchantingPlugin,
            pets::PetPlugin,
            quests::QuestPlugin,
            structures::StructuresPlugin,
            skills::SkillsPlugin,
            automation::AutomationPlugin,
        ))
        .run();
}
