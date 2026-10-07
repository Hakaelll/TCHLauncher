# TCHLauncher 0.16.0 — Backups and server management

## Backups

- Add an optional automatic copy after a successful exit from a server session that reached Running. Enable it per server in Backups; it is off by default.
- Track bytes, allow cancellation before publication and apply the saved retention policy. Persist a session claim so failed, cancelled or interrupted attempts do not replay the same stop.
- Wait for owned file operations and automatic stop copies when exiting. Reject new starts while shutdown is in progress.
- Show the saved reason, Minecraft version and server engine in backup lists and restore confirmation. Support older copies with missing metadata and report damaged metadata without hiding other copies.

## Players and performance

- Use the same normalized player event for console tags and connected-player tracking. Recognize colored Paper output and reject chat, command and plugin text that imitates joins or leaves.
- Add an editor for existing operators' levels and permission to join a full server. Changes require a stopped server and take effect at its next start. Preserve identities, other entries and unknown JSON fields; retain an exact copy of the original `ops.json` before replacement.
- Measure current-process startup time from launching Java to the first recognized server-ready message. Show it in Performance; leave it unavailable before readiness or after process removal.

## Reliability

- Preserve the latest failed process report across later successful sessions for an explicit, dated historical review. Keep current-session crash diagnostics separate.
- Record unexpected scheduler/automatic-restart worker failures and release worker capacity on every exit.
- Validate saved task dates, intervals and actions before claim or reactivation. Disable malformed tasks without blocking valid ones; allow review and removal from the task list.
- Isolate damaged NAT-PMP records and individual renewal/cleanup failures. Continue maintenance for valid mappings and retain failed cleanup for retry.

## Verification

- All 166 offline native tests pass, with 20 integration tests excluded from that run.
- All 24 frontend tests, TypeScript and the production frontend build pass.
- The explicit Java 21 failure/success integration passes, checking real stderr, exit codes, session separation and retained historical failure. It does not launch Minecraft, accept EULA or create worlds.
- Windows NSIS x64 packaging completes; native application product/file version is `0.16.0`.
- Embedded small and large icons in the application and installer match `icons/icon.ico` using `scripts/verify_windows_icons.ps1`.

Native desktop interaction, real Minecraft gameplay/permissions, scheduled Java restart,
automatic stop-copy/shutdown behavior with Minecraft, physical-router checks and Linux
distribution remain pending. This is a Windows preview and does not complete the full
original application specification. Existing account verification and player UUIDs are
not migrated by this release.

## Windows preview

- Installer: `TCHLauncher_0.16.0_x64-setup.exe` (6.03 MiB, 6,323,459 bytes).
- SHA-256: `f5098d6288e6742c8d2396eb0e198d40e46a94ec3ecf9d18080c48ea5ce0d81e`.
- The installer and adjacent `.sha256` file are in `src-tauri/target/release/bundle/nsis/`. The 0.15.1 installer remains available.
