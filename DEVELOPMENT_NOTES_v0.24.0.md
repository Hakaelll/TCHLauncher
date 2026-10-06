# Unreleased

## Startup preferences and scheduled-task recovery

- Read OS autostart and server-start preferences independently. Expose retries and retain explicitly labelled last-known settings after a read failure; disable unconfirmed preference edits.
- Publish native acknowledgements into the read controller, invalidating queries started before the write. Keep acknowledged settings after failed synchronization; disposed/different selections reject acknowledgements.
- Guard preference dispatch against duplicate calls and obsolete components. Refresh startup options when the library's server identities/names change; late listener delivery is ignored after disposal.
- Close scheduled-task creation after native acknowledgement, show completed changes separately from follow-up list errors, reject duplicate dispatch and validate whole-number intervals. Existing native scheduler ownership and execution semantics remain unchanged.
- Refine startup section headings, retry controls, server icon slots and text-backed selection states with current End materials. Preserve native checkboxes, keyboard focus and narrow layouts.
- All 72 frontend behavioral tests, TypeScript and Vite production build pass. Two new read-controller tests cover acknowledgement/read races, failed synchronization, retry and disposed/different scopes. Native visual/keyboard, IPC races and actual OS-session autostart remain unverified.
- Three native autostart tests and nine scheduler tests pass using isolated storage/registration fixtures, including durable preferences, unique ownership, concurrent claims and interrupted-job non-replay. No live OS autostart registration was changed.
- Source-only after the generated 0.23.0 installer; no additional installer generated.

## Coordinated application shutdown

- Request graceful stops for every selected active process before waiting. Use one shared 45-second deadline, aggregate server-labelled failures and still run configured post-stop copies for cleanly stopped servers. A failed request, unclean exit or copy cannot skip the remaining owned servers.
- Keep the application open on batch errors; automatic force killing is not added. Retain owned reservations until stop observation and configured copies finish.
- Recheck operation/process activity in final exit handling, including a start admitted before shutdown. Restore the close choice if Java remains active; reject new ordinary stopped/running reservations and imports while shutting down. Internal post-stop copy reservations remain available during cleanup.
- Keep close errors in the protected confirmation dialog, preserve background/cancel choices after errors and guard hide/stop dispatch against duplicates. A successful exit request stays busy until the application exits.
- Three shutdown orchestration regressions, two process tests and two post-stop backup tests pass. The process test terminates/reaps only its explicitly owned child fixture; no user server is stopped. All 72 frontend tests, TypeScript and Vite build pass.
- Native window/tray/IPC shutdown races, real Minecraft graceful shutdown and router cleanup during app exit remain pending. These helper gates do not prove the complete desktop lifecycle. Source-only after 0.23.0; installer unchanged.

## Installed inventory refinement

- Combine independent log-state and source filters with trimmed search across filenames, titles and all declared IDs. Preserve separate files with equal titles and severity-first ordering without altering native inventory order.
- Consolidate search, source, ordering and presentation into one workbench. Add text-backed state counts, visible filter reset and result totals, and explain disabled file mutations while the server or an operation is active.
- Refine End-themed file slots, warning/error edges, status separation, compact spacing and empty inventory presentation. Retain existing details, version, removal and filtered-console actions.
- All 74 frontend tests, TypeScript and the production build pass. Two new regressions cover combined filters, secondary IDs, stable equal-name files and non-mutating ordering. Native visual and keyboard review remains pending. Source-only after 0.23.0; no installer generated.

## Linux verification preparation

- Add a Debian Bookworm container build route with locked package installation, frontend/native tests, Debian packaging and exported verification logs/checksums. Exclude host build artifacts and local configuration from its context; restrict output to an empty workspace directory.
- The PowerShell runner parses successfully. Docker Desktop startup was requested, but the Linux engine did not become available and its status query was cancelled after remaining unresponsive. The container build and Linux package are unverified; this preparation does not establish Linux support.

## Fabric dependency presence review (Windows development)

- Review required dependency IDs across dedicated-server Fabric descriptors, provided aliases and recursively declared nested JARs. Associate missing providers, including transitive declarations, with their owning top-level file; client-only files cannot supply server dependencies.
- Read archives without executing or extracting them. Bound descriptor/JAR bytes, entry counts, node count and depth; reject unsafe paths and ambiguous descriptor entries.
- An unreadable/unsupported descriptor, included archive, exceeded budget or dependency override prevents negative absence conclusions. Per-file incomplete reads remain visible. Presence is not a version-range, loader-selection or gameplay verdict; optional declarations are not required automatically.
- Four real archive/filesystem audit regressions pass, including aliases, nested/transitive dependencies, client-only providers, missing nested data, override preservation and read limits. TypeScript and production build pass. Native interaction and real modded-world verification remain pending; source-only after 0.23.0.
- Linux work is deferred at the user's request. Preparation and remaining gates are retained in LINUX_LATER.md; Windows is the active platform.

## World creation and pre-start settings

- Add an optional text seed to the new-world step and final review. Keep signed 64-bit numbers as exact strings; encode backslashes, spaces and Unicode for Java properties. Reject control characters/oversized values before creating server files.
- Persist initial game rules in new server.properties before Java runs. Expose missing core settings for existing servers through a read-only editor merge; real values win, unknown properties are preserved and saving publishes selected values through the existing recoverable settings operation.
- Retain numeric legacy choices and omit simulation distance before 1.18. Seed selection is ignored for copied existing worlds; changing a seed does not regenerate an existing world.
- Integrate wizard actions with the form surface, remove the separate dark footer band, show the next step and group the seed field with world selection. Native visual/keyboard review remains pending.
- Content inventory reads now use the scoped read controller, preserving labelled last-known data after failures. Native installation/removal acknowledgement closes confirmation before refresh; synchronization failure stays separate and retries reads only. Mutation ownership rejects repeated dispatch and ignores late results after unmount; keep active installation confirmation across emitted installing/stopped state changes.
- 218 offline native tests pass; 25 external/runtime/helper integrations are excluded. All 74 frontend tests, TypeScript and Vite pass. New creation/property tests verify real files before Java/world/EULA exist, lossless seeds, read-only defaults and injection rejection. Actual gameplay and native interaction remain unverified.
