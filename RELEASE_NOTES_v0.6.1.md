# TCHLauncher 0.6.1

## Visual and interaction updates

- Replace the plain empty library with two TFT-inspired creation and import cards, using original geometric artwork, ivory/brass and sage accents.
- Add restrained color to navigation, controls, server states, destructive actions and console level labels.
- Refine card entrance, hover, press and context-menu motion while respecting reduced-motion preferences.
- Add sidebar context menus with persistent pin/unpin, settings and confirmed deletion. Pinned servers appear first.
- Move performance into its own section and show RAM and CPU charts together, with current readings, peaks and disk throughput.
- Display real pixelated Minecraft skin heads for players with available UUID profiles, backed by a local cache.
- Use the same 64 × 64 PNG server icon in the launcher and Minecraft server list; migrate legacy launcher artwork on startup.
- Disable WebView2 autofill suggestions and command-field autocomplete.

## Runtime and download fixes

- Fix Minecraft 1.17.1 installation by falling back from an unavailable Temurin JRE 16 to the matching JDK 16.
- Add visible Java version, executable path, verification and managed runtime install/reinstall actions.
- Add cancellable downloads, measured bytes/speed, bounded transient retries and explicit retry actions.
- Verify downloaded artifacts before publication and clean incomplete downloads on failure or cancellation.
- Preserve per-server progress during navigation and serialize shared-runtime preparation.

## Validation

- Frontend type checking and production build.
- 30 offline Rust tests and 6 telemetry tests.
- Both official-service integration checks passed and cover Minecraft 1.17.1/Temurin 16 installation and a real Minecraft skin head.
- No EULA is accepted and no server is started by these integration checks.
- Native visual review and joining a server from Minecraft remain manual checks.

## Windows installer

- File: `TCHLauncher_0.6.1_x64-setup.exe`
- Size: 4,689,402 bytes
- SHA-256: `5DBA7793B83410DE2534D57437F7C3504104A669CBC2F5FA89ED9868E41CF4DB`
