# Development notes — 0.29.0

## Real chart inspection

`chartData.ts` owns filtering by server/PID and selected time window, finite-value
inspection, keyboard sample selection and memory axis sizing. Windows are anchored
to the latest native sample. Data is copied before sorting; another process or
server cannot appear merely because its PID matches.

The memory domain includes both resident RAM and measured heap and fits actual
values with headroom. Without a positive reading it falls back to configured Xmx.
Missing readings are never interpolated as zero. Existing segmented polylines
continue to preserve gaps; interval changes reset interpolation identity. Actual
segment endpoints remain visible even when there is only one sample.

Shared interval selection affects the display only: no new polling, timer, native
request or third-party dependency. Heap/process toggles cannot hide every available
series. A retained time selection must belong to the current visible samples.
Selected readings announce on interaction; live polling does not continuously
announce the reading row. Reduced-motion preferences remain authoritative.

## Layout

Dedicated `performance.css` retains End/Obsidian/Nether role tokens and Minecraft
lettering, with conventional reading text for units, timestamps and help. Plots
adapt their height to the window. Their legends use explicit pressed buttons;
Java explanations and startup/report detail use native disclosures.

The Modrinth catalog refinement from the preceding source iteration is included.
Its project icons, category selectors, loading state and publication actions retain
the existing real catalog and installation workflows.

## Evidence

- 88 frontend regressions passed, including four new graph-data tests.
- TypeScript and Vite production compilation passed.
- 24 performance text/theme pairs passed 4.5:1 contrast; minimum 6.55:1.
  The preceding catalog pass checked 30 pairs with the same minimum.
- Source encoding and diff whitespace checked.
- Native engine code is unchanged in this milestone. The 0.28.0 native suite
  previously passed 239 tests with 28 conditional tests ignored; it was not rerun
  merely for the frontend layout changes.
- Native screenshots, keyboard interaction, actual gameplay, physical router
  reachability and the friend's reported RAM/internet stutters remain unverified.

Suggested commit: `feat: refine Minecraft charts and content discovery`

## Windows artifact

The final offline NSIS build completed successfully after the compact plot-height
adjustment. Installer: `TCHLauncher_0.29.0_x64-setup.exe`, 6,675,710 bytes.
Executable and installer file/product versions are 0.29.0. Embedded icon pixels
match the supplied logo resource. Four license destinations are present in the
generated NSIS script. The installer has a neighboring SHA-256 file.

SHA-256: `7b94e40536f2f69bed2883e0e6b49bfb4d414f14d7328c3166a2044382bf53a0`.
