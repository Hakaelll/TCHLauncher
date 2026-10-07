# TCHLauncher 0.8.0

## Added

- Import local `.mrpack` modpacks through a three-step wizard with optional server-file selection and capacity controls.
- Prepare exact Fabric/NeoForge loader versions, verify package file hashes and apply server overrides in an isolated instance before publishing it.
- Schedule one-off or periodic backups, starts, restarts and shutdowns, with pause controls, next-run times and visible results.
- Offer an explicit stop/backup/restart action for consistent backups of running servers.
- Configure crash recovery with a delay, bounded attempts and a reset after ten minutes of actual running time. Requested stops do not restart automatically.
- Add opt-in application autostart and selected-server startup preferences in a global dialog accessible from the server rail.
- Register application autostart for the current Windows user or a Linux desktop session; development builds cannot register their executable for autostart.

## Improved

- Store Minecraft/platform versions, relative launcher paths, memory/storage settings and JVM configuration with backups.
- Restore the matching server generation and configuration transactionally, with a safety copy and interruption recovery.
- Configure opt-in retention by backup count and measured storage usage. Preserve the newest copy and a restore source in use.
- Review existing copies before manual cleanup and reject stale confirmations.
- Recover interrupted modpack publication, restoration and retention through durable journals and SQLite transactions.
- Prevent simultaneous launchers from operating the same library through an exclusive ownership lock.
- Use viewport-centered confirmation dialogs for restoration and cleanup, and refresh backup lists after scheduled work.

## Verification

- All 86 local Rust tests and six telemetry tests pass; 14 tests requiring network access, Java or external resources are ignored in the local suite.
- TypeScript checking and frontend production compilation pass.
- Prior isolated integration checks installed real Fabric/Lithium and NeoForge/FerriteCore packages and reached Minecraft's EULA refusal without accepting it.
- Filesystem/SQLite checks cover backup rollback, interrupted recovery, retention, scheduling, bounded retries and persisted startup selection. Desktop-entry checks use temporary files and do not change OS autostart preferences.
- The Windows x64 NSIS installer builds successfully; its SHA-256 checksum is provided beside the installer.

## Preview limitations

This is a development preview. Native visual/keyboard review, scheduled Java stop/restart/crash behavior, OS login startup, real Minecraft client connections, gameplay restoration and Linux integration still need verification. Forge and Quilt modpacks are not prepared automatically. Public-network automation and loader updates remain pending. Startup tasks validate the existing runtime and remembered EULA choice; they do not silently install a missing runtime or accept the agreement.
