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
- Paper with managed Chunky also passed three real plugin enablements and
  restored plugin JAR equality. This covers one plugin, not all plugin behavior.
- Source includes deferred server workflows/settings/console/backups, improved
  operation/plot/catalog/history presentation and shared validation. Latest checks:
  106 frontend tests, TypeScript and production Vite passed.
- Ctrl+F selects a rendered eligible search; Players participates. Modrinth pauses
  scheduling when hidden and preserves completed page/compatibility evidence.
  Installed icon reads now pause when hidden and resume uncached files; inventory
  sorting and catalog lookups no longer repeat per icon/result.
- Removed obsolete interface CSS and exact duplicate rules: Windows initial CSS
  is 218.04 kB (35.66 kB gzip), down from 225.09 kB. Native comparison remains open.
  Native keyboard and browsing behavior still require review.
- Current packaged preview is 0.33.0, including the accumulated changes recorded
  after 0.32.0. See [source evidence](docs/development-notes/DEVELOPMENT_NOTES_v0.31.0.md)
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
| MOD-01 | REVIEW | Mods and Paper plugins | Verified plans, dependency traversal, hashes, enable/disable, updates, removal and diagnostics connected. Lithium/Fabric and Chunky/Paper load through the real install/restore harness. Verify native reviews, broader content and gameplay. Fabric candidate checks remain conservative, not a global solver. |
| PACK-01 | REVIEW | Modpack import | Reviewed .mrpack file selection, exact loader and staged publication implemented. Verify complex third-party packs and native/client outcomes. |
| IMPORT-01 | REVIEW | Existing server import | Detection, copy/manage choice and recoverable publication implemented. Verify real renamed/legacy/custom launchers through native import. |
| IMPORT-02 | REVIEW | World import during creation | Wizard world copying and validation implemented. Verify native source selection and worlds from different versions. |
| PLAYER-01 | REVIEW | Players and permissions | File inventories, online commands, UUID/skin evidence, whitelist import, OP and bans connected. Verify live action outcomes and imported Minehut account identity on the actual host. |
| WORLD-01 | REVIEW | World operations and combined export | Import/export, world-plus-content ZIP, duplicate, activate, backup and confirmed deletion connected. Verify native/client round trips and dimensions. |
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
| REL-02 | REVIEW | Current installer | 0.33.0 NSIS x64 generated with verified version/icons/licenses/checksum and 251 native / 105 frontend tests. Native install/upgrade and usage gates remain open. |

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
