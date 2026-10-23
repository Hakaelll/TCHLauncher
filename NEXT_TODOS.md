# TCHLauncher — current roadmap

Windows is the active platform. The full original manager objective remains open.
`REVIEW` means implemented with evidence recorded, but the complete native/user
flow is not yet verified. `TODO` identifies an outstanding gate; `DEFERRED` follows
an explicit user instruction. These labels do not imply feature completion.

## Current evidence

- Vanilla, Paper, Fabric with Lithium and NeoForge each passed a real isolated
  Minecraft 1.21.4 lifecycle: managed Java/install, three ready/status/command/
  graceful-stop sessions, tracked backup/restore and restored boot. Restore checks
  level.dat bytes; this is not a full Minecraft client gameplay test.
- Two real Vanilla instances also passed simultaneous ready states, distinct
  PIDs/ports/directories, isolated commands/logs and independent graceful stops.
- Vanilla 1.21.4 to 1.21.5 also passed a generated-world conversion and rollback:
  all 20 original world-file hashes and JAR bytes restored, converted safety copy
  preserved, managed Java reverified and three real ready/command/clean-stop sessions.
- Fabric/Lithium world-plus-content ZIP also passes a real second-instance import
  and boot: 23 manifest hashes verified, all 20 world files equal before boot and
  the source unchanged. This follows the archive's manual content-copy instructions,
  not an automatic combined-archive importer or native UI/client test.
- Paper with managed Chunky also passed three real plugin enablements and
  restored plugin JAR equality. This covers one plugin, not all plugin behavior.
- Paper 1.21.4 also passes a populated three-dimension folder export/import into
  a second instance: all 45 world-file hashes equal before boot, distinct blocks
  verified through Minecraft commands in Overworld/Nether/End after boot, source
  and exported files unchanged, and no replacement default world generated.
  The existing Paper scenario now checks this without increasing the test count.
- Source includes deferred server workflows/settings/console/backups, improved
  operation/plot/catalog/history presentation and shared validation. Latest checks:
  Five retained frontend tests, TypeScript and Windows production Vite passed.
  Content reviews now publish their plan, kind and version catalog together;
  failed replacement retains the previous complete review and obsolete replies
  cannot replace it. These are frontend contract checks, not native modal QA.
  Minecraft upgrades retain native warnings and expose failed metadata refresh
  separately, with a read-only retry and blocked replanning until recovery.
  Scheduled tasks now use the shared newest-read owner: failed reads retain
  labeled unconfirmed rows and block changes until a successful retry.
  Current and historical crash reports offer read-only recovery; nullable report
  envelopes distinguish unread/failed observation from a confirmed absent report.
  Latest native gate: ten passed, nine real Minecraft integrations ignored.
  The compact suite has 24 cases overall, including import and permission-boundary
  checks; it remains approximately 94.3% smaller than the original 422-case suite.
  Retained cases prioritize recovery, data integrity and real engine scenarios;
  retired low-level checks no longer provide current regression coverage.
  Paper family checks cover distinct copied/exported dimension bytes, source
  equality, interrupted publication, conflicts and ownership changes.
- Ctrl+F selects a rendered eligible search; Players participates. Modrinth pauses
  scheduling when hidden and preserves completed page/compatibility evidence.
  Installed icon reads now pause when hidden and resume uncached files; inventory
  sorting and catalog lookups no longer repeat per icon/result. Installed files
  now render and read icons in pages of 24, with complete inventory search/counts.
- LAN/router UI reads now share visibility scheduling: paused when hidden and
  serialized while visible. Native lease maintenance remains independent; physical
  router and actual Internet-play checks are still open.
- Console rows retain identities at the 1,000-line limit and skip unchanged row
  rendering; clock/log formatting is shared. Source/build checks cover identity
  and locale behavior; native frame-time/RAM measurements remain pending.
- Removed obsolete interface CSS and exact duplicate rules: Windows initial CSS
  is 218.04 kB (35.66 kB gzip), down from 225.09 kB. Native comparison remains open.
  Native keyboard and browsing behavior still require review.
- Development compiler caches were removed: workspace logical size fell from
  65.238 GiB to 297.9 MiB; installers/executable hashes unchanged. Dev/test symbols
  and incremental caches are disabled; release profile remains unchanged.
- Current packaged preview is 0.36.0, including reviewed Paper family import,
  notes for every updated project and exact-UUID whitelist removal. See
  [source evidence](docs/development-notes/DEVELOPMENT_NOTES_v0.31.0.md)
  and [release history](docs/release-notes/README.md).

## Remaining work and acceptance evidence

