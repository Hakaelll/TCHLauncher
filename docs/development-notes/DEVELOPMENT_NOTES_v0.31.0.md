# Development notes - 0.31.0

This Windows milestone packages the source follow-ups recorded in
[operations evidence](OPERATIONS_AFTER_0.29.0.md): supervision history, bounded
output, whitelist review and session-bound dispatch.

The new player composition uses existing theme tokens, native counts and
conservative connected identities. Up to four current heads accompany the summary;
row heads remain visible at 40 px. Search, pagination, permission editing and
native player commands are retained. The review's scrolling list is keyboard
focusable. Hover color transitions respect reduced motion. No synthetic statistics,
new polling or external service was added.

Unused .player-avatar and .whitelist-import-preview selectors were removed after
checking their references. The obsolete player-row selection stripe was removed.
Old generated installer cleanup is limited to verified files inside the local
NSIS output directory, after the new installer passes resource checks.

Verification: 251 offline native tests passed with 29 conditional entries ignored;
98 frontend tests and TypeScript passed. A real Java 21 diagnostic test passed
separately. The final production frontend and Windows NSIS build completed
successfully. The installer is 6,693,727 bytes; executable/installer versions,
embedded icons and four license destinations were verified. The checksum is in
the release notes and an adjacent file. After verification, 61 obsolete generated
installer/checksum files were removed, freeing 204,793,436 bytes (195.31 MiB).
Only 0.30.0 and 0.31.0 packages and their checksums remain in the NSIS output.
Native UI and gameplay remain pending; package verification does not prove them.

Suggested commit: `feat: release 0.31 with player polish and Java reliability fixes`

## Source cleanup after packaging

The existing 0.31.0 installer is unchanged by this follow-up. Remove bore.rs and
tunnel.rs plus their test-only module declarations: the retired relay protocol
and provider lifecycle had no production or remaining source consumers. Their
eight obsolete test entries are retired as well; active LAN, UPnP and NAT-PMP
implementations are retained. Historical release evidence is not rewritten.

Four unused production helpers are now cfg(test): untracked backup creation and
restore wrappers, the default storage creation fixture and process_snapshot.
Production uses tracked operations, seed-aware creation and session-aware metric
identity. These helpers remain available to their existing tests.

PlayersView now uses the same deferred-panel boundary as other secondary sections,
including loading, local failure and retry states. TypeScript and production Vite
passed. The generated manifest confirms PlayersView is a dynamic entry outside
the initial static import graph: initial JS 557,292 bytes, player entry 22,255
bytes. These are bundle sizes, not measured launch time or resident memory.

Verification: cargo check --offline --lib passed without the previous dead-code
warnings, and cargo test --offline --lib --no-run compiled the remaining tests.
The previously recorded 251/29 run belongs to the packaged source; this follow-up
did not rerun the entire suite. Native first access/retry/keyboard remain pending.

Suggested commit: `refactor: remove retired tunnel code and defer the players panel`

## Further source refinement (not packaged)

Startup now presents the same observed Java/world/ready phases as a connected
track, with pixel check marks and a fully completed ready state. Download/task
headings have semantic status marks, wrapped actions and quieter inset framing.
All states retain their explicit text. Retry attempts and screen-reader phase
descriptions are localized; reduced motion disables phase color transitions.
Duplicate startup CSS moved out of end.css into operations.css.

Removed ten tracked NeoForge research files, including two historical installer
archives and copied Java source, totalling 11,535,194 bytes (11.00 MiB). A repository
reference search found no consumers. Production installation code, regression
fixtures and historical release documentation are retained. Removing tracked
files reduces the current checkout; existing Git history still contains them.

The process event adapter is separated from the shared process engine to allow
headless verification. A real owned test child exercises the production output
pump, observed readiness, shared stop request and terminal release. Nine focused
process tests passed; three conditional fixture entries were ignored. This is
not a real Minecraft lifecycle or gameplay check.

