# Ralph Agent Instructions

You are an autonomous coding agent working on the Driftlands game (Bevy 0.15 + Rust).

## Your Task

1. Read the PRD at `ralph/prd.json`
2. Read the progress log at `ralph/progress.txt` (check Codebase Patterns section first)
3. Check you're on the correct branch from PRD `branchName`. If not, check it out or create from main.
4. Pick the **highest priority** user story where `passes: false`
5. Implement that single user story
6. Run `cargo check` to verify typecheck passes
7. If checks pass, commit ALL changes with message: `feat: [Story ID] - [Story Title]`
8. Update the PRD (`ralph/prd.json`) to set `passes: true` for the completed story
9. Append your progress to `ralph/progress.txt`

## Project Context

- **Game**: Driftlands -- top-down open-world sandbox (Bevy 0.15 + Rust, macOS)
- **Bevy version**: 0.15 with dynamic_linking (`cargo run --features dev`)
- **Architecture**: Chunk-based tilemap -- tiles as data arrays, NOT individual ECS entities
- **Module count**: 43 modules in src/ (see main.rs for the full list)
- **Key Patterns**:
  - Max 15 plugins per `.add_plugins()` tuple -- split into multiple calls (main.rs uses 3 tuples)
  - `EventWriter::send()` not `write()` for events
  - `Sprite.image` is `Handle<Image>` not `Option`
  - Color: `Color::srgb()` / `Color::srgba()`
  - Deps: bevy 0.15, noise 0.9, rand 0.8, serde 1, serde_json 1, bincode 1
  - Run condition: `not_paused` exported from `hud.rs` -- guard ALL gameplay systems with `.run_if(not_paused)`
  - `MainMenuActive` resource also gates gameplay via `not_paused`
  - `CraftingStation` component on placed station buildings, checked via proximity (64px)
  - `ChestStorage` component on placed Chest buildings
  - `FloatingText` + `spawn_floating_text()` in `hud.rs` for damage numbers / pickup text
  - `SpawnParticlesEvent` in `particles.rs` for visual effects
  - `SoundEvent` in `audio.rs` for audio hooks (currently logs, no audio files shipped)
  - `DroppedItem` component in `gathering.rs` for item pickup animations
  - `ScreenShake` resource in `camera.rs` for combat feedback
  - `Knockback` component in `combat.rs` for hit feedback
  - `GatheringState` resource tracks current gathering target for progress bars
  - `SaveData` uses `#[serde(default)]` on all new fields for backwards compat
  - Biome-specific tile colors via `TileType::biome_color(biome)` in `tile.rs`
  - `SpatialGrid` (128px cells) in `spatial.rs` -- use for proximity queries, not raw ECS iteration
  - Enchant on-hit effects: call `enchanting::weapon_on_hit_effect(weapon)` from combat damage resolution
  - Status effects: apply via `StatusEffectsPlugin` -- use `StatusEffectType` enum values
  - Skill XP: fire `SkillXpEvent` with the appropriate `SkillType` when relevant actions complete
  - Quest progress: fire `QuestProgressEvent` with `QuestType` when quest-relevant actions complete
  - Automation buildings (`AutoSmelter`, sprinkler, alarm bell) tick independently in `automation.rs`
  - World structures spawn via `check_chunk_structures` in `structures.rs` on chunk load
  - Settings persisted to disk via `GameSettings::load()` / `save()` in `settings.rs`
  - Animation: attach `SpriteAnimation` component; `AnimationPlugin` drives frame cycling
  - Debug perf: update `DebugPerfTiming` resource fields in `debug_perf.rs` for timing data

## Progress Report Format

APPEND to ralph/progress.txt (never replace, always append):
```
[PASS] US-XXX -- Story Title -- cargo check passes
```

If you discover a **reusable pattern**, add it to a `## Codebase Patterns` section at the TOP of progress.txt.

## Quality Requirements

- ALL commits must pass `cargo check`
- Do NOT commit broken code
- Keep changes focused and minimal
- Follow existing code patterns in the codebase
- When adding new modules, remember to add `mod newmodule;` to main.rs and register the plugin
- Do NOT add a 4th `.add_plugins()` tuple without filling the 3rd one to capacity first (max 15 per tuple)

## Stop Condition

After completing a user story, check if ALL stories have `passes: true`.

If ALL stories are complete, reply with:
<promise>COMPLETE</promise>

If there are still stories with `passes: false`, end your response normally.

## Important

- Work on ONE story per iteration
- Commit after each story
- Keep cargo check green
- Read existing source files before modifying them
