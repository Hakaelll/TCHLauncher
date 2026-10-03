# TCHLauncher 0.15.1 - Configuration saving hotfix

## Configuration saving

- Fix native property-name validation so standard dotted names such as `query.port`, `rcon.port` and `rcon.password` no longer block saving unrelated settings.
- Preserve existing query/RCON settings and account verification when saving. Retain checks against invalid keys, injected newlines and out-of-range server ports.
- Cover the full-properties save path with a regression that changes PvP while retaining dotted properties, comments and authentication settings.

## Failure review

- Add local, deterministic crash diagnostics for explicit Java-version, heap/native-memory, bind, mod-loader and plugin-load failures.
- Capture bounded output from the current process session, preserve the exit code and requested-stop status, and write a local report before publishing the terminal state.
- Wait up to two seconds for output streams and mark incomplete capture honestly. Reject reports belonging to a previous process session.
- Add an Overview review with recommendations, expandable technical evidence and navigation to configuration, connection, content and console. Recent mod/plugin timestamps are separate context, not proof of a cause.
- Prevent player chat and commands containing error words from receiving false error tags; preserve classification of genuine plugin errors.

## Verification

- A Java 21 integration passed using a deliberately unsupported test class and a subsequent successful Java process. It verifies stderr/log capture, actual exit codes, terminal state, saved reports and separation between sessions. Minecraft was not launched, and no EULA or world was created.
- All 155 offline native tests and 24 frontend tests pass. Twenty integration tests requiring external resources or an installed Java runtime are excluded from the offline run; the Java 21 diagnostic integration was run explicitly and passed. TypeScript and the production frontend build pass.
- Native desktop interaction, Minecraft crash/gameplay behavior, physical-router checks and Linux distribution remain manual or pending. This preview does not complete the full original specification.

## Windows preview

- Installer: `TCHLauncher_0.15.1_x64-setup.exe` (5.98 MiB, 6,271,884 bytes).
- Native application product/file version: `0.15.1`.
- SHA-256: `636b47f0d92313c5dd28162bc8ded088faf68e27798f9eb13ac5af750899a090`.
- Embedded small and large application/installer icons match `icons/icon.ico` using `scripts/verify_windows_icons.ps1`.
- Installer and adjacent checksum file are in `src-tauri/target/release/bundle/nsis/`. The previous 0.15.0 preview is preserved.
