# TCHLauncher 0.21.0 — Content diagnostics and controlled shutdown

## Mod and plugin inventory

- Add card and compact views, alphabetical or attention-first ordering, an attention filter and direct filtered-console actions. Keep search, selection, reviewed updates, cancellation and confirmed removal with safety backups.
- Read NeoForge TOML metadata: declared primary and secondary IDs, display name, literal version and the primary Minecraft dependency expression. Detect duplicate secondary IDs and associate explicit diagnostic references with their shared JAR.
- Display declared local NeoForge PNG logos through the bounded icon reader already used for Fabric. Missing or unsupported images retain the inventory glyph.
- Read plugin names, literal versions, main classes and declared API versions from Paper YAML descriptors. Bound parsing, support scalar aliases and reject malformed or ambiguous identity fields without executing plugins. API declarations are shown separately from Minecraft compatibility.
- Keep unresolved version placeholders unknown. Metadata and inventory presence do not certify successful loading or gameplay. Paper descriptors have no standard icon field.
- Compile console-reference patterns once per installed file. Support raw Log4j/NeoForge diagnostic headers while excluding INFO discussions, player messages and commands. Related errors remain visible even after later warnings; associations retain evidence rather than claiming a root cause.

## Save and shutdown feedback

- Distinguish save start, per-dimension completion, whole-save notification and explicit failure. Overview shows the observed state and last whole-save message; a failure review appears only with actual failure evidence.
- Do not infer a successful save from shutdown or per-dimension output. Incomplete saves remain unconfirmed, and observed startup clears old-session evidence. Minecraft messages are not an independent disk-durability check.
- Use structured severity instead of generic keyword matching for console tags and saved diagnostic findings. Recent history reads freeze the observed file extent and exclude unfinished boundary records.
- Remove automatic Java termination after thirty seconds of graceful stopping. Offer an explicit force-close confirmation only after normal stop has been requested.
- Revalidate the process session and state before forced termination. Persist the forced-exit reason, including zero OS exit codes, while retaining readable older reports. Forced exits exclude clean-stop backups and automatic crash retries; restart refuses to continue after an unsuccessful exit.

## Appearance and interaction

- Preserve Minecraft/End styling, the three themes, existing font choices and reduced-motion preferences.
- Keep text labels beside semantic colors and expose diagnostic evidence from inventory details and console actions.
- Native visual and keyboard review remains manual; this preview does not claim a completed visual redesign.

## Verification

- TypeScript and Vite production build pass.
- All 41 frontend behavioral tests pass, including nine content-attribution cases and three observed-save cases.
- All 188 offline native tests pass; 24 external/runtime/helper checks are excluded from that suite. The owned-child force-stop regression verifies actual termination, session protection and durable forced-exit reporting.
- A separately selected integration installs a real LuckPerms release into an isolated Paper plugins directory, with reviewed download verification and no Java execution or EULA acceptance.
- Descriptor tests cover real ZIP/TOML/YAML/PNG reads, secondary IDs, duplicate identities, malformed input, unsafe icon paths, YAML bounds and scalar aliases.
- Minecraft gameplay, slow-save shutdown in a real world, third-party failures, native visual/keyboard review, physical-router connectivity and Linux packaging remain unverified. The original missing backpack/structure mods and the friend's RAM session are not diagnosed by these checks.

## Windows preview

- Windows NSIS x64 packaging completes with the final frontend embedded.
- Native application and installer product/file versions are `0.21.0`. Embedded large/small icons match the supplied application logo in both files.
- Installer: `TCHLauncher_0.21.0_x64-setup.exe` — 6,440,897 bytes (6.14 MiB).
- SHA-256: `56950b2ff5e0cb3e25050fd4e06ff37742e73d0ff23045e3aa9733806077528e`.
- Installer and adjacent `.sha256` file are in `src-tauri/target/release/bundle/nsis/`.
- The installer was not installed and the application was not opened for native visual QA.

No commit, push or GitHub publication was performed.

## Suggested commit

Improve content metadata, save feedback and confirmed force shutdown

- Add compact inventory presentation and direct attention/console controls.
- Parse bounded NeoForge and Paper metadata and declared NeoForge icons.
- Associate secondary mod identities with explicit diagnostics without INFO false positives.
- Track observed save phases and preserve forced-exit reports across sessions.
- Replace automatic stop timeout termination with confirmed session-bound force close.
- Package and verify the 0.21.0 Windows preview.
