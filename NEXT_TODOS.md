# TCHLauncher: remaining acceptance work

Windows is the active platform. The full original manager objective remains open.
`REVIEW` means implemented with recorded evidence, with the complete native/user
flow still awaiting acceptance. `TODO` means an outstanding gate. `DEFERRED`
follows an explicit maintainer instruction. These labels do not mean completion.
The maintainer has accepted the current visual design; visual approval is no
longer a completion blocker. Accessibility and functional checks remain scoped
to their own evidence.

The maintainer owns pending manual validation and explicitly asks that its absence
not block implementation. Report unverified behavior without claiming a pass;
continue actionable work without repeated acceptance or screenshot requests.

## Build and publication state

- Latest published preview: [0.48.0](https://github.com/Hakaelll/TCHLauncher/releases/tag/v0.48.0).
- Latest local Windows installer: 0.49.0, unsigned and not published on GitHub.
- Published source: `382aca1b4b645c0952cc5c485d33e4186dc5d95e`. See the
  [0.48.0 changes](docs/release-notes/RELEASE_NOTES_v0.48.0.md) for animation,
  connection, console and download-buffer fixes and their scoped checks.
- Local 0.49.0 source: `daac81c7d11068ce06d792d2d002ed586d34598d`. See the
  [0.49.0 changes](docs/release-notes/RELEASE_NOTES_v0.49.0.md). TypeScript,
  Windows frontend, optimized offline NSIS packaging, versions and icons pass.
  Only the 0.48/0.49 installers are retained locally; checksums stay local.
- Deferred post-0.49 cache cleanup is complete: 1,923,146,108 logical bytes of
  generated files removed, with both retained setups and the executable unchanged.
- The maintainer requests publishing the version that closes the current batch.
  Intermediate versions remain local; the usual 3–4-version cadence does not
  postpone that final publication. The 0.48.0 publication fulfills this request.
  Attach only the EXE and keep checksums local. Commit and push continue
  independently; the usual cadence applies to later intermediate versions.
- Pending manual interaction checks remain scoped below and maintainer-owned.
- Unreleased after local 0.49.0: the task panel can hide completed results in one
  action, preserving failures, cancelled/active work and replacement task IDs.
  Focus returns to its heading. Eight ES/EN callback cases and frontend checks
  pass; native popover interaction remains manual validation.
- Console errors now use danger coloring and alert semantics, while successful
  copy/export notices remain status messages. Twelve scoped ES/EN callback/JSX
  cases and frontend checks pass; native clipboard/export remains manual validation.
- Eight overridden CSS declarations are removed with unchanged remaining order,
  context and final root values. The Windows frontend build passes; this is
  maintainability cleanup, not a measured performance or native visual claim.
- Local list searches share accent/case normalization with settings, commands
  and menu typeahead, preserving original names and distinct IDs. Eighteen
  focused cases, five retained frontend tests and frontend checks pass. Native
  interaction remains maintainer validation; console/API queries are unchanged.
- Packaged in local 0.49.0: startup-preference read failures now translate both
  cached-data notices. Before/after component renders, TypeScript and the Windows
  frontend build pass; native startup/IPC is outside that check.
- JVM installation-preview and operator-conflict notes also use existing
  translations. Focused paragraph renders cover English/Spanish and conditional
  visibility; no native runtime or permission-operation result is claimed.
- Modrinth pagination moves focus and the viewport to the results status. Both
  directions and an absent target pass focused callback checks; TypeScript and
  the Windows frontend build pass. Native interaction remains manual validation.
- Players and pre-import file lists now follow the same paging behavior, with
  focused result counts or labeled lists. Both directions and absent refs pass
  eight callback cases; TypeScript and the Windows frontend build pass.
- RAM and CPU now share inspected time, preserving gaps and supporting Escape
  back to recent readings. Server/process/range ownership and unchanged pointer
  samples pass 32 scoped ES/EN component cases; frontend checks pass. Native
  interaction remains maintainer validation, not an implementation blocker.

## Evidence already available

- Current 0.48 application-source Paper 1.21.4 passes three lifecycle sessions,
  backup/restore and a populated three-dimension folder export/import into a
  second instance. All 45 copied file hashes match before boot and three saved
  blocks remain after boot. Five sessions return real TPS readings; source and
  export are unchanged. Temporary data is removed and package hashes are
  unchanged. This verifies the native engine with a test event collector, not
  Tauri interaction or Minecraft client gameplay.

- Current 0.48 application-source Vanilla 1.21.4 lifecycle passes with real managed
  Java/Minecraft downloads, three ready/status/command/graceful-stop sessions,
  tracked backup, byte-identical world restoration before boot and restored-world
  startup. An isolated temporary library uses loopback and an existing explicit
  EULA record. Owned processes exit and temporary data is removed. The test event
  adapter is not the Tauri UI; native interaction and affected-host cases remain
  unproven. Installer/executable hashes are unchanged.

- The maintainer explicitly accepts the current visual appearance: "tu sigue que
  visualmente esta bien". This closes the general visual-design review. It does
  not assert screen-reader behavior, every keyboard path or affected-host outcomes.
- The maintainer reports that Windows preview 0.38.0 works well. Individual scenario
  steps and results were not supplied.
- For 0.39.0, the maintainer explicitly confirms all four requested scenarios:
  hide to tray and restore with an active server, stop and exit, startup after a
  Windows login, and a friend's connection from the Internet. This is user-reported
  acceptance, not an automated observation. Router protocol/lease cleanup, account
  identity and the later tray-language change are outside that report's scope.
- Installed 0.37.0 silently in an isolated directory, upgraded it to 0.38.0 and
  uninstalled it successfully. Upgrade preserved the initialized database and a
  marker; uninstall preserved the populated test library. Original installation,
  database, WebView data and registry were restored after the probe.
- The installed 0.38.0 executed its Windows Run command with `--autostart`, started
  a selected real Vanilla 1.20.4 server and completed scheduled backup/restart,
  stop and stopped-server backup. Both sessions reached Minecraft readiness;
  both backup directories contained world/level.dat. Preferences/tasks were
  seeded directly in the isolated database, so that probe did not exercise their
  UI/IPC or an actual Windows login. No desktop interaction is claimed for it.
- Real isolated Minecraft 1.21.4 lifecycles passed for Vanilla, Paper, Fabric with
  Lithium and NeoForge: managed Java/install, three ready/status/command/clean-stop
  sessions, backup/restore and restored boot. Restore checks level.dat bytes.
- Historical NeoForge 1.20.1-47.1.106 also passed with managed Temurin 17, three
  sessions, level.dat/MOTD restoration and restored boot. Its temporary library
  was removed. This does not cover native selector scrolling or additional mods.
- Two Vanilla instances passed simultaneous independent PIDs, ports, commands and
  stops. Vanilla 1.21.4 to 1.21.5 conversion/rollback restored 20 original world-file
  hashes and the JAR; this is separate from Minecraft client gameplay.
- Real Fabric world/content archive import checked 23 manifest hashes, unchanged
  source files and a second-instance boot. A controlled Fabric/Lithium `.mrpack`
  additionally checked exact loader, optional/client exclusions, overrides,
  protected settings, receipt/hashes and a real target session.
- Paper's populated three-dimension export/import checked 45 world-file hashes
  before boot and three distinct saved blocks through Minecraft commands after
  boot. Paper/Chunky also passed real plugin enablement and restored JAR equality.
- World-publication recovery passed eight real child-process kills at durable
  publication/cleanup boundaries with exact synthetic dimension bytes, repeated
  recovery and fixture removal. Power loss and pre-journal death remain unproven.
- The compact retained suite has 24 cases: five frontend read/recovery checks,
  ten native safety/recovery checks and nine opt-in real Minecraft scenarios.
  Most former tests were removed at the maintainer's request. Old test counts
  do not describe current regression coverage.
- Recent focused checks cover shared access-file writes, UUID-selected OP grants,
  task error/attempt reporting, NeoForge catalog interpretation and streamed
  SHA-1/SHA-256/SHA-512 transfers. Frontend production builds and relevant ES/EN
  React checks passed where recorded. No throughput/RAM improvement is claimed.

[Architecture](ARCHITECTURE.md) describes current owners and safeguards.
[Integration setup](src-tauri/tests/fixtures/README.md) identifies retained engine
scenarios. Detailed observations remain in
[development notes](docs/development-notes/DEVELOPMENT_NOTES_v0.31.0.md).

## Remaining acceptance by area

Source review confirms implemented UI/API/native paths for content, existing-server
import, players and worlds. Their remaining work is acceptance of the recorded
flows and maintainer-owned manual checks, so they use `REVIEW`, not `DONE`. The review does
not establish unobserved native interaction or client outcomes.

| ID | Status | Area | Remaining acceptance evidence |
| --- | --- | --- | --- |
| CORE-01 | REVIEW | Multiple servers and persistence | Two real Vanilla processes pass independent simultaneous states/commands/stops. Verify native creation/reselection, event presentation and multi-client gameplay. |
| CORE-02 | REVIEW | Managed downloads and Java | Four real isolated engine installs pass. Streaming downloads now compute only the requested digest; focused local transfer checks preserve hash verification and existing files on mismatch. Verify native download/runtime recovery and custom Java selection. |
| CORE-03 | REVIEW | Start, console, stop and restart | All four real engines pass three ready/status/command/graceful-stop sessions. Paper startup Stop verified with real Java. Console publication now pauses when the document is hidden and resumes the bounded latest-line buffer; direct App-effect/buffer checks cover ordering, reset, disposal and stale callbacks. Post-0.47 source also skips unchanged follow-scroll animations and counts pending lines without another filtered array; actual effect/count checks and frontend builds pass. Verify native IPC, tray visibility events and slow saves. |
| CORE-04 | REVIEW | Properties and capacity | Shared integer limits and recoverable file/SQLite saves implemented. The shared property reader/writer now handles colon/whitespace separators, escaped keys and continued logical records while preserving unrelated source bytes; direct native module checks and full Cargo check pass. The existing real Vanilla 1.21.4 case now saves hard difficulty/creative mode through the native settings owner before any world exists, then verifies startup mode and read-only seed/difficulty replies (42/Hard). It retains the three-session backup/restore flow and adds no test case; installer unchanged. Verify native edits and other version-specific gameplay rules. |
| CORE-05 | REVIEW | Backup and restore | Tracked real world restore and restored boot pass for all four engines; level.dat byte equality checked. Version 0.47.0 keeps post-restore refresh failures by section, localizing at render time. Actual refresh callback/component checks cover all four reads, language changes, successful retry, selected-server consent and stale revisions; retry never repeats restoration. Read/React boundaries are controlled. Verify native restore/retention and interruption. |
| BACKUP-STOP-01 | REVIEW | Copy after clean stop | Real Paper exit produces one durable copy without replay; startup-stop is excluded. Verify native observer, immediate restart and window exit. |
| CORE-06 | REVIEW | Close with active processes | Maintainer confirms 0.39.0 hide/restore with an active server and stop/exit. Current source returns structured stop/timeout/backup failures per server and renders each separately in ES/EN; exit requires an empty failure list and failures permit retry. Actual shutdown function, serialized fixture, React component and App callback checks cover mixed outcomes, all-success, delayed clean stop, retry, IPC rejection, hide, duplicate invocation and unmount. Process/window/API boundaries are controlled. Cancel, native failure feedback and multiple-server interaction still need direct evidence. |
| CORE-07 | REVIEW | Measured telemetry | CPU, resident RAM, heap, I/O, uptime, storage and startup observations connected. A missing RAM reading now preserves available CPU/disk samples and leaves a gap in the memory line; focused checks cover continuity, gaps, retention and process isolation. Memory report sessions now reset before clock/order or unavailable-RAM checks, and startup clears only that server’s chart history to prevent PID-reuse mixtures; production recording/callback probes and native/frontend builds pass. Real Paper TPS response verified across three sessions; verify native plots and process-session transitions. |
| CRASH-DIAG-01 | REVIEW | Crash evidence | Bounded durable session reports and real Java failure/relaunch verified. Verify native recovery actions and actual Minecraft crash cases. |
| CORE-08 | REVIEW | Download cancellation and retries | Temporary publication, validation and retries implemented. Post-0.47 source returns response buffers directly instead of copying each chunk to another Vec. Real loopback transfers preserve SHA-1/256/512 verification, no-hash downloads, cancellation, previous targets and partial cleanup; native test compilation passes. No CPU/RAM benchmark or HTTPS/native interaction claim. Verify native cancellation/retry and interrupted installs; HTTP Range resume is not implemented. |
| JVM-01 | REVIEW | JVM settings | Profiles, custom arguments and persistence connected. Verify custom profiles through native restart and runtime selection. |
| JAVA-01 | REVIEW | Custom runtime | Executable inspection and version validation implemented. Verify native picker, corrupt runtime recovery and custom-runtime lifecycle. |
| LOAD-01 | REVIEW | Fabric | Real 1.21.4 / Loader 0.19.5 lifecycle with installed Lithium and restored mod bytes passes. Verify native imports/updates and client mod behavior. |
| LOAD-02 | REVIEW | NeoForge | Real 1.21.4 / 21.4.158 and historical 1.20.1 / 47.1.106 lifecycles pass, including managed Java, three command/stop sessions and restored-world boot. Verify native flows and an actual modded client session. |
| MC-VERSIONS-01 | REVIEW | Version catalog and experimental opt-in | Official catalogs and opt-in filtering implemented. NeoForge catalog and loader selection now share the bounded release parser, including the existing 1.20.1 artifact identity; numeric/prerelease/size checks pass. Historical 1.20.1 installation and lifecycle pass. Verify native selection/scrolling and snapshots. |
| MC-UPDATE-01 | REVIEW | Minecraft upgrades and rollback | Real Vanilla 1.21.4 to 1.21.5 world conversion/rollback passed; all 20 world-file hashes restored before boot. Verify native flow, other engines/content migrations and client gameplay. |
| MOD-01 | REVIEW | Mods and Paper plugins | Verified plans, dependency traversal, hashes, enable/disable, updates, removal and diagnostics connected. Main Fabric JARs now show advisory Minecraft requirement mismatches for configured stable releases; real local JAR audit checks cover OR alternatives and excluded uncertain cases. Lithium/Fabric and Chunky/Paper load through the real install/restore harness. Verify native reviews, broader content and gameplay. Fabric environment strings now normalize case/defaults and reject non-string types; a focused native JAR/provider-alias/client-exclusion check and ES/EN message checks pass. Version 0.47.0 preloads explicit root versions before dependency traversal, preventing order-dependent rejection of valid bulk plans, including a retained beta. A local production-graph probe reproduces the previous conflict and checks order, aliases, exact conflicts, cycles, compatibility/identity failures and bounds with controlled catalog/artifact boundaries. Fabric candidate checks remain conservative, not a global solver. |
| PACK-01 | REVIEW | Modpack import | Reviewed selection and staged publication implemented. Real Fabric 1.21.4/Lithium mrpack install and boot passed, including exact loader, override precedence, excluded client/optional files and protected settings. Completed imports retain acknowledgement if library refresh fails; retry refreshes data only. Complex third-party packs and native/client outcomes remain open. |
| IMPORT-01 | REVIEW | Existing server import | Detection, copy/manage choice and recoverable publication implemented. Acknowledged servers remain visible and continue preparation after a library refresh failure. Isolated native inspection and SQLite import pass eight copy/manage cases for renamed Paper/Fabric, legacy Vanilla and an explicitly confirmed unknown launcher; source/copied bytes and permission inventories remain intact and scripts are never run. These are metadata fixtures, not executable Minecraft launchers. Verify native interaction and real custom-launcher boot. |
| IMPORT-02 | REVIEW | World import during creation | Wizard world copying and validation implemented. Verify native source selection and worlds from different versions. |
| PLAYER-01 | REVIEW | Players and permissions | File inventories, online commands, whitelist import, OP and bans connected. Online rows now use session-observed UUIDs from server logs and show unknown identity explicitly; historical names do not select their skins. Known identities can be added directly to the whitelist by UUID while stopped, with existing-entry status and preserved native acknowledgement. Banned IP rows now support stopped-server removal with a safety copy, preserving live Minecraft commands while running. Earlier imported Minehut identity/skin report is resolved by maintainer confirmation on October 31, 2026; broader client flows remain manual validation. |
| WORLD-01 | REVIEW | World operations and combined export | Real Fabric/Lithium ZIP export/import and populated Paper three-dimension folder export/import pass. Paper checks 45 world-file hashes before boot and three saved blocks after boot. Verify native/client round trips and broader content. |
| NET-01 | REVIEW | LAN connection | Local address, port state and loopback Minecraft status implemented. Post-0.47 LAN/NAT-PMP copy feedback is scoped to the actual address and latest request; controlled component checks preserve unrelated errors, disposal, LAN timing and ES/EN labels. Frontend builds pass. Verify native clipboard and connection from a separate LAN client. |
| NET-TUNNEL-01 | DEFERRED | External tunnels | Bore removed at user request; no external tunnel is required. Revisit only if requested. |
| STORAGE-USAGE-01 | REVIEW | Storage readings and warnings | Bounded cached directory measurements and targets implemented. Refresh now reconciles untouched warning controls with stored values, including recovery from an initial read failure, while preserving unsaved edits; direct callback checks pass. Verify native access failures, large worlds and threshold feedback. |
| NET-02 | REVIEW | Internet and router mapping | Maintainer confirms a friend's Internet connection on 0.39.0. UPnP/NAT-PMP discovery, temporary leases, renewal/cleanup and manual fallback are implemented; Post-0.47 source separates polling errors from operation errors for both protocols; successful reads clear only read failures, and unverified status blocks new opening/confirmation while preserving removal. Controlled component callbacks and ES/EN markup checks plus frontend builds pass. Physical-router protocol/cleanup and CGNAT feedback remain unverified. |
| TASK-01 | REVIEW | Local scheduling and autostart | Installed 0.38.0 passed selected-server autostart and scheduled real Vanilla backup/restart, stop and stopped-server backup. Maintainer confirms actual Windows-login startup on 0.39.0. Preference/task UI and error recovery still need direct evidence. |
| UX-01 | DONE | Visual design acceptance | The maintainer explicitly accepts the current application appearance. Earlier isolated library captures cover three desktop sizes. General visual approval no longer requires screenshots. Keyboard and assistive-technology behavior are tracked separately. |
| UX-ACCESS-01 | REVIEW | Keyboard and accessibility | Shared labels, ES/EN control markup, modal focus recovery and reduced-motion component checks passed. Shared action menus now support accent-insensitive prefix navigation and repeated-letter cycling; production keyboard callbacks pass ES/EN, disabled/empty-list, timeout and composition checks. Broader keyboard paths and screen-reader interaction remain unverified; the maintainer's visual approval does not establish them. |
| PLAYER-BANS-01 | REVIEW | Ban metadata | Player and IP records show a visible expiry summary and labeled reason, source, creation date and expiry details. Missing expiry remains unknown; the native forever sentinel is localized. Values/time zones remain verbatim and pardon actions are unchanged. ES/EN component checks, TypeScript and the frontend build pass. |
| RAM-ADVICE-01 | REVIEW | Local memory guidance | Creation, server settings and modpack import show an advisory starting heap based on physical RAM, server type and configured slots. Rules reserve at least 2 GiB or 25% for other work, omit unsupported estimates and never change saved memory. Local rule and ES/EN component checks pass. Real workload needs depend on content; no measured RAM reduction is claimed. |
| UX-TASKS-01 | REVIEW | Background operations | Measured task events and reconciliation connected. Verify native popover, navigation, cancellation and partial/error outcomes. |
| UX-MOTION-01 | REVIEW | Motion preferences | Persisted preference and system reduced-motion precedence tested. Shared dialogs now cancel pending exits when an operation becomes busy; an isolated headless component check verifies the race, subsequent close, focus return and reduced-motion close. Post-0.47 source skips unchanged initial numeric/chart frames and reduced-motion list geometry/cloning, clears snapshots, and recovers numeric interpolation from non-finite starts. Controlled actual hook/component checks and frontend builds pass; no measured CPU/RAM gain claimed. Verify native transitions and focus stability. |
| UX-KEY-01 | REVIEW | Commands and search shortcuts | Ctrl+K/Ctrl+N connected; Ctrl+F skips hidden/inert/disabled searches and includes Players. The palette now opens server folders without navigating and restarts running servers through the existing lifecycle. Commands are revalidated against current state before execution. Verify native keyboard paths and modals. |
| APP-SETTINGS-01 | REVIEW | Application preferences | Categories, themes/language, Java, paths, networking, retention and advanced settings connected. The selected ES/EN language now synchronizes Show/Quit tray labels through a validated native command. Application info/location reads have a read-only retry. Native interaction review remains pending; library-location migration is not implemented. |
| I18N-NATIVE-01 | REVIEW | English native messages | Added 843 messages across administration, recovery, networking, content, engines, startup, tasks and diagnostics. Fourteen wrappers mark error-owned arguments while preserving names, paths and external output. Content notes and close failures translate separately. Actual locale/catalog checks, React summaries and frontend builds pass; native contract changes also pass offline checking/packaging. A Rust token reader reviews 4,415 literals beyond the first inventory; exclusions remain documented. Five native picker commands receive the current language; fifteen actual API calls cover ES/EN/ES transitions with IPC substituted. Complete native dialogs, language switching during normal use and copy paths outside the audited cases still need interaction evidence. Packaged in 0.46.0. |
| APP-LOG-01 | REVIEW | Launcher diagnostics | Bounded rotated local JSONL logs implemented. Verify native folder opening and OS-session error reporting. |
| UX-02 | REVIEW | Server image | Launcher and Minecraft server-icon.png use the selected image. Verify native crop/remove and the Minecraft multiplayer list. |
| UX-03 | REVIEW | Automatic sidebar | Hover/focus expansion, contextual server actions and empty-library handling implemented. Verify native resizing, long lists and pointer/keyboard transitions. |
| UX-04 | REVIEW | Creation and settings experience | Deferred wizard, seed, pre-start defaults and searched gameplay-first settings with sticky save controls implemented. Shared capacity sliders now use translated visible labels and announce MB/GB/percent units; ES/EN static React markup checks pass. Verify native flow, validation, focus, screen-reader behavior and narrow layouts. |
| UX-05 | REVIEW | Deletion confirmation | Centered modal, explicit managed/external scope and focus return implemented. Confirmed native deletion now removes the local server/history/tasks immediately and uses the existing read-only recovery banner when refreshing the library fails. Production callback checks pass for deletion failure and successful deletion with either refresh result. Verify from scrolled settings and last-server deletion. |
| REL-01 | REVIEW | Windows distribution | Silent clean install 0.40.0, upgrade to 0.43.0 and uninstall passed with a preserved user-owned fixture, unchanged original executable/database hashes and exact original registry restoration. No application launch or native UI was exercised. Maintainer confirms 0.39.0 login startup, tray/exit and external play. Interactive installer choices and broader usage remain open. Linux is deferred; unsigned Windows builds are previews. |
| REL-02 | REVIEW | Current installer | Local 0.49.0 NSIS x64 built from commit daac81c, unsigned and not published. TypeScript, Windows frontend, optimized offline packaging, executable/setup versions and embedded icons pass. It groups synchronized RAM/CPU inspection, pagination focus and translated notices. GitHub remains at 0.48.0; local retention keeps 0.48/0.49. Native interaction and gameplay were not repeated; maintainer-owned validation remains nonblocking. |

## Implemented scope from earlier audits

The former implementation-gap entries now have source implementations and recorded
checks. Their remaining native/client boundaries are represented in the table
above. Keep these IDs for tracing history rather than treating their implementation
notes as new tasks:

`WORLD-PAPER-01`, `WORLD-PAPER-RECOVERY-01`, `IMPORT-PREVIEW-01`, `WORLD-PAPER-IMPORT-01`, `PLAYER-OFFLINE-01`, `MOD-CHANGELOG-01`, `WHITELIST-REMOVE-01`, `ACCESS-REMOVE-01`, `NOTIFY-01`, `ACTION-RECOVERY-01`, `ACCESS-WRITER-01`, `OP-GRANT-01`, `PLAYER-ACTIONS-01`, `TASK-ERROR-01`, `GENERAL-ERROR-01`.

Version 0.39.0 includes general action-error guidance/log access, one NeoForge
release parser for discovery and selection, computation of only the requested
download checksum, and cancellation of interrupted dialog exits.

## Closure checks

1. Exercise native creation/import, first consent, managed download, readiness,
   console command, stop/restart and backup/restore through the Windows UI. Record
   the version and actual result; the existing engine runs do not replace native
   interaction evidence.
2. General visual design is accepted by the maintainer. Review remaining keyboard,
   focus, screen-reader and reduced-motion behavior where evidence is still absent.
   Native UI automation remains excluded; do not reopen the screenshot request as
   a prerequisite for accepting the appearance.
3. Exercise close cancellation/failure handling and scheduling/preferences UI.
   Background tray, stop/exit and actual login startup passed in the maintainer's
   0.39.0 report. Real installed-app scheduled actions also passed independently.
4. Review interactive installation choices. Silent clean install, upgrade and
   uninstall passed against isolated data; the original environment was restored.
5. A friend's Internet connection passed in the maintainer's 0.39.0 report, and
   the later affected-host reports are now resolved by maintainer confirmation. Where
   router mapping is supported, verify temporary lease cleanup; otherwise verify
   the manual fallback and accurate unsupported-state feedback.

No gate should be marked complete from a compiled button, an unobserved operation,
a sample value or a broad claim about a different version/scenario.

## Earlier reports resolved by the maintainer

On October 31, 2026, the maintainer confirms that the earlier issues on the other
computer have been fixed for some time: Internet stalls, Fabric memory growth,
missing content and the imported account/skin mismatch. Close these reports on
that user-provided evidence; do not keep asking for their old logs or treat them
as blockers. This is maintainer confirmation, not a newly reproduced local test.

The maintainer owns remaining manual validation and explicitly requests continued
implementation. Unobserved checks stay documented without stopping independent
improvements or reopening these resolved reports.

## Deferred scope

- Linux: [LINUX_LATER.md](LINUX_LATER.md), explicitly deferred by the user.
- Fabric client instances and Microsoft sign-in: explicitly withdrawn for now.
- External tunnel providers: Bore removed; Playit rejected by the user.

Historical iteration evidence belongs under [development notes](docs/development-notes/).
Older roadmap wording is recoverable in Git history. Do not treat historical
pending lists or test counts as the current state.
