# TCHLauncher 0.27.0

## Lighter observations during gameplay

- Batch live console updates in a bounded 1,000-line buffer, retaining order and
  immediate native lifecycle events.
- Read player files only for active live-server views; parse them on a worker.
- Pause periodic player observations while the document reports hidden and slow
  metrics to fifteen seconds. Refresh on return without overlapping automatic reads.
- Collect only needed process counters and system RAM. Bound retained counter
  sessions to 32 observed servers and reset the CPU baseline on every new launch,
  including when Windows reuses a PID.
- Share mod/plugin diagnostic parsing across inventory cards.
- Avoid repeated Minecraft TCP probes in routine router maintenance. Explicit
  opening validation, mapping ownership, renewal and cleanup remain enforced.

No relay, external service or new dependency is added. These changes do not establish
that the user's internet-only connection stutters or Minecraft RAM growth are fixed.

## Clearer contextual actions

- Keep Export and a contextual menu beside worlds, preserving activation,
  duplication, full-server backup, folder opening and confirmed deletion.
- Group installed mod/plugin version review, validated Modrinth page opening and
  removal in contextual menus. Keep Details/Notice and related Console actions visible.
- Support menu buttons and right-click, viewport clamping, enabled-action keyboard
  navigation and focus restoration without resetting focus on ordinary refreshes.
- Dismiss obsolete content menus after a file hash/filter/view change.
- Distinguish failed world reads from an empty world list. Retain last-known worlds,
  provide manual refresh/retry and block file mutations until the inventory is confirmed.
- Preserve End, Obsidian and Nether materials, existing motion preferences and languages.

## Verification and limits

- All 84 frontend tests passed; TypeScript and production frontend compilation passed.
- The native offline suite passed 232 tests with 28 condition-dependent tests ignored.
- Synthetic diagnostic analysis of 100 files / 1,000 log lines returned identical
  evidence while averaging 25.35 ms before and 1.04 ms after shared parsing.
- Synthetic console-buffer evidence and native counter/maintenance regressions are
  recorded in development notes. They are not native rendering, resident RAM or
  gameplay latency measurements.
- Native visual/keyboard/browser-dispatch review, actual gameplay, physical router
  behavior and the friend's mod/RAM/internet case remain pending. Linux is deferred.
- Vite's initial-chunk size advisory remains. No user server or installer was run.

## Windows package

- Installer: `TCHLauncher_0.27.0_x64-setup.exe` (6,644,406 bytes; 6.34 MiB).
- SHA-256: `17dc6a1e8894ed258c19c9ab874e508ff92f00ef81f4ecdd7921363e5452c6e8`.
- Executable and installer file/product versions are 0.27.0; embedded icons match the supplied logo.
- All four copied license resources match their source hashes and are included in the NSIS script.
- The final build includes the latest world inventory read/retry changes. A checksum file is adjacent to the installer.
- Generated locally; not installed or published.