| ID | Status | Area | Remaining acceptance evidence |
| --- | --- | --- | --- |
| CORE-01 | REVIEW | Multiple servers and persistence | Two real Vanilla processes pass independent simultaneous states/commands/stops. Verify native creation/reselection, event presentation and multi-client gameplay. |
| CORE-02 | REVIEW | Managed downloads and Java | Four real isolated engine installs pass. Verify native download/runtime recovery and custom Java selection. |
| CORE-03 | REVIEW | Start, console, stop and restart | All four real engines pass three ready/status/command/graceful-stop sessions. Paper startup Stop verified with real Java; verify native IPC and slow saves. |
| CORE-04 | REVIEW | Properties and capacity | Shared integer limits and recoverable file/SQLite saves implemented. Verify native edits and actual version-specific gameplay rules. |
| CORE-05 | REVIEW | Backup and restore | Tracked real world restore and restored boot pass for all four engines; level.dat byte equality checked. Verify native restore/retention and interruption. |
| BACKUP-STOP-01 | REVIEW | Copy after clean stop | Real Paper exit produces one durable copy without replay; startup-stop is excluded. Verify native observer, immediate restart and window exit. |
| CORE-06 | REVIEW | Close with active processes | Stop, keep in background and cancel paths implemented. Verify native close/tray lifecycle and process ownership. |
| CORE-07 | REVIEW | Measured telemetry | CPU, resident RAM, heap, I/O, uptime, storage and startup observations connected. Real Paper TPS response verified across three sessions; verify native plots. |
| CRASH-DIAG-01 | REVIEW | Crash evidence | Bounded durable session reports and real Java failure/relaunch verified. Verify native recovery actions and actual Minecraft crash cases. |
| CORE-08 | REVIEW | Download cancellation and retries | Temporary publication, validation and retries implemented. Verify native cancellation/retry and interrupted installs; HTTP Range resume is not implemented. |
| JVM-01 | REVIEW | JVM settings | Profiles, custom arguments and persistence connected. Verify custom profiles through native restart and runtime selection. |
| JAVA-01 | REVIEW | Custom runtime | Executable inspection and version validation implemented. Verify native picker, corrupt runtime recovery and custom-runtime lifecycle. |
| LOAD-01 | REVIEW | Fabric | Real 1.21.4 / Loader 0.19.5 lifecycle with installed Lithium and restored mod bytes passes. Verify native imports/updates and client mod behavior. |
| LOAD-02 | REVIEW | NeoForge | Real 1.21.4 / 21.4.158 lifecycle passes; loader/update/import regressions exist. Verify native flows and an actual modded client session. |
| MC-VERSIONS-01 | REVIEW | Version catalog and experimental opt-in | Official catalogs and opt-in filtering implemented. Verify native selection/scrolling, snapshots and older supported families. |
| MC-UPDATE-01 | REVIEW | Minecraft upgrades and rollback | Real Vanilla 1.21.4 to 1.21.5 world conversion/rollback passed; all 20 world-file hashes restored before boot. Verify native flow, other engines/content migrations and client gameplay. |
| MOD-01 | IN_PROGRESS | Mods and Paper plugins | Verified plans, dependency traversal, hashes, enable/disable, updates, removal and diagnostics connected. Lithium/Fabric and Chunky/Paper load through the real install/restore harness. Verify native reviews, broader content and gameplay. Fabric candidate checks remain conservative, not a global solver. |
| PACK-01 | REVIEW | Modpack import | Reviewed .mrpack file selection, exact loader and staged publication implemented. Verify complex third-party packs and native/client outcomes. |
| IMPORT-01 | IN_PROGRESS | Existing server import | Detection, copy/manage choice and recoverable publication implemented. Verify real renamed/legacy/custom launchers through native import. |
| IMPORT-02 | REVIEW | World import during creation | Wizard world copying and validation implemented. Verify native source selection and worlds from different versions. |
| PLAYER-01 | IN_PROGRESS | Players and permissions | File inventories, online commands, UUID/skin evidence, whitelist import, OP and bans connected. Verify live action outcomes and imported Minehut account identity on the actual host. |
| WORLD-01 | IN_PROGRESS | World operations and combined export | Real Fabric/Lithium ZIP export/import and populated Paper three-dimension folder export/import pass. Paper checks 45 world-file hashes before boot and three saved blocks after boot. Verify native/client round trips and broader content. |
| NET-01 | REVIEW | LAN connection | Local address, port state and loopback Minecraft status implemented. Verify connection from a separate LAN client. |
| NET-TUNNEL-01 | DEFERRED | External tunnels | Bore removed at user request; no external tunnel is required. Revisit only if requested. |
| STORAGE-USAGE-01 | REVIEW | Storage readings and warnings | Bounded cached directory measurements and targets implemented. Verify native access failures, large worlds and threshold feedback. |
| NET-02 | REVIEW | Internet and router mapping | UPnP/NAT-PMP discovery, temporary leases, renewal/cleanup and manual fallback implemented. Verify a physical router, shutdown cleanup, CGNAT feedback and real external clients. |
| TASK-01 | REVIEW | Local scheduling and autostart | Persistent tasks, reservation/recovery and explicit OS/session preferences connected. Verify a real login and scheduled Java backup/restart/stop. |
| UX-01 | TODO | Native desktop QA | Review 1280×720, 1366×768 and 1920×1080; keyboard, screen reader, themes, reading font and reduced motion. No native UI automation is claimed. |
| UX-TASKS-01 | REVIEW | Background operations | Measured task events and reconciliation connected. Verify native popover, navigation, cancellation and partial/error outcomes. |
| UX-MOTION-01 | REVIEW | Motion preferences | Persisted preference and system reduced-motion precedence tested. Verify actual transitions, interruption and focus stability. |
| UX-KEY-01 | REVIEW | Commands and search shortcuts | Ctrl+K/Ctrl+N connected; Ctrl+F now skips hidden/inert/disabled searches and includes Players. Verify native keyboard paths and modals. |
| APP-SETTINGS-01 | REVIEW | Application preferences | Categories, themes/language, Java, paths, networking, retention and advanced settings connected. Native review pending; library-location migration is not implemented. |
| APP-LOG-01 | REVIEW | Launcher diagnostics | Bounded rotated local JSONL logs implemented. Verify native folder opening and OS-session error reporting. |
| UX-02 | REVIEW | Server image | Launcher and Minecraft server-icon.png use the selected image. Verify native crop/remove and the Minecraft multiplayer list. |
| UX-03 | REVIEW | Automatic sidebar | Hover/focus expansion, contextual server actions and empty-library handling implemented. Verify native resizing, long lists and pointer/keyboard transitions. |
| UX-04 | REVIEW | Creation and settings experience | Deferred wizard, seed, pre-start defaults and searched gameplay-first settings with sticky save controls implemented. Verify native flow, validation, focus and narrow layouts. |
| UX-05 | REVIEW | Deletion confirmation | Centered modal, explicit managed/external scope and focus return implemented. Verify from scrolled settings and last-server deletion. |
| REL-01 | TODO | Windows distribution | Verify clean install, upgrade/uninstall, OS login and production usage. Linux is deferred; unsigned Windows builds are previews. |
| REL-02 | REVIEW | Current installer | 0.36.0 NSIS x64 generated with verified version/icons/licenses/checksum, ten native checks and five frontend checks. The populated Paper round trip passed separately; other eight real-engine integrations were not rerun. Native install/upgrade and usage gates remain open. |

