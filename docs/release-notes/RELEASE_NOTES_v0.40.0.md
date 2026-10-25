# TCHLauncher 0.40.0

## Mods and diagnostics

- The installed Fabric inventory flags recognized Minecraft requirement mismatches
  against the configured stable release. Experimental IDs, unknown predicates,
  client-only files and dependency overrides remain outside this inference.
  Warnings help review a file; they do not block startup or prove gameplay behavior.
- CPU and disk samples remain available when a RAM reading is missing. Memory
  plots show the gap rather than bridging an unavailable measurement.
- Chart history stores only plotted values instead of full diagnostic responses.
  No measured reduction in application or server RAM is claimed.

## Settings and recovery

- Native tray Show/Quit labels follow the selected Spanish or English language.
- Library counts and management action names use the selected language.
- Capacity sliders use the translated visible label and expose memory, disk and
  percentage units to assistive technology.
- Refreshing storage measurements updates untouched warning controls while
  preserving unsaved edits, including recovery from an initial read failure.
- A confirmed server deletion remains successful if the following library read
  fails. The server and its chart/startup/task data leave the local view, and the
  existing refresh banner retries the read without repeating deletion.

## Verification and limits

A focused native audit over real local JAR metadata passed mismatch and exclusion
cases. Focused checks cover partial telemetry, chart field retention, deletion
acknowledgement and storage refresh recovery. ES/EN React markup checks cover
library counts and slider labels. The native tray command compiled and its
language validation passed. These checks do not replace native UI or client play.

This remains an unsigned Windows x64 preview. The full manager objective remains
open. Acceptance reported for 0.39.0 is not automatically transferred to this
build. Linux and Minecraft client launching remain deferred.

Only `TCHLauncher_0.40.0_x64-setup.exe` is attached to the GitHub release. Checksum
files remain local.

TypeScript, the Windows frontend production build and the optimized native build
passed. Executable and installer product versions are 0.40.0; their embedded icons
match the application emblem. The installer is 6,867,997 bytes. The native linker
emitted an informational library-creation message. Installer interaction and
native visual review remain pending for this build.
