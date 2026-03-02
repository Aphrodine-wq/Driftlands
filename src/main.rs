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
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
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
            minimap::MinimapPlugin,
        ))
        .run();
}
