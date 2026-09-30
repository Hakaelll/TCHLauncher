# TCHLauncher v0.6.0

This release refreshes the desktop interface and adds live process monitoring for testing a local Vanilla or Paper server.

## Changes

- Simplified the first-run screen to a short heading and clear Create and Import actions. Removed the large server icon and repetitive introductory copy.
- Reworked the overview, forms, settings, buttons and navigation around the supplied grayscale design: lighter surfaces, consistent controls, quieter borders and clearer spacing.
- Added short transitions for view changes, wizard steps, the sidebar and dialogs. Busy indicators animate only during an operation; reduced-motion preferences disable these effects.
- Added a two-minute RAM/CPU history with current and peak values. History remains available when switching between Summary, Console and Settings, and resets for a different server process.
- Added measured disk read/write rates, Java process ID and the configured heap limit. Process RAM is shown separately from the heap limit. Missing samples and long gaps are not displayed as zero or joined with invented data.
- Kept player counts, uptime, folder usage, recent events, log warning/error counts and backups together in the overview. Local connection information and the TCP port check remain available beside performance data.
- Moved deletion and other confirmation dialogs into the native modal top layer. They are centered in the visible window even when Settings is scrolled down, block background interaction and return focus without scrolling on cancellation.
- Added a Stop action while a server is starting. Startup output cannot change a stopping process back to Running, and a finished crash remains visible after the process exits.
- Refresh server properties after lifecycle changes while preserving unsaved property edits.
- Updated Paper download requests to identify the application version and project contact URL, as required by the [PaperMC downloads service](https://docs.papermc.io/misc/downloads-service/).

## Try a real server

1. Create a Vanilla server, select a Minecraft release and allocate RAM appropriate for your computer. Paper is also available when a stable build exists for the selected version.
2. Install the server and its managed Java runtime. Read and explicitly accept the Minecraft EULA before starting.
3. Start the server. Use Console to follow startup; Running is reported after Minecraft finishes loading the world.
4. Open Minecraft Java Edition with the same version. Use the displayed LAN address, or `localhost:<configured port>` when playing on the host computer.
5. Check the live metrics and player list, send a console command, then Stop the server before editing properties or making a backup.

The launcher uses actual files and a managed Java process. It does not simulate a Minecraft server. No EULA was accepted automatically during development.

## Verification

- TypeScript compilation and the production frontend build passed.
- All 21 Rust tests and 6 telemetry tests passed. Coverage includes file operations, local port checks, crash-state retention, history ordering and measured disk rates.
- The Windows NSIS x64 installer was generated locally.
- The follow-up code review scored all four requested fixes resolved: retained graph history, refreshed properties, grayscale states and stopping during startup. This review did not certify visual rendering.
- Native installation, a complete server start/stop session and joining from Minecraft remain for manual validation. The interface was not operated through computer use or browser automation.

## Known limitations

- Graph history is kept in memory for the current launcher session. CPU usage represents the Java process's share of the whole computer; these are not TPS measurements.
- Warning and error counts cover the recent log records loaded by the launcher, rather than an all-time total. Folder size is cached for up to one minute.
- Paper availability depends on its published stable builds. The version selector currently uses Mojang's release catalog.
- Internet access requires manual firewall and router configuration. Automatic UPnP/NAT-PMP mapping and remote port checks are still pending.
- Fabric, NeoForge, mod/plugin management, modpacks and scheduled tasks are not included yet.
- The Windows installer is not digitally signed. A Linux release package has not been built for this version.
