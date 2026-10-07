# TCHLauncher 0.20.0 — Visual mod inventory and console diagnostics

## Installed content

- Replace the file list with responsive Minecraft inventory cards: icon slot, name, version, origin, size, explicit loading status and separate detail/version/remove actions.
- Read declared PNG icons from Fabric JARs, including size maps, without executing the mod or contacting an image service. Normalize them to at most 64 pixels with strict byte/dimension/allocation limits. Missing or unsupported icons use an authored inventory glyph; Paper/NeoForge local icons are not parsed yet.
- Add installed-content search and attention/source filters. Preserve them and the open detail view while auditing files again. Keep existing reviewed updates, stopped-server restrictions, cancellation and confirmed removal with a safety backup.

## Related console messages

- Show red error states and amber warnings when actual recent diagnostics explicitly reference a mod ID/name, JAR filename or logger. Recognize Fabric dependency continuations only within a bounded real diagnostic block.
- Keep text labels and evidence alongside color. Open the console with the relevant file/ID filter from the detail dialog; ordinary navigation clears this contextual filter.
- Exclude chat, player commands, generic mentions and INFO discussions containing error keywords. Retain a recent related error even if later warnings arrive.
- Present associations rather than a guaranteed root cause. A dependency may be involved. Evidence is capped to three distinct messages per file and the available recent console window; an unresolved reference is not attached to an arbitrary mod.

## Visual refinement

- Use themed End/Obsidian/Nether inventory surfaces, relief on icon slots, compact semantic states and selected-tab accents.
- Add restrained item entrance and icon hover feedback, keeping card order and workspace position stable as diagnostics update. Respect both application and operating-system reduced motion.
- Read the rail version from the package manifest instead of a stale label.

## Verification

- TypeScript and the production frontend build pass.
- All 36 frontend tests pass, including seven diagnostic attribution cases: explicit references, token boundaries, chat/INFO exclusions, warnings, bounded evidence, loader continuations and error retention.
- The complete offline native suite passes 180 tests; 23 external/runtime integrations remain excluded. Four focused content-diagnostic tests include real local ZIP/PNG reading, icon normalization, unsafe paths, malformed images and bounds.
- Native visual/keyboard review, third-party gameplay and a real mod failure in the user's world remain unverified. The checks do not diagnose the original missing backpack/structure mods.

## Windows preview

- Windows NSIS x64 packaging completes with the final frontend changes embedded.
- Native executable and installer product versions are `0.20.0`; embedded small/large logo pixels match `icons/icon.ico` in both files.
- Installer: `TCHLauncher_0.20.0_x64-setup.exe` — 6,338,750 bytes (6.05 MiB).
- SHA-256: `0f0c45d20904c362a04da49e42bcc7190eb29d58fbc3d841c60761c835ae675d`.
- Installer and adjacent `.sha256` file are in `src-tauri/target/release/bundle/nsis/`.

No commit, push or GitHub publication was performed.

## Suggested commit

Make installed mods visual and surface related console diagnostics

- Add themed inventory cards with local Fabric icons, search and attention filters.
- Associate explicit runtime diagnostics with mod files and expose evidence in a detail dialog.
- Open filtered console context while preserving update/removal safeguards and reduced motion.
- Synchronize the rail version and package the 0.20.0 Windows preview.
