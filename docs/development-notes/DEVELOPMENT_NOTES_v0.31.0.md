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

## Two real independent server processes

The opt-in Vanilla 1.21.4 integration installs two managed server directories with
separate JAR paths, ports, seeds and player limits, while reusing the verified Java
runtime. Both actual Java PIDs reach Running at once. Duplicate launch is rejected;
SLP reports each server's configured limit. Unique commands appear only in their
own callback logs and persisted launcher logs. After the first clean stop, the
second retains its session, answers a new command and still responds to SLP. Both
finally stop with code zero, complete output reports and no owned process left.
The integration passed in 60.12 seconds; its temporary library was removed and
the durable local transcript is artifacts/lifecycle/multiple-servers.log (ignored).

This proves the production process/console/file path for two Vanilla instances,
not native UI event presentation, player login or multi-server gameplay. No
production behavior or installer changes in this test-only iteration.

Commit: `test: verify independent simultaneous Minecraft servers`

## Windows preview 0.33.0 and incremental cleanup

The accumulated settings, locale, Paper TPS and real-engine evidence above are
packaged in preview 0.33.0. Storage now uses the shared byte formatter with its
existing two-decimal GB precision. Chart limits avoid mapped timestamp arrays
and flattened/filtered memory arrays while preserving unavailable readings,
server/process isolation and the configured-memory fallback. Across these three
frontend files, normalized UTF-8 source decreases by 26 bytes; this small cleanup
is not a runtime memory benchmark. All 105 frontend tests and TypeScript pass.
Native suite: 251 passed, 36 conditional integrations ignored. Package evidence
and remaining native/client/router limits are recorded in the 0.33.0 release notes.

Commit: `refactor: streamline chart calculations and package Windows preview 0.33.0`

## Content inventory work while hidden

InstalledContentView previously shadowed its active prop with a local variable;
icon reads could therefore continue being scheduled while the inventory was
hidden. Its effect now observes visibility, cancels further scheduling on leave,
rejects late replies, resumes uncached items and retains completed icon evidence.
Cached items no longer occupy worker slots. Filtering/sorting is memoized so an
icon reply does not sort the inventory again. Discovery uses one category lookup
and a memoized installed-project set instead of rebuilding category arrays and
scanning all installed files for every result. First-category precedence and
language switching remain intact; search, compatibility and install guards stay.

106 frontend tests, TypeScript and the Windows-target Vite build passed. Worker
coverage includes initially hidden work, bounded dispatch and cached resumption;
this is not native UI acceptance or a measured RAM/connection improvement. Initial
JavaScript remains 460.12 kB; the deferred content chunk is 44.40 kB. The 0.33.0
installer is unchanged: these source-only changes await the next significant
preview. No Minecraft files, dependencies or downloaded content were removed.

Commit: `perf: pause hidden mod icon reads and reuse catalog lookups`

## Real generated-world upgrade and rollback

The new opt-in production-engine integration goes beyond the earlier Java/EULA
smoke check: managed Vanilla 1.21.4 creates a real world and stops gracefully.
The test hashes every file under that world, including generated region files.
Unconfirmed conversion warnings leave the world and backup list untouched.
Production review/update publishes Minecraft 1.21.5 with a BeforeMinecraftUpdate
copy, preserving the unbooted world, Java path, properties and custom JVM settings.
The new Java process reaches actual Running, advertises 1.21.5, answers a unique
command and stops cleanly. Its saved level.dat differs from the original.

Production tracked restore then retains the converted world's hashes in its
BeforeRestore safety copy and restores all 20 original world-file hashes, the
original JAR bytes, properties and JVM configuration. Cross-version runtime reset
is resolved through the normal managed-runtime preparation/verification path.
The restored 1.21.4 process reaches Running, responds to status/command and stops
with code zero and complete output. No publication stage/old directory/journal
remains. Three sessions took 17,889 / 12,886 / 10,142 ms to reach readiness; the
integration passed in 114.95 seconds. Its temporary library and copied consent
record were removed. The durable local transcript is artifacts/lifecycle/world-upgrade.log
(ignored). Existing explicit consent was reused; no new acceptance was recorded.

This proves one Vanilla 1.21.4 to 1.21.5 conversion and whole-world file restoration
before the restored boot. It does not prove native IPC/UI, Minecraft client
behavior, mod/plugin migration, other version families or physical networking.
No production changes or new installer in this test-only iteration.

Commit: `test: verify real world conversion and complete Vanilla rollback`

## Obsolete visual rules removed

The active React surface no longer produces the former sidebar/collapse control,
legacy installed-content rows/status/actions, chart-switch/tooltip, network-guide
and old empty/table controls. A reviewed explicit class list removed their selector
branches, preserving live branches of grouped rules and their declarations. Dynamic
pixel/glyph classes and functional selector groups were retained. Five exact repeat
rules in visual.css were removed only when an identical later rule remained in the
same parent context. The unused chart-tooltip keyframe was also removed.

This removes 8,128 normalized UTF-8 source bytes from four stylesheets and reduces
the Windows initial CSS from 225.09 to 218.04 kB (gzip 36.78 to 35.66 kB). Initial
JavaScript remains 460.12 kB. All 106 frontend tests, TypeScript and the Windows
production Vite build passed. CSS parsing and the reviewed diff provide source
and build evidence; native rendered/keyboard/theme comparison remains pending.
No lower runtime RAM, faster frames or connection improvements are claimed.
These source changes are not yet packaged in the unchanged 0.33.0 installer.
Local before-files and the removed-selector report are under artifacts/lifecycle
(ignored), rather than new permanent tooling/dependencies in the repository.

Commit: `refactor: remove obsolete interface styles and duplicate chart rules`

## Console identities and shared clock formatting

Disk-history occurrences now receive distinct negative client identities; live
arrivals keep their positive identities. Reconciliation retains matched objects
and IDs, including duplicate lines. Console rows use these stable keys and React
memoization with an explicit language prop. Appending at the 1,000-row limit retains
all 999 surviving row identities instead of shifting their index-based keys.
Typing commands or changing unrelated controls no longer reexecutes unchanged row
markup/time formatting. Search, copy/export, selectable text and clear boundaries
retain their behavior. Native timestamps and disk logs are unchanged.

Removed one redundant bounded-array copy per log batch. Shared clock/log-time
formatting replaces repeated date/locale handling across eight components; NAT-PMP
also reuses the existing error formatter. Clock precision and legacy/invalid log
handling are retained; lease/storage/TPS clocks now follow the selected language.
Production source is 186 normalized UTF-8 bytes smaller overall, despite the added
identity/memoization safeguards. No new dependency or permanent audit script.

All 108 frontend tests, TypeScript and the Windows production Vite build passed.
New regression coverage checks duplicate historical identities, retention at the
capacity limit, clear/refresh boundaries, language changes and timestamp precision.
Initial Windows JavaScript is 460.09 kB (previously 460.12); CSS remains 218.04 kB.
This is source/build evidence, not a measured native frame-rate/RAM improvement.
The existing 0.33.0 installer has not been regenerated for these source changes.

Commit: `perf: preserve console rows and share clock formatting`

## Network observation scheduling

LAN, UPnP and NAT-PMP panels now reuse the existing visibility scheduler instead
of independent five-second setInterval loops. Hidden windows pause these UI reads;
foreground return refreshes immediately. Slow reads cannot overlap periodic reads.
The shared scheduler also skips immediate reads when mounted hidden and rejects
already queued hidden callbacks. Metrics keep their configured 15-second hidden
cadence. Existing response revision/disposal guards and manual actions are retained.

Router maintenance remains in the native start_router_maintenance worker, which
runs independently every two seconds: UI polling is not responsible for renewing
or withdrawing leases. No router/firewall/protocol behavior was changed. These are
source and scheduler-test findings, not physical-router or Internet-play evidence.

All 110 frontend tests, TypeScript and Windows production Vite passed. Regression
cases cover hidden initial mounting, queued callbacks, foreground return, slow
reads, disposal and slower background telemetry. Windows initial JavaScript falls
from 460.09 to 460.01 kB; CSS remains 218.04 kB. Native hidden-window behavior, actual
RAM/frame timing and the reported Internet stalls still require host evidence.
The existing 0.33.0 installer has not been regenerated for these source changes.

