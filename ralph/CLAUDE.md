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

- **Game**: Driftlands — top-down open-world sandbox (Bevy 0.15 + Rust, macOS)
- **Bevy version**: 0.15 with dynamic_linking
- **Architecture**: Chunk-based tilemap — tiles as data arrays, NOT individual ECS entities
- **Key Patterns**:
  - Max 15 plugins per `.add_plugins()` tuple — split into multiple calls
  - `EventWriter::send()` not `write()` for events
  - `Sprite.image` is `Handle<Image>` not `Option`
  - Color: `Color::srgb()` / `Color::srgba()`
  - Deps: bevy 0.15, noise 0.9, rand 0.8, serde 1, bincode 1
- **Module structure**: 21 modules in src/ — see main.rs for the full list

## Progress Report Format

APPEND to ralph/progress.txt (never replace, always append):
```
[PASS] US-XXX — Story Title — cargo check passes
```

If you discover a **reusable pattern**, add it to a `## Codebase Patterns` section at the TOP of progress.txt.

## Quality Requirements

- ALL commits must pass `cargo check`
- Do NOT commit broken code
- Keep changes focused and minimal
- Follow existing code patterns in the codebase
- When adding new modules, remember to add `mod newmodule;` to main.rs and register the plugin

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
