# Development record for 0.23.0

The following entries record checks while changes were source-only. These changes are now bundled in 0.23.0; see RELEASE_NOTES_v0.23.0.md for final verification and artifact details.

# TCHLauncher — Unreleased

### Runtime observation and acknowledged changes

- Give Java checks independent selected-server read ownership and reject obsolete responses. Refresh when the configured executable, Minecraft version or process state changes.
- A failed check retains last-known details but shows “Not checked” instead of a stale ready label. Distinguish missing, invalid, incompatible and verified-version-with-unknown-compatibility states.
- Guard custom-runtime selection and saving against duplicate operations and obsolete component scopes. Close the confirmation only after native save acknowledgement; distinguish successful persistence from later server-metadata refresh failure.
- Keep success acknowledgement and observation errors separate, with readable inline feedback and End warning tokens.
- Four focused frontend cases, TypeScript and Vite production build pass. Two native runtime tests and a separate real-Temurin-21 compatibility/selection test pass. Desktop interaction review remains pending. These source changes are not included in the already generated 0.22.0 installer.

### JVM profile loading and preview ownership

- Show an unconfirmed profile label until settings are successfully read. Provide an explicit reload, keep local edits protected from reload and surface unavailable reads beside their controls.
- Bind launch previews to server identity, platform/version, directory, launcher, Java path, RAM limits and effective profile arguments. Clear changed previews immediately and prevent saving without a current successful preview.
- Guard saves against duplicate dispatch and component disposal. Keep the native checked-save path and acknowledge persistence only after it succeeds.
- Refine JVM radio choices, focus treatment, reading area and inline unsaved feedback with existing End materials.
- Two preview-context regressions, TypeScript/Vite and two native JVM tests pass. A separate real-Temurin-21 test verifies valid profiles, invalid-option rejection and preservation of the previous profile after failure. Native keyboard/visual review and actual Minecraft startup remain pending. Source-only after 0.22.0.

### On-demand frontend sections

- Load Worlds, Content, Performance and Application Settings on entry instead of bundling them into the initial application script. Preserve the eager library, process controls, console, creation flow and modal entry points.
- Keep a local loading state and error boundary around each deferred section. Module/render failures show an explicit retry and technical detail without replacing the application shell or changing native process state.
- Use a compact End surface without artificial percentages, shimmer or extra ambient animation.
- TypeScript and production Vite build pass. The build manifest confirms four dynamic entries and no eager section imports. Initial JavaScript decreases from 501.30 kB to 437.31 kB (gzip 148.35 to 130.96 kB); this is bundle-size evidence, not a measured native startup-time or memory improvement.
- Native first-entry, failed-module recovery and keyboard/visual assessment remain pending. Source-only after the generated 0.22.0 installer.

### Loader acknowledgement and version-catalog recovery

- Close loader confirmation immediately after the native update is acknowledged. Keep the installed-target notice and native cleanup warnings separate from subsequent server-metadata read failures; offer a read-only metadata retry rather than repeating a completed update.
- Guard loader review/update calls against duplicate dispatch and obsolete component scopes. Ignore late progress-listener delivery after teardown; revalidate stopped state and selected reviewed versions in the UI while retaining native fingerprint checks.
- Distinguish a failed Minecraft-version catalog from a valid empty result and provide an explicit catalog retry. Reject duplicate review/update dispatch, ignore unmounted catalog/review replies and disable update actions when server state or operation availability changes.
- Use compact End acknowledgement and recovery rows. Six native loader publication/recovery tests pass, including rollback, interrupted commits, imported launchers and active-generation/fingerprint protection. TypeScript passes; real downloaded-loader startup and native async/keyboard/visual review remain pending.
- Source-only after 0.22.0; no additional installer generated.

### Operator drafts and acknowledged player actions

- Preserve unsaved operator permission edits during player polling. Compare the original permission baseline with new native values; block conflicting writes and show current permissions with an explicit discard-and-load action.
- Keep expected baseline values in the native checked-write payload. Acknowledged saves reset the draft to submitted permissions; duplicate save dispatch and obsolete component completion are guarded.
- Separate generic action acknowledgement from later server-list refresh failure. Show a read-only recovery notice rather than returning mutation failure or requesting the action again. Stopped-server whitelist changes refresh properties after acknowledgement, preserving the successful-write result.
- Refine operator editing/conflict details with End materials. All 68 frontend behavioral tests, TypeScript and Vite production build pass. Six native player tests pass, including stale-permission rejection, unknown-field preservation and malformed/duplicate identity handling; keyboard/visual review, IPC race assessment and actual Minecraft command outcomes remain pending.
- Source-only after 0.22.0; installer unchanged.

### Historical NeoForge release identity groundwork

- Verify the official historical `net.neoforged:forge` catalog and the 1.20.1-47.1.106 installer SHA-256 without executing it. The historical profile identifies `1.20.1-forge-47.1.106` and stores launch libraries under `net/neoforged/forge`, unlike modern NeoForge artifacts.
- Add a bounded, strict release identity model for modern and historical coordinates, numeric sorting, official installer URLs, profile identity and library paths. Integrate the shared identity into modern installation layout and stable-loader sorting while preserving explicitly catalog-confirmed prereleases.
- Integrate historical 1.20.1 catalog routing, installation coordinates, imported `--fml.forgeVersion` identity and numeric updates. Offer the official 1.20.1 release in creation alongside the modern catalog. Do not imply universal Forge-mod compatibility.
- Official sources: https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/forge and https://maven.neoforged.net/releases/net/neoforged/forge/1.20.1-47.1.106/.
- Verified installer SHA-256: `c1ea1c3be532c4444004efd7f9cc71e1590d010ff44845b93effba61e3bd1526` (7,846,890 bytes). No Java, server execution or EULA acceptance occurred during this inspection. Source-only after 0.22.0.

- An isolated real 1.20.1-47.1.106 installation and copied import both boot with managed Temurin 17 to EULA refusal; properties remain intact and no world is created. Preserve catalog-confirmed modern prerelease import/layout support. Gameplay and historical mod compatibility remain unverified.

### Library inventory and browsing

- Add online/error/pinned filters with actual counts, combined search and name/recent ordering without mutating native order or merging same-name servers.
- Refine square card materials, pinned markers, capacity/action grouping, readable narrow list rows and larger search/view targets using existing End/Obsidian/Nether tokens.
- Expose import actions together, retain creation and artwork, add a filtered-empty reset and enable existing context menus from the cards. Reuse existing list motion and reduced-motion behavior.
- Two browsing regressions and all 70 frontend tests pass; TypeScript and production Vite build pass. Native visual/keyboard review is pending. Source-only after 0.22.0; installer unchanged.
