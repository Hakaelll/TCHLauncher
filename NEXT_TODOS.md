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

- Latest published preview: [0.44.0](https://github.com/Hakaelll/TCHLauncher/releases/tag/v0.44.0).
- Latest local Windows installer: 0.45.0, unsigned and unpublished.
- This installer includes the
  [changes packaged in 0.45.0](docs/development-notes/README.md#changes-packaged-in-0450).
  Native interaction and affected-host validation remain scoped below.
- The maintainer requests publishing the final version of the current changes
  when ready, without intermediate releases. The usual cadence remains three or
  four version milestones. Attach only the EXE; keep checksums local. Commit and
  push continue independently.
- Packaged changes belong in the [release notes](docs/release-notes/README.md),
  rather than another chronological list here.

## Evidence already available

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

| ID | Status | Area | Remaining acceptance evidence |
| --- | --- | --- | --- |
| CORE-01 | REVIEW | Multiple servers and persistence | Two real Vanilla processes pass independent simultaneous states/commands/stops. Verify native creation/reselection, event presentation and multi-client gameplay. |
| CORE-02 | REVIEW | Managed downloads and Java | Four real isolated engine installs pass. Streaming downloads now compute only the requested digest; focused local transfer checks preserve hash verification and existing files on mismatch. Verify native download/runtime recovery and custom Java selection. |
| CORE-03 | REVIEW | Start, console, stop and restart | All four real engines pass three ready/status/command/graceful-stop sessions. Paper startup Stop verified with real Java. Console publication now pauses when the document is hidden and resumes the bounded latest-line buffer; direct App-effect/buffer checks cover ordering, reset, disposal and stale callbacks. Verify native IPC, tray visibility events and slow saves. |
| CORE-04 | REVIEW | Properties and capacity | Shared integer limits and recoverable file/SQLite saves implemented. The shared property reader/writer now handles colon/whitespace separators, escaped keys and continued logical records while preserving unrelated source bytes; direct native module checks and full Cargo check pass. Verify native edits and actual version-specific gameplay rules. |
| CORE-05 | REVIEW | Backup and restore | Tracked real world restore and restored boot pass for all four engines; level.dat byte equality checked. Verify native restore/retention and interruption. |
| BACKUP-STOP-01 | REVIEW | Copy after clean stop | Real Paper exit produces one durable copy without replay; startup-stop is excluded. Verify native observer, immediate restart and window exit. |
| CORE-06 | REVIEW | Close with active processes | Maintainer confirms 0.39.0 hide/restore with an active server and stop/exit. Cancel, failure feedback and multiple-server close outcomes still need direct interaction evidence. |
| CORE-07 | REVIEW | Measured telemetry | CPU, resident RAM, heap, I/O, uptime, storage and startup observations connected. A missing RAM reading now preserves available CPU/disk samples and leaves a gap in the memory line; focused checks cover continuity, gaps, retention and process isolation. Memory report sessions now reset before clock/order or unavailable-RAM checks, and startup clears only that server’s chart history to prevent PID-reuse mixtures; production recording/callback probes and native/frontend builds pass. Real Paper TPS response verified across three sessions; verify native plots and process-session transitions. |
| CRASH-DIAG-01 | REVIEW | Crash evidence | Bounded durable session reports and real Java failure/relaunch verified. Verify native recovery actions and actual Minecraft crash cases. |
| CORE-08 | REVIEW | Download cancellation and retries | Temporary publication, validation and retries implemented. Verify native cancellation/retry and interrupted installs; HTTP Range resume is not implemented. |
| JVM-01 | REVIEW | JVM settings | Profiles, custom arguments and persistence connected. Verify custom profiles through native restart and runtime selection. |
| JAVA-01 | REVIEW | Custom runtime | Executable inspection and version validation implemented. Verify native picker, corrupt runtime recovery and custom-runtime lifecycle. |
| LOAD-01 | REVIEW | Fabric | Real 1.21.4 / Loader 0.19.5 lifecycle with installed Lithium and restored mod bytes passes. Verify native imports/updates and client mod behavior. |
| LOAD-02 | REVIEW | NeoForge | Real 1.21.4 / 21.4.158 and historical 1.20.1 / 47.1.106 lifecycles pass, including managed Java, three command/stop sessions and restored-world boot. Verify native flows and an actual modded client session. |
| MC-VERSIONS-01 | REVIEW | Version catalog and experimental opt-in | Official catalogs and opt-in filtering implemented. NeoForge catalog and loader selection now share the bounded release parser, including the existing 1.20.1 artifact identity; numeric/prerelease/size checks pass. Historical 1.20.1 installation and lifecycle pass. Verify native selection/scrolling and snapshots. |
| MC-UPDATE-01 | REVIEW | Minecraft upgrades and rollback | Real Vanilla 1.21.4 to 1.21.5 world conversion/rollback passed; all 20 world-file hashes restored before boot. Verify native flow, other engines/content migrations and client gameplay. |
| MOD-01 | IN_PROGRESS | Mods and Paper plugins | Verified plans, dependency traversal, hashes, enable/disable, updates, removal and diagnostics connected. Main Fabric JARs now show advisory Minecraft requirement mismatches for configured stable releases; real local JAR audit checks cover OR alternatives and excluded uncertain cases. Lithium/Fabric and Chunky/Paper load through the real install/restore harness. Verify native reviews, broader content and gameplay. Fabric environment strings now normalize case/defaults and reject non-string types; a focused native JAR/provider-alias/client-exclusion check and ES/EN message checks pass. Fabric candidate checks remain conservative, not a global solver. |
| PACK-01 | REVIEW | Modpack import | Reviewed selection and staged publication implemented. Real Fabric 1.21.4/Lithium mrpack install and boot passed, including exact loader, override precedence, excluded client/optional files and protected settings. Completed imports retain acknowledgement if library refresh fails; retry refreshes data only. Complex third-party packs and native/client outcomes remain open. |
| IMPORT-01 | IN_PROGRESS | Existing server import | Detection, copy/manage choice and recoverable publication implemented. Acknowledged servers remain visible and continue preparation after a library refresh failure. Isolated native inspection and SQLite import pass eight copy/manage cases for renamed Paper/Fabric, legacy Vanilla and an explicitly confirmed unknown launcher; source/copied bytes and permission inventories remain intact and scripts are never run. These are metadata fixtures, not executable Minecraft launchers. Verify native interaction and real custom-launcher boot. |
| IMPORT-02 | REVIEW | World import during creation | Wizard world copying and validation implemented. Verify native source selection and worlds from different versions. |
| PLAYER-01 | IN_PROGRESS | Players and permissions | File inventories, online commands, whitelist import, OP and bans connected. Online rows now use session-observed UUIDs from server logs and show unknown identity explicitly; historical names do not select their skins. Known identities can be added directly to the whitelist by UUID while stopped, with existing-entry status and preserved native acknowledgement. Banned IP rows now support stopped-server removal with a safety copy, preserving live Minecraft commands while running. Verify actual client logins and imported Minehut account identity on the affected host. |
| WORLD-01 | IN_PROGRESS | World operations and combined export | Real Fabric/Lithium ZIP export/import and populated Paper three-dimension folder export/import pass. Paper checks 45 world-file hashes before boot and three saved blocks after boot. Verify native/client round trips and broader content. |
| NET-01 | REVIEW | LAN connection | Local address, port state and loopback Minecraft status implemented. Verify connection from a separate LAN client. |
| NET-TUNNEL-01 | DEFERRED | External tunnels | Bore removed at user request; no external tunnel is required. Revisit only if requested. |
| STORAGE-USAGE-01 | REVIEW | Storage readings and warnings | Bounded cached directory measurements and targets implemented. Refresh now reconciles untouched warning controls with stored values, including recovery from an initial read failure, while preserving unsaved edits; direct callback checks pass. Verify native access failures, large worlds and threshold feedback. |
| NET-02 | REVIEW | Internet and router mapping | Maintainer confirms a friend's Internet connection on 0.39.0. UPnP/NAT-PMP discovery, temporary leases, renewal/cleanup and manual fallback are implemented; physical-router protocol/cleanup and CGNAT feedback remain unverified. |
| TASK-01 | REVIEW | Local scheduling and autostart | Installed 0.38.0 passed selected-server autostart and scheduled real Vanilla backup/restart, stop and stopped-server backup. Maintainer confirms actual Windows-login startup on 0.39.0. Preference/task UI and error recovery still need direct evidence. |
| UX-01 | DONE | Visual design acceptance | The maintainer explicitly accepts the current application appearance. Earlier isolated library captures cover three desktop sizes. General visual approval no longer requires screenshots. Keyboard and assistive-technology behavior are tracked separately. |
| UX-ACCESS-01 | REVIEW | Keyboard and accessibility | Shared labels, ES/EN control markup, modal focus recovery and reduced-motion component checks passed. Shared action menus now support accent-insensitive prefix navigation and repeated-letter cycling; production keyboard callbacks pass ES/EN, disabled/empty-list, timeout and composition checks. Broader keyboard paths and screen-reader interaction remain unverified; the maintainer's visual approval does not establish them. |
| PLAYER-BANS-01 | REVIEW | Ban metadata | Player and IP records show a visible expiry summary and labeled reason, source, creation date and expiry details. Missing expiry remains unknown; the native forever sentinel is localized. Values/time zones remain verbatim and pardon actions are unchanged. ES/EN component checks, TypeScript and the frontend build pass. |
| RAM-ADVICE-01 | REVIEW | Local memory guidance | Creation, server settings and modpack import show an advisory starting heap based on physical RAM, server type and configured slots. Rules reserve at least 2 GiB or 25% for other work, omit unsupported estimates and never change saved memory. Local rule and ES/EN component checks pass. Real workload needs depend on content; no measured RAM reduction is claimed. |
| UX-TASKS-01 | REVIEW | Background operations | Measured task events and reconciliation connected. Verify native popover, navigation, cancellation and partial/error outcomes. |
| UX-MOTION-01 | REVIEW | Motion preferences | Persisted preference and system reduced-motion precedence tested. Shared dialogs now cancel pending exits when an operation becomes busy; an isolated headless component check verifies the race, subsequent close, focus return and reduced-motion close. Verify native transitions and focus stability. |
| UX-KEY-01 | REVIEW | Commands and search shortcuts | Ctrl+K/Ctrl+N connected; Ctrl+F skips hidden/inert/disabled searches and includes Players. The palette now opens server folders without navigating and restarts running servers through the existing lifecycle. Commands are revalidated against current state before execution. Verify native keyboard paths and modals. |
| APP-SETTINGS-01 | REVIEW | Application preferences | Categories, themes/language, Java, paths, networking, retention and advanced settings connected. The selected ES/EN language now synchronizes Show/Quit tray labels through a validated native command. Application info/location reads have a read-only retry. Native interaction review remains pending; library-location migration is not implemented. |
| I18N-NATIVE-01 | IN_PROGRESS | English native messages | Added 330 messages across core server administration, backups/worlds, recovery and UPnP/NAT-PMP diagnostics. Reviewed native copy has its own catalog, including existing Minecraft-query translations; stored Error-prefixed messages use native patterns without general UI label matching. Actual ES/EN, argument/wrapper, opaque-detail and catalog-regrouping checks pass with TypeScript/frontend build. Removed the stale tunnel suggestion without changing router operations. Installation/content and composed errors still need review; the literal inventory is not a complete census. These changes postdate the local 0.45 installer. |
| APP-LOG-01 | REVIEW | Launcher diagnostics | Bounded rotated local JSONL logs implemented. Verify native folder opening and OS-session error reporting. |
| UX-02 | REVIEW | Server image | Launcher and Minecraft server-icon.png use the selected image. Verify native crop/remove and the Minecraft multiplayer list. |
| UX-03 | REVIEW | Automatic sidebar | Hover/focus expansion, contextual server actions and empty-library handling implemented. Verify native resizing, long lists and pointer/keyboard transitions. |
| UX-04 | REVIEW | Creation and settings experience | Deferred wizard, seed, pre-start defaults and searched gameplay-first settings with sticky save controls implemented. Shared capacity sliders now use translated visible labels and announce MB/GB/percent units; ES/EN static React markup checks pass. Verify native flow, validation, focus, screen-reader behavior and narrow layouts. |
| UX-05 | REVIEW | Deletion confirmation | Centered modal, explicit managed/external scope and focus return implemented. Confirmed native deletion now removes the local server/history/tasks immediately and uses the existing read-only recovery banner when refreshing the library fails. Production callback checks pass for deletion failure and successful deletion with either refresh result. Verify from scrolled settings and last-server deletion. |
| REL-01 | REVIEW | Windows distribution | Silent clean install 0.40.0, upgrade to 0.43.0 and uninstall passed with a preserved user-owned fixture, unchanged original executable/database hashes and exact original registry restoration. No application launch or native UI was exercised. Maintainer confirms 0.39.0 login startup, tray/exit and external play. Interactive installer choices and broader usage remain open. Linux is deferred; unsigned Windows builds are previews. |
| REL-02 | REVIEW | Current installer | 0.45.0 local NSIS x64 generated from commit 121c1f3, with matching executable/installer versions, expected embedded icons and local checksum. TypeScript, Windows frontend and optimized offline packaging pass. Feature evidence and limits are in the version/development notes. Dependencies/legal resources are unchanged. This unsigned local build is not a GitHub release; 0.44.0 remains the preview published by this thread. The maintainer owns pending manual interaction and gameplay validation without blocking implementation. |

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
5. Check a separate LAN client and account/skin identity on the affected host.
   A friend's Internet connection passed in the maintainer's 0.39.0 report. Where
   router mapping is supported, verify temporary lease cleanup; otherwise verify
   the manual fallback and accurate unsupported-state feedback.

No gate should be marked complete from a compiled button, an unobserved operation,
a sample value or a broad claim about a different version/scenario.

## Reported issues needing host evidence

- Internet-only connection stalls without observed CPU/RAM spikes: capture host
  server/client timing and router conditions before changing network/JVM values.
- Friend-hosted Fabric 1.21.11 with about 30 mods, Xms 1 GB / Xmx 12 GB, resident RAM
  rising from 3 to 9.2 GB: use the existing memory report and actual host logs.
  Resident RAM growth alone does not establish a leak; no forced GC is used.
- Missing backpack/structure behavior: exact mod names, client/server versions
  and startup log are still unavailable. Inventory presence does not prove loading
  or content generation in already-created chunks.
- Imported Minehut UUID/skin mismatch requires the configuration and authenticated
  connection evidence from the other computer. Distinct UUIDs are not merged by name.

## Deferred scope

- Linux: [LINUX_LATER.md](LINUX_LATER.md), explicitly deferred by the user.
- Fabric client instances and Microsoft sign-in: explicitly withdrawn for now.
- External tunnel providers: Bore removed; Playit rejected by the user.

Historical iteration evidence belongs under [development notes](docs/development-notes/).
Older roadmap wording is recoverable in Git history. Do not treat historical
pending lists or test counts as the current state.
