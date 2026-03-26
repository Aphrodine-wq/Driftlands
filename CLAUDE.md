# Driftlands -- Claude Development Guide

## What This Is

2D top-down survival crafting game built in Rust with Bevy 0.15. Targeting Steam Early Access. ~24K lines across 47 source files, 43 registered Bevy plugins.

## Build Commands

```bash
cargo run --features dev      # Dev mode with dynamic linking (fast iteration)
cargo run                     # Dev mode without dynamic linking
cargo build --release         # Optimized release binary
cargo check                   # Type-check without building (use this for validation)
```

The `dev` feature enables `bevy/dynamic_linking` for faster incremental compiles.

## Project Structure

- `src/main.rs` -- App entry point. All 43 plugins registered here in 3 `.add_plugins()` tuples (max 15 per tuple) plus individual `.add_plugins()` calls.
- `src/world/` -- World generation subsystem (mod.rs, generation.rs, chunk.rs, tile.rs). Chunk-based tilemap where tiles are data arrays, NOT individual ECS entities.
- `assets/` -- 37 sprite/asset directories plus `shaders/` with WGSL files.
- `ralph/` -- Ralph autonomous agent workspace (PRD, progress log, CLAUDE.md for Ralph).
- `.planning/PROJECT.md` -- Original PRD with requirements and key decisions.
- `.aeonrc.yml` -- AEON verification config (safety profile, all engines enabled).

## Critical Architecture Rules

1. **Tiles are data, not entities.** Each chunk is 1 ECS entity with 1 mesh. Tiles stored as data arrays. This is THE critical performance decision -- never create individual tile entities.

2. **Max 15 plugins per `.add_plugins()` tuple.** main.rs uses 3 tuples. Do not add a 4th tuple without filling the 3rd to capacity first.

3. **Pause guard on all gameplay systems.** Use `.run_if(not_paused)` from `hud.rs`. The `MainMenuActive` resource also gates gameplay via `not_paused`.

4. **`SpatialGrid` for proximity queries.** 128px cells in `spatial.rs`. Use this instead of raw ECS iteration for finding nearby entities. Tracks enemies, buildings, farms, and world objects.

5. **Save backwards compatibility.** All new fields on `SaveData` must use `#[serde(default)]`.

## Key Patterns

| Pattern | Location | Usage |
|---------|----------|-------|
| `FloatingText` + `spawn_floating_text()` | `hud.rs` | Damage numbers, pickup text |
| `SpawnParticlesEvent` | `particles.rs` | Visual effects |
| `SoundEvent` | `audio.rs` | Audio hooks (currently logs, no audio files shipped) |
| `DroppedItem` | `gathering.rs` | Item pickup animations |
| `ScreenShake` | `camera.rs` | Combat feedback |
| `Knockback` | `combat.rs` | Hit feedback |
| `GatheringState` | `gathering.rs` | Tracks current gathering target for progress bars |
| `CraftingStation` | `building.rs` | Component on placed station buildings, proximity check (64px) |
| `ChestStorage` | `building.rs` | Component on placed Chest buildings |
| `SkillXpEvent` + `SkillType` | `skills.rs` | Fire when relevant actions complete |
| `QuestProgressEvent` + `QuestType` | `quests.rs` | Fire when quest-relevant actions complete |
| `StatusEffectType` | `status_effects.rs` | Apply effects via `StatusEffectsPlugin` |
| `weapon_on_hit_effect()` | `enchanting.rs` | Call from combat damage resolution |
| `SpriteAnimation` component | `animation.rs` | Attach to entity; `AnimationPlugin` drives frame cycling |
| `DebugPerfTiming` resource | `debug_perf.rs` | Update fields for timing data |
| `GameSettings::load()` / `save()` | `settings.rs` | Persistent settings to disk |
| `check_chunk_structures` | `structures.rs` | World structures spawn on chunk load |
| `TileType::biome_color(biome)` | `world/tile.rs` | Biome-specific tile colors |

## Bevy 0.15 API Notes

- `EventWriter::send()` -- not `write()`.
- `Sprite.image` is `Handle<Image>` -- not `Option`.
- Color constructors: `Color::srgb()` / `Color::srgba()`.
- Window: set via `WindowPlugin` in `DefaultPlugins`.
- Pixel art: `ImagePlugin::default_nearest()` is set globally.

## Dependencies

bevy 0.15, noise 0.9, rand 0.8, serde 1 (with derive), serde_json 1, bincode 1.

## Largest Files (by line count)

| File | Lines | Notes |
|------|-------|-------|
| `combat.rs` | ~3,100 | Enemy AI, damage, spawning, boss fights |
| `hud.rs` | ~2,500 | All in-game UI, pause system, floating text |
| `world/mod.rs` | ~2,100 | World state, chunk objects, object spawning |
| `crafting.rs` | ~1,400 | All recipes, crafting UI, station logic |
| `saveload.rs` | ~980 | Full world serialization |
| `building.rs` | ~920 | Structure placement, chest UI |
| `assets.rs` | ~850 | All asset handle loading |

## Adding New Systems

1. Create `src/newsystem.rs` with a `pub struct NewSystemPlugin;` implementing `Plugin`.
2. Add `mod newsystem;` to `main.rs`.
3. Register `.add_plugins(newsystem::NewSystemPlugin)` in the appropriate tuple in main.rs (fill existing tuples before creating new ones).
4. Guard gameplay systems with `.run_if(not_paused)`.
5. If the system has saveable state, add fields to `SaveData` in `saveload.rs` with `#[serde(default)]`.
6. Run `cargo check` to verify.

## Do NOT

- Create individual tile entities (use chunk data arrays).
- Use `sudo -s` in this project directory.
- Run ML inference or training on this machine (use vast.ai for heavy compute).
- Skip `cargo check` before committing.
- Use gradients or emojis in any UI/visual work.
