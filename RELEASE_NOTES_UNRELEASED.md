# TCHLauncher — Unreleased

The latest Windows preview is documented in [0.11.0](RELEASE_NOTES_v0.11.0.md). The sections below retain development history; completed additions are included in that preview.

## Loader updates (development source)

- Add stable Fabric/NeoForge update review for the current Minecraft version, with installed-version identification and stale-review detection.
- Prepare a separate verified loader generation using the configured Java runtime and create a backup before changing launch paths.
- Publish through a durable journal and SQLite update; recover interrupted publication and preserve the previous generation on database failure.
- Connect a stopped-server settings panel, explicit version selection, confirmation, transfer progress and cancellation.
- Verify preservation of worlds/configuration, failed publication rollback, interruption recovery and active-generation protection with local tests.

- Handle canonical Windows launch paths during installed-version review, prepared-generation publication and rollback.
- Verify official Fabric 0.16.10 → 0.19.5 → restored 0.16.10 and NeoForge 21.4.157 → 21.4.158 → restored 21.4.157 on Minecraft 1.21.4 with Java 21. Updated and restored launchers reach EULA refusal, preserving Java/JVM/configuration without accepting the agreement or creating a world.
- Identify imported classic Fabric launchers, embedded unattended Fabric metadata and NeoForge argument files without relying on filenames or managed metadata. Reject contradictory versions and unsafe referenced paths.
- Preserve imported launchers during update publication and database failure, and restore their original launch paths from the safety backup.
- Verify real imported Fabric and NeoForge update/restore cycles with Java 21, and normalize Windows verbatim/UNC paths for Java classloading. Unattended Fabric identity is locally tested; its full live cycle remains pending.
- Pass all 95 local Rust tests; 15 external-resource tests are excluded from the offline suite, with the loader integration run separately for both platforms.

These changes are not included in the 0.8.0 installer. Native visual/gameplay and Linux checks remain pending. Legacy shaded/custom launchers without sufficient identity data need additional identification support.

## Remaining verification and features

- Native visual/keyboard review and Minecraft client/gameplay validation.
- Scheduled Java restart/shutdown/crash and OS-login startup verification.
- Linux builds and integration checks.
- Legacy/custom imported-loader identification, unattended Fabric live integration and complex third-party modpack compatibility.
- Public-network automation and reliable TPS telemetry.
- Remaining distribution/signing and full requirement audit.

These items are not claimed as completed by the preview. See [NEXT_TODOS.md](NEXT_TODOS.md) for the implementation and verification status.

## Minecraft version upgrade review foundation (in progress)

- Added a native read-only review command for stable release chronology, runtime requirements, matching Fabric/NeoForge/Paper catalogs, and installed content compatibility backed by SHA-512.
- Manual or modified files and unavailable catalog metadata remain unverified. Reviews include configuration and launcher evidence to detect stale input.
- Transactional publication, content migration, UI, startup verification and real upgrade/restore integration are still pending. This foundation is not included in the 0.8.0 installer.

## Imported player identity correction

- Avoid selecting an old offline UUID simply because it appears first in usercache when displaying an authenticated connected player. Ambiguous proxy/offline identities retain a neutral avatar.
- Preserve separate historical UUIDs, and show authentication-aware access copy rather than claiming account validation for offline servers.
- Covered by three identity selection regressions and TypeScript/Vite builds; not included in the existing 0.8.0 installer.

## Minecraft version publication (in progress)

- Added guarded native update execution with review fingerprint revalidation, incompatible-content blocking, warning acknowledgement, a safety backup, isolated preparation and cancellation before publication.
- Publish the server directory, Minecraft version and Java path using a SQLite recovery token. Imported directories keep their temporary folders on their own volume.
- Added exact stable Paper build preparation with required SHA-256 and preservation/validation of a compatible configured Java runtime.
- Four local publication tests cover rollback on SQLite failure, recovery before/after commit, backup restore and staged path rejection. UI, content migration and real Java upgrade/start/restore checks remain pending; not included in the 0.8.0 installer.

## Minecraft update interface and integration checks

