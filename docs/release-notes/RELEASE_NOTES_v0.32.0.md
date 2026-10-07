# TCHLauncher 0.32.0

## Server reliability

- Recognize Paper's Windows terminal prompt before its structured logs, so a real
  ready server no longer remains stuck at Starting because of that prefix.
- Report incomplete console capture truthfully after read failures or truncation.
- Avoid false Fabric dependency blocks caused by unused optional bundled mods.
  Ambiguous candidate selection remains the loader's responsibility.
- Share integer capacity validation between creation and settings, preventing
  fractional values from reaching native integer serialization.

## Interface and loading

- Present backup history before automation preferences, with date/reason/version
  search, reason filtering, clearer restore rows and latest-copy identification.
- Load creation, server/modpack imports, settings, console and backup controls on
  demand while retaining the application shell and retry paths.
- Focus the visible current-view search with Ctrl+F, including Players, without
  selecting hidden installed-mod inputs while browsing Modrinth.
- Pause new compatibility checks while discovery is hidden; retain the page and
  completed release checks when returning. Already dispatched requests may finish.
- Retain the End/Obsidian/Nether identity, Minecraft typography and reduced-motion
  handling. Clean duplicate helpers/imports and outdated roadmap chronology.

## Real engine verification

Isolated Minecraft 1.21.4 tests passed for Vanilla, Paper, Fabric with installed
Lithium and NeoForge. Each used managed Java, reached actual readiness, answered
loopback status and unique console commands, stopped gracefully three times and
booted after tracked backup/restore. Restore checks include byte-identical level.dat;
Fabric additionally retained the installed mod bytes and listed Lithium each time.
These engine tests do not establish native interface behavior or client gameplay.

## Remaining verification

Native Windows appearance, keyboard/screen-reader paths, clean install/upgrade,
actual player connections and physical-router checks remain pending. Imported
Minehut account behavior, internet stalls and the friend-hosted Fabric RAM/mod
reports still need evidence from their actual host. Linux and the Fabric client
launcher remain deferred. The full original objective remains in progress.

## Windows package

Installer: `TCHLauncher_0.32.0_x64-setup.exe` (6,720,432 bytes / 6.41 MiB).
Tauri/NSIS packaging completed. Both embedded icons match the supplied golden logo;
executable and installer file/product versions are 0.32.0. All four legal resource
destinations are present in the generated NSIS script; the legal inventory matches
the locked Windows Cargo graph and the installed frontend runtime packages.

251 native tests passed; 33 conditional entries were ignored in that suite.
All 102 frontend tests, TypeScript and the production Windows build passed.
The real engine integrations above were executed separately before this release.
The Windows-target initial JavaScript is about 459.23 kB; bundle size is not a
runtime memory or network benchmark.

SHA-256: `219631a9379e0b5f9a5af436b6a368981d8b9212c37ab0144840da8d2a52ecba`.

Generated locally with adjacent checksum files; unsigned, not installed or
published as a GitHub release. Version 0.31.0 is retained for fallback; the obsolete
0.30.0 generated installer and checksum are removed after artifact verification.
