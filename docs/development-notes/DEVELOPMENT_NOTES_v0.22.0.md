# TCHLauncher — 0.22.0 development record

### Independent console and player reads

- Load metrics, console history, EULA and players independently so one failed query cannot discard successful data from another panel. Preserve live console lines while history loads or retries.
- Scope player reads and manual retries to the selected server; discard superseded responses, including selection away and back. Retain last-known player data and label failed refreshes in Players and Overview.
- Use compact End-material loading/error strips with readable details and direct recovery controls. Refine player summary inventory slots, numeric hierarchy and console line-count styling without adding ambient motion.
- All 60 frontend behavioral tests, TypeScript and Vite production build pass. Desktop visual/keyboard review remains manual.


### Recoverable properties loading and protected drafts

- Give server properties an independent selected-server read owner with explicit loaded/loading/error and dirty states. Reject superseded replies, invalidate pending reads on edits/write acknowledgement and preserve unsaved properties during automatic status updates.
- Show an inline End-themed properties status and retry action. Hide property/capacity controls before a successful read; block editing and saving after read errors or during refresh while retaining last-known values for review. Keep runtime and maintenance controls available independently.
- Preserve unsaved RAM/storage capacity changes when server metadata refreshes. Reset them on server selection or acknowledged save/restoration. The combined native settings write remains the persistence path.
- Separate successful settings writes from later metadata-refresh failures. Route restoration property replacement through the same owner, explicitly discarding obsolete drafts only after native restoration has succeeded.
- Four asynchronous properties-draft regressions, TypeScript, production Vite build and whitespace checks pass. Native UI races, capacity editing and keyboard/visual review still require desktop assessment. Source-only after 0.21.0.

### Backup list ownership and restoration follow-up

- Load backups independently from metrics/logs. Distinguish initial loading, a valid empty list, a failed read and retained last-known copies. Add explicit list refresh and prevent restoration from a stale or refreshing list.
- Use one selected-server read owner for initial loads, scheduler updates and callbacks from world/content/retention operations. Discard superseded replies, including when switching away and back to the same server; invalidate reads on disposal.
- Close restoration confirmation after the native operation succeeds. Refresh properties, backups, EULA and server metadata independently; show a per-server follow-up with a read-only retry action when any query fails. Keep configuration edits disabled until synchronization succeeds and reject pre-restoration properties/EULA replies.
- A successful manual backup remains recorded as created even if later list/server refresh or retention reports a warning. Refine backup-row hierarchy and inline restoration follow-up using existing End materials.
- Three asynchronous frontend regressions pass for read failure/retry, overlapping responses, server reselection, disposal and last-known data. TypeScript, production Vite build and whitespace checks pass. Native desktop interaction, gameplay and full restoration QA remain pending; source-only after 0.21.0.

### Backup retention controls and cleanup feedback

- Add manual policy/list refresh, an unavailable initial-read state and guarded replies scoped to the current server. Refresh a clean draft while preserving unsaved local limits; disable editing during a policy write or cleanup.
- Validate count and byte limits before saving, including the requirement for at least one effective limit when automatic deletion is enabled. Associate errors with fields and focus the first invalid input.
- Close the cleanup confirmation after native deletion is acknowledged. Report a subsequent list-read failure separately, allowing manual refresh without offering to repeat a completed deletion. Preserve native cleanup warnings.
- Use End materials and readable dates, sizes and recorded Minecraft versions in retention review. Improve restoration details at narrow widths and use explicit disclosure state and inline unsaved feedback.
- Two frontend boundary tests, six native retention tests, TypeScript and production Vite build pass. Desktop async interaction, visual/keyboard review and real-world Minecraft backup/restoration remain pending. Source-only after 0.21.0.

### Automatic restart settings recovery

- Add manual status refresh and an explicit unavailable state after initial read failure. Keep observed policy changes synchronized when the form is clean, while retaining unsaved local edits.
- Separate acknowledged saves from subsequent status-read errors. Do not report a successful native write as a failed save or display old attempt counts after that write. Ignore superseded and previous-server replies.
- Validate integer delay/attempt limits before submitting, associate errors with their fields and focus the first invalid field. Show unsaved/acknowledged-save feedback inside the panel, without extra notifications.
- Two frontend policy-boundary/reconciliation tests and five existing native restart tests pass; TypeScript, production build and whitespace checks pass. Asynchronous desktop interaction, real Minecraft restarts and visual/keyboard review remain unverified. Source-only after 0.21.0.

### Scheduled task presentation and recovery

- Give scheduled tasks raised inventory materials, pixel clock slots, readable next-run dates, rectangular state badges and a separate last-result area. Add a purposeful empty state and give automatic restart settings the same End theme materials; retain responsive controls and existing actions.
- Distinguish recorded errors, interruptions and cancellations using labels and semantic color. A requested start remains a request, without claiming Minecraft is ready.
- Add manual refresh and a recoverable initial read failure. Ignore superseded list replies and replies from a previous server; prevent an old mutation from closing the new server's form.
- Two presentation regressions, TypeScript, production Vite build and whitespace checks pass. Native visual/keyboard review and real scheduled Minecraft execution remain pending. Source-only changes after 0.21.0; no new installer generated.

The latest Windows preview is documented in [0.21.0](../release-notes/RELEASE_NOTES_v0.21.0.md). The development history below is retained for context.

## Included in 0.21.0

Release gate: 188 native tests and 41 frontend tests pass; 24 external/runtime/helper checks are excluded from the offline suite. Separate real Paper plugin installation, TypeScript/Vite, Windows NSIS packaging, native versions and embedded icons pass. See the dedicated notes for the checksum and manual verification limits.

