# TCHLauncher 0.29.0

## Performance charts

- Rework the RAM and CPU plots with cleaner square framing, clearer scales,
  quieter guides and more readable sampled lines. Both remain visible together
  in standard desktop windows.
- Select a shared 30-second, one-minute or two-minute interval.
- Fit the memory axis to actual samples instead of flattening the plot against
  a large configured Xmx. Resident process memory and heap use share the scale;
  Xmx remains visible separately and is not treated as a process RAM limit.
- Show or hide the process RAM and heap series independently, keeping at least
  one available series visible. Missing heap readings remain unavailable.
- Inspect real readings and timestamps with the pointer or arrow keys. The
  readout sits above the graph and no longer covers its lines.
- Keep gaps in measurement visible and show a square marker for isolated samples.
- Group Java, Minecraft response, TPS and process/disk readings more clearly;
  move detailed explanations into optional disclosures.

## Modrinth discovery

- Larger project artwork, distinct pixel category selectors and grouped search
  and filters give the catalog a more coherent Minecraft inventory appearance.
- Separate project information from compatibility and installation actions.
- Add quiet loading placeholders and improve narrow-window layouts and pagination.
- Localize result totals, authors and release actions in Spanish and English.

## Verification and limits

88 frontend tests passed, including time-window isolation, heap-only inspection,
keyboard navigation and memory scaling. TypeScript and production compilation
passed. Text token contrast was checked across End, Obsidian and Nether.

Native visual/keyboard review and Minecraft gameplay remain pending. These changes
do not diagnose a memory leak, improve measured internet latency or certify mod
behavior in a world. Linux remains deferred.

## Windows package

Installer: `TCHLauncher_0.29.0_x64-setup.exe` (6,675,710 bytes / 6.37 MiB).
File/product versions and the embedded golden logo were checked for the executable
and installer. The four license resources are included in the NSIS file list.
Generated locally; not installed or published.

SHA-256: `7b94e40536f2f69bed2883e0e6b49bfb4d414f14d7328c3166a2044382bf53a0`.