Commit: `perf: share network polling and pause hidden observations`

## Real Fabric world and content export/import

Added an opt-in integration through the production install/process/content and
tracked world-export/import/activation functions. A real Fabric Minecraft 1.21.4
server with managed Lithium generates and saves a world, answers a nonce command
and stops cleanly. The shared ZIP records the actual engine/version and verifies
all 23 listed file sizes and SHA-256 hashes. Extracted world hashes match all 20
original files and the two files in mods/ match byte-for-byte. The archive has
only its listed world/content/config records plus manifest and instructions.

A separately created Fabric instance installs its own launcher, imports the
extracted world under a different name, selects it with a safety copy, and receives
the exported mods/config as described by the archive instructions. All imported
world hashes match before boot. It then reaches actual Running, reports Lithium,
answers Minecraft status and a nonce command, and stops with code zero and complete
output. No default world is generated in place of the selected world. Mod bytes
remain equal and the original server's world stays unchanged throughout.

The integration passed in 87.85 seconds; startup times were 25,647 and 19,984 ms.
Its isolated library and copied existing consent were verified removed. No new
EULA acceptance was recorded. Local evidence: artifacts/lifecycle/world-content.log
(ignored). The existing streamed file-hash helper is shared with the world-upgrade
test to avoid duplicate verification code. The native regression suite also passes:
251 passed, 38 opt-in tests ignored, zero failures (66.59 seconds).

This verifies one real Fabric/Lithium round trip through native functions, including
the archive's documented manual content-copy step. It does not establish a native
UI round trip, automatic combined-archive import, Minecraft-client gameplay or
populated Paper dimension migration. Those acceptance gates remain open. No
production changes, frontend changes or regenerated installer in this iteration.

Commit: `test: verify real Fabric world and content round trip`

## Development workspace storage cleanup

The user's folder-size report was reproduced with a no-symlink filesystem scan:
70,048,605,663 logical bytes (65.238 GiB), 54,038 files. Compiler output accounted
for 61.320 GiB in target/debug and 3.676 GiB in target/release. No Cargo/rustc or
TCHLauncher process was running. Resolved cache paths were checked beneath the
workspace target directory and redirected roots were refused before deletion.

Removed target/debug and release build/deps/fingerprint/incremental/examples
caches plus the editor build cache. Preserved the release executable, 0.33.0 and
0.32.0 installers, legal resources and checksums. SHA-256 comparisons confirm the
three binaries are unchanged. Source, Git history, dependencies, fixture evidence
and the user's application library were retained.

The same scan after cleanup measured 312,320,916 logical bytes (297.9 MiB), 2,170
files: 64.947 GiB removed, a 99.55% reduction. Logical file sizes are reported;
filesystem allocation and hard links can differ. Ignored evidence is stored in
artifacts/lifecycle/storage-cleanup.json and preserved-builds.json.

Cargo's development profile now sets debug=0 and incremental=false; test inherits
the development profile. Debug assertions, overflow checks, panic handling and the
release profile remain at their existing settings. Cargo metadata accepts the
manifest offline. This limits regenerable development output at the cost of source
symbols and longer rebuilds. README documents cleanup, installer preservation and
explicit symbol-enabled debugging. No native rebuild was run solely to regenerate
the removed caches; the preceding 251-test native suite remains the latest run.

Reference: https://doc.rust-lang.org/cargo/reference/profiles.html
Commit: `build: limit development artifacts and document cache cleanup`

## Shared error formatting and unused artwork

Eleven identical error-message branches across eight modules now use the existing
errorText helper. Error objects retain message text; other rejection values retain
String conversion. Read revisions, mutation acknowledgements, retries and translated
display stay unchanged. This removes 74 normalized source bytes after imports.

Removed three unused full-resolution inventory illustrations (3,679,975 bytes).
Their production PNGs remain byte-identical in the Vite output; source logo/cutout
remain available for icon generation and provenance. The removed originals are
recoverable from Git history; asset documentation now explains the current layout.

All 110 frontend tests, TypeScript and Windows production Vite pass. Initial
JavaScript is 459.94 kB, down from 460.01; deferred Console/ResourceMonitor/Players
chunks also shrink. CSS remains 218.04 kB. Native compiler caches were not rebuilt,
and no new installer or native visual/performance claim accompanies this cleanup.
Commit: `refactor: share error formatting and remove unused artwork`

## Bounded mod evidence and Windows Paper diagnostics

Frontend diagnostics now strip leading Windows Paper terminal prompts only before
bracketed log headers, after ANSI removal. This restores severity recognition for
plugin enablement errors without changing console/disk logs. INFO headers still
reset continuation evidence; player chat and embedded severity text remain excluded.

Per-file evidence retains at most three recent distinct signals during matching,
plus the latest error when it falls outside that window. Deduplication examines the
bounded window instead of an accumulating history; final large-array copies are
removed. A repeated old message receives its latest timestamp. This bounds working
storage without claiming measured application RAM or frame-time improvements.

All 112 frontend tests, TypeScript and Windows production Vite pass. Regression
cases cover prompted/ANSI Paper logs, INFO/chat exclusion and a 1,000-warning burst
with an earlier error and repeated occurrences. Initial JavaScript remains
459.94 kB and CSS 218.04 kB; the deferred content chunk is 44.41 kB. No native cache
rebuild, installer regeneration or native UI/client verification in this iteration.
Commit: `fix: recognize Windows plugin diagnostics and bound mod evidence`

## Installed inventory pages and code review

Installed content displays at most 24 files per page. Complete-inventory filters,
counts, diagnostic associations and file actions remain available. Page changes
focus the actual range above the results; controls appear at both ends. Filter and
sort changes return to page one. Refreshes clamp an out-of-range page. Context
menus retain filename/hash ownership and close when their file leaves the page.
Icon reads now use the displayed page, with the existing four-read concurrency
limit, cache and stale-completion guards. This is a bounded rendering/read change;
no measured native RAM or frame-time improvement is claimed.

Players and installed content share listWindow for page bounds and slicing.
Player search retains its 50-entry pages and separate UUIDs. Page sizes are a
24-or-50 type; select values narrow at the DOM boundary instead of forced casts.
No dependency was added. Existing player styles cover both pagination controls.

The requested Ponytail and Thermo-Nuclear skills guided a small shared calculation
and two independent read-only reviews. Pstack workflow guidance was read; Cursor
model defaults and tools are unavailable here, so reviewers inherit the Codex
model. Both reviews found no concrete pagination blocker. The structural review
identified existing duplicated mutation/review lifetimes in ContentManager. A
future refactor needs separate ownership/concurrency coverage before changing
those paths. This iteration does not add a generic mutation wrapper.

All 114 frontend tests, TypeScript and Windows production Vite pass. New tests
cover 49 distinct files with equal titles, complete search/source filtering,
shrinking/empty inventories and invalid requested page numbers. Windows initial
JavaScript is 460.24 kB, content is 45.44 kB, shared page code is 0.20 kB and CSS
is 218.12 kB. The added navigation increases bundle size slightly; it limits mounted
items and first-page icon reads. Native UI review and a new installer remain pending.
Commit: `perf: paginate installed content and share list bounds`

## Shared content operation ownership

The four content operations now use one synchronous lock/finalization function:
retrying metadata sync, removing files, enabling/disabling files and installing
reviewed content. A local adapter retains the exact owner identity/activity check
and React busy setter. Individual handlers still own their guards, acknowledgement,
cancellation suppression, error destination and subsequent reads. Toggle dismisses
completed progress in its original position. Installation failure still refreshes
the inventory while holding the lock. No mutation is retried by this helper.

Three promise-based tests verify one write under concurrent invocations, retaining
the lock through follow-up reading, error propagation and release, and an obsolete
completion while a replacement owner is busy. Inactive owners cannot start work.
The separate read-only contract and final diff reviews found no scoped blocker.
Ponytail kept the extraction limited to repeated lifecycle code; the Thermo-Nuclear
review drove removal of four independent lock/finalization implementations.

All 117 frontend tests, TypeScript and Windows production Vite pass. Content code
is 45.46 kB versus 45.44 kB before extraction; this is a maintainability change,
not a bundle-size or measured RAM improvement. Initial JS remains 460.24 kB and
CSS 218.12 kB. Native UI validation, Cargo checks and packaging were not rerun for
this frontend-only refactor. Preparation/review lifecycle consolidation remains
separate work and was not included.
Commit: `refactor: share content operation ownership and locking`

