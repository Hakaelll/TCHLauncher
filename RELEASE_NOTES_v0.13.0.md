# TCHLauncher 0.13.0 - Automatic navigation and background tasks

## Navigation and appearance

- Start with a compact server rail that expands smoothly on pointer hover or keyboard focus. Reveal labels without moving icon targets or resizing the workspace. Keep the rail open for its context menu and task panel.
- Remove the manual collapse button and resting frames around utility icons. Use the current theme's hover/selected accents and retain the Minecraft lettering and transparent gold emblem.
- Add a persistent Reduce Animations setting. Respect the operating system's reduced-motion preference in CSS, navigation, dialogs, console following and metric animation. Existing saved themes and fonts remain intact.
- Add Ctrl+K action/server search with keyboard navigation and current native-state restrictions. Ctrl+N opens server creation; Ctrl+F focuses the current library, console or content search without interfering with confirmation dialogs.

## Background tasks and file operations

- Add a non-blocking task panel for installations, Java preparation, content/modpack downloads, loader/Minecraft updates, manual backups and restores. Show real stages, measured transfer data and recent outcomes from the current session.
- Query current native jobs every three seconds without overlapping requests or replacing newer events. Recover active progress after a frontend reload and remove vanished jobs without inventing success.
- Cancel by exact task identity and open the owning server. Protect active/replaced tasks from dismissal and include current jobs in action/menu restrictions.
- Measure manual backup and restore copy phases using actual byte totals and bounded file copies. Preserve file permissions and sync copied files. Final metadata publication and retention remain indeterminate rather than displaying a fabricated percentage.
- Keep backup/restore cancellation available before publication, then reject it at an exclusive publication boundary. Cancelled restores preserve the current server and the completed safety copy. The restore dialog also exposes its own progress and cancellation control.
- Hold the native operation reservation until task completion and registry cleanup. A stopped server remains stopped while its files are copied.

## Diagnostics

- Add internal JSONL launcher logs for startup, shutdown, process-state changes and router-maintenance failures. Records use defined info/warn/error events, launcher version and validated server identifiers.
- Retain the current log and one rotated previous log, up to 2 MB each, with an Open Application Logs action in Settings. Console text, player names, commands, credentials and arbitrary error payloads are excluded from these diagnostic records.
- Correct the Windows loopback UPnP test fixture so accepted sockets use blocking reads; production router transport remains bounded.

## Verification and remaining work

- The offline native suite passes 128 tests; 18 external-resource integration tests are excluded. All 24 frontend tests pass, with TypeScript and production frontend compilation successful.
- Native visual/keyboard/popover review, Windows log-folder opening, physical-router behavior and Minecraft client gameplay still need manual verification.
- Global progress for scheduled/safety copies outside restores and server/world imports remains pending. NAT-PMP, remote reachability checks, reliable TPS telemetry and Linux distribution remain outside this preview's verification.
- Imported authentication settings and player data are preserved. This preview does not claim to repair a server's offline/proxy authentication configuration or migrate player UUID data.

## Windows preview

- Installer: `TCHLauncher_0.13.0_x64-setup.exe` (5.77 MiB, 6,051,696 bytes).
- Native application product/file version: `0.13.0`.
- SHA-256: `c65c405f0a50f0aa8633f10bf1f3aba5fd9cc4e2573b4b105c9d43375b277e1a`.
- Small and large embedded application/installer icons match `icons/icon.ico` pixel for pixel using `scripts/verify_windows_icons.ps1`.
- Installer and adjacent `.sha256` file are in `src-tauri/target/release/bundle/nsis/`. The previous 0.12.0 preview is preserved.