## Implementation gaps from the current scope audit

The source audit at `ad4528d` identified missing paths beyond the native QA gates
listed above. These remain required work, rather than implemented features awaiting
only manual verification:

- **WORLD-PAPER-01 — REVIEW:** Paper's overworld and separate Nether/End folders
  share discovery across listing, duplicate, folder/ZIP export, activation and
  deletion. Retained duplicate/export checks compare all dimension bytes and
  unchanged source hashes. A real Paper restart now also verifies three distinct
  saved blocks after family import. Native UI/client gameplay remain unverified;
  older broader unit checks were retired.
- **WORLD-PAPER-RECOVERY-01 — REVIEW:** trusted SQLite plans, ownership markers
  and streamed fingerprints recover interruptions between dimension renames.
  Startup attempts all plans; unavailable external destinations remain pending
  without blocking unrelated servers. Internal pending copies block mutations
  until recovery at application restart; background startup also recovers them.
  Recovery never deletes visible world folders, and preserves modified files or
  conflicting destinations for review. Retained checks exercise every durable
  publication/cleanup boundary and repeat recovery. These simulate interruption
  through unwinding, not OS termination or power loss. A process killed before
  the durable plan exists may leave hidden staging folders; pre-journal ordinary
  failures clean their stages. Native recovery presentation remains unverified.