## Atomic content review replacement

Project installation reviews and bulk update reviews now share one preparation
lifecycle. A discriminated selection holds the native review token/plan together
with its kind and, for projects, its compatible version catalog. Preparation keeps
new versions local until the plan succeeds. A failed replacement retains the entire
previous selection and reports the error inside its review. Release preference,
explicit version selection, stopped/busy guards, sequence invalidation, close and
installation behavior remain in place.

Previously, preparation changed the kind and version catalog before planning
finished. A successful catalog read followed by a failed plan could leave the old
token with new version options. Cross-kind replacement was also structurally
inconsistent, but a normal native UI path reproducing it was not established;
the modal restricts background interaction. This change fixes the state contract.

Five injected-read tests cover release/explicit/experimental selection, complete
replacement publication, retained review on planning failure, obsolete catalog and
plan completion, empty catalogs and unmodified read errors. The caller retains a
final sequence check at publication: a separate promise-ordering case covers
invalidation between the helper's completion and the caller resuming. They run the shared
preparation function, not Tauri IPC or a mounted React modal. All 122 frontend
tests, TypeScript and Windows production Vite pass. Content JS is 45.75 kB versus
45.46 kB; initial JS remains 460.24 kB and CSS 218.12 kB. This is a consistency and
maintainability fix, not a measured RAM or bundle reduction. Native UI validation
and a new installer remain pending. The preparation consolidation noted above is
now implemented.
Commit: `fix: publish content review plans atomically`

## Minecraft upgrade read recovery

An acknowledged Minecraft upgrade now uses the existing action/read boundary.
The helper returns the native value alongside any subsequent read error, retaining
warnings without turning a successful write into a failed installation. The panel
shows a separate localized refresh error and an Update data button which only
calls the parent read. Catalog preparation, inspection and update controls stay
blocked until that read succeeds, preventing new plans from stale server metadata.
The same recovery is available inside a failed-update dialog or outside a closed
successful review. The existing End error material wraps the controls on narrow
windows; no new theme, animation or polling is added.

Native warnings render independently of startup notices, so a halt, timeout or
start exception cannot hide them. Startup observation still checks the real native
running state. Its final metadata read also reports failure instead of silently
discarding it. A disposed panel no longer updates local UI or initiates startup
after the update/read result returns.

The new helper test retains a native warning across a read rejection and verifies
that recovery reads do not repeat the update. All 123 frontend tests, TypeScript
and Windows production Vite pass. Initial JS is 460.37 kB, settings is 27.65 kB and
CSS remains 218.12 kB. The read-only review found no new helper/caller regression
and identified the previous hidden-warning behavior, now fixed. Tests cover the
action/read contract; native dialog interaction, IPC and installer validation
remain pending. No native rebuild or new installer was produced for this change.
Commit: `fix: expose Minecraft upgrade refresh recovery`

## Content diagnostic localization

Content inventory read failures, Fabric log counts, log dates, error-file counts,
installation file counts and dependency alternatives now translate their entire
application wording. Most labels already existed in the English dictionary but
their callers bypassed translation or inserted Spanish fragments into translated
sentences. Native audit issues in the card summary now use the same translation
path as the detail view. Original console evidence remains verbatim.

A locale test exercises English counts, dates and dependency alternatives, checks
that version ranges and raw log text stay unchanged, and switches back to Spanish.
All 124 frontend tests, TypeScript and Windows production Vite pass. Initial JS
is 460.43 kB, content is 45.80 kB and CSS remains 218.12 kB. This is a scoped copy
correction; native layout review and packaging remain pending.
Commit: `fix: localize content diagnostic fragments`

## Windows preview 0.34.0

The accumulated frontend improvements since 0.33.0 are packaged in a new NSIS
x64 preview. All four version locations agree on 0.34.0 and Cargo updated only
the root package version in its lockfile. The offline license generator produces
the same inventory with no missing local texts. All 124 frontend tests, TypeScript
and Tauri's optimized Windows build pass. Native tests were not rerun for these
frontend changes. Embedded executable/installer icons match the supplied logo;
versions, checksum and all four legal resource destinations are verified.

The installer is 6,719,128 bytes / 6.41 MiB. Its SHA-256 and scoped verification
limits are recorded in the 0.34.0 release notes. It remains unsigned and has not
been installed or published as a GitHub release. Preserved executable, 0.33.0
fallback and 0.34.0 installer hashes remain equal after deleting 2,087,480,229
bytes of rebuildable release caches. No Cargo/rustc/NSIS process was active before
cleanup; every target was resolved inside the workspace and checked for reparse
points. Obsolete 0.32.0 generated installer/checksum files are also removed.

README records the user's preference for detailed English commits: a concise
title followed by changes, rationale, checks and material limitations.
Commit: `release: package Windows preview 0.34.0`

## Scheduled task read recovery

Tasks now use the existing shared server-read owner instead of a separate revision
counter, initial-load flags and read-error handling. A pending or failed inventory
read blocks adding, pausing, enabling and removing tasks. Last-known rows remain
visible with an explicit unconfirmed-list message and a read-only retry. Recovery
is available inside an open creation dialog as well as in the main panel. The
initial unavailable list remains distinct from a successfully read empty list.
Native action acknowledgement and its notice remain separate from read failures;
retry does not repeat the action. The scheduler event subscription and mutation
owner still guard server selection and disposal; no new polling is added.

Task status/result labels, intervals and remove-button accessible names now follow
the selected language. The action selector narrows its DOM value instead of casting
it. A new test checks state/result localization while preserving native result
classification. Existing server-read tests cover retained values, failed/retried
reads, overlapping replies, reselection, acknowledgements and disposal. These
tests do not mount React or exercise native scheduler events.

All 125 frontend tests, TypeScript and Windows production Vite pass. Initial JS
is 460.87 kB, tasks is 23.71 kB and CSS remains 218.12 kB. This is read recovery and
localization work, not a measured memory or bundle-size reduction. Native UI and
live task execution remain pending; this source change is not in the 0.34.0
installer. The existing installer is unchanged.
Commit: `fix: guard scheduled task changes behind confirmed reads`

## Crash report read recovery and English diagnosis copy

Current diagnostics and crash history now use the existing shared server reader
and offer a read-only retry. A report envelope distinguishes the initial unread
state from a successful native null response, so missing history is not presented
as a failed read or an indefinite loading state. History stays absent when no
record exists; expanding a known report does not fetch it a second time. Existing
keyed current/history lifetimes, report timestamps, native exit facts, action
destinations and raw evidence are preserved. No server process operation or
additional polling accompanies retry.

Authored native diagnosis titles, explanations and recommendations now translate
at render time. English copy covers all six diagnostic rules plus unknown and
forced exits, preserving uncertainty and the difference between heap exhaustion
and other allocation failures. Technical evidence and changed-file names remain
verbatim. The existing End read-failure material provides the recovery controls.

New shared-reader coverage distinguishes unread, confirmed absent, failed and
recovered reports. Locale coverage checks crash labels/recommendations, technical
evidence and switching back to Spanish. All 127 frontend tests, TypeScript and
Windows production Vite pass. Initial JS is 464.27 kB versus 460.87 kB, largely
additional English diagnostic copy; CSS remains 218.12 kB. This is recovery and
localization work, not a size or memory improvement. Mounted React, native crash
IPC, visual/keyboard review and packaging remain pending. The 0.34.0 installer
is unchanged.
Commit: `fix: recover crash report reads and translate diagnoses`

## Scope audit and Paper world families

The source audit against the original requirements found implementation gaps
beyond native QA: Paper dimension families, existing-server inventory previews,
stopped-server identity-preserving whitelist imports and per-project bulk update
changelogs. NEXT_TODOS now marks those parent areas IN_PROGRESS and records the
specific missing paths. No full-goal completion is claimed.

Paper worlds now share one family resolver across listing, duplication, folder/ZIP
export, activation and deletion. A parent row includes total size and paired folder
names. Direct dimension activation/deletion is rejected, active families remain
protected and inactive deletion retains its full-server safety copy. Vanilla,
Fabric and NeoForge keep internal dimension directories unchanged. Destination
checks prevent a copied/imported primary from becoming an existing world's
dimension, including exports into an external destination with a matching parent.
Unsafe or invalid siblings produce a named read error instead of a false empty
inventory. Valid 64-character primaries support their longer derived folder names
without relaxing primary input validation.

