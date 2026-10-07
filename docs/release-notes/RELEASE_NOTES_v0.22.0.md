# TCHLauncher 0.22.0 — Recoverable management and End interface refinement

## Interface and everyday controls

- Refine Minecraft/End materials in creation steps, world choices, import controls, player summaries, backup rows and maintenance panels. Preserve End, Obsidian and Nether themes, bundled font choices and reduced-motion preferences.
- Add category search and 50-row pagination for players without merging separate UUID identities. Show action recipients and effects, validate names/reasons and focus invalid fields.
- Show unavailable reads, last-known data, unsaved changes and recovery actions inside the affected panel. Console history no longer depends on successful metrics reads; live output remains visible during history loading.
- Keep player refreshes scoped to the selected server and reject obsolete responses. Label stale player totals instead of presenting them as current observations.

## Configuration, backups and scheduled recovery

- Publish properties and RAM/storage settings through one stopped-server operation with recoverable file/database publication. Preserve unknown and unchanged legacy property values; expose grouped controls only for properties actually present.
- Preserve unsaved property and capacity drafts during background refresh. Prevent saving before a successful properties read or after read failure; provide explicit reload.
- Separate completed backup creation, restoration, retention cleanup and settings writes from subsequent read failures. Restoration closes after native acknowledgement and offers a read-only synchronization retry if follow-up queries fail.
- Validate retention and crash-restart policies, protect unsaved drafts and distinguish schedule requests from confirmed Minecraft startup. Keep native cleanup warnings visible.

## Import and content metadata

- Identify renamed imported launchers from bounded metadata and revalidate platform/version before publication. Unknown or ambiguous launchers require explicit selection rather than guessing the newest version.
- Read the official stable Paper catalog with bounded pagination and verified download evidence.
- Expand Fabric dependency and NeoForge/Paper identity metadata. Preserve declared alternatives and unknown versions; installed inventory and dependency declarations do not certify successful gameplay loading.
- Bound player-list reads and preserve malformed source files instead of rewriting them.

## Verification and preview limits

- All 60 frontend behavioral tests pass, including asynchronous server selection, overlapping reads, retries, draft preservation and successful-write follow-up scenarios.
- TypeScript and Vite production build pass.
- All 204 offline native tests pass; 25 external/runtime/helper checks remain excluded. The Windows x64 NSIS installer builds successfully. Executable version resources report 0.22.0; both executable and installer icons match the supplied logo.
- Native visual/keyboard assessment and real Minecraft gameplay, physical router access and Linux packaging remain pending. This preview does not claim those gates completed.

## Windows installer

- Artifact: `src-tauri/target/release/bundle/nsis/TCHLauncher_0.22.0_x64-setup.exe` (6,495,003 bytes).
- SHA-256: `8b51f4defdf7a5447e19d56516ee41308e9a56a283447d4b0de959e096f25835`.
- Generated locally; not installed, committed, pushed or published to GitHub.