- Read bounded Paper YAML descriptors for declared plugin name, literal version, main class and API version. Reject malformed or ambiguous identity fields without executing plugins or expanding YAML collections. Scalar aliases remain supported. A real LuckPerms installation integration passes without Java or EULA acceptance.
- Read NeoForge TOML metadata for declared IDs (including multiple mods in one JAR), display name, literal version and the primary mod's Minecraft dependency expression. Show all declared IDs in details, detect duplicate secondary IDs and associate their explicit console references with the shared file. Expressions remain informational; metadata does not certify loading or gameplay.
- Read declared NeoForge PNG logos through the existing bounded local icon pipeline. Malformed TOML, invalid/duplicate IDs and escaping icon paths are rejected or shown as unavailable. Unresolved version placeholders stay unknown. Format follows the [official NeoForge mod-file documentation](https://docs.neoforged.net/docs/gettingstarted/modfiles/).
- Compile diagnostic reference patterns once per installed file rather than for every log row. Nine frontend attribution tests, five native descriptor/icon tests and five native content-storage regressions pass; three external content-storage integrations were excluded. TypeScript/production build pass. NeoForge gameplay and native visual review remain pending; included in 0.21.0.
- Persist an explicit forced-exit flag with backwards-compatible reads of existing diagnostic files. Retain forced shutdowns in failure history even if the OS exit code is zero, preserve earlier real error findings and present a clear forced-close heading instead of an unknown crash cause.
- Require diagnostic severity or recognized unheaded JVM/loader failures for saved error findings. Structured INFO mentions, player messages and commands cannot become known failure diagnoses. Four diagnostic persistence/rule tests and two process tests pass (including actual owned-child termination and durable report); two external/helper fixtures stay excluded. TypeScript and production build pass; Minecraft/native visual review remains pending.
- Remove the automatic Java kill after thirty seconds of graceful shutdown. Offer confirmed force termination only after normal stop has been requested, with a native process-session token to reject stale dialogs. Forced exits are unsuccessful and user-requested, excluding clean-stop backups and crash retries; a pending restart refuses to launch a successor after an unsuccessful exit.
- Verify force termination against an owned native test child, including state/session rejection, PID preview, actual termination/reaping and forced-success override. TypeScript and production build pass; native Minecraft shutdown and dialog review remain pending. Included in the 0.21.0 preview.
- Add installed-content card/compact presentation, alphabetical or attention-first ordering, a direct attention filter and filtered-console actions on related diagnostic cards. Preserve existing file review, version planning and confirmed removal. Use theme tokens and reduced-motion-aware state feedback.
- Recognize raw NeoForge/Log4j diagnostic headers and explicit warning prefixes when associating console evidence with installed files; INFO headers terminate diagnostic continuations. Eight attribution tests and TypeScript/production build pass. Native visual review remains pending; included in 0.21.0.
- Separate world-save start, per-dimension completion, whole-save notification and explicit failure in the native parser. Require recognized server messages for operational events; mentions in chat, commands or plugin-prefixed output do not become save completion.
- Show observed save state and the last whole-save notification in Overview. Only an actual failure creates the failure review; exit without completion stays unconfirmed, and a new observed startup clears old-session save evidence. A server notification is not an independent disk-durability check.
- Derive console severity from structured headers, Java exception lines or explicit diagnostic prefixes. Preserve the observed unbracketed Log4j warning format and stop coloring ordinary INFO descriptions as errors from keyword matches.
- Bound history reads to the initially observed file extent and 512 KB, ignoring unfinished records at the snapshot boundary. Export remains a separate byte-preserving snapshot operation.
- TypeScript/production build pass. All 39 frontend tests passed, followed by ten focused attribution/save tests after the final fallback changes. Sixteen related local native tests pass (log/application-log, startup, player replay and crash diagnostics). A separate read-only integration against the user's supplied Paper log also passes. No Minecraft process or native visual review was exercised.
- Recognized save-message examples were checked against the [Paper issue log](https://github.com/PaperMC/Paper/issues/9142). These changes are included in 0.21.0.

## Included in 0.20.0

- Replace the installed-content file list with Minecraft inventory cards, bounded local Fabric icons, search, attention/source filters and expanded file details. Preserve reviewed updates, stopped-server mutations and confirmed removal with backups.
- Associate recent real console diagnostics with explicit mod IDs, names, filenames and bounded Fabric dependency continuations. Show related errors in red and warnings in amber with evidence and direct filtered-console navigation; associations do not establish causation. Chat, commands, generic mentions and INFO discussion are excluded.
- Add restrained item entrance/hover motion and selected-tab feedback using the existing themes and reduced-motion controls. Keep filters and details while refreshing inventory. Read the rail version from the package manifest.
- Native visual review and real third-party mod failure verification remain manual; see the dedicated release notes for validation and packaging evidence.

## Included in 0.19.0

- Add release-level compatibility, category/sort/version filters and persistent browsing to Modrinth discovery. Add read-only JAR descriptor checks and bounded Fabric startup-list comparison, with explicit unknowns instead of claiming that inventory presence proves loading.
- Reject modern Fabric client-only or malformed descriptors during reviewed Modrinth installation. Keep legacy descriptors explicit and preserve installed files. Actual friend's-world symptoms remain unverified without the exact mod names/log.
- Remove duplicate chart meters, simplify chart framing and legends, add sampled area shading and keyboard exploration, and move lengthy network explanations into optional help and port confirmation.
- See the dedicated 0.19.0 release notes for test and packaging evidence.

## Included in 0.18.0

- Remove Bore from production. Discover routers automatically, preserve confirmed openings and show internet access without a disclosure. Move Minecraft status queries to Performance and hide historical failure review when no report exists.
- Adopt Minecraft inventory compartments, authored pixel icons, measured segmented indicators, orthogonal charts and native segmented progress bars. Keep unavailable values, real transfer totals, cancellation, themes and reduced motion.
- See the dedicated 0.18.0 release notes for verification and packaging.

## Included in 0.17.0 (changes after 0.16.0)

- Add explicit Minecraft Server List Ping in Connection, with a validated status response and matching pong, local query latency, advertised version/player counts and sample time. Bound packets and the complete exchange, keep unavailable data explicit, respect disabled status queries and preserve authentication/properties. IPv6 wildcard listeners now use IPv6 loopback for local checks. Three protocol/socket regressions and three network regressions pass; live Minecraft and native UI review remain pending.

- Distinguish resident Java process RAM from used and committed heap. Read bounded, validated HotSpot counters for the managed PID without JDK tools, JMX ports, forced garbage collection or new JVM flags. Overlay measured heap usage with a dashed line on the existing memory chart, preserve gaps and drops, and show collector invocation totals separately. Clarify that Xmx limits heap rather than all process RAM. Parser and explicit Java 21 integration, seven frontend telemetry tests, TypeScript and Vite pass; shared counters are unavailable in the execution sandbox and the real-Java check passed outside it. This does not diagnose the reported Fabric 1.21.11/30-mod retention on the friend's computer; gameplay observations remain necessary. Not included in the 0.16.0 installer.

- Measure server storage with checked filesystem reads instead of silently accepting partial totals. Keep unavailable storage distinct from an empty folder and return its error alongside CPU/RAM metrics. Cache successful measurements for 60 seconds and failures for 15 seconds, independently per server and directory; file operations invalidate the existing cache. Show the measurement time, omitted symbolic links and a scoped error with access to storage settings. Three real-filesystem storage regressions, TypeScript and Vite pass. Native desktop review and unreadable-folder permission checks remain pending. Not included in the 0.16.0 installer.

- Add an observed Java → World → Ready startup path in Overview, with explicit world-loading and spawn-preparation status from native server logs. Never infer readiness from spawn percentages; accept only the existing ready event. Keep progression monotonic per process and clear old metrics on restart, rejecting outstanding replies from an earlier start. Emit phase changes in real time with the process session, keeping the metrics snapshot as a fallback. Reject older-session phase updates and repeated/regressing events. Use a compact semantic list, status text and reduced-motion-aware color transitions. Two local native regressions, two frontend event regressions, an explicit read-only integration with the supplied actual Paper log, TypeScript and Vite pass. Live Minecraft/Tauri and native visual review remain pending. Not included in the 0.16.0 installer.

## Included in 0.16.0

- Use one normalized, validated player event for console classification and connected-player tracking. Support colored Paper output without losing joins/leaves; reject chat, commands and plugin text that merely end with those phrases. Preserve chat delimiters, including unprefixed and unsigned chat, instead of interpreting them as console headers. Keep raw log bytes and use the same events on saved-log reads. A real-file regression verifies live/read-back identity agreement, duplicate events and forged join/leave text. Minecraft connections and native UI review remain pending. Included in the 0.16.0 preview.

- Add a stopped-server editor for existing operators: levels 0–4 and joining when the player limit is reached. Preserve UUIDs, names, other operators and unknown JSON fields; reject stale permissions, missing/duplicate identities and malformed files. Sync a copy of the exact original ops.json before replacement and retain it in the server directory. Keep grant/revoke actions on native Minecraft commands. Four player tests, TypeScript and Vite pass; actual server permission effects and native keyboard/visual review remain pending. Included in the 0.16.0 preview.

- Measure startup time for the current Java process from launch to the first recognized server-ready message. Show the duration in Performance, keep it unavailable before readiness or after process removal, and reset it on each launch. Use the same native process snapshot as PID and uptime; no estimates or historic values are substituted. TypeScript and Vite pass; native Minecraft readiness and desktop review remain pending. Included in the 0.16.0 preview.

- Add opt-in automatic backups after a clean server stop, including stops from the console, restarts and stopping before launcher exit. Require a successful native exit from a session that reached Running, reserve stopped files and use tracked copying, cancellation and saved retention. Claim each session durably before copying; cancelled/failed/interrupted attempts do not replay. Show the stored result in Backups and mark snapshots with the stop reason. Shutdown waits for file operations and stop copies instead of exiting while files are being copied. Two actual filesystem/SQLite regressions pass; native Minecraft/desktop shutdown remains pending. Included in the 0.16.0 preview.

- Store explicit backup reasons for manual/scheduled copies and safety copies before restore, Minecraft/loader/content updates, content removal and world changes/deletion. Show the saved reason, Minecraft version and engine in the backup list and restore confirmation. Read metadata in a native worker with bounded reads and managed-path checks; retain other rows when a copy has invalid metadata. Old backups show unrecorded information instead of the current server settings. Twelve backup tests and TypeScript/Vite pass; native visual review remains pending. Included in the 0.16.0 preview.

- Validate saved scheduled occurrences before claiming or re-enabling them. Disable malformed rows without replay or blocking valid tasks, use checked recurrence arithmetic and preserve the original stored values for review/removal. Show invalid tasks safely in the task list, with an explicit status and removal instead of activation. Nine scheduler tests and TypeScript/Vite pass; native desktop interaction remains pending. Included in the 0.16.0 preview.

- Catch unexpected scheduled-action and automatic-restart worker failures, persist a readable failure through the existing result path and release worker capacity on every exit via an owned permit. A local SQLite/thread regression verifies recorded failure, released capacity and no replay of a one-time task, including unwind during result handling. Seven scheduler tests pass; native Java scheduled stop/restart remains pending. Included in the 0.16.0 preview.

- Keep the latest failed process report separately from the latest exit. Successful later sessions preserve the failure for an explicit, dated historical review in Overview, including after reopening the launcher. The current-crash endpoint still requires the exact current session. Support the previous single-file format on upgrade. Three local diagnostic tests, the real Java 21 failure/success integration and TypeScript/Vite pass; native desktop interaction remains unverified. Included in the 0.16.0 preview.

- Isolate malformed NAT-PMP records and individual transport/cleanup failures so other servers still receive lease renewal and shutdown cleanup. Preserve rejected records without contacting their destinations, retain failed cleanup for retry and report errors after processing all valid records. Two new local regressions cover malformed JSON, mismatched destinations, invalid stored ports, renewal, shutdown and retry. Included in the 0.16.0 preview; physical-router verification remains pending.

## Included in 0.15.1

- Fix saving standard dotted Minecraft properties, including query.port, rcon.port and rcon.password. A full-properties save regression preserves comments, query/RCON values and authentication settings while changing PvP, and rejects injected keys/values without changing the original file.
- Verify the native exit-report path with actual Java 21 using a deliberately unsupported class followed by a successful process. Preserve stderr/log evidence, real exit codes, terminal states and separate session reports without launching Minecraft, accepting EULA or creating a world.
- Add deterministic crash diagnostics from bounded output of the current Java process. Preserve the exit code, session identity, stop request and output-drain status in an atomic local report before publishing the terminal state. Detect known Java version, heap/native memory, bind, mod-loader and plugin-loading errors without sending logs to an external service.
- Connect a failure review in Overview with recommendations, navigation to configuration/content/connection/console, expandable evidence and separately identified recent mod/plugin file changes. File timestamps are not presented as causal evidence. Old process reports are rejected for a new session.
- Prevent chat, joins and genuine player commands from acquiring error tags merely because they contain error keywords. Keep real plugin errors containing command-related text classified as errors.
- Two local diagnostic rule/persistence tests and six log/application-log tests pass; TypeScript and production build pass. Native Java crash lifecycle, keyboard/visual review and Minecraft gameplay remain unverified. These changes are included in the 0.15.1 hotfix. Native Minecraft/desktop verification remains pending.

## Included in 0.15.0

- Connect NAT-PMP as an optional router alternative in Connection: detect the gateway, confirm temporary TCP opening, retain the assigned external port, renew while the server stays available and attempt removal after stop or launcher exit. Persist intents and failed cleanup for retry. Three local transport/lease tests pass; physical router, native controls and internet reachability remain unverified. An account-free Bore alternative is included below; Playit was declined.
- Add an explicit Paper TPS query in Performance with 1/5/15-minute readings, semantic low-rate feedback and a timestamp. Send the namespaced native command through the current Java process, accept only recognized INFO responses from stdout within ten seconds of a request, rate-limit requests and expire readings after one minute. Never reuse previous-process logs or estimate TPS from CPU. Two local parser/lifecycle tests, TypeScript and production build pass. Real Paper command/response and native interaction remain unverified; included in 0.15.0.
- Track primary-world changes and world deletion, including their measured safety copies. Allow cancellation before file mutation, retain completed backups on cancellation, revalidate the selected world after backup and protect publication from cancellation. Keep active worlds undeletable and avoid a backup for an unchanged selection. Refresh the backup list after cancelled/failed changes. Five local world tests pass; native interaction review remains pending. Included in 0.15.0.
- Track world duplication and export with measured copy progress, cancellation before publication and validation of staged world data. Preserve the original world and existing destinations, remove cancelled/invalid staging copies and route task review back to Worlds. Keep the stopped-server reservation through terminal reporting and registry cleanup. Native visual interaction remains pending; these changes are included in 0.15.0.
- Add measured storage breakdowns for worlds, mods/plugins, logs, other files and backups, with actual free disk space and an explicit refresh time. Persist a configurable warning threshold and apply it to the server overview without enforcing a quota. Reject incomplete scans instead of reporting partial totals as success. Two local native storage tests, TypeScript and the production build pass; native visual review remains pending. Included in 0.15.0.
- Integrate Bore as an optional account-free TCP tunnel in Connection. Confirm the local destination, copy the assigned public address, monitor worker state and close control/player streams after stop or application exit. Five contract/lifecycle tests and two local protocol/forwarding tests pass. An explicit real bore.pub echo integration passed; native interaction and Minecraft gameplay remain manual checks. No Playit account or external executable is used.

## Included in 0.14.0

- Organize application preferences into General, Java, Storage, Network, Backups, Advanced and About, reusing native controls with explicit server scope. Add real library/folder locations and opening actions, native binary/platform information and EULA status. Complete offline verification passes 135 native tests (18 external integrations excluded) and 24 frontend tests; TypeScript, production build, Windows packaging and embedded-icon/version checks pass. Native visual/keyboard/folder-opening review remains pending. See the release notes for the installer checksum.
- Track mod/plugin removal in the native registry and confirmation dialog. Measure the server safety copy and content preparation separately, allow cancellation before the content switch, retain any completed backup and preserve configuration folders. Keep the stopped process state, protect both publication boundaries and clear previous terminal progress when confirming another removal. A local regression covers cancellation during backup, after backup and during content preparation, plus successful removal with measured byte totals, retained configuration, original backup files and registry cleanup. Native interaction review remains pending; this is not in the 0.13.0 installer.
- Track server imports before they enter the library. Show measured folder-copy progress and cancellation in the import view and task center, validate the staged launcher and protect publication. Remove cancelled staging copies and prepared managed directories on database-insert failure; keep source files intact when copying. Register external folders with honest stages and no copy percentage. Hold the new-server reservation in the worker through task cleanup. A local registry integration test covers copied/external imports, cancellation, database failure and original-file preservation; native interaction review remains pending. This is not in the 0.13.0 installer.
- Track world imports in the native task registry with measured file-copy progress, cancellation before publication and a verified staged world. Keep the original world intact, remove partial copies on cancellation or validation failure, preserve the stopped process state and route task review to Worlds. Three local world tests cover the existing lifecycle plus measured completion, cancellation, invalid staged data, publication protection and registry cleanup. TypeScript and the production frontend build pass; native interaction review remains pending. This is not in the 0.13.0 installer.
- Refine automatic sidebar motion with a 280 ms eased expansion, a faster 190 ms collapse and a brief delayed label reveal. Fade labels out before the rail closes; keep icon targets and workspace position fixed. Respect application and system reduced-motion preferences.
- Track scheduled backups through the same native task registry and measured copy path as manual backups. Preserve the scheduled stop/resume behavior when a copy fails or is cancelled; report cancellation without claiming an inactive server was restarted.
- Add byte progress for safety copies before Minecraft, loader and managed-content updates within the existing parent task. Reject cancellation during every task's backup-publication phase.
- Release registered backup jobs by exact identity on completion or worker failure, preserve an occupied registry entry and turn a worker panic into a bounded failure outcome. Two local tests exercise a real scheduled claim, completed and cancelled file copies, current-file preservation, active registry visibility, terminal cleanup, duplicate protection and worker failure. Publication-boundary tests cover backup, restore and all three update kinds. Scheduled Java stop/resume and native task-panel verification remain pending. These changes are not in the 0.13.0 installer.

## Included in 0.13.0

- Include tracked restores and their safety copies in the task registry and confirmation dialog. Measure the two copy phases separately and allow cancellation before restoration publication; keep the current server and completed safety copy on cancellation. Keep operation reservations through task completion and registry cleanup. Full local verification: 128 native tests and 24 frontend tests pass; 18 external integrations remain excluded. Build and embedded-icon/version verification pass. See the release notes for the installer checksum and manual verification boundaries.

- Include manually requested backups in native task snapshots and the global task popover. Scan actual file sizes, copy in bounded chunks with measured byte progress and preserve permissions; keep final publication/retention indeterminate. Cancellation and publication share an exclusive boundary: cancellation removes the staged copy before publication, while publication rejects further cancellation. Do not report a stopped server as Installing just because it is being backed up. Nineteen backup-related native tests pass (one external integration test remains ignored), including byte measurements, staged cancellation, retention/recovery and the publication boundary; TypeScript, production build and four task-state tests also pass. Scheduled/safety-copy tracking, restore/import tracking and native UI review remain pending. This change is included in the 0.13.0 preview.
- Add a non-blocking background-task popover for server installations, Java preparation, content/modpack installs and loader/Minecraft updates. Show native download stages and measured transfer data, cancel by exact task ID and open the owning server without repeating an operation. Keep active tasks before recent session outcomes; only matching terminal tasks can be dismissed.
- Reconcile current native task snapshots every three seconds without overlapping requests or overwriting newer events. Recover active progress after frontend reload, remove vanished jobs without inventing a result, and include these tasks in action/menu restrictions. Nine native download tests, four task-state tests, TypeScript and the production build pass. Native popover/keyboard verification and global scheduled/safety-copy, restore and import progress remain pending; these additions are included in the 0.13.0 preview.
- Keep the server rail compact by default and expand it smoothly on pointer hover or keyboard focus. Reveal labels without moving icon targets or resizing the workspace; keep context menus usable outside the rail. Remove the manual collapse control and resting frames around utility icons, with themed hover/selected feedback and reduced-motion support. TypeScript and the production build pass; native visual/keyboard verification is pending. This is included in the 0.13.0 preview.
- Add a persistent Reduce Animations setting alongside theme and typography. Apply it before first paint, disable CSS transitions/animations and respect it in navigation, dialogs, console following and metric animation. Never override the operating system's reduced-motion preference. Existing saved themes/fonts migrate without reset; preference persistence, malformed input, unavailable storage and OS priority are covered by four passing tests. Native visual verification is pending; this is included in the 0.13.0 preview.
- Add a Ctrl+K action palette with server/action search, keyboard navigation and native-state restrictions for start, stop and backup. Keep Ctrl+N and Ctrl+F available outside confirmation dialogs; Ctrl+F focuses the current library, console or content search. Three action/search tests, TypeScript and the production frontend build pass. Native keyboard verification is pending; these changes are included in the 0.13.0 preview.
- Add bounded internal JSONL launcher diagnostics for startup, shutdown, server process states and router-maintenance failures. Records use defined info/warn/error events and validated server identifiers; console messages, player names, commands, credentials and arbitrary error payloads are excluded.
- Keep the current log and one rotated previous log, up to 2 MB each. Add an Open Application Logs action in Settings through a native Windows/Linux folder-opening command.
- Pass two local log serialization/rotation tests, TypeScript and the production frontend build. Native folder-opening/visual verification remains pending. These additions are included in the 0.13.0 preview.

## Included in 0.12.0

- Recognize Paper's short console startup message and ANSI-colored output when promoting a server from Starting to Running. Keep command input available during startup while its Java process is alive. Native verification with the affected imported server is pending.
- Connect explicit UPnP detection, confirmed temporary TCP opening, verification and removal to the Connection panel. Renew only owned mappings while the server remains running at the same LAN address/port, and queue cleanup after stop, configuration changes or exit. Ten local lifecycle and HTTP transport tests cover durable ownership, interrupted requests, cleanup retries, malformed records, renewal and bounded SOAP responses. Physical router and native exit verification remain pending; NAT-PMP and remote reachability checks are not implemented.
- Replace the legacy cube on the populated library's New Server card with the transparent End island artwork used during first setup. Size the illustration separately for grid and list views and respect reduced motion during hover feedback. This change is not included in the 0.11.0 installer.

## Loader updates (development source)

- Add stable Fabric/NeoForge update review for the current Minecraft version, with installed-version identification and stale-review detection.
- Prepare a separate verified loader generation using the configured Java runtime and create a backup before changing launch paths.
- Publish through a durable journal and SQLite update; recover interrupted publication and preserve the previous generation on database failure.
- Connect a stopped-server settings panel, explicit version selection, confirmation, transfer progress and cancellation.
- Verify preservation of worlds/configuration, failed publication rollback, interruption recovery and active-generation protection with local tests.

- Handle canonical Windows launch paths during installed-version review, prepared-generation publication and rollback.
- Verify official Fabric 0.16.10 → 0.19.5 → restored 0.16.10 and NeoForge 21.4.157 → 21.4.158 → restored 21.4.157 on Minecraft 1.21.4 with Java 21. Updated and restored launchers reach EULA refusal, preserving Java/JVM/configuration without accepting the agreement or creating a world.
- Identify imported classic Fabric launchers, embedded unattended Fabric metadata and NeoForge argument files without relying on filenames or managed metadata. Reject contradictory versions and unsafe referenced paths.
- Preserve imported launchers during update publication and database failure, and restore their original launch paths from the safety backup.
- Verify real imported Fabric and NeoForge update/restore cycles with Java 21, and normalize Windows verbatim/UNC paths for Java classloading. Unattended Fabric identity is locally tested; its full live cycle remains pending.
- Pass all 95 local Rust tests; 15 external-resource tests are excluded from the offline suite, with the loader integration run separately for both platforms.

These changes are not included in the 0.8.0 installer. Native visual/gameplay and Linux checks remain pending. Legacy shaded/custom launchers without sufficient identity data need additional identification support.

## Remaining verification and features

- Native visual/keyboard review and Minecraft client/gameplay validation.
- Scheduled Java restart/shutdown/crash and OS-login startup verification.
- Linux builds and integration checks.
- Legacy/custom imported-loader identification, unattended Fabric live integration and complex third-party modpack compatibility.
- Public-network automation and reliable TPS telemetry.
- Remaining distribution/signing and full requirement audit.

These items are not claimed as completed by the preview. See [NEXT_TODOS.md](../../NEXT_TODOS.md) for the implementation and verification status.

## Minecraft version upgrade review foundation (in progress)

- Added a native read-only review command for stable release chronology, runtime requirements, matching Fabric/NeoForge/Paper catalogs, and installed content compatibility backed by SHA-512.
- Manual or modified files and unavailable catalog metadata remain unverified. Reviews include configuration and launcher evidence to detect stale input.
- Transactional publication, content migration, UI, startup verification and real upgrade/restore integration are still pending. This foundation is not included in the 0.8.0 installer.

## Imported player identity correction

- Avoid selecting an old offline UUID simply because it appears first in usercache when displaying an authenticated connected player. Ambiguous proxy/offline identities retain a neutral avatar.
- Preserve separate historical UUIDs, and show authentication-aware access copy rather than claiming account validation for offline servers.
- Covered by three identity selection regressions and TypeScript/Vite builds; not included in the existing 0.8.0 installer.

## Minecraft version publication (in progress)

- Added guarded native update execution with review fingerprint revalidation, incompatible-content blocking, warning acknowledgement, a safety backup, isolated preparation and cancellation before publication.
- Publish the server directory, Minecraft version and Java path using a SQLite recovery token. Imported directories keep their temporary folders on their own volume.
- Added exact stable Paper build preparation with required SHA-256 and preservation/validation of a compatible configured Java runtime.
- Four local publication tests cover rollback on SQLite failure, recovery before/after commit, backup restore and staged path rejection. UI, content migration and real Java upgrade/start/restore checks remain pending; not included in the 0.8.0 installer.

## Minecraft update interface and integration checks

- Connected the settings dialog to stable version selection, native review, per-file compatibility results, runtime/loader summary, acknowledgement and cancellable update progress.
- Real Java 21 upgrade/restore cycles from Minecraft 1.21.4 to 1.21.5 and back passed for Vanilla, Fabric and NeoForge. Java/JVM/configuration preservation and backup recovery were checked; EULA was never accepted and no world was generated.
- Paper 1.21.5 was correctly rejected because the live catalog has no STABLE build. Paper 1.21.8 build 60 was verified as STABLE and its 1.21.4→1.21.8→1.21.4 upgrade/restore cycle passed with Java 21 and no EULA acceptance.
- Native visual QA, incompatible-content migration, actual client gameplay/world conversion and Linux remain pending. No new installer has been generated.

## Managed content migration and interface refinement

- Resolve target-compatible stable mod/plugin releases as one dependency graph during Minecraft upgrade review. Include exact required dependencies, reject conflicts and show proposed changes/changelogs before confirmation.
- Install reviewed content into the isolated prepared instance, preserve unverified files and avoid redundant backups. Update inventory metadata when distinct catalog releases share the same artifact.
- Verify Fabric/Lithium and NeoForge/FerriteCore upgrade/restore cycles using real downloads and Java 21 up to EULA refusal. No EULA acceptance, world generation or gameplay is claimed.
- Present creation, folder import and modpack import directly in the empty library, with responsive layouts and deliberate action placement.
- Consolidate overview metrics, add process/configuration context and emphasize the selectable LAN address. Preserve anthracite controls, semantic data colors and reduced-motion support.

TypeScript builds and the content inventory regression pass. Native visual validation and application startup verification after upgrades remain pending. These changes are in development source, not the existing 0.8.0 installer.

## Post-upgrade startup observation

- Offer explicit startup after a reviewed Minecraft change, disclose world conversion and leave the server running.
- Confirm readiness only from native state. Stopped/crashed processes and unavailable or timed-out observation remain visibly unconfirmed.
- Link failures to the console and retain the safety backup without silently restoring worlds. Navigating away stops observation, not Java.
- Four startup-observation regressions passed; the frontend suite now has 13 passing tests. The full offline Rust suite has 108 passing tests and 16 ignored external-resource checks.

These additions are included in the 0.9.0 Windows preview. Native/client validation remains pending.

## Development after the 0.9.0 installer

- Add opt-in experimental Minecraft catalogs in Vanilla/Fabric creation, with snapshot labeling and an official Mojang/Fabric ID intersection. Keep stable choices as the default and preserve catalog chronology.
- Remove the fifty-release catalog truncation; omit legacy alpha/beta categories and unsafe IDs.
- Require a complete native server-thread startup message and a valid finite duration before treating Minecraft as ready. Chat, plugin task text and incomplete completion messages do not establish readiness.

Snapshot installation/gameplay and native interaction remain unverified. These source changes are not included in the already generated 0.9.0 installer.

Verification for subsequent source: 110 offline Rust tests passed (17 external-resource checks ignored), including catalog opt-in and startup-message regressions. A separate live catalog check passed for official Vanilla/Fabric entries and Minecraft 26.4-snapshot-2 server metadata. TypeScript and Vite production builds passed. Snapshot installation/boot and native UI checks are still pending.

Separate live installation checks passed for Minecraft 26.4-snapshot-2 on Vanilla (42.43 seconds) and Fabric (41.12 seconds) with Java 25. Both reached EULA refusal, preserved initial properties and verified no consent or world was created. This proves snapshot preparation and boot to that gate, not client gameplay. The current 0.9.0 installer remains unchanged.

## Minecraft / End visual refresh — 0.10.0

- Replace the light graphite presentation with a dark Tableland-derived surface system and End-inspired lavender details.
- Add locally bundled Monocraft 4.2.1 to headings and primary actions, retaining readable system/monospace faces for dense information.
- Add raised, resting-dark controls with interaction lighting, square slider grips and coherent dark menus, modals, fields, charts and console tags.
- Replace the application emblem and desktop/tray/browser icons with the supplied gold voxel source.
- Retain all current native command wiring and first-server creation/import/modpack actions.
- Include post-0.9 experimental catalog and strict readiness parsing source changes.
- Verify 110 offline Rust tests and 13 frontend behavioral tests; 18 external integration tests are excluded from the offline run. Validate 24 text/semantic token combinations (minimum 5.23:1) and actual Spanish font glyph coverage.
- Native screenshot/gameplay verification is not performed: the user requested manual visual assessment and prohibited computer use.

### Memory report export

- Performance now exports a native JSON report through the Save dialog, with resident memory, used/committed heap and collector events. It retains up to 30 minutes / 901 actual samples for the latest observed Java session while that server is selected, bounded to 32 server histories in memory.
- Missing heap readings remain null and gaps are preserved. Reports label current configured heap limits separately from measured JVM values; no logs, commands, executable paths, player identifiers or credentials are included.
- Session tokens discard readings interrupted by a restart. Collection does not force GC or change JVM flags. Saving stages a complete file and preserves the previous destination if publication fails.
- Two native regressions cover sample limits, session changes, unavailable heap, excluded private fields and file publication. Native Save-dialog review and the friend's Fabric session remain pending. This is not included in the 0.16.0 installer.

## Development after 0.21.0 (not packaged)

- Replace the creation wizard's Minecraft popup with a bounded, scrollable native selection list. Preserve full catalog chronology, keyboard selection and loader-switch selection validation. Present the four server platforms as native radio choices with themed raised surfaces and selected/focus feedback.
- Explain that the integrated modern NeoForge catalog starts at Minecraft 1.20.2; pre-1.20.1 versions do not exist, and legacy NeoForge 1.20.1 is not integrated. Live official catalog inspection confirms eleven stable Minecraft families, so the lack of older pre-1.20 entries is not a hidden-scroll omission. See the [NeoForge user guide](https://docs.neoforged.net/user/docs/).
- Expose all five Fabric dependency declaration kinds from local JAR metadata, preserving separate OR range alternatives. Use a native disclosure and readable list in file details. Do not equate declarations with satisfied dependencies or successful gameplay.
- Bound declaration counts, alternatives, IDs and range strings; malformed declarations are rejected by the existing descriptor validation before reviewed installation. Follow the [Fabric descriptor specification](https://wiki.fabricmc.net/documentation:fabric_mod_json_spec).
- Seven native descriptor/icon tests, five content-storage tests and nine frontend attribution cases pass; three external content-storage checks remain excluded. TypeScript and production build pass. Native visual/scroll/keyboard review remains manual. These changes are not included in the already generated 0.21.0 installer.

### Paper creation availability

- Replace the general Minecraft list for Paper creation with official stable-build availability intersected with Mojang releases, preserving release chronology and older compatible versions.
- Use the current GraphQL `channels` filter after checking the actual public schema. Bound response sizes, page count and aggregate request time; reject partial GraphQL errors and stalled pagination instead of silently truncating the catalog.
- Require a published stable build, official HTTPS server download, regular JAR filename and SHA-256 metadata before offering a version. Installation still requeries and verifies the exact build; no automatic version substitution is introduced.
- Two local catalog regressions pass for pagination, chronology, snapshot/unrelated exclusion, partial failure, invalid downloads and query limits. A separately selected live integration passes through the application catalog and confirms REST stable-build/checksum evidence for the newest returned version and Minecraft 1.8.8. No JAR was downloaded, Java executed or EULA accepted.
- Follow the [Paper Downloads Service](https://docs.papermc.io/misc/downloads-service/); its displayed GraphQL example currently uses the older singular channel field, while the live API uses channels. These source changes are not in the 0.21.0 installer. Native selection/gameplay remains unverified.

### Import identity and creation polish

- Identify renamed Paper, Vanilla and Fabric launchers from bounded JAR metadata and NeoForge from its validated argument files. Expose the platform/version per launch candidate; filenames alone do not establish identity.
- Reject contradictory or malformed metadata and require manual confirmation when a version cannot be established. Multiple known launchers require selection. Revalidate detected platform and Minecraft version immediately before import, preserving the original files.
- Remove the latest-version fallback from import, including the asynchronous catalog response; the catalog supplies suggestions without replacing missing evidence.
- Refine creation with raised numbered steps, a themed form/action footer, an End-island configuration preview and native world-choice cards. Improve import source and location choices, long-path wrapping, narrow layouts and keyboard focus when returning to the first step. Retain existing motion preferences and action wiring.
- Three native identity/import regressions, ten storage/content regressions and 41 frontend tests pass. Three live content checks remain excluded. TypeScript and production build pass. Native visual and actual imported-server startup assessment remain pending; these changes are not in the 0.21.0 installer.

### Existing server property controls

- Expand labelled settings for forced game mode, hardcore, flight checks, command blocks, spawn protection, world seed/generator/structures, dimension and mob switches, world radius and idle timeout. Group rules, distances and generation separately, using existing Minecraft typography and theme tokens.
- Render controls only for keys present in the actual server properties. Preserve unknown current choice values and nonstandard booleans without silently replacing them; keep other properties in the advanced editor.
- Explain that generation settings do not recreate an existing world and that saved changes apply at the next start. Follow the [Paper server properties reference](https://docs.papermc.io/paper/reference/server-properties/).
- Validate changed numeric and boolean values natively, including query/RCON ports. Retain unchanged imported legacy values so an unrelated edit does not require rewriting the server's settings.
- Four native file-based property tests pass, covering real save/reload, comments, legacy values, unknown keys, rejected changes and unchanged file bytes after rejection. TypeScript and production build pass. Native visual assessment and version-specific gameplay remain pending. Source-only after 0.21.0.

### Recoverable configuration save

- Save properties, initial/max memory and storage target through one native command and one stopped-server reservation, replacing three separate frontend writes.
- Validate all requested settings before publication. Retain a bounded SQLite recovery record until both file publication and the capacity transaction succeed. Restore original file bytes when database publication fails or a save is interrupted.
- Recover before stopped-server operations, startup and application scheduling. Preserve externally edited properties and reject an altered registered directory instead of silently overwriting either.
- Keep committed settings if only old-file cleanup fails. Disable form editing while saving or while the server is active, and avoid a redundant post-save properties read being reported as a save failure.
- Five native settings scenarios pass using actual files/SQLite, including invalid-request isolation, a real SQL trigger failure after file publication, interruption at four publication/restoration points, idempotent recovery, external-change preservation and an unavailable folder without pending work. The full suite before that final folder regression passes 199 native tests (25 external/helper cases excluded) and all 41 frontend tests; TypeScript and production build pass. Native interrupted-app testing remains pending. Source-only after 0.21.0.

### Configuration file recovery refinement

- Preserve an orphaned old copy and staged bytes when the current properties file is missing without a pending recovery record. Report the ambiguity for review instead of deleting the remaining copy or restoring mismatched settings.
- Refuse to replace an existing recovery destination. Reject normal saves of read-only properties before publication and copy file permissions to the staged replacement; interrupted restoration can inherit permissions from the retained old copy.
- Eight native settings regressions pass, including three added cases for retained copies, actual read-only properties, existing recovery destinations and a real filesystem rename error after staging. Original file bytes and SQLite capacity remain unchanged after the rename failure, and the same pending record recovers successfully. Linux permission-mode verification and native interrupted-app testing remain open. Source-only after 0.21.0.

### Player action clarity

- Show the entered player/IP and the selected action's effect before submitting. Use named actions and the existing destructive styling for kicks/bans rather than a generic Apply button. Explain that an IP ban can affect multiple players sharing an address and that OP grants administrative commands.
- Associate name/reason validation errors with their fields and focus the first invalid field. Disable editing during submission while preserving the ability to prepare an action with a stopped server. Keep final IP validation in the native command builder.
- Include the recipient in command-sent feedback without claiming the action completed. Keep existing row actions, list imports and operator settings connected.
- Count native reason limits by Unicode characters rather than UTF-8 bytes, retaining rejection of embedded command controls. Five native player tests and two frontend boundary tests pass; TypeScript and production build pass. Native keyboard/visual review and actual Minecraft command processing remain unverified. Source-only after 0.21.0.

### Player history search and bounded reads

- Add category search by name, UUID or IP and pagination of 50 matching entries. Preserve native ordering, duplicate-name identities, full category totals and all row actions. Clamp pagination when snapshots shrink; keep unavailable data distinct from a valid empty list.
- Restrict list-motion comparison to the visible page instead of serializing the complete history on every search update.
- Check regular files and sizes before opening player lists, then bound reads using the opened file handle. Retain the existing 5 MB native list and 1 MB whitelist-import limits even if the file grows during reading. Invalid JSON remains an error and is never rewritten.
- Six native player tests and two frontend pagination/search regressions pass, covering oversized real files, malformed data preservation, directory rejection, separate same-name UUIDs, input immutability and shrinking/empty results. TypeScript and production build pass. Native navigation and real long-history assessment remain pending; these source changes are not in 0.21.0.


The source-only entries above describe intermediate checks. Their changes are now included in the locally generated 0.22.0 installer. The final aggregate gate is 60 frontend and 204 native tests passing, with 25 external/runtime/helper tests excluded. See RELEASE_NOTES_v0.22.0.md for distribution evidence and outstanding manual gates.