Copies stage and validate every part before publication. All reserved destination
names are rechecked after staging and publication callbacks, including dimensions
absent from the source. Cancellation/copy failures clean staging; an in-process
later rename failure rolls back the operation's already-published copies and
reports rollback cleanup failures. This is not a crash-recovery journal: process
interruption between family directory renames remains separate work. External
world-folder import still copies only the chosen folder; reviewed family import
also remains pending. ZIP instructions state the actual manual sibling-copy step.

Six new direct native tests cover distinct dimension files, aggregate listing,
renamed duplicates, folder/ZIP export, backup-protected activation/deletion,
collisions, cancellation, invalid stages, later publication failure, aliases,
reserved suffixes, unsafe siblings and long names. The final full native gate
passes 257 tests with 38 conditional integrations ignored in 88.87 seconds.
All 127 frontend tests, TypeScript and Windows production Vite pass. Initial JS
is 464.84 kB, worlds is 12.44 kB and CSS is 218.19 kB. Read-only review found two
edge cases, destination aliases and long derived names; both were fixed and
covered before the final gate. Native UI, real Paper gameplay and interruption
recovery remain unverified. This source change is not packaged in installer 0.34.0.
Commit: `fix: preserve Paper world dimension families across operations`

## World-copy recovery and requested test reduction

Paper family duplicate/folder-export publication now records a versioned plan in
library SQLite before the first visible rename. Fresh ownership markers and
streamed SHA-256 tree fingerprints identify its prepared parts. Recovery checks
the entire family and its reserved dimension names before completing remaining
renames. It confirms publication durably before removing markers; cleanup can be
repeated. Conflicting or modified folders are retained, never recursively removed.
Imported server-local journals do not authorize recovery operations.

Startup attempts each pending record independently. An unavailable external
export stays pending without stopping unrelated servers. Mutating a server with
an internal pending copy is blocked until application restart recovery; background
startup also performs recovery. Listing refuses an unresolved internal plan.
Normal errors before a durable record clean the operation's staging folders;
errors after it retain recoverable files. Abrupt termination during staging or
fingerprinting before that record can still leave hidden .part folders. The
journal covers process interruption, not a proven power-loss durability guarantee.
Hashing adds file reads; no RAM, network or runtime speedup is claimed.

At the maintainer's explicit request, the suite was reduced from 422 cases to 22,
a net reduction of 94.8%, including four new combined world-publication checks.
The retained set has five frontend read/concurrency checks, eight native integrity
and recovery checks, and nine opt-in real Minecraft scenarios. Many removed cases
were meaningful low-level coverage; removing them intentionally reduces regression
protection. Standalone Java predicate/properties probes and their obsolete setup
instructions were retired. This reduces source maintenance, not installed app size.
Historical verification records above remain historical evidence.

Current verification: five frontend checks, TypeScript, Windows production Vite
and eight native checks passed; nine real Minecraft integrations were not rerun.
The world checks cover all durable rename/cleanup interruption points, repeated
recovery, foreign destinations, changed files/markers, a disconnected export,
invalid trusted records and actual tracked duplicate/export task wiring with
valid compressed level.dat fixtures and distinct dimension files. They do not
exercise native UI, actual process killing, client gameplay or physical routers.
The existing 0.34.0 installer is unchanged by these source changes.

## Read-only existing-server inventory

Import selection now returns a flattened launcher inspection plus an inventory
of safe world folders, mod/plugin filenames, allowlisted properties and counts
from whitelist, operator and ban files. Permission identities, IP values and
arbitrary settings such as RCON passwords are excluded. Missing permission files
count as zero; invalid/unreadable files are unavailable and produce warnings.
World validity uses the existing bounded level.dat check. Content discovery lists
.jar and .jar.disabled files; it makes no claim that Minecraft loaded them.

Inventory reads stop at 5,000 directory entries, 32 world candidates, 2 MB per
configuration/permission file, and 32 detailed warnings plus a truncation notice.
Incomplete inventory remains explicit. Validating a world can decompress up to
8 MB, so the aggregate candidate cap limits repeated decompression. Checked
content parents are canonicalized to support Windows folders named Mods/Plugins.
The UI paginates each category at 24 names, preserves the current themes and
provides English/Spanish labels. Picking/submission disables import fields.

Launcher-only inspection remains the internal copy/backup validation path;
collecting the preview does not add inventory scans to those operations. No
new dependencies, network requests or background polling were added.

Verification: nine native checks and five frontend checks passed; TypeScript
and Windows production Vite passed. Nine real Minecraft integrations were not
rerun. The single added combined filesystem check calls the actual preview
boundary, verifies flattened fields and unchanged fixture bytes, and covers
invalid metadata/JSON, private data exclusion, Windows casing and read limits.
Source review found no remaining required corrections; native UI/client checks
remain open. The existing 0.34.0 installer has not been rebuilt for this source.

## Identity-preserving whitelist import

The picker now distinguishes TXT name requests from standard Minecraft JSON
identities. JSON validation retains canonical UUIDs, rejects malformed identities
and conflicting names for a repeated UUID, and keeps different UUIDs with the same
name. The review displays identities and source-profile heads; its prospective
counts never decide which JSON identities reach the native merge.

