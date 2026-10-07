# TCHLauncher 0.14.0 - Preferences and tracked maintenance

## Application preferences

- Organize Settings into General, Java, Storage, Network, Backups, Advanced and About, using one main surface and a wrapping category bar.
- Keep End, Obsidian and Nether themes, Minecraft/reading typography and reduced-motion preferences in General, alongside existing operating-system and server startup controls.
- Identify the selected server explicitly for Java/JVM, network, backup retention and scheduled-task controls. Reuse the real native workflows and retain access through each server's workspace.
- Show actual library, server, managed Java and backup locations in Storage, with native folder-opening actions and access to the server's storage target.
- Move launcher diagnostics and keyboard shortcuts to Advanced. About reads the native binary version, operating system, architecture and persisted EULA consent.
- Refine automatic sidebar motion: expand over 280 ms, collapse over 190 ms and reveal labels with a short delay. Keep icon targets and workspace positions fixed; respect reduced motion.

## Imports and maintenance tasks

- Track server and world imports in the native task registry. Measure real copied bytes, validate staged files and allow cancellation before publication.
- Keep copied source folders intact, remove cancelled/invalid staging copies and remove a prepared managed server directory if its database insert fails. Registering an external folder shows stages without a fabricated copy percentage.
- Track scheduled backups and safety copies before Minecraft, loader and managed-content updates through the same registry and measured copy path. Preserve scheduled stop/resume behavior on backup failure or cancellation.
- Track mod/plugin removal in its confirmation dialog and the task center. Measure the whole-server safety copy and prepared content tree separately; cancellation preserves current content/configuration and any completed backup.
- Protect publication from cancellation, release registrations by exact task identity and preserve occupied registry entries. Keep stopped file-operation tasks from being reported as Installing.

## Verification boundaries

- The complete offline native suite passes 135 tests; 18 external-resource integration tests are excluded. All 24 frontend tests pass, with TypeScript and production frontend compilation successful.
- Native visual, keyboard and folder-opening review is pending. The user reviews the application directly; no computer use or delegated agents were used.
- Scheduled Java stop/resume, physical-router behavior and Minecraft client gameplay still need manual verification.
- Imported authentication settings and player data remain preserved. This release does not claim to migrate offline/proxy UUIDs or repair an imported server's authentication configuration.
- NAT-PMP, remote reachability checks, reliable TPS telemetry and Linux distribution remain unverified or incomplete. The full original application specification is still in progress.

## Windows preview

- Installer: `TCHLauncher_0.14.0_x64-setup.exe` (5.80 MiB, 6,082,746 bytes).
- Native application product/file version: `0.14.0`.
- SHA-256: `6fda30451840e15bdcde21ccc0f6b07ceb9acaaf21ac3f296168c883176caf87`.
- Small and large embedded application/installer icons match `icons/icon.ico` pixel for pixel using `scripts/verify_windows_icons.ps1`.
- Installer and adjacent `.sha256` file are in `src-tauri/target/release/bundle/nsis/`. The previous 0.13.0 preview is preserved.
