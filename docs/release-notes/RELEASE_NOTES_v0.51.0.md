# TCHLauncher 0.51.0

Windows x64 preview. The installer is unsigned.
Prepared for publication after the 0.49, 0.50 and 0.51 version milestones.
Only the EXE installer will be attached; checksums stay local.

## Changes since the published 0.48.0

- RAM and CPU charts inspect the same timestamp with pointer or keyboard input.
  Missing readings remain gaps; Escape returns both charts to recent readings.
- Process metrics no longer wait for a directory-size scan. Storage measurement
  runs in the background and keeps its measured timestamp. Initial calculation
  is distinguished from a read failure.
- Retention review visits backup files in deterministic order without retaining
  the whole tree's entry list. The newest and protected copies remain protected;
  changed files or policy still invalidate a reviewed cleanup.
- Stop-backup preferences confirm the committed value without a second read that
  could misreport a successful save. Manual refresh recovers failed queries;
  duplicate and disposed writes are guarded.
- Download tasks reuse separate catalog, transfer and strict modpack HTTP pools,
  preserving per-task cancellation, progress, hashes, limits and redirect rules.
- Hide completed background tasks together while keeping failures and active work.
- Console command, copy and export failures are visibly marked as errors.
- Local list searches consistently ignore accents and case without changing names
  or merging distinct identities.
- Modrinth, player and import-file pagination returns focus to the result list.
  Startup, JVM and operator recovery notes respect the selected language.
- Remove obsolete CSS declarations without changing effective colors.

## Validation and limits

Targeted component, callback, reader and local-filter checks cover these changes,
including Spanish and English feedback. Native temporary filesystem/SQLite cases
cover saved preferences, protected retention, changed-tree refusal, background
size publication, cache expiry, directory changes and failed-read recovery.
Local HTTP fixtures cover connection reuse, cancellation isolation, hash failures,
preserved destinations, modpack size limits and rejected redirects.

These scoped checks do not establish measured RAM or latency improvements,
Minecraft-network gains, remote-provider outcomes or native keyboard/screen-reader
acceptance. Manual validation remains with the maintainer and does not block
independent implementation. No dependency or retained test is added in this batch.

Packaging evidence will be recorded after the installer is built.