A stopped-server reservation owns JSON publication. The merge rereads the real
whitelist, preserves existing JSON objects and unknown fields, and adds only
missing UUIDs. It writes a synced adjacent stage plus an exclusive original-byte
safety copy, compares the current original and publishes with one rename.
[Rust's file rename](https://doc.rust-lang.org/std/fs/fn.rename.html) replaces an
existing regular file on Windows, avoiding the two-rename gap of the generic
legacy replacement helper. This is not a proven power-loss durability guarantee.
Invalid existing files remain unchanged; a no-op creates no backup or stage.

TXT imports keep the reserved running-session command path. Every reviewed name
is sent, even when the cached whitelist has that name, because a name alone does
not establish account identity. Exact JSON identities require stopping; editing
the live file would race Minecraft's own whitelist writes. The outcome distinguishes
commands submitted from identities saved. Acknowledgement clears the draft before
refresh; a failed refresh cannot invite an accidental repeat write.

Verification: ten native checks and five frontend checks passed, with nine real
Minecraft integrations not rerun. TypeScript and Windows production Vite passed.
The one added combined filesystem check covers JSON/TXT parsing, UUID/name
conflicts, separate same-name accounts, preserved unknown fields/source bytes,
exact safety copies, corruption refusal, no-op behavior and missing-file creation.
Source review caught and fixed cached-identity filtering in submission; the backend
now receives all reviewed identities. Native IPC, actual client admission and
visual/keyboard behavior remain open. Source remains ahead of installer 0.34.0.

## Windows 0.35.0 package

Version 0.35.0 now packages the source follow-ups since 0.34.0, including task and
crash read recovery, Paper dimension families and publication recovery, existing
server inspection and UUID-preserving whitelist imports. App/Cargo/config versions
match. Ten native tests, five frontend tests, TypeScript and optimized Windows
NSIS compilation passed; nine real-engine scenarios remain opt-in and were not
rerun. Resource checks verify executable/installer version and exact logo pixels,
plus all four legal resources in the NSIS script. The regenerated inventory has
349 entries and 616 notice texts, with none missing and no dependency changes.

The installer is 6,787,166 bytes; its checksum and verification limits are in
[0.35.0 release notes](../release-notes/RELEASE_NOTES_v0.35.0.md). Package 0.34.0
remains as fallback. Compiler-cache cleanup removed 3,962,938,326 bytes and obsolete
0.33.0/debug artifacts another 18,806,610 bytes. Executable and retained installer
hashes match before/after cleanup. Cleanup excluded application data, libraries,
servers, worlds and downloaded runtimes. Native installation, UI and gameplay
verification remain pending; this is an unsigned locally generated preview.

## Release notes for every reviewed project

The content review previously displayed only its first main project's changelog.
It now lists each changed project and required dependency in a bulk review, while
individual reviews retain all planned entries. Each disclosure identifies the
exact reviewed version and explicitly labels unavailable author notes. Note bodies
mount only while open, use plain text and have bounded, focusable scrolling.
Deferred content CSS owns the theme styling; obsolete changelog rules were removed.
No backend, dependency, network request, polling or installation guard changed.

Verification: TypeScript, five retained frontend checks and Windows production
Vite pass. One-off React server-render fixtures cover multiple projects, changed
dependencies, unchanged-entry exclusion, missing notes, English/Spanish labels,
escaped author titles and closed-body omission. A separate source review found
no required corrections. These checks do not prove native disclosure interaction,
keyboard scrolling or visual layout. No additional retained tests were added.
Installer 0.35.0 remains unchanged; this follow-up is source-only.

## Reviewed external Paper world families

The world picker now inspects the selected primary folder plus adjacent Paper
Nether/End folders. Both creation and World Manager show the complete source and
destination names with the total family size. The exact reviewed dimension names
travel with the selection through the typed frontend API. Native import repeats
detection and refuses a changed family, unsafe sources, dimension-only selection,
occupied destinations or split Paper worlds targeting a different engine.

Import now shares the staged copier and trusted SQLite publication recovery used
by duplicate/export. The old separate single-folder copier was removed. World
activation metadata is read before publication so a failed properties read cannot
turn a completed copy into a reported failure. Original files are not modified.
The user must stop the source server before copying; there is no cross-engine
dimension conversion or automatic combined-archive extraction in this change.

The existing combined publication test now also exercises reviewed external
family import, dimension hashes, unchanged sources, stale/missing reviews, wrong
engines, destination conflicts and cleanup after rejected journal persistence.
The retained test count is unchanged. One-off React rendering checks cover three
dimension rows, destination renaming, standalone worlds, escaped names and bilingual
incompatibility feedback. Native picker interaction, keyboard/layout and real
Paper restarts with imported dimensions remain manual acceptance gates. This
follow-up has not been packaged into installer 0.35.0.

Verification: ten native checks and five frontend checks pass, as do TypeScript,
React render fixtures and Windows production Vite. Nine real Minecraft scenarios
were not rerun. Native compilation required the ordinary unsandboxed build path
after Visual Studio's archiver could not overwrite its generated archive in the
restricted environment. A focused source review found no required corrections.

## Real populated Paper dimension round trip

The existing opt-in Paper scenario now additionally runs a world round trip after
its three lifecycle/backup/restore sessions. A fourth source session loads a chunk
in each dimension and places a distinct diamond/gold/emerald block. Tracked folder
export and reviewed import copy the complete family into a fresh Paper instance
under a different world name. All 45 world-file hashes match before target boot.
The target session uses Minecraft's conditional block commands to verify the saved
blocks without placing new ones. Probes require a `[Server]` response with a unique
marker, so command parser errors containing the submitted text cannot pass.

The Minecraft 1.21.4 run passed in 234.10 seconds, including managed Java/Paper
downloads, five successful sessions and graceful stops. The three dimension blocks
survived, the source and export stayed byte-identical and no default replacement
world was generated. TPS responses were observed in all five sessions. The isolated
temporary library was confirmed absent after fixture cleanup. The existing explicit
EULA record was reused without creating a new acceptance. Test count remains 24;
the other eight real-engine scenarios were not rerun. Evidence is recorded in the
ignored `artifacts/lifecycle/paper-world-import.log`.

The generated archive README now describes reviewed family import instead of
obsolete manual sibling-folder copying. This run exercises folder export/import;
it does not claim a new ZIP round trip, native picker/UI or Minecraft client
gameplay verification. The source audit of original sections 32–46 found no further
missing production paths in those sections; their remaining native/host gates stay
open. This follow-up is not included in installer 0.35.0.
The final native compilation and all ten compact native checks also passed.

## Remove the selected whitelist identity

The focused source audit found a remaining mismatch: same-name accounts could be
imported with distinct UUIDs, but both row buttons sent the same name-only removal
command. Rows now submit their displayed UUID through a registered stopped-server
command. A shared bounded writer handles merge and removal, preserving unrelated
JSON objects and unknown fields, syncing an original-byte safety copy and checking
the original again before a single adjacent-file replacement. Missing UUIDs are
no-ops; corrupt or invalid lists are not rewritten. Duplicate records for the
selected UUID are removed together. The legacy name-only removal action is gone.

The UI explains stopping beside the whitelist, disables unconfirmed/UUID-less
rows and retains the acknowledged UUID/safety filename near the list after refresh
failure. A local owner suppresses unmounted feedback and duplicate submissions;
the native reservation owns the mutation through completion. The existing combined
whitelist check now covers same-name separation, duplicate records, preserved
objects, original-byte copies, invalid input, corruption and absent-file no-ops.
The retained test count remains 24. Ten native checks, five frontend checks and
TypeScript pass; focused source review found no required corrections. Native row
interaction and actual permission enforcement remain open. This fix is being
included in the 0.36.0 package before delivery.

## Windows 0.36.0 package

Package 0.36.0 includes per-project release notes, reviewed external Paper
dimension families and the exact-UUID whitelist removal found during the final
source audit. Package/Cargo/config versions match. Ten native checks, five frontend
checks and TypeScript passed before the final optimized NSIS build. The first full
native release build took 6m 47s after cache cleanup; the rebuild including the
whitelist fix took 2m 00s. The initial package was superseded before delivery.

The final installer is 6,812,804 bytes; the executable is 24,442,880 bytes. Both
report 0.36.0 and have verified expected icon pixels. All four staged legal files
match their source hashes and appear in the generated NSIS script. Regenerating
the legal inventory left 349 entries and 616 notice texts, none missing, without
dependency changes. Package SHA-256 and acceptance limits are in the foldered
0.36.0 release notes. The artifact was generated locally, not installed or published.

After artifact verification, cleanup removed 3,980,120,461 bytes of regenerable
compiler/debug files and the obsolete 0.34.0 installer/checksum. Checked literal
paths stayed inside the workspace, contained no redirected directories and were
removed only after builds ended. The current executable and retained 0.35.0/0.36.0
installer hashes were unchanged. Server data, worlds, downloaded runtimes, app data
and dependencies were excluded. Ignored proof files are `artifacts/package-0.36.json`
and `artifacts/cleanup-0.36.json`. Native installation, UI and client/physical-router
acceptance gates remain open; the full original goal is not complete.

## Exact operator and ban removal

The remaining operator and banned-player row actions still sent names, so accounts
sharing a name could not be addressed individually. An isolated official Vanilla
1.21.4 probe first checked direct UUID commands with both profiles in usercache.
`deop UUID`, `pardon UUID` and `whitelist remove UUID` each reported that the player
did not exist; all three JSON lists retained both profiles. The JAR matched the
official manifest hash, Java reused the existing managed runtime, the recorded
EULA was reused and the server bound only to loopback. Java exited cleanly. This
is evidence for the tested version, not an assertion about every Minecraft build.

The shared file writer now lives in `access_lists.rs`. Its closed deserialized
list enum maps whitelist, operators and banned players to fixed standard filenames.
Only whitelist import can merge entries; selected-row removal accepts the list and
UUID. The existing stopped-server reservation, bounded validation, exact safety
copy, changed-original check and single-file publication remain in place. The old
whitelist-only IPC and name-based deop/pardon helper routes were removed after
migrating callers. Other records, including operator levels, bypass flags, ban
metadata and unknown fields, are preserved. Absent UUIDs remain no-ops.

The UI displays operator/ban UUIDs, explains stopping beside the list and includes
the list label in its retained acknowledgement. Adding permissions/bans and raw
console commands remain available while running. Ten native checks, five frontend
checks, TypeScript and Windows production Vite passed; the retained test count is
unchanged. The combined filesystem case now also covers operator/ban name clashes,
field and safety-copy preservation, no-op/corrupt/missing files and invalid list
selectors. Focused source review found no required corrections. Native interaction
and actual player admission/permissions remain open. Installer 0.36.0 is unchanged.

The requirements 47–100 source audit also identified missing actionable background
notices and structured recovery for ordinary action errors. These are recorded as
NOTIFY-01 and ACTION-RECOVERY-01; existing crash reports and task inventories do not
prove those requirements complete.

## Actionable background outcomes

Added session-only global in-app notices for observed crashes and failed tasks.
Successful tasks notify when their server/section is not visible or the window is
hidden/unfocused. Notices open the console or relevant task section and never
retry a mutation. Task navigation is shared with the existing background inventory.

The observer suppresses repeated terminal IDs, initial historical snapshots and
cancelled tasks. Crash observations reset after a different state, allowing a new
crash to notify. Session maps retain at most 128 entries each; the visible queue
retains eight notices without auto-dismiss timers. Task/crash records remain in
their existing managers. The theme-aware square surface has a 140 ms entry with
system/application reduced-motion support, translated controls and a polite live
region. No dependency, native change or retained test case was added.

One-off actual observer checks and React ES/EN rendering passed for duplicate
suppression, foreground silence, background completion, failures, cancellation,
repeat crashes, section routing, escaped names and removed-server actions. Five
retained frontend checks, TypeScript and Windows production Vite passed. Native
window delivery, layout and keyboard interaction remain unverified. No release
was built or published; installer 0.36.0 remains unchanged.

## Startup and runtime recovery actions

Ordinary Start/Restart requests capture the requested server ID/name and operation
before awaiting the existing acknowledged action. A failed request displays its
native reason, review advice, Settings/Console navigation and expandable raw
technical evidence. The same error state still holds ordinary string messages for
other operations. Removed-server navigation is disabled. No IPC shape changed and
no substring classifier infers a root cause. Existing update/crash diagnostics
remain separate. No automatic retry or runtime replacement was added.

Runtime request failures without a failed task use the same contextual surface.
Failed task views now disclose task ID, kind, attempt and original message, with
runtime-specific review advice and their existing explicit retry control. A later
failed refresh preserves an earlier runtime error. Corrected the global notice
entry animation name so it no longer collides with the existing banner keyframe.

One-off actual React checks passed for all three operation titles in ES/EN,
context preservation, escaped names/evidence, closed technical disclosures,
Settings/Console/dismiss actions and unavailable servers. Failed runtime task
rendering also passed in both languages without invoking a native API. Five
retained frontend checks, TypeScript and Windows production Vite passed. No native
code/dependency changed and no retained test case was added. Native failure
delivery, layout and keyboard interaction remain manual gates. No installer or
GitHub release was generated; the current installer remains 0.36.0.

## World publication after real process termination

Extended the existing compact family-publication safety case with eight actual
Windows child-process terminations. Each child runs the production publication
code in the current test executable against a synthetic three-dimension fixture.
An atomically published signal confirms the requested durable boundary before
the parent kills and waits for its owned child. A guard also kills/waits if the
boundary is not reached within 15 seconds or an assertion fails.

The boundaries are prepared journal, each of three dimension renames, committed
journal and each of three ownership-marker cleanup steps. Before recovery the
parent confirms the surviving journal and expected number of visible/staged
folders. Recovery then checks all three dimension payloads, absent stages and
markers, cleared journal, repeat recovery and fixture deletion. Existing unwind,
foreign-data, source-preserving copy/import and publication-failure checks remain.
No retained test case or production behavior was added.

The focused Windows check passed; the final compact native suite passed ten
cases with nine real-Minecraft scenarios ignored. The final suite log records
successful recovery/cleanup at phases 0 through 7. These checks exercise native
publication code after process termination, not an installed application crash,
a Minecraft client, pre-journal termination or power loss. Existing real Paper
round-trip evidence remains separate. No frontend code/dependency changed, so
frontend and Minecraft-engine checks were not repeated for this test-only change.
No installer or GitHub release was generated.

## Finish modpack import after a library read failure

Pack import now records native success before asking the application to refresh
and open the server. A failed refresh retains a completed result with a read-only
Refresh data action; retry does not install the package or create another server.
The result replaces the wizard controls and redundant progress. A synchronous
submission guard blocks duplicate requests. The shared acknowledged action now
passes its result to the refresh callback; existing callbacks ignore that value.
TypeScript and Windows Vite passed. Native interaction/layout remain manual.

The Fabric scenario already in progress also completed its mrpack extension:
Minecraft 1.21.4, Fabric 0.20.1 and Lithium loaded successfully after reviewed pack
installation. The small generated pack verifies exact loader, downloaded JAR
hashes, an omitted optional file, excluded client overrides, server-overrides
precedence, unchanged local properties/EULA, receipt selection and unchanged
source data/archive. Four sessions completed across the existing lifecycle and
pack target. This is one controlled pack, not broad third-party pack coverage.
The retained case count remains unchanged; no other engine scenario was rerun.
Following the user's instruction, further checks are limited to what each
functional change needs. No installer or GitHub release was generated.

## Continue creation after library refresh failures

Creation/import now insert the acknowledged native server into the visible list
and use the shared server navigation. Removed the extra library read from their
native operation, so a failed refresh no longer prevents subsequent world/Java
preparation. Successful post-install refresh clears the stale read warning.
Application locations/About use the shared recoverable reader with retry controls;
folder opening preserves its acknowledgement if the following info read fails.
TypeScript and Windows Vite passed. Native failure delivery remains manual.
No new test cases, dependencies, installer or GitHub release were added.

## Observe connected-player UUIDs

The supplied Paper log includes UUID lines before player joins. The process
collector now consumes these recognized server/authenticator INFO observations,
retains bounded recent pending identities and publishes name/UUID objects for
connected players. Pending observations expire after 60 seconds and are consumed
on join; conflicting same-name observations remain unresolved. Leave removes the
connection and pending evidence. Each Java process owns a fresh collector.

Connected heads/detail/search use that observed UUID. Removed the cache-name
selection helper; unknown sessions retain a neutral head and explicit label.
Known history and permission files are unchanged. The source contract migrated
all online-player consumers together. This does not alter server authentication,
merge UUIDs, move player data or verify Microsoft account ownership.

TypeScript and Windows Vite passed; native compilation checked separately.
No server sessions or broad suites were run, following the user's request for
brief checks. Real-client login and native visual behavior remain unverified.
No test case, dependency, installer or GitHub release was added.

## Authorize known identities without a file

Known-player rows now add the selected UUID to the whitelist through the existing
stopped-server identity writer. They show already-authorized status, require a
confirmed read and share the mutation owner with identity removal. Native
acknowledgement includes the safety-copy name and survives a failed refresh.
The import-file review and row status share UUID-key normalization. No name
resolution, new native command or automatic whitelist activation was added.
TypeScript and Windows Vite passed; native row interaction remains manual.
No new tests, dependencies, installer or GitHub release were generated.

## Unban IPs while stopped

Banned-IP rows now use a stopped-server command that validates IP addresses and
removes the selected address from banned-ips.json through the shared safe writer.
Equivalent IPv6 spellings match; other records and unknown fields survive. A
synced original-byte safety copy precedes publication. Missing IPs write nothing
and invalid inputs/files are preserved. Live servers keep Minecraft's pardon-ip
command. The frontend retains acknowledgement through refresh errors.

The existing access-file case passed with focused IP coverage; no new case was
added. TypeScript and Windows Vite passed. No broad suite or server run was
performed. Native row interaction remains manual; no installer/release was built.


## Shared operator permission persistence

Moved OperatorEdit and its stopped-server mutation into access_lists. The IPC
payload and safety-copy acknowledgement are unchanged. Removed the separate
ops.json read/write implementation; editing now inherits bounded serialization,
Windows reparse rejection and original comparison immediately before publication.
Unknown fields and other same-name UUIDs remain unchanged. The retained access-file
case exercises edits, preserved bytes, stale expected permissions, invalid level,
missing/duplicate identity, malformed JSON and absent file. Focused native check
passed; no broad engine run or native UI claim. Source is newer than installer
0.37.0; no new installer or release was generated for this change.


## Prepare known-player operator grants

Added a stopped-server grant_operator IPC using access_lists and Minecraft's
standard ops.json. It validates name/UUID, rejects nil identities and levels above
4, preserves existing operators and original bytes, and returns added versus
already present with the previous-file name. Same-name identities remain separate.
Known-player rows offer an inline level/bypass review with confirmation. The local
submission owner prevents repeated requests; read failures preserve acknowledgement.
Existing operators show status, and running/stale/loading states block submission.
English and Spanish strings and narrow-window CSS are included.

The existing access-file case passed in 1.18 seconds after native compilation. It
now exercises first grant, same-UUID no-op without overwritten permissions, a
second same-name identity, original-byte copy, invalid inputs and corrupt JSON.
Actual React ES/EN rendering checks cover the displayed UUID, level default, running
state and existing-operator status. TypeScript and Windows Vite passed. No new
retained test case or dependency. Native interaction and actual Minecraft access
remain manual gates; no new installer or release was generated.


## Player action acknowledgements and read guards

Commands and whitelist toggles now retain native acknowledgement if their refresh
callback rejects, so a sent form can clear without repeating its command. The
existing component owner serializes mutations and ignores late completion after
unmount. Inventory-based actions block during pending/failed reads, including OP
settings and whitelist import. The shared read hook still owns read error display;
its current callback normally resolves after publishing that error. Result counts
now translate to English. Five existing frontend cases and TypeScript pass;
the acknowledgement case includes a failed read and a read-only retry with exactly
one mutation. No new case/dependency, native UI check or installer was added.


## Task-specific recovery messages

Consolidated DownloadTask terminal publication into one result match. Installation,
Java, modpack and update failures/cancellations now name their actual operation;
existing world/content/backup results keep their labels. Failure publication keeps
the latest attempt number and original reason. InstallProgress now supplies local,
operation-specific recovery advice for all 17 task kinds, with exhaustive TypeScript
coverage and English/Spanish copy. No error-string heuristic infers a root cause.

A temporary native check passed real DownloadTask publication for attempt-3 failures
and loader cancellation; its source/log remain ignored under artifacts, and its
module was removed from production source. Actual React rendering passed all kinds
in both languages with escaped technical content. TypeScript passed. No retained
case, dependency, broad engine run, native UI claim or new release was added.


## Command palette server folder and restart

Added the original prompt's Open server folder action and a restart action for
running servers. Folder opening uses the existing validated native command and
keeps the current workspace selection. Restart uses the existing contextual
lifecycle/error path. The dispatcher regenerates available commands before acting,
so a stopped/removed server or newly active installation cannot execute a stale
restart. Command types require a server identity for server actions and dispatch
is exhaustive. English/Spanish generator checks cover labels, filtering, distinct
server IDs, active operations and state changes; TypeScript passed. No dependency,
retained case, native Explorer/keyboard check or new installer was added.


## General application error recovery

The former string-only application banner now uses GeneralActionError with state
review advice, raw escaped technical disclosure and Open application logs. Failed
folder opening has separate local feedback, preserving the original failure;
requests are serialized and each new message owns its own component state. Existing
contextual startup errors are unchanged. Removed the unused error-banner selectors
and shared existing notice motion with both error presentations. TypeScript and
actual React ES/EN checks pass for guidance, the logs control, escaped content and
closed disclosure. Native Explorer, keyboard and visual review remain pending;
no retained case, dependency or new installer was added.


## Shared NeoForge catalog interpretation

Removed the duplicate modern version parser from neoforge.rs. Catalog discovery,
stable loader lists and requested loader identity now use neoforge_release, with
bounded entries/version strings and numeric ordering. Release iteration avoids an
extra catalog vector; sorting is limited to the requested Minecraft family.
Existing historical 1.20.1 identity and catalog-selected prereleases remain valid.
A temporary focused native check passed numeric ordering, deduplication, stable
versus explicit prerelease selection, wrong/absent versions, legacy identities,
26.x mapping and catalog bounds. The check was removed after execution; no retained
case or dependency was added. This is parser/selection evidence, not a new real
legacy installer run or native selector review. Source follows installer 0.38.0.


## Compute only the requested download checksum

The transfer initializes one SHA-1/SHA-256/SHA-512 hasher paired with its expected
value, or none when no digest is provided. Unsupported algorithms fail before the
request or partial-file creation. Chunk writes, limits, retry policy, checksum
comparison, synced publication and cleanup remain intact. A temporary native check
passed streamed local HTTP transfers for all three algorithms, uppercase expected
hex, no hash, mismatch preservation/partial cleanup and unsupported-algorithm
rejection. The private transfer fixture uses loopback HTTP; it does not verify
production HTTPS or provide a throughput/CPU benchmark. No retained case or
dependency was added. Source is ahead of installer 0.38.0.


## Real historical NeoForge lifecycle

The existing NeoForge integration passed with TCH_TEST_MINECRAFT=1.20.1 and the
existing explicit consent record. It selected 1.20.1-47.1.106, downloaded/verified
Temurin 17 and official server/libraries, prepared the installer, then completed
three ready/status/command/clean-stop sessions. Backup/restore checked exact
level.dat bytes, restored MOTD and metadata, a before-restore safety copy and a
third boot from the restored world. The isolated temporary library was confirmed
absent after the run. Log: artifacts/lifecycle/neoforge-1.20.1.log. The scenario
passed in 174.89 seconds; this is a functional result, not a performance benchmark.
No case, dependency or production change was added. Other engines, native selector
scrolling, additional mods and actual Minecraft client play were not tested here.

## Installed Windows acceptance, 2026-10-23

The maintainer asked for direct testing, then excluded Computer Use. The checks
below used installed executables, subprocesses, SQLite and saved files. They did
not operate the desktop UI or replace tray/keyboard acceptance.

The original Roaming and Local application folders were preserved by renaming
them. The two installation registry keys were exported before testing. No existing
server process was active. The original Downloads installation was left intact.

- The published 0.37.0 NSIS installer completed a silent clean installation in
  artifacts/native-acceptance/install with /NS to avoid replacing shortcuts.
- The published 0.38.0 installer upgraded that installation. Both executable and
  registered versions were correct. The initialized database hash and marker file
  were unchanged. This upgrade database did not yet contain a test server.
- A temporary fixture registered Vanilla 1.20.4 with an official SHA-1-verified
  server JAR, existing Temurin 17 and previously recorded EULA consent. It bound
  only 127.0.0.1. Startup selection and one-shot tasks were seeded in SQLite.
- The installed 0.38.0 executed the temporary Run command with --autostart. Java
  created the world and reached Done. The scheduled backup_restart saved a copy,
  stopped Java cleanly and requested a new process, which also reached Done.
  Scheduled stop reported Servidor detenido; a stopped-server backup then reported
  Copia guardada. Both backup directories contained world/level.dat and nonzero
  recorded sizes. No Minecraft client joined.
- The initial fixture had incorrectly placed its JAR outside the server folder.
  Backup rejected it and requested the server's restart. The fixture was corrected
  before the successful run. This was a test setup error, not a production fix.
- The 0.38.0 uninstaller exited successfully, removed its test executable and
  uninstall registration, and preserved the database and marker. Application
  processes were terminated only after the scheduler stopped Java in the passing
  run; this does not verify the application's graceful tray-exit handler.
- The temporary Run entry was restored in a finally block. Original application
  folders and registry entries were restored. Original database/executable hashes
  matched, the registered Downloads installation was again 0.38.0, and no test
  TCHLauncher or Java process remained.

Local evidence is under artifacts/native-acceptance: clean-install.json,
upgrade.json, execution.json, minecraft-launcher.log, uninstall.json and
restoration.json. The temporary probe is ignored and adds no retained test,
dependency, production change or release. Large generated fixtures are removed
after evidence capture. Interactive installation, preference/task UI and IPC,
actual Windows logout/login, and close/cancel/background tray remain unverified.

## Dialog exit interrupted by an operation

The shared Modal previously completed an exit using the state from the close
request, even when an operation became busy before the animation finished. An
isolated React page reproduced the failure in Chromium headless: the confirmation
closed while aria-busy was true. Modal now cancels the exit when busy changes and
ignores animation completion callbacks that no longer own the current exit.

The same component check passes after the fix, including Escape while busy,
closing after the operation finishes, opener focus restoration and reduced-motion
closing. It uses no desktop interaction or Computer Use. This does not verify the
native tray/close flow. Local before/after logs and result JSON are under
artifacts/modal-acceptance*. Temporary fixture sources are removed after checking;
no dependency or retained test is added. Source remains ahead of installer 0.38.0.
TypeScript and the Windows-targeted frontend production build also passed.

## Native tray language

The Show/Quit menu previously stayed Spanish after selecting English. Tray setup
now lives in app_menu.rs; the frontend synchronizes the current language at mount
and after preference changes. The native command accepts only es/en, updates the
existing items and retains their close/restore event handling. Update failures use
the shared application error surface, with stale failures ignored after a language
change or unmount. No second preference store or dependency is introduced.

A temporary focused native check passed label mapping and invalid-locale rejection.
The full native test target compiled, including command registration and menu types.
TypeScript, the Windows frontend build and an IPC payload/rejection check passed.
The temporary test was removed;
the existing retained suite is unchanged. The new translated labels have not been
inspected in the actual Windows menu. This source change follows installer 0.39.0.

## Maintainer acceptance for 0.39.0

In response to the four named acceptance scenarios, the maintainer reported
"Lo he probado y funciona todo". Record user-reported passes for tray hide/restore
with an active server, stop/exit, startup after a Windows login and an external
friend's connection. The report supplies no router method/lease evidence, client
identity details or resolution/accessibility review. It also predates packaging
the later tray-language change. Keep those boundaries open without requesting
the same four tests again. No additional execution or release was needed.

## Library layout and English counts

An isolated React/Chromium fixture captured empty and populated libraries at
1280x720, 1366x768 and 1920x1080 using current styles/assets and long sample names.
No horizontal overflow or actions clipped outside the main column were observed.
The captures exposed untranslated plural counts in the heading/footer and Manage
action names. These now use the existing locale function and two English entries.
The check failed before the correction and passed afterwards, including English
singular/plural and Spanish plural summaries. TypeScript passed.

Evidence remains in artifacts/library-review.json, before/after logs and six PNGs.
The fixture used illustrative server records and an empty rail; it does not verify
native IPC, the real sidebar, other screens or desktop interaction. Its sources
and generated Vite cache were removed afterwards. No retained test or dependency
was added. This change follows installer 0.39.0.

## Partial telemetry readings

The history recorder rejected every sample without resident RAM, discarding valid
CPU and disk counters from the same observation. It now accepts timestamped
samples from a live PID independently of RAM availability. The existing plot
segmentation leaves a memory gap and retains the continuous CPU readings.

A focused executable check failed before the change and passed afterwards. It
checked missing RAM, CPU continuity, disk rate, memory gaps, stale observations,
two-minute retention and PID changes. TypeScript passed. The temporary check was
removed after recording its result in artifacts/partial-telemetry-check.log.
No native build, desktop interaction, dependency or retained test was added.
This source correction follows installer 0.39.0 and does not establish the cause
of the previously reported gameplay connection stalls or resident RAM growth.

## Accessible capacity controls

Shared sliders now reference their translated visible label instead of a fixed
Spanish aria-label. Numeric fields expose their unit as a description; range
controls include MB, GB or percent in their accessible value. Unitless player
controls keep the native numeric value. This reaches creation, imports, server
settings and storage warnings without changing limits or save behavior.

An isolated React static-markup check failed before the correction and passed
for ES/EN with MB, GB, percent and unitless inputs afterwards. TypeScript passed.
The result is recorded in artifacts/slider-accessibility-check.log; temporary
check sources were removed. This does not verify assistive-technology interaction
on Windows. No new dependency or retained test was added.

## Storage warning refresh recovery

Refreshing storage measurements now updates an untouched warning control to the
stored percentage, including the first successful retry after an initial read
failure. Unsaved edits remain intact. Previously the saved baseline changed but
the input retained its old value, falsely appearing edited and ready to overwrite
the stored warning. A direct execution of the production refresh callback failed
before the fix and passed afterwards for initial retry, clean/dirty refresh,
read failure and disposal. TypeScript passed. This callback check does not prove
native UI interaction. Temporary check source was removed after recording
artifacts/storage-refresh-check.log. No dependency or retained test was added.

## Deletion acknowledgement and session data

A native deletion could succeed and then be reported as failed when the library
refresh rejected. The server also remained in the local list. Deletion now uses
the existing acknowledged-action contract, removes the confirmed server and its
chart history, startup phase and task record immediately, and reports a later
read failure through the read-only refresh banner. A rejected deletion preserves
all local records. Current metrics/logs are cleared when the deleted server was
selected. The task revision invalidates snapshots begun before this cleanup.

A direct execution of the production deletion callback reproduced the refresh
rejection before the fix and passed afterwards for native failure and successful
deletion with either refresh result, preserving another server's records. No
native deletion or desktop interaction was performed by this check.

Chart samples now copy only their eight scalar fields, avoiding references to
unused heap/TPS objects and error strings from the full metrics response. Focused
checks cover the retained field set, partial readings, disk rate, memory gaps,
process isolation and history bounds. No measured RAM reduction is claimed.
TypeScript and the Windows frontend production build are recorded for this batch.
Temporary checks are removed; results remain under artifacts/server-delete-ack-check.log
and artifacts/compact-telemetry-check.log. No dependency or retained test was added.

## Fabric Minecraft requirement warnings

The installed inventory now compares main Fabric JAR depends.minecraft ranges
with the configured stable numeric release. A recognized mismatch becomes an
advisory issue beside the file, using the existing warning presentation. It does
not block startup, alter JARs or claim that the configured version was observed
inside a running process. Client-only files, experimental IDs, unknown syntax,
and present/unreadable dependency overrides do not produce this inference.
The existing Fabric predicate comparator is reused; no dependency was added.

A focused native check ran the production audit over real local ZIP/JAR metadata.
It failed before the change and passed afterwards for eight cases: mismatch,
matching OR alternative, unknown syntax, client-only content, prerelease ID,
overrides, 26.1 release naming and non-Fabric content. TypeScript and the English
version/range message passed. This proves inventory behavior for controlled JARs,
not native UI, broad modpack compatibility or client gameplay. Temporary test
source was removed and the retained suite is unchanged. Evidence is recorded in
artifacts/minecraft-audit-result.json.

The policy follows Fabric's metadata and dependency-override documentation:
https://docs.fabricmc.net/develop/loader/fabric-mod-json
https://docs.fabricmc.net/players/troubleshooting/dependency-overrides

## Windows 0.40.0 package

Packaged the accumulated tray/localization, telemetry, storage/deletion recovery
and Fabric inventory changes as Windows preview 0.40.0. All four source version
entries agree. TypeScript, the Windows frontend production build and the optimized
native build passed. The native linker emitted its library-creation informational
message. Embedded executable/installer icons match the application emblem and
both product versions are 0.40.0. The NSIS installer is 6,867,997 bytes; its local
hash and package metadata are recorded in artifacts/package-0.40.json.

The release attaches only the installer. Native visual/interaction acceptance
and client/router observations are not inferred from packaging or earlier user
reports. Dependency versions and bundled legal resources are unchanged.

## Configured authentication and observed local UUIDs

The identity panel now shows a review notice when saved files indicate direct
authentication but displayed connected players have local-type UUIDs while the
server is running. It reuses the existing UUID-format heuristic and does not
claim a cause, verify a Microsoft account or merge identities. Historical UUIDs,
unknown UUIDs and stopped snapshots do not trigger the notice; proxy/offline/read
failure guidance retains priority. Both new messages have English translations.

A React static-render check failed before the change and passed afterwards for
ES/EN and the excluded cases. TypeScript passed. This source change follows the
0.40.0 installer and does not prove that the affected host's login/skin problem is
resolved. Temporary checks were removed; the retained suite is unchanged. Results
remain in artifacts/authentication-session-check.log.

## Visual acceptance and local memory guidance

The maintainer explicitly accepted the current appearance: "tu sigue que visualmente
esta bien". General visual approval is now recorded as DONE and no longer depends
on screenshots. Keyboard/accessibility and functional outcomes retain their own
evidence boundaries. The roadmap, README and product context reflect that scope.

Original requirement 11 also requested local RAM recommendations; capacity views
previously exposed physical RAM and oversubscription warnings only. Creation,
server settings and modpack import now share an advisory starting-point rule.
It reserves at least 2 GiB or 25% of physical RAM, uses platform/slot heuristics,
omits unsupported estimates and changes no configuration automatically. The
current validation limits, Java profiles and collection behavior are unchanged.

Focused local-rule checks, actual ES/EN component rendering, TypeScript and the
Windows frontend production build passed. This does not measure workload needs
or establish lower RAM use. Temporary checks and generated frontend files were
removed; no dependency or retained test was added. Source follows installer 0.40.0.
