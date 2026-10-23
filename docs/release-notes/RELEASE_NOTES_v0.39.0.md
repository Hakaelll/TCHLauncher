# TCHLauncher 0.39.0

## Recovery and interaction

- General action errors retain the original reason, show recovery advice and offer
  access to application logs. Raw technical details stay in a disclosure.
- A dialog stays visible if an operation becomes busy during its exit animation.
  Cancelled animations cannot later dismiss that dialog. Closing after the
  operation, focus return and reduced-motion behavior remain available.

## NeoForge and downloads

- NeoForge discovery and explicit loader selection share one bounded catalog
  parser, including historical Minecraft 1.20.1 artifact identities. Stable
  releases retain numeric ordering; explicit prereleases require catalog membership.
- Downloads compute only the requested SHA-1, SHA-256 or SHA-512 digest. Existing
  validation, streaming limits, retries and atomic file publication remain in place.
  No performance benchmark or RAM reduction is claimed.

## Verification and limits

Focused parser and streamed-transfer checks passed. A real historical NeoForge
1.20.1-47.1.106 lifecycle passed three sessions and backup/restore with managed
Temurin 17. An isolated React component reproduced the dialog race before the fix
and passed afterwards, including focus return and reduced motion. TypeScript and
Windows-targeted frontend builds passed. No dependency or retained test was added.

The previous published 0.38.0 installer also passed isolated silent install/upgrade
and uninstall checks, plus selected-server autostart and scheduled real Vanilla
backup/restart/stop. Original application data and registration were restored.
These observations do not establish installer acceptance for every newer build.

This remains an unsigned Windows x64 preview. Interactive desktop/tray behavior,
an actual Windows login, broader Minecraft client gameplay and physical-router
Internet use still require acceptance. The full manager objective remains open.
Linux and Minecraft client launching remain deferred.

Only `TCHLauncher_0.39.0_x64-setup.exe` is attached to the GitHub release. Checksum
files remain local.

The optimized Windows build completed successfully. Installer size is 6,837,482
bytes; executable and installer versions are 0.39.0. Embedded icons match the
application emblem. The linker emitted an informational library-creation message.