TypeScript and the production Vite build passed; ten locale/startup frontend tests
passed. The existing main-bundle size warning remains. Native appearance,
keyboard operation and full Minecraft lifecycle remain unverified. These source
changes are not included in the existing 0.31.0 installer.

## Console capture and initial-load efficiency (not packaged)

The output reader records complete capture only after normal EOF without line
truncation. Read errors, unfinished threads and panics cannot be reported as
complete merely because a reader ended. Interrupted reads still retry. Read
errors emit an explicit console error through the production event sink without
changing the process state or claiming that Minecraft crashed. Exit reports
retain incomplete-capture information; the localized crash-panel explanation
now covers read errors as well as open pipes.

Ten process tests passed with three conditional entries ignored. The owned-child
integration checks the live notice, unchanged running state, graceful stop and
persisted incomplete report. Its fixture now creates launcher-logs, as required
by the production writer. Production cargo check, TypeScript and Vite passed;
four locale tests passed.

ServerSettings is a deferred section using the existing loading/error/retry
boundary. Runtime, JVM, loader/Minecraft updates, properties and tasks are outside
the initial import graph. ServerArtwork, Icon and byte formatting are shared in
ServerPresentation; two unused collapse/expand icon paths were removed. The
extracted settings component body was compared with HEAD and is unchanged.

The production manifest confirms 508,159 bytes in the initial static JavaScript
graph, down from approximately 557.80 KB in the preceding build (about 8.9%). The
settings entry is 25,876 bytes plus its deferred shared TasksPanel dependency.
This reduces initial code loading, not total functionality or measured RAM.
The existing >500 KB main-chunk warning remains. Twelve properties/runtime/read/
acknowledgement frontend tests passed. Native first access/retry/keyboard remain
pending. Installer 0.31.0 is unchanged. Real Minecraft lifecycle, RAM and network
performance remain unverified.

Suggested commit: `perf: defer server settings and verify console capture failures`

## Deferred console and production Java launch verification (not packaged)

ConsoleView uses the existing deferred-panel loading/error/retry boundary. Log
collection and the bounded console timeline remain in App and continue while
that module loads. The production manifest confirms the console entry is outside
the initial static import graph: 7,706 deferred bytes, 500,595 initial static JS
bytes (previously 508,159). This is initial bundle loading, not a measured RAM or
startup-time result; the existing >500 KB chunk warning remains.

Production launch now uses the shared acceptance-file predicate rather than its
own unbounded read. The shared predicate also bounds the actual file read to
65,537 bytes, rejecting contents beyond 65,536 even if the file grows after the
metadata check. Missing, false, invalid UTF-8 and oversized files are rejected
before a child or launcher log is created. The exact accepted-size boundary and
explicit consent persistence across servers are verified. Acceptance behavior
for normal files is unchanged.

An isolated integration test launches a real Temurin 21 process through the
production launch, output and monitor paths. Its non-Minecraft JAR contains only
a deliberately unsupported class header, rejected before execution. It verifies
starting/crashed events, complete captured output, persisted version diagnosis,
released ownership and a second independent session. The EULA marker belongs
only to that non-Minecraft fixture; no real game terms, world or server is changed.
Eleven process tests passed / four conditional entries ignored; this conditional
Java integration passed separately, as did the existing consent persistence test.
Eight console frontend tests, TypeScript, Vite and production cargo check passed.
These tests do not prove the full Minecraft ready/command/stop/restart/backup/
restore flow, which remains pending.

The Windows icon verifier no longer hardcodes the obsolete 0.21 installer. Its
default resolves the product name/version from tauri.conf.json; explicit installer
arguments remain supported. The default verified the existing 0.31 executable
and installer icons. Native first console access/retry/keyboard remain pending.
The 0.31 installer has not been rebuilt with these source changes.

Suggested commit: `perf: defer console loading and bound consent file reads`

## Real Paper lifecycle and Windows terminal readiness (not packaged)

