# TCHLauncher 0.49.0

Windows x64 preview, built locally. The installer is unsigned.
This version is not a GitHub release; the published preview remains 0.48.0.
Packaged source: `daac81c7d11068ce06d792d2d002ed586d34598d`.

## Changes since 0.48.0

- RAM and CPU charts inspect the same timestamp with pointer or keyboard input.
  Missing values stay visible as gaps and dashes. Escape returns both charts to
  recent readings; selection belongs to its server, process and time range.
- Repeated pointer movement within the same sample preserves inspection state
  instead of updating both charts again. Sampling frequency is unchanged.
- Modrinth, player and pre-import file pagination restores focus and scrolls to
  the beginning of results, following the installed-content and backup lists.
- Startup recovery, JVM installation-preview and operator-conflict notices use
  the existing English translations when English is selected.
- The maintainer confirms the earlier reports from the other computer are
  resolved. Those reports are closed on that confirmation, and remaining manual
  validation does not block independent improvements.

## Validation and limits

Focused checks reproduce the previous independent chart timestamps and verify
shared inspection, missing values, Escape, pointer exit, context ownership and
unchanged-state identity across Spanish/English. Actual paging callbacks cover
both directions and absent focus targets. Component/paragraph renders verify the
translated notices. TypeScript and the Windows frontend build pass.

Hooks, reads and focus/pointer boundaries are controlled in these checks. Native
visual, keyboard and screen-reader validation remains with the maintainer. No
measured CPU/RAM improvement is claimed. No dependency or retained test is added.
Optimized offline Windows NSIS packaging, executable/setup versions and embedded
icons pass. The installer is 6,901,187 bytes. Local retention keeps 0.48 and 0.49;
the obsolete 0.47 installer was removed after hash verification. Build-cache
cleanup is deferred while other compiler processes are active.
