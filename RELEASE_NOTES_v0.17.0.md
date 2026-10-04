# TCHLauncher 0.17.0 — Startup and memory diagnostics

## Server monitoring

- Show Java, world loading, spawn preparation and Ready from actual native log events. Reset progress on restart and reject replies from previous sessions; readiness is never inferred from a spawn percentage.
- Add an explicit Minecraft status query in Connection: measured local response time, advertised version and player counts. Validate the response and matching pong, bound the exchange and respect disabled status queries. Support IPv6 wildcard listeners through IPv6 loopback.
- Keep storage measurement failures separate from empty folders. Cache checked measurements per server/directory and show sample time, skipped links and scoped errors.

## Java memory

- Display used and committed HotSpot heap alongside resident process RAM, with a dashed heap curve and collector event counts. Preserve missing readings and sampling gaps.
- Export a native JSON memory report from Performance through the Save dialog: up to 30 minutes of actual samples while the server is selected. Retain only the latest observed Java session, with bounded in-memory history.
- Reports include current configured heap limits, explicitly distinguished from measured launch limits. They exclude logs, commands, credentials, player identifiers and executable paths.
- Diagnostics do not force garbage collection, add JVM flags or open JMX ports. Increasing resident memory alone does not prove a leak; the reported Fabric gameplay session still needs observations from its hosting computer.

## Scope

Client Fabric instances and Microsoft authentication remain deferred at the user's request. Existing server functionality is retained. No account identities or authentication settings are migrated.

Native desktop appearance, Save-dialog interaction, gameplay, physical-router checks and Linux distribution remain pending. This Windows preview does not complete the full original specification.

## Verification and installer

- All 175 offline native tests pass; 22 integrations requiring external services, installed Java or explicit files are excluded from this run.
- All 27 frontend tests pass. TypeScript and the production frontend build pass.
- Previously explicit HotSpot Java 21 and actual Paper-log checks passed; this packaging run does not repeat gameplay or claim a fix for the friend’s reported RAM growth.

- Windows NSIS x64 packaging completes. Native file/product version: `0.17.0`.
- Small and large embedded icons match the provided logo in both the application and installer.
- Installer: `TCHLauncher_0.17.0_x64-setup.exe` — 6,393,365 bytes (6.10 MiB).
- SHA-256: `c3c39e7fde45c78cd7bc6ad4e5b352124b95399e863f026f42c7676bc437434e`.
- The installer and adjacent `.sha256` file are in `src-tauri/target/release/bundle/nsis/`; the previous installer remains available.