A fresh isolated Paper 1.21.4 server exposed a production readiness bug: Windows
JLine prefixes redirected console lines with `> ` and carriage-return prompt
clearing. The actual `Done (...)!` line was present, but the parser did not
recognize its header. Parsing now removes that leading prompt only before a
console header, preserving original log records and message contents. This also
restores severity, startup phases and save-event classification on those lines.
Regression tests preserve the prohibition on chat/plugin readiness spoofing.

An opt-in integration test uses the production install, Java launch, output,
monitor, command, stop, backup and restore paths. It copies an already recorded
explicit EULA acceptance into a temporary database; it never calls acceptance.
The server binds only to loopback on a temporary port. The test cleans its owned
processes and temporary library, without changing the regular application DB.

Verified with downloaded and checked Temurin 21.0.12.1 and Paper 1.21.4 build 232:

- Three independent ready sessions with real Minecraft status responses.
- Three nonce command responses and three graceful zero-code stops with complete
  console capture and persisted exit reports.
- Start-to-ready times: 36,756 / 13,718 / 11,181 ms, including local preflight.
  These fixture measurements are not general performance benchmarks.
- A 256,167,620-byte tracked backup, a safety copy before restore, byte-identical
  restoration of world/level.dat and successful boot of the restored world.
- The integration passed in 129.91 seconds. Thirteen related log/catalog/app-log
  tests and two startup tests passed; two unrelated conditional tests stayed
  ignored. Production cargo check and git diff --check passed.

The first failed attempt was stopped and cleaned before the corrected rerun.
This verifies the backend cycle, not native UI/tray handling, real client
participation, router forwarding or Internet performance. The existing 0.31.0
installer does not include this source fix and was not rebuilt.

Commit: `fix: recognize Paper terminal prompts and verify real server lifecycle`

## Real Fabric mod lifecycle (not packaged)

The existing isolated lifecycle harness now covers both Paper and Fabric without
duplicating installation, command, stop or restore checks. The Fabric entry
installs a compatible Lithium release through the production Modrinth plan,
review and SHA-512-validated content installation paths, then executes it in the
isolated Minecraft instance. It requires the same previously recorded explicit
EULA acceptance; no acceptance command is called.

Verified Minecraft 1.21.4 with downloaded Temurin 21.0.12.1 and Fabric Loader
0.19.5: three independent sessions reached ready and returned real Minecraft
status and nonce command responses; each loader log listed Lithium. All three
stops were graceful, with zero exit codes and complete console capture. A tracked
149,491,545-byte backup and before-restore safety copy were created. world/level.dat
and the installed mod JAR matched the original bytes after restore, and the
restored instance booted and listed Lithium again. Start-to-ready times were
55,506 / 17,989 / 14,173 ms; the test passed in 149.02 seconds. These fixture times
are not a general performance benchmark.

The loopback-only temporary library and copied consent record were removed after
the test. The regular application database was read without modification.
Only test code changed; production behavior and the existing installer remain
unchanged. This verifies installation and loader recognition, not player-side
mod behavior, the friend's missing backpack/structure mods, native interaction,
physical routing or the full Fabric dependency solver.

Commit: `test: verify real Fabric mod startup and backup restoration`

## Fabric optional bundled dependency correction (not packaged)

The static Fabric review previously checked every bundled candidate's missing
requirements, even when no main mod required that candidate. This could create a
false red inventory issue and block activating a valid mod through content
controls. Review now begins with main mods and follows uniquely required bundled
providers, matching recognized version ranges and aliases. Unselected optional
candidates and ambiguous alternatives do not create definite missing-dependency
or selected-incompatibility claims. Required transitive failures remain attributed
to the owning top-level JAR.

Provider identities are indexed once; a work queue visits each required candidate
at most once, including cycles. Partial metadata, read limits and dependency
overrides still suppress negative conclusions. English/Spanish evidence copy
states the revised scope. This does not implement Fabric's global candidate
selection solver; alternative combinations, built-in runtime versions and actual
loading remain the loader's authority.