- **IMPORT-PREVIEW-01 — REVIEW:** existing-server selection now previews valid world
  folders, mod/plugin filenames, safe gameplay/access settings and permission-file
  counts. Incomplete/unreadable inventory is labeled; private permission records
  and arbitrary configuration values are not displayed. Category lists render
  24 entries per page and the preview supports English/Spanish. Inspection limits
  file reads, directory entries, world validations and warning count. Inventory
  runs only for the preview, not internal copy or legacy-backup revalidation.
  One combined filesystem check covers cross-layer output, unchanged source
  bytes, invalid files, private-value exclusion, Windows folder casing and limits.
  Native selection/review/copy/manage presentation remains unverified.
- **WORLD-PAPER-IMPORT-01 — REVIEW:** external world selection detects adjacent
  Paper Nether/End folders, previews each source/destination and reports total size.
  Creation and World Manager submit the reviewed dimensions; native import rereads
  that family and rejects changed dimensions, dimension-only selection, unsafe
  folders, occupied destinations and non-Paper targets for split worlds. Imports
  reuse staged family copying and trusted SQLite publication recovery. Original
  folders remain unchanged; users must stop the external source server first.
  Retained filesystem and React preview checks cover the new boundary; the real
  Paper check also verifies imported dimension hashes and three saved blocks after
  boot. Native picker/keyboard/layout and client gameplay remain unverified. No automatic
  conversion between Paper's split folders and other engines is claimed.
- **PLAYER-OFFLINE-01 — REVIEW:** standard whitelist JSON preserves explicit UUIDs
  and merges against the current file under a stopped-server reservation. Existing
  objects and unknown fields survive; same-name/different-UUID accounts stay
  separate. A synced original-byte safety copy precedes a single adjacent-file
  rename. Corrupt lists are preserved; no-op imports write nothing. TXT files use
  the running session's command path so Minecraft resolves names, with no name-only
  exclusion against cached identities. JSON requires stopping to avoid concurrent
  Minecraft file writes. Review shows UUIDs/heads, prospective counts and exact
  saved-versus-submitted results. One combined filesystem check passes; native
  selection/review and actual Minecraft permission enforcement remain unverified.
- **MOD-CHANGELOG-01 — REVIEW:** update review exposes notes for every changed
  project, including required dependencies, and labels missing author notes.
  Single-project review retains notes for all planned entries. Native disclosures
  mount plain-text notes only while open. React server-render checks cover bulk
  filtering, dependencies, missing notes, escaped titles and English/Spanish labels;
  TypeScript, five frontend checks and Windows production Vite pass. Native toggle,
  keyboard and layout verification remain pending. Included in installer 0.36.0.
- **WHITELIST-REMOVE-01 — REVIEW:** selected whitelist rows now remove the exact
  UUID under a stopped-server reservation, preserving same-name accounts and
  unrelated fields. The shared bounded file writer retains an original-byte copy;
  absent UUIDs do not write files. The obsolete name-only removal command path is
  rejected. Existing whitelist filesystem coverage includes removal without adding
  a test case. Native row interaction and actual client access remain unverified.
- **ACCESS-REMOVE-01 — REVIEW:** operator removal and player unbanning now submit
  the selected UUID through the same stopped-server writer as whitelist removal.
  A closed native list enum selects only whitelist/ops/banned-players JSON files.
  Unrelated records and unknown fields survive, original bytes receive a safety
  copy, and missing identities write nothing. Name-only deop/pardon UI routes are
  retired; raw console commands remain available. Operator/banned rows display UUIDs.
  Ten native checks, five frontend checks, TypeScript and Windows production Vite
  pass with the retained case count unchanged. An isolated Vanilla 1.21.4 probe
  confirmed that the three direct UUID commands did not remove the selected test
  profile, so no live-command UUID shortcut is assumed. Native row interaction and
  actual client permissions remain unverified. Source is ahead of installer 0.36.0.
- **NOTIFY-01 — REVIEW:** in-app notices identify unexpected server crashes and
  failed tasks, plus successful tasks completed outside their visible workflow.
  Open the console or the task's section directly. Terminal task IDs and crash
  transitions suppress duplicates; initial snapshots do not replay past outcomes.
  Keep eight dismissible notices and bounded session observations. Existing task
  inventory and crash state remain available after dismissal. One-off observer
  checks and actual React ES/EN rendering pass; five retained frontend checks,
  TypeScript and Windows Vite pass. Native delivery, keyboard and layout remain
  manual checks. No operating-system notification or new dependency was added.
  Original requirement 76; source is ahead of installer 0.36.0.
- **ACTION-RECOVERY-01 — TODO:** ordinary startup/runtime action errors still use
  the generic dismissible string banner. Add a concrete recovery action and
  expandable technical evidence for actionable failures; retain richer existing
  crash-report diagnostics. Original requirement 49.

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