- Connected the settings dialog to stable version selection, native review, per-file compatibility results, runtime/loader summary, acknowledgement and cancellable update progress.
- Real Java 21 upgrade/restore cycles from Minecraft 1.21.4 to 1.21.5 and back passed for Vanilla, Fabric and NeoForge. Java/JVM/configuration preservation and backup recovery were checked; EULA was never accepted and no world was generated.
- Paper 1.21.5 was correctly rejected because the live catalog has no STABLE build. Paper 1.21.8 build 60 was verified as STABLE and its 1.21.4→1.21.8→1.21.4 upgrade/restore cycle passed with Java 21 and no EULA acceptance.
- Native visual QA, incompatible-content migration, actual client gameplay/world conversion and Linux remain pending. No new installer has been generated.

## Managed content migration and interface refinement

- Resolve target-compatible stable mod/plugin releases as one dependency graph during Minecraft upgrade review. Include exact required dependencies, reject conflicts and show proposed changes/changelogs before confirmation.
- Install reviewed content into the isolated prepared instance, preserve unverified files and avoid redundant backups. Update inventory metadata when distinct catalog releases share the same artifact.
- Verify Fabric/Lithium and NeoForge/FerriteCore upgrade/restore cycles using real downloads and Java 21 up to EULA refusal. No EULA acceptance, world generation or gameplay is claimed.
- Present creation, folder import and modpack import directly in the empty library, with responsive layouts and deliberate action placement.
- Consolidate overview metrics, add process/configuration context and emphasize the selectable LAN address. Preserve anthracite controls, semantic data colors and reduced-motion support.

TypeScript builds and the content inventory regression pass. Native visual validation and application startup verification after upgrades remain pending. These changes are in development source, not the existing 0.8.0 installer.

## Post-upgrade startup observation

- Offer explicit startup after a reviewed Minecraft change, disclose world conversion and leave the server running.
- Confirm readiness only from native state. Stopped/crashed processes and unavailable or timed-out observation remain visibly unconfirmed.
- Link failures to the console and retain the safety backup without silently restoring worlds. Navigating away stops observation, not Java.
- Four startup-observation regressions passed; the frontend suite now has 13 passing tests. The full offline Rust suite has 108 passing tests and 16 ignored external-resource checks.

These additions are included in the 0.9.0 Windows preview. Native/client validation remains pending.

## Development after the 0.9.0 installer

- Add opt-in experimental Minecraft catalogs in Vanilla/Fabric creation, with snapshot labeling and an official Mojang/Fabric ID intersection. Keep stable choices as the default and preserve catalog chronology.
- Remove the fifty-release catalog truncation; omit legacy alpha/beta categories and unsafe IDs.
- Require a complete native server-thread startup message and a valid finite duration before treating Minecraft as ready. Chat, plugin task text and incomplete completion messages do not establish readiness.

Snapshot installation/gameplay and native interaction remain unverified. These source changes are not included in the already generated 0.9.0 installer.

Verification for subsequent source: 110 offline Rust tests passed (17 external-resource checks ignored), including catalog opt-in and startup-message regressions. A separate live catalog check passed for official Vanilla/Fabric entries and Minecraft 26.4-snapshot-2 server metadata. TypeScript and Vite production builds passed. Snapshot installation/boot and native UI checks are still pending.

Separate live installation checks passed for Minecraft 26.4-snapshot-2 on Vanilla (42.43 seconds) and Fabric (41.12 seconds) with Java 25. Both reached EULA refusal, preserved initial properties and verified no consent or world was created. This proves snapshot preparation and boot to that gate, not client gameplay. The current 0.9.0 installer remains unchanged.

## Minecraft / End visual refresh — 0.10.0

- Replace the light graphite presentation with a dark Tableland-derived surface system and End-inspired lavender details.
- Add locally bundled Monocraft 4.2.1 to headings and primary actions, retaining readable system/monospace faces for dense information.
- Add raised, resting-dark controls with interaction lighting, square slider grips and coherent dark menus, modals, fields, charts and console tags.
- Replace the application emblem and desktop/tray/browser icons with the supplied gold voxel source.
- Retain all current native command wiring and first-server creation/import/modpack actions.
- Include post-0.9 experimental catalog and strict readiness parsing source changes.
- Verify 110 offline Rust tests and 13 frontend behavioral tests; 18 external integration tests are excluded from the offline run. Validate 24 text/semantic token combinations (minimum 5.23:1) and actual Spanish font glyph coverage.
- Native screenshot/gameplay verification is not performed: the user requested manual visual assessment and prohibited computer use.