Eight dependency archive/audit cases passed, including unused optional candidates,
unique transitive requirements and alternative versions filtered by ranges.
Thirteen content storage cases passed, including the production enable/disable
round trip preserving original JAR bytes and creating safety copies; three
external integrations were intentionally ignored. Four locale tests, TypeScript,
production cargo check and diff validation passed. No native visual review,
new gameplay diagnosis or installer generation is claimed.

Commit: `fix: avoid false Fabric dependency blocks for optional bundled mods`

## Backup history refinement and deferred loading (not packaged)

Backup history now comes before automation/retention preferences, with one
primary create action, refresh, date/reason/version search and a reason filter.
Rows emphasize readable dates, reasons and saved engine/version, then align sizes
and restore controls. An authored file mark and labeled latest-date accent follow
the End system. Narrow layouts reflow without hiding metadata or restore. Loading,
last-known read errors, empty history and no filter matches retain distinct states;
filters can be cleared. English/Spanish copy and dated restore accessible names
are provided. Existing restore confirmation, busy/read-error/read-loading/stopped
server guards and mutation acknowledgements remain owned by App/backend.

BackupsView uses the shared deferred loading/error/retry boundary and owns its
list-motion hook, so its list still animates after the module mounts. Policy code
is shared with WorldView and deferred from the initial static import graph.
Removed obsolete backup-table/description styles and the former App list ref.
Final production manifest: 490,605 bytes initial static JavaScript, 9,990 fewer
than the previous measured entry; BackupsView and BackupPolicyPanel are outside
that graph. This is bundle evidence, not a RAM or launch-time measurement.

TypeScript, production Vite, three backup read-state tests, four locale tests and
diff validation passed. Native visual/keyboard/reduced-motion/theme checks remain
manual and are not established by compilation. The existing installer is unchanged.

Commit: `feat: refine backup history and defer backup controls`

## Shared frontend helpers and real NeoForge lifecycle (not packaged)

Consolidated 36 duplicate locale imports and shared equivalent byte formatting
and error conversion across existing panels. Display formats with different
precision remain separate. Frontend source is 1,596 bytes smaller with repository
LF line endings, including the new shared module. Initial static production
JavaScript is 490,372 bytes, 233 fewer than the preceding build. These are code
and bundle measurements, not evidence of reduced runtime RAM or network latency.
TypeScript, all 98 frontend tests and production Vite passed.

The shared opt-in lifecycle harness now includes NeoForge. Using the existing
persisted consent record without accepting it again, production installation
selected NeoForge 21.4.158 for Minecraft 1.21.4 and downloaded/verified Temurin 21.
Three sessions reached real readiness, answered loopback status and unique console
commands, then stopped gracefully with exit code zero and complete output capture.
Startup times were 33,372 / 22,701 / 16,904 ms. The tracked backup was 202,933,466
bytes; restore created a safety backup, preserved level.dat bytes, restored the
configuration and booted the restored instance. The test passed in 181.90 seconds.
The owned temporary library was removed and the consent copy was deleted.

No additional NeoForge mod was installed in this test. Actual client gameplay,
native interface and physical router verification remain pending. Existing
installer and application version are unchanged.

Commit: `refactor: share frontend helpers and verify NeoForge lifecycle`

## Deferred server creation and imports (not packaged)

Extracted CreateView and ImportView from App and loaded them on demand, alongside
PackImport, through the existing per-panel loading/error/retry boundary. Creation,
seed selection, world import, capacity validation, version catalog, image picking,
step motion and native import callbacks retain their existing implementation.
An exact comparison against the preceding committed component bodies confirmed
the extraction did not alter them. Platform display naming now has one shared
owner; dynamic module failure cannot replace the application shell.

The final production manifest establishes that all three workflow modules are
outside the initial static graph. Initial JavaScript is 461,913 bytes, down 28,459
from 490,372 (5.8%). SliderField is shared between deferred workflows. This is a
startup bundle reduction, not deletion of those workflows or a measured RAM,
launch-time or network improvement. No CSS or visual identity change is included.

