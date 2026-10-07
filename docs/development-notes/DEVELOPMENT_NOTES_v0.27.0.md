# Development notes — 0.27.0

This milestone packages the performance and contextual-action work after 0.26.0:

- [Performance implementation and measured limits](PERFORMANCE_AFTER_0.26.0.md)
- [World and installed-content actions](WORLD_ACTIONS_AFTER_0.26.0.md)

World inventory now uses the existing selected-server read controller. Failed
reads retain previous rows and display labelled unavailability with retry; a valid
empty list alone shows the empty state. File operations are unavailable during
loading or failed reads, but folder inspection remains read-only. Manual refresh
is available after external world generation. An acknowledged operation remains
successful if its inventory refresh fails. Server reselection invalidates earlier
reads through the same controller covered by existing read regressions.

The final frontend checks passed all 84 regressions and TypeScript. The current
native implementation previously passed 232 offline tests / 28 ignored. Production
NSIS packaging is performed locally, with native versions, logo resources and
bundled license resources checked before delivery. Native UI/gameplay/router behavior
is not certified by these tests. No install, commit, push or publication is performed
by this packaging workflow.

## Final Windows package

- Installer: `TCHLauncher_0.27.0_x64-setup.exe` (6,644,406 bytes; 6.34 MiB).
- SHA-256: `17dc6a1e8894ed258c19c9ab874e508ff92f00ef81f4ecdd7921363e5452c6e8`.
- Executable and installer file/product versions are 0.27.0; embedded icons match the supplied logo.
- All four copied license resources match their source hashes and are included in the NSIS script.
- The final build includes the latest world inventory read/retry changes. A checksum file is adjacent to the installer.
- Generated locally; not installed or published.
