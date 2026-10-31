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

- Latest published preview: [0.54.0](https://github.com/Hakaelll/TCHLauncher/releases/tag/v0.54.0), grouping the
  0.52, 0.53 and 0.54 milestones. Only the unsigned Windows x64 EXE is attached;
  GitHub asset size/digest and its source tag are verified.
- Published/package source: `1f4f7a795471f61d3f3eb7c6395ac95df8a858c1`. See the
  [0.54.0 changes](docs/release-notes/RELEASE_NOTES_v0.54.0.md). TypeScript,
  the retained frontend tests, Windows frontend, optimized offline NSIS packaging,
  executable/setup versions and embedded icons pass.
- Latest local installer: 0.55.0, unsigned and unpublished, from
  `ee1669d5610902bce956fcabc02a9f05756ebb03`. See the
  [0.55.0 changes](docs/release-notes/RELEASE_NOTES_v0.55.0.md). Five retained frontend
  tests, TypeScript, production frontend, optimized offline NSIS packaging,
  versions and embedded icons pass. GitHub remains on published 0.54.0.
- Local retention keeps 0.54/0.55; checksums stay local. Generated-cache cleanup is
  deferred because the existing guard detects other active builds; caches remain
  untouched. Retained package/executable hashes are unchanged. The obsolete local
  0.53 installer was verified and removed.
- Post-0.52 source cleanup removes 13 unused platform icon derivatives
  (3,413,415 logical bytes). Four configured Windows outputs keep their hashes;
  actual CLI generation and current executable/setup icon-pixel checks pass.
  These changes are included in local 0.53; Git history/server libraries remain untouched.
- Post-0.52 ZIP features remove unused password encryption and seven Windows
  registry packages, preserving compression/time features. Before/after native
  archive checks pass on identical fixtures; legal inventories are regenerated.
  Local 0.53 is 26,970 installer bytes smaller than 0.52 across all grouped changes;
  isolated dependency-size and runtime memory effects remain unmeasured.
- Publication follows the usual 3–4-version cadence. The requested final batch
  release remains authorized; commits and pushes proceed independently.
- Post-0.55 task reconciliation reuses unchanged maps for idle snapshots and
  retained outcomes, avoiding needless state replacement. Thirteen direct helper
  checks and frontend compilation pass; runtime performance remains unmeasured.
- Application event registrations now retry failures and reconcile missed state,
  logs and backup data after recovery. An ES/EN status notice tracks unavailable
  subscriptions. Eleven controlled helper checks and frontend compilation pass;
  actual native event failure/recovery remains unverified.
- Post-0.53 storage/retention notices now localize skipped links and cleanup
  results at render time, preserving external details. Scoped ES/EN component
  and callback checks and frontend compilation pass; real operations are unchanged.
- Post-0.54 source routes loader/Minecraft update progress through shared tasks,
  recovering active tasks when settings opens and removing duplicate subscriptions.
  Scoped ES/EN progress/acknowledgement recovery checks and frontend compilation pass.
- The modpack importer also recovers shared progress with server/kind ownership,
  blocks duplicate active imports and preserves read-only refresh after installation.
  Forty-two scoped ES/EN checks and frontend compilation pass; no real import or
  native UI is exercised. These source changes are included in local 0.55.0.
- Task cancellation now rejects repeated callbacks before a render, keeps accepted
  requests guarded and releases declined/failed requests for retry. Fifteen scoped
  component checks pass; native cancellation remains outside that evidence.
- Active UPnP mappings now offer the existing copyable address-row pattern, with
  failed-copy selection fallback and address-scoped feedback. Seventeen scoped
  panel checks pass; physical router/clipboard and rendered appearance remain
  outside that evidence. Pending, attention, closing and expired endpoints hide.
- Scheduler creation enforces its per-server capacity inside the insert statement.
  An isolated native SQLite race previously saved 131 tasks from 99; the corrected
  module saves exactly 100 under the same 32 concurrent additions, preserving
  independent-server limits and validation. Full-engine/UI execution is untested
  by this probe; included in local 0.55.0.
- Scheduled-task subscription notices now survive successful mutations and manual
  reads, remaining separate from action errors. Scoped component checks cover
  persistence and owner reset; frontend compilation passes without native UI.
- Pending manual interaction checks remain scoped below and maintainer-owned.
- Packaged in local 0.50.0: the task panel can hide completed results in one
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
- Download tasks reuse separate catalog/transfer/strict-modpack HTTP clients while preserving
  task-owned cancellation and progress. A focused native loopback case verifies
  connection reuse, cancellation isolation and hash/partial-file safety; the
  native test target compiles. No measured speed/RAM or gameplay claim is made.
  A second focused native case covers strict modpack pool reuse, SHA-512, file
  size limits, forbidden redirects and preserved targets; original builder and
  origin-validation rules remain unchanged. Native interaction stays manual.
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
| CORE-03 | REVIEW | Start, console, stop and restart | All four real engines pass three ready/status/command/graceful-stop sessions. Paper startup Stop verified with real Java. Console publication now pauses when the document is hidden and resumes the bounded latest-line buffer; direct App-effect/buffer checks cover ordering, reset, disposal and stale callbacks. Post-0.47 source also skips unchanged follow-scroll animations and counts pending lines without another filtered array; actual effect/count checks and frontend builds pass. Post-0.53 send acknowledgements preserve intervening command edits and history drafts, including bounded-history eviction; scoped ES/EN callbacks and frontend compilation pass. Verify native IPC, tray visibility events and slow saves. |
| CORE-04 | REVIEW | Properties and capacity | Shared integer limits and recoverable file/SQLite saves implemented. The shared property reader/writer now handles colon/whitespace separators, escaped keys and continued logical records while preserving unrelated source bytes; direct native module checks and full Cargo check pass. The existing real Vanilla 1.21.4 case now saves hard difficulty/creative mode through the native settings owner before any world exists, then verifies startup mode and read-only seed/difficulty replies (42/Hard). It retains the three-session backup/restore flow and adds no test case; installer unchanged. Verify native edits and other version-specific gameplay rules. Post-0.51 choices follow the native known-version numeric/named boundary and preserve imported values until edited; scoped ES/EN component checks and frontend compilation pass. |
| CORE-05 | REVIEW | Backup and restore | Tracked real world restore and restored boot pass for all four engines; level.dat byte equality checked. Version 0.47.0 keeps post-restore refresh failures by section, localizing at render time. Actual refresh callback/component checks cover all four reads, language changes, successful retry, selected-server consent and stale revisions; retry never repeats restoration. Read/React boundaries are controlled. After 0.50, retention scans use a sorted walker instead of a whole-tree entry list; a temporary real SQLite/filesystem case preserves totals, selection, protected copies, changed-tree refusal and prune results. Large-library RAM is not benchmarked. Verify native restore/retention and interruption. |
| BACKUP-STOP-01 | REVIEW | Copy after clean stop | Real Paper exit produces one durable copy without replay; startup-stop is excluded. After 0.50, the preference setter returns its committed state, the panel recovers failed reads through manual refresh, and duplicate/disposed writes are guarded. Nine controlled component/reader checks and a temporary native persistence case pass; native UI is not automated. Verify native observer, immediate restart and window exit. |
| CORE-06 | REVIEW | Close with active processes | Maintainer confirms 0.39.0 hide/restore with an active server and stop/exit. Current source returns structured stop/timeout/backup failures per server and renders each separately in ES/EN; exit requires an empty failure list and failures permit retry. Actual shutdown function, serialized fixture, React component and App callback checks cover mixed outcomes, all-success, delayed clean stop, retry, IPC rejection, hide, duplicate invocation and unmount. Process/window/API boundaries are controlled. Cancel, native failure feedback and multiple-server interaction still need direct evidence. |
| CORE-07 | REVIEW | Measured telemetry | CPU, resident RAM, heap, I/O, uptime, storage and startup observations connected. A missing RAM reading now preserves available CPU/disk samples and leaves a gap in the memory line; focused checks cover continuity, gaps, retention and process isolation. Memory report sessions now reset before clock/order or unavailable-RAM checks, and startup clears only that server’s chart history to prevent PID-reuse mixtures; production recording/callback probes and native/frontend builds pass. Real Paper TPS response verified across three sessions. Post-0.52 memory scaling follows visible series, so hiding resident RAM no longer flattens heap inspection; 30 scoped helper/component/callback cases and frontend checks pass. Verify native plots and process-session transitions. |
| CRASH-DIAG-01 | REVIEW | Crash evidence | Bounded durable session reports and real Java failure/relaunch verified. Post-0.53 crash-restart notices preserve subscription failures separately from status reads; controlled ES/EN completion-order, refresh, language and disposal checks plus frontend compilation pass. Verify native recovery actions, event delivery and actual Minecraft crash cases. |
| CORE-08 | REVIEW | Download cancellation and retries | Temporary publication, validation and retries implemented. Post-0.47 source returns response buffers directly instead of copying each chunk to another Vec. Real loopback transfers preserve SHA-1/256/512 verification, no-hash downloads, cancellation, previous targets and partial cleanup; native test compilation passes. No CPU/RAM benchmark or HTTPS/native interaction claim. Verify native cancellation/retry and interrupted installs; HTTP Range resume is not implemented. |
| JVM-01 | REVIEW | JVM settings | Current engine runs three real Vanilla 1.21.4 sessions using existing external Java 21. Checked custom arguments produce an actual GC log, persist through restart and reject an invalid option without replacing the saved profile. Restoring the recommended profile permits another clean session without writing the custom log. Post-0.53 custom option verification uses the server working directory, matching launch; a real Java case covers relative GC output, invalid profile preservation, missing directories, unchanged global CWD and plain version inspection. Native settings interaction and other loaders remain manual validation. |
| JAVA-01 | REVIEW | Custom runtime | Production inspection/candidate validation and storage persist existing external Java 21; Java 17 and a corrupt executable are rejected with the valid path preserved. Three isolated real Vanilla 1.21.4 ready/status/command/clean-stop sessions pass without managed Java download. Temporary world data is removed and existing executable/package hashes are unchanged. Native picker, Tauri command reservation/UI and other engines remain unverified. |
| LOAD-01 | REVIEW | Fabric | Real 1.21.4 / Loader 0.19.5 lifecycle with installed Lithium and restored mod bytes passes. Verify native imports/updates and client mod behavior. |
| LOAD-02 | REVIEW | NeoForge | Real 1.21.4 / 21.4.158 and historical 1.20.1 / 47.1.106 lifecycles pass, including managed Java, three command/stop sessions and restored-world boot. Verify native flows and an actual modded client session. |
| MC-VERSIONS-01 | REVIEW | Version catalog and experimental opt-in | Official catalogs and opt-in filtering implemented. NeoForge catalog and loader selection now share the bounded release parser, including the existing 1.20.1 artifact identity; numeric/prerelease/size checks pass. Historical 1.20.1 installation and lifecycle pass. Verify native selection/scrolling and snapshots. |
| MC-UPDATE-01 | REVIEW | Minecraft upgrades and rollback | Real Vanilla 1.21.4 to 1.21.5 world conversion/rollback passed; all 20 world-file hashes restored before boot. Verify native flow, other engines/content migrations and client gameplay. |
| MOD-01 | REVIEW | Mods and Paper plugins | Verified plans, dependency traversal, hashes, enable/disable, updates, removal and diagnostics connected. Main Fabric JARs now show advisory Minecraft requirement mismatches for configured stable releases; real local JAR audit checks cover OR alternatives and excluded uncertain cases. Lithium/Fabric and Chunky/Paper load through the real install/restore harness. Verify native reviews, broader content and gameplay. Fabric environment strings now normalize case/defaults and reject non-string types; a focused native JAR/provider-alias/client-exclusion check and ES/EN message checks pass. Version 0.47.0 preloads explicit root versions before dependency traversal, preventing order-dependent rejection of valid bulk plans, including a retained beta. A local production-graph probe reproduces the previous conflict and checks order, aliases, exact conflicts, cycles, compatibility/identity failures and bounds with controlled catalog/artifact boundaries. Fabric candidate checks remain conservative, not a global solver. Post-0.52 empty-result broadening preserves query/draft, kind and sort, resets the page and restores heading focus; 48 scoped ES/EN component checks and frontend compilation pass, without Modrinth/native UI traffic. |
| PACK-01 | REVIEW | Modpack import | Reviewed selection and staged publication implemented. Real Fabric 1.21.4/Lithium mrpack install and boot passed, including exact loader, override precedence, excluded client/optional files and protected settings. Completed imports retain acknowledgement if library refresh fails; retry refreshes data only. Complex third-party packs and native/client outcomes remain open. |
| IMPORT-01 | REVIEW | Existing server import | Detection, copy/manage choice and recoverable publication implemented. Acknowledged servers remain visible and continue preparation after a library refresh failure. Isolated native inspection and SQLite import pass eight copy/manage cases for renamed Paper/Fabric, legacy Vanilla and an explicitly confirmed unknown launcher; source/copied bytes and permission inventories remain intact and scripts are never run. These are metadata fixtures, not executable Minecraft launchers. Verify native interaction and real custom-launcher boot. |
| IMPORT-02 | REVIEW | World import during creation | Wizard world copying and validation implemented. Verify native source selection and worlds from different versions. |
| PLAYER-01 | REVIEW | Players and permissions | File inventories, online commands, whitelist import, OP and bans connected. Online rows now use session-observed UUIDs from server logs and show unknown identity explicitly; historical names do not select their skins. Known identities can be added directly to the whitelist by UUID while stopped, with existing-entry status and preserved native acknowledgement. Banned IP rows now support stopped-server removal with a safety copy, preserving live Minecraft commands while running. Post-0.51 shared icon/head caching rejects redirected folders and invalid PNGs and uses temporary checked publication; real fixture junctions reproduce the old read and are rejected afterward, with bounded/concurrent PNG checks passing. A further native cached-profile case removes idle successful/expired-failure registry entries while preserving active locks and five-minute failure backoff; no RAM benchmark or server-heap reduction is claimed. No remote or native UI result is inferred. Earlier imported Minehut identity/skin report is resolved by maintainer confirmation on October 31, 2026; broader client flows remain manual validation. |
| WORLD-01 | REVIEW | World operations and combined export | Post-0.54 source adds accent-insensitive world search and 50-row pages using shared local-list helpers; scoped ES/EN component/callback checks preserve ordering, identities, export/duplicate payloads and guards, with frontend compilation passing. Real Fabric/Lithium ZIP export/import and populated Paper three-dimension folder export/import pass. Paper checks 45 world-file hashes before boot and three saved blocks after boot. Verify native/client round trips and broader content. |
| NET-01 | REVIEW | LAN connection | Local address, port state and loopback Minecraft status implemented. Post-0.47 LAN/NAT-PMP copy feedback is scoped to the actual address and latest request; controlled component checks preserve unrelated errors, disposal, LAN timing and ES/EN labels. Frontend builds pass. Verify native clipboard and connection from a separate LAN client. |
| NET-TUNNEL-01 | DEFERRED | External tunnels | Bore removed at user request; no external tunnel is required. Revisit only if requested. |
| STORAGE-USAGE-01 | REVIEW | Storage readings and warnings | Bounded cached directory measurements and targets implemented. Refresh now reconciles untouched warning controls with stored values, including recovery from an initial read failure, while preserving unsaved edits; direct callback checks pass. After 0.50, process metrics queue directory-size scans independently and reuse timestamped samples; initial absence displays calculation rather than a false read failure. A focused native temporary-files/cache case and eight ES/EN meter renders pass, with no latency/RAM benchmark. Verify native access failures, large worlds and threshold feedback. |
| NET-02 | REVIEW | Internet and router mapping | Maintainer confirms a friend's Internet connection on 0.39.0. UPnP/NAT-PMP discovery, temporary leases, renewal/cleanup and manual fallback are implemented; Post-0.47 source separates polling errors from operation errors for both protocols; successful reads clear only read failures, and unverified status blocks new opening/confirmation while preserving removal. Controlled component callbacks and ES/EN markup checks plus frontend builds pass. Post-0.53 NAT-PMP status reads no longer block enabled action controls or release an active mutation lock; 72 controlled ES/EN component/effect/callback checks cover pending/stale observations, duplicate actions and reconciliation, and frontend compilation passes. Physical-router protocol/cleanup and CGNAT feedback remain unverified. |
| TASK-01 | REVIEW | Local scheduling and autostart | Installed 0.38.0 passed selected-server autostart and scheduled real Vanilla backup/restart, stop and stopped-server backup. Maintainer confirms actual Windows-login startup on 0.39.0. Post-0.51 task date validation rejects normalized/nonexistent local times, mirrors the five-year native bound and focuses associated date/interval errors. A reproduced Madrid DST-gap case, 34 scoped ES/EN callback/markup checks, TypeScript and frontend build pass. Native date-picker and broader preference/task interaction still need direct evidence. |
| UX-01 | DONE | Visual design acceptance | The maintainer explicitly accepts the current application appearance. Earlier isolated library captures cover three desktop sizes. General visual approval no longer requires screenshots. Keyboard and assistive-technology behavior are tracked separately. |
| UX-ACCESS-01 | REVIEW | Keyboard and accessibility | Shared labels, ES/EN control markup, modal focus recovery and reduced-motion component checks passed. Shared action menus now support accent-insensitive prefix navigation and repeated-letter cycling; production keyboard callbacks pass ES/EN, disabled/empty-list, timeout and composition checks. Broader keyboard paths and screen-reader interaction remain unverified; the maintainer's visual approval does not establish them. |
| PLAYER-BANS-01 | REVIEW | Ban metadata | Player and IP records show a visible expiry summary and labeled reason, source, creation date and expiry details. Missing expiry remains unknown; the native forever sentinel is localized. Values/time zones remain verbatim and pardon actions are unchanged. ES/EN component checks, TypeScript and the frontend build pass. |
| RAM-ADVICE-01 | REVIEW | Local memory guidance | Creation, server settings and modpack import show an advisory starting heap based on physical RAM, server type and configured slots. Rules reserve at least 2 GiB or 25% for other work, omit unsupported estimates and never change saved memory. Local rule and ES/EN component checks pass. Real workload needs depend on content; no measured RAM reduction is claimed. |
| UX-TASKS-01 | REVIEW | Background operations | Measured task events and reconciliation connected. Post-0.51 shared Win32 filesystem advice covers full disks, denied access and files in use for general/server/task failures; 66 scoped ES/EN SSR checks preserve technical payloads and unknown-message fallback. TypeScript and frontend build pass; OS failures/native UI are not driven. Verify native popover, navigation, cancellation and partial/error outcomes. |
| UX-MOTION-01 | REVIEW | Motion preferences | Persisted preference and system reduced-motion precedence tested. Shared dialogs now cancel pending exits when an operation becomes busy; an isolated headless component check verifies the race, subsequent close, focus return and reduced-motion close. Post-0.47 source skips unchanged initial numeric/chart frames and reduced-motion list geometry/cloning, clears snapshots, and recovers numeric interpolation from non-finite starts. Controlled actual hook/component checks and frontend builds pass; no measured CPU/RAM gain claimed. Post-0.53 shared motion preserves unchanged indicator state and captures list widths in one traversal; controlled baseline/after operation counts, lifecycle checks and frontend compilation pass without an FPS/CPU/RAM benchmark. Verify native transitions and focus stability. |
| UX-KEY-01 | REVIEW | Commands and search shortcuts | Ctrl+K/Ctrl+N connected; Ctrl+F skips hidden/inert/disabled searches and includes Players. The palette now opens server folders without navigating and restarts running servers through the existing lifecycle. Commands are revalidated against current state before execution. Verify native keyboard paths and modals. |
| APP-SETTINGS-01 | REVIEW | Application preferences | Categories, themes/language, Java, paths, networking, retention and advanced settings connected. The selected ES/EN language now synchronizes Show/Quit tray labels through a validated native command. Application info/location reads have a read-only retry. Native interaction review remains pending; library-location migration is not implemented. |
| I18N-NATIVE-01 | REVIEW | English native messages | Added 843 messages across administration, recovery, networking, content, engines, startup, tasks and diagnostics. Fourteen wrappers mark error-owned arguments while preserving names, paths and external output. Content notes and close failures translate separately. Actual locale/catalog checks, React summaries and frontend builds pass; native contract changes also pass offline checking/packaging. A Rust token reader reviews 4,415 literals beyond the first inventory; exclusions remain documented. Five native picker commands receive the current language; fifteen actual API calls cover ES/EN/ES transitions with IPC substituted. Complete native dialogs, language switching during normal use and copy paths outside the audited cases still need interaction evidence. Packaged in 0.46.0. |
| APP-LOG-01 | REVIEW | Launcher diagnostics | Bounded rotated local JSONL logs implemented. Verify native folder opening and OS-session error reporting. |
| UX-02 | REVIEW | Server image | Launcher and Minecraft server-icon.png use the selected image. Verify native crop/remove and the Minecraft multiplayer list. |
| UX-03 | REVIEW | Automatic sidebar | Hover/focus expansion, contextual server actions and empty-library handling implemented. Verify native resizing, long lists and pointer/keyboard transitions. |
| UX-04 | REVIEW | Creation and settings experience | Deferred wizard, seed, pre-start defaults and searched gameplay-first settings with sticky save controls implemented. Shared capacity sliders now use translated visible labels and announce MB/GB/percent units; ES/EN static React markup checks pass. Verify native flow, validation, focus, screen-reader behavior and narrow layouts. |
| UX-05 | REVIEW | Deletion confirmation | Centered modal, explicit managed/external scope and focus return implemented. Confirmed native deletion now removes the local server/history/tasks immediately and uses the existing read-only recovery banner when refreshing the library fails. Production callback checks pass for deletion failure and successful deletion with either refresh result. Verify from scrolled settings and last-server deletion. |
| REL-01 | REVIEW | Windows distribution | Silent clean install 0.40.0, upgrade to 0.43.0 and uninstall passed with a preserved user-owned fixture, unchanged original executable/database hashes and exact original registry restoration. No application launch or native UI was exercised. Maintainer confirms 0.39.0 login startup, tray/exit and external play. Interactive installer choices and broader usage remain open. Linux is deferred; unsigned Windows builds are previews. |
| REL-02 | REVIEW | Current installer | Local 0.54.0 NSIS x64 from commit 1f4f7a7, unsigned and published on GitHub with one verified EXE asset. Frontend tests, TypeScript, Windows frontend, optimized offline packaging, versions and icons pass. It groups the 0.52–0.54 milestones described in the release notes. Local retention keeps 0.53/0.54; protected cleanup preserves package/executable hashes. Native interaction and gameplay were not repeated for packaging; maintainer-owned validation remains nonblocking. |

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