TypeScript, all 98 frontend regressions, production Vite and diff validation passed.
Native wizard/import interaction and visual inspection remain manual. The app
version and existing installer are unchanged.

Commit: `perf: defer server creation and import workflows`

## Integer capacity validation and real Vanilla lifecycle (not packaged)

Creation and settings now share the same integer capacity validator. Previously,
finite fractional values could leave actions enabled even though native integer
serialization rejects them. The validator preserves existing limits and rejects
fractional, nonfinite, out-of-range and reversed memory values before submission.
Settings omit the creation-only port input. Two boundary/regression tests, all
100 frontend tests, TypeScript and production Vite passed. This consolidates
validation behavior; no measured RAM or connection improvement is claimed.

Added Vanilla to the shared opt-in production lifecycle harness. Minecraft 1.21.4
with downloaded/verified Temurin 21 completed three readiness/status/unique command
and graceful exit sessions, tracked backup/restore and boot of the restored world.
Startup times were 39,886 / 21,078 / 14,686 ms; the backup was 124,188,011 bytes.
Restore created a safety backup, preserved level.dat bytes and restored the
configuration. All exit codes were zero with complete captured output. The test
passed in 128.02 seconds. Its temporary library was confirmed absent and the
copy of the existing consent record was removed; no new consent was accepted.

Vanilla, Paper, Fabric with Lithium and NeoForge now each have real isolated
lifecycle evidence. This does not establish client gameplay, native interface
behavior, physical router operation or the friend's reported mod/network issues.
Existing installer and app version are unchanged.

Commit: `fix: validate integer capacity and verify Vanilla lifecycle`

## Visible search shortcuts and paused discovery (not packaged)

Ctrl+F previously selected the first search input, including the hidden installed
inventory when browsing Modrinth. It now skips disabled fields (including disabled
fieldsets), hidden/inert/aria-hidden ancestry and inputs without rendered rectangles.
Only a valid target consumes the shortcut, focuses and selects its content. Players
now exposes its existing filter as a search input. Existing dialog guards remain.
Two DOM-adapter regressions cover eligible selection and native-find fallback; they
do not establish real Windows focus or layout behavior.

ContentManager now activates discovery only in its visible mode. Search and
compatibility work have separate lifecycles: hiding stops new compatibility checks,
rejects late observations and clears transient searching/opening indicators, while
retaining the result page and completed compatibility evidence. Reopening reuses
the page and checks only unanswered releases; explicit failure retries remain.
Already dispatched native requests may finish; they are not forcibly cancelled.
Existing bounded-worker/cancellation regressions pass. TypeScript, 102 frontend
tests and production Vite pass. Native switching/keyboard/network behavior remains
manual, and no measured latency or RAM improvement is claimed.

NEXT_TODOS.md is now an English current roadmap rather than an appended release
journal. All 39 work IDs remain, current real-engine evidence and source/installer
boundaries are stated, and unresolved host reports/deferred scope are retained.
Duplicate/stale chronology was removed; existing docs and Git retain history.
The roadmap shrank from 72,282 to 10,139 bytes. Installer/version are unchanged.

Commit: `fix: focus visible searches and pause hidden mod browsing`

## Packaged in Windows preview 0.32.0

The source refinements and fixes recorded above are now included in 0.32.0.
251 offline native tests passed / 33 conditional entries ignored; all 102 frontend
tests, TypeScript and Windows production Tauri/NSIS packaging passed. The four
real lifecycle checks were run separately. Binary versions, supplied-logo icons,
legal inventory/resource destinations and the installer checksum are verified.
See [release details](../release-notes/RELEASE_NOTES_v0.32.0.md). The preview was
not installed or published as a GitHub release; native UI/client/router acceptance
remains outstanding. Source-only statements above describe their historical state.

## Source refinements after 0.32.0

Shared locale formatters now serve dates/numbers across twelve panels, removing
465 bytes of frontend production source and making previously OS-localized dates
follow the selected application language. No installer-size or RAM reduction is
claimed. TypeScript, 103 frontend tests and the Windows Vite build pass.

A real Paper 1.21.4 run reproduced missing TPS because JLine repeats cleared
prompts before its command response. Shared prompt normalization now serves both
logs and TPS, preserving raw records and rejecting chat/plugin imitations. The TPS
parser uses a fixed three-value array instead of allocating a temporary vector.
Two TPS and 13 matching log tests pass. The real production lifecycle test passes
three TPS queries (20/20/20), process resets, commands, graceful stops, backup and
restored-world boot in 154.02 seconds. Native UI/client/router checks remain open.
These changes are source-only; the existing 0.32.0 installer is unchanged.

Commit: `refactor: share locale formatting and fix Paper TPS responses`

## Real startup stop and stop-copy evidence

The opt-in Paper harness now shares its managed installation setup. A real Paper
1.21.4 test sends Stop twice while Starting: Java exits gracefully with code zero,
the application states remain Starting/Stopping/Stopped, and no eligible running
exit or stop-copy is invented. A subsequent ready/TPS/command/clean-stop session
feeds its actual exit to the production stop-copy worker. Exactly one ServerStop
snapshot preserves level.dat bytes; persisted recovery and another worker call do
not replay it. The registry ends empty. The integration passed in 106.71 seconds;
both stop-copy filesystem/SQLite regressions also pass. Its temporary library was
removed. This does not exercise the native scheduler observer, IPC or window exit.

ARCHITECTURE.md no longer describes removed Bore/tunnel source and test modules.
No production behavior or installer version changes in this iteration.

Commit: `test: verify Paper startup stop and durable stop backups`

## Server configuration organization and search

The server settings save heading is sticky in the current workspace scroller;
capacity/gameplay controls now precede maintenance. Search filters localized
property groups, labels/descriptions and native keys without changing drafts,
handles accents, reveals matching advanced keys and remains available with the
server running. A server-keyed settings instance clears old search/add drafts
when switching selection. Existing save/load/dirty guards and all runtime, JVM,
update, task, image and delete controls remain. Scoped deferred styles preserve
End tokens and stack the add-property inputs at narrow widths.

Adding a property now refuses duplicate names instead of overwriting an existing
draft, validates the native 80-character ASCII key and 1024-byte/newline value
limits, and displays translated inline recovery. Native save validation remains
authoritative; imported unchanged legacy values are not newly rejected.
TypeScript, all 105 frontend tests and Windows production Vite pass. New tests
cover duplicate preservation, UTF-8 limits and localized search. Native visual,
keyboard and scrolling verification remains open. Installer 0.32.0 is unchanged.

Commit: `feat: make server settings searchable and keep save controls visible`

## Real Paper plugin lifecycle

The opt-in engine harness now supports managed content independently of its
platform. It selects the primary project from the reviewed production plan,
installs through the normal Modrinth/hash/publication path and checks that exact
managed JAR after restoration. Paper 1.21.4 with Chunky passed three loader-reported
plugin enablements, actual TPS/status/console responses and graceful code-zero
stops. A tracked backup and pre-restore safety copy were created; level.dat and
the plugin JAR matched their saved bytes, and the restored world/plugin booted.
The test passed in 119.51 seconds, with startup observations of 37,138 / 14,521 /
11,780 ms. The temporary library was removed. The first lost terminal session has
no retained pass evidence and is not counted; the successful rerun has a durable
local transcript under ignored artifacts/lifecycle/paper-plugin.log.

Project metadata: https://api.modrinth.com/v2/project/chunky . This verifies one
plugin's installation/loading, not every plugin, its in-game features, native IPC,
client gameplay or physical internet access. Installer 0.32.0 remains unchanged.

Commit: `test: verify real Paper plugin loading and backup restoration`
