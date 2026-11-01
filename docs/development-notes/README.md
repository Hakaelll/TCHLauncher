# Development notes

Engineering records, test evidence and pending native checks.

## Windows 0.58 package

Source `e66a4d1a11842222bcc9064e8be4f9349318a082` produces the unsigned
6,888,131-byte NSIS x64 installer. Five frontend tests, TypeScript, production
frontend, optimized offline packaging, versions and embedded icons pass.
Tauri's Cargo.toml formatting is restored after parsed TOML equality is checked.
Local retention keeps 0.57/0.58 and removes the hash-verified 0.56 setup. Cache
cleanup is deferred by the active-build guard before deletion; retained package
and executable hashes are unchanged. GitHub stays on 0.57. No native dialog,
keyboard or Minecraft client interaction is repeated for packaging.

## Source changes after 0.57.0

### Shared-theme text contrast

On November 1, 2026, a PostCSS read of the actual End/Obsidian/Nether role tokens
checks 168 opaque text/surface pairs. All exceed the normal-text 4.5:1 threshold;
the minimum is 4.640816966481406 for Obsidian muted text `#b5bac7` over control
hover `#444955`. The calculation uses linearized sRGB luminance and unrounded
comparisons, following [WCAG contrast minimum](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html).
This is token evidence only: no rendered cascade, alpha compositing, disabled
controls, non-text borders, keyboard or screen-reader pass is claimed. No palette
change, dependency or retained test is added; the temporary probe is removed.

### NAT-PMP transport evidence

The actual Rust transport passes against an owned loopback UDP responder:
address/open/close packet bytes, granted external port/lifetime, connected-peer
and opcode filtering, mismatched internal-port rejection, and rejection of zero
internal ports or invalid lease durations. The private fixture endpoint bypasses
the production constructor only inside the removed probe; that constructor still
rejects loopback. No physical gateway, discovery, durable manager/UI, renewal or
Internet reachability is exercised. Temporary source/binary/PDB files are removed;
no production behavior, retained test or dependency changes.

### Source improvements

Creation's new-world review now uses the existing localized label. Actual
expression checks cover ES/EN, random/text/numeric seed choices and unchanged
imported world names. TypeScript/frontend checks pass; no native creation or
rendered UI is exercised. The narrow JSX-copy audit is not a full locale audit.

Creation also shares ownership between world/image selection. Step changes and
submission wait for selection completion, with busy controls and an `aria-busy`
form state. Actual picker/navigation callbacks preserve selected data, cancellation,
retry, busy rejection and disposal; TypeScript/frontend checks pass. Step names
also localize the accessible "Step" prefix. Native dialogs, keyboard interaction
and actual server creation are outside this probe; temporary files are removed.

Existing-server import now shares immediate ownership between folder and image
pickers. Folder inspection and catalog/picker results are ignored after leaving
the view. Actual effect/handler checks reproduce overlapping native requests,
then preserve selected path/JAR/image, cancellation, errors, retry and disposal.
TypeScript/frontend checks pass; native dialogs and actual import are not driven.
The temporary probe is removed; no retained test or dependency is added.

The November 1 workspace inventory sums regular-file lengths without following
symlinks or junctions, including Git history, dependencies, artifacts and packages.
The result is about 161 MB; this excludes redirected dependency targets and is not
allocated disk usage or installed application size. A further generated-cache
cleanup is deferred by the active-build guard before deletion. Both retained
0.56/0.57 installers and the executable keep their verified hashes. No Git-history,
dependency or server-data cleanup is performed.

Player search processes only the active group's rows; ban presentation records
are built only for the banned tab, and authorization/operator indexes only for
known players. Actual initializer/helper checks cover all five groups, two pages,
source order and original identities. TypeScript/frontend checks pass. These are
scoped operation checks, not a runtime RAM/FPS benchmark or native interaction.

Storage measurements and warning saves now share immediate operation ownership.
Controlled actual callbacks reproduce overlapping requests and verify exclusion,
draft preservation, acknowledged warning values, error recovery and disposal.
TypeScript/frontend checks pass; native disk measurement, rendered interaction
and runtime memory are not measured. Temporary probes are removed.

Minecraft update blockers now use the existing native-message translator at
render time. The actual list-item callback reproduces the untranslated English
case, then passes ES/EN switching and unchanged unknown-detail checks in static
React rendering. TypeScript and the frontend build pass. No native update or
interactive UI is exercised; the temporary probe is removed.

The remaining stop-backup, startup-result and server-artwork subscriptions now
reuse the shared registration retry helper. Recovery refreshes missed data once;
panel notices clear only when registration succeeds. Six controlled contexts run
the actual effects in ES/EN and check retry, reconciliation, foreign-server
events and disposal. TypeScript and the frontend build pass. Native event delivery
is not exercised, and no retained test or dependency is added.

## Windows 0.57 package

Source `9c7bd682b25768994b46b5ae8cb7d7ce6cdf037b` produces the unsigned
6,895,327-byte NSIS x64 setup. Five retained frontend tests, TypeScript,
production frontend, optimized offline build, versions and embedded icons pass.
The Tauri CLI's Cargo.toml formatting is restored only after parsed TOML equality
is confirmed. Protected cache cleanup removes 2,151,068,244 logical bytes;
executable and 0.56/0.57 installer hashes remain unchanged. The obsolete 0.55 setup
is verified and removed. No native interaction or client gameplay is repeated
for packaging. Publication attaches only the verified EXE, with checksums local.
GitHub preview `v0.57.0`, release ID `430226897`, is published with exactly that
EXE. Its source tag, remote asset size and SHA-256 match the verified package.

## Annual NeoForge releases after 0.56.0

NeoForge discovery now accepts the four-component annual version format and maps
zero/nonzero Minecraft patch numbers correctly. Explicit selections preserve
snapshot, pre-release and release-candidate identities; installation still
requires exact catalog membership. The parser replay recognizes all 110 numeric
annual stable entries in the captured official catalog and preserves legacy
version selection and rejection checks.

Annual deobfuscated Minecraft releases can omit server mappings. Their verified
official installer runs offline with verified staged libraries; older releases
still require mappings and use the existing mapping-cache wrapper. See
[NeoForge versioning](https://docs.neoforged.net/docs/gettingstarted/versioning/)
and the [26.1 migration announcement](https://neoforged.net/news/26.1release/).

The retained isolated engine case passes on Minecraft 26.3 / NeoForge 26.3.0.100
with managed Java 25: three status responses, console commands and graceful
stops, backup/restore with byte-identical world data, and a restored-world boot.
The case now waits separately for status availability after the startup message.
Immediate queries initially closed; a 500 ms retry, alternate protocol number
and alternate packet ordering did not resolve that timing window. The unchanged
production protocol succeeds when status becomes available. Temporary diagnostic
code and downloaded probe artifacts are removed. No mod-loading, Minecraft
client gameplay, native UI or physical-router claim follows from this run.

Loader-update comparisons also accept four numeric components. The actual helper
probe reproduced the omitted annual update, then passed annual/legacy/Fabric,
same-build prerelease promotion, downgrade rejection, invalid numbers, sorting
and deduplication checks. The optimized native test target compiles offline.
Exact catalog membership, installation fingerprints and update publication are
unchanged. This check does not execute a loader update; the temporary probe is
removed without adding a retained test or dependency.

## Inventory name-order locale after 0.56.0

Name ordering follows the selected language instead of always using Spanish.
The library memo includes language so existing rows reorder when it changes.
Actual helper checks reproduce the English ñ-order mismatch and preserve Spanish,
numeric names, search, other ordering modes and source identity. TypeScript and
the frontend build pass. No native/rendered UI is checked; temporary probe removed.
The local 0.56 installer is unchanged.

Installed-content names and diagnostic tie-breaks also follow the selected
language. Actual content/search helper checks preserve attention priority,
managed/local filters, aliases, numeric order and original file identities.
TypeScript and the frontend build pass; no native file change or rendered UI is
exercised. Temporary probe removed; no retained test or dependency added.

Console search uses locale-independent lowercasing so host-language rules do not
hide `INFO` matches. Actual search/timeline checks reproduce Turkish-default
failure and preserve clear boundaries, server scope, spaces and original rows.
TypeScript/frontend checks pass; native keyboard/rendering remain untested.
Temporary probe removed; local 0.56 packaging is unchanged.

Console command recall now survives navigation for the current application
session, with 50 acknowledged sends per server and no disk storage. The App keeps
late acknowledgements; new console mounts receive that server's history, and
deletion releases it. Actual send/initialization/scope callbacks and helpers check
remounts, isolation, rejected sends, failed reads, late replies and bounds.
TypeScript/frontend checks pass; native commands, deletion and keyboard behavior
remain untested. Temporary probe removed; no retained test or dependency added.

## Local Windows 0.56.0 package

Unsigned local x64 installer from `44aeb3d4ce3b4e3b3a911a0da6d63c525f931856`, 6,882,989 bytes.
Five retained frontend tests, TypeScript, production frontend, optimized offline
NSIS packaging, versions and embedded icons pass. Source changes recorded below
after 0.55 are included. Generated-cache cleanup is deferred while
other builds are active. Retained 0.55/0.56 installers and the executable preserve
their verified hashes. The obsolete local 0.54 installer is verified and removed.
GitHub stays on 0.54; checksums stay local.
Native UI/gameplay is not repeated, and the full manager objective remains open.

## Paged server library after 0.55.0

Library cards now use the shared 50-item window. Search/filter/order changes reset
the page; layout changes preserve it. Pages clamp when inventory shrinks, preserve
server identities and request result-count focus/scroll. Existing creation/import
and empty states remain available. Shared pagination styling and labels are reused.
Actual component/helper checks compare 253 old live cards with 50 per page and
cover boundaries, select/menu payloads, layout, accent search, filters, ordering,
empty recovery and shrinkage. TypeScript and the frontend build pass. Hooks,
motion and assets are controlled; no rendered/native UI or RAM/FPS benchmark is
claimed, and motion ghosts may overlap live cards. Temporary probe removed;
one English label added. Local 0.55 packaging is unchanged.

## Visible memory readings after 0.55.0

With resident RAM hidden, the memory chart headline, peak and description now
follow heap usage. Showing resident RAM restores its readings; CPU is unchanged.
Actual chart/helper and legend-callback checks reproduce the resident/heap mismatch
and verify switching, peaks and missing heap. TypeScript and the frontend build
pass. No native telemetry or rendered pixels are checked. Temporary probe removed;
local 0.55 packaging is unchanged.

## Superseded sizing declarations after 0.55.0

Removed 36 earlier sizing/spacing declarations, saving 707 stylesheet-source
bytes. Each has a later unconditional declaration with the same selector,
property and importance and a supported fixed value. PostCSS checks preserve
final declarations and conditional rules; the frontend build passes. No rendered
or native visual check is claimed. Probe removed; local 0.55 remains unchanged.

## Deferred cache cleanup completed after 0.55.0

The existing cleanup guard now permits removal after active builds finish.
It removes 1,916,190,619 logical bytes from generated release dependencies,
build metadata, symbols and frontend output. The 0.54/0.55 installers and current
executable retain their verified hashes before/after cleanup. Server libraries
and source are untouched. Counts describe file lengths, not allocated disk space
or runtime memory. Compact receipts remain local; no new package or release.

## Immediate ownership of application actions after 0.55.0

The existing global busy state now has a synchronous owner. General actions,
settings saves, manual backups, deletion and restore claim it before sending
native work and release it in `finally`. Repeated callbacks cannot send another
operation before React disables controls. Independent installation jobs retain
their existing task ownership. Deletion contention returns existing wait guidance.
Actual action/backup callback checks reproduce duplicate sends and cover shared
contention, success/failure release and retry. TypeScript and the frontend build
pass. Native operations, deletion/restore and Windows UI are not exercised by the
probe. Temporary source removed; no retained test added and local 0.55 unchanged.

## Backup inventory derivation after 0.55.0

The backup view memoizes its latest-copy selection and available reason values
by inventory reference. A controlled actual-component rerender with 253 copies
previously repeats two full derived-data scans; afterward it repeats neither.
Changed inventories recompute both, preserving latest markers, reason ordering,
restore identity/guards and empty states. TypeScript and the frontend build pass.
This counts scans, not CPU/RAM savings. Hooks and presentation boundaries are
controlled; no native restore or rendered UI is exercised. Temporary probe
removed, with no dependency or retained test added. Local 0.55 is unchanged.

## Focus after server removal after 0.55.0

Closing the deletion dialog after its server leaves the inventory requests focus
on the current page heading after dialog cleanup. Cancellation preserves normal
return to the opener. Library/server headings accept programmatic focus without
adding Tab stops. Actual App callback checks reproduce the missing fallback and
cover deletion, cancellation and missing/disposed targets. TypeScript and the
frontend build pass. Focus/frame boundaries are controlled; native keyboard,
rendered appearance and deletion are not exercised. Temporary probe removed;
local 0.55 packaging unchanged.

## Server snapshot ordering after 0.55.0

Library reads carry request and process-event revisions. Superseded reads cannot
replace a newer list, while state events arriving during a read override only
that server's state. Other snapshot metadata still updates. Confirmed creation,
rename and deletion invalidate pending reads; deletion also releases its cached
event state. This prevents a late list from reverting a newer process transition.

Twelve scoped checks extract the actual App callbacks through the TypeScript AST
and control list/native-acknowledgement boundaries. They reproduce the old state
regression and cover event cycles, independent servers, ordering, new rows,
failed reads, unchanged input and creation/rename acknowledgement. TypeScript and
the frontend build pass. No native Tauri event, deletion or Windows UI is exercised.
The temporary probe is removed; the local 0.55 installer is unchanged.

## Application event recovery after 0.55.0

Application subscriptions retry failed registrations every five seconds. A shared
ES/EN status notice remains until all failed subscriptions recover. Recovered
server-state, log and scheduler subscriptions request their existing read paths
to reconcile missed data; task/metric polling remains unchanged. Disposal cancels
retries, releases late registrations and ignores callbacks from the old owner.
Eleven controlled helper checks cover failures, recovery, healthy registration,
late completion and cleanup errors. TypeScript and the frontend build pass.
Native event failure/recovery and Windows interaction are not exercised; the
temporary probe is removed. The local 0.55 installer is unchanged.

Task and crash-restart panels now reuse the same retry helper. A recovered
subscription clears its warning and refreshes missed data once. Controlled actual
panel effects reproduce missing retries, then pass recovery, foreign/disposed
event and late-registration checks. TypeScript and the frontend build pass;
native event delivery/crash/UI remain untested. Temporary probe removed.

## Idle task-state reuse after 0.55.0

Task reconciliation reuses its previous map when the keys and task references
are unchanged. Empty snapshots and retained terminal outcomes no longer create
new application state on every poll. Active snapshots still remain authoritative:
missing jobs are removed, replacement tasks are accepted and changed progress
gets a new map. Poll timing, native reads and event handling are unchanged.

Thirteen direct helper checks reproduce the old idle identity change and cover
terminal/stale snapshots, updates, removals, replacements and source preservation.
TypeScript and the production frontend build pass. This proves state reuse, not
a measured CPU, RAM or frame-rate improvement; no native UI is exercised. The
temporary probe is removed with no retained test added. Local 0.55 is unchanged.

## Local Windows 0.55.0 package

Unsigned local x64 installer from `ee1669d5610902bce956fcabc02a9f05756ebb03`.
Five retained frontend tests, TypeScript, production frontend, optimized offline
NSIS packaging, versions and embedded icons pass. The installer is 6,881,643 bytes.
Generated-cache cleanup is deferred because other builds remain active; the guard
is preserved. Package and executable hashes remain unchanged. Local retention
keeps 0.54/0.55 and removes the verified obsolete
0.53 installer. Source changes recorded below after 0.54 are included in 0.55.
GitHub remains on 0.54; checksums stay local. Native interaction and gameplay are
not repeated for packaging, and the full manager objective remains open.

## Scheduler capacity and observation after 0.54.0

Task creation checks its per-server limit inside the same SQLite statement that
inserts the task. The previous separate count allowed concurrent callers to pass
the limit together. The existing limit message, validation and returned task
metadata are preserved; no scheduler schema or UI change is required.

An isolated native probe compiles the actual scheduler module against SQLite,
with a minimal storage adapter and temporary server records. Starting from 99
tasks, 32 concurrent additions previously persisted 131 tasks. After the fix,
one succeeds and 31 return the capacity error, leaving exactly 100. The same run
checks an independent server, removal/re-addition, saved task metadata and invalid
input preservation. This is not a full-engine, scheduled-action or native UI test.
Temporary fixture, probe source and compiled dependencies are removed; compact
logs remain local. Published 0.54 packaging is unchanged.

The scheduled-task panel also keeps event-subscription failures separate from
action errors. Successful changes or manual reads no longer hide the instruction
to refresh manually when automatic events remain unavailable. Actual-panel hook
and callback checks reproduce the old cleared notice, then verify success/failure
and manual-read persistence, independent action errors and owner reset. TypeScript
and the frontend build pass; subscription/IPC boundaries are controlled, with no
native scheduling or UI exercised and no retained test added.

## Copyable active UPnP endpoint after 0.54.0

UPnP now uses the existing address-row input/button pattern to copy its active
external endpoint. The readonly input remains selectable when clipboard access
fails, with existing translated recovery text. Pending, closing, attention and
expired mappings do not offer a shareable endpoint. Address changes invalidate
pending copy feedback, so an old response cannot mark a new address as copied.
This does not claim that router confirmation proves Internet reachability.

Seventeen scoped actual-panel effect/callback checks cover clipboard payloads,
failure recovery, stale replies, mapping phases/expiry and input selection.
TypeScript and the production frontend build pass. Status and clipboard boundaries
are controlled; no physical router, native clipboard or rendered visual check is
claimed. Existing layout, tokens and translations are reused. Temporary probe
source is removed; published 0.54 packaging is unchanged.

## Guarded task cancellation after 0.54.0

Progress controls now guard cancellation synchronously as well as disabling the
button. Repeated callbacks before a React render send one request for the task.
An accepted request remains guarded; a declined request or IPC failure releases
the guard for retry. Publishing and terminal stages cannot submit cancellation.
The existing task-ID component key isolates the guard from subsequent jobs.

The baseline actual-component callback check sends two IPC requests. Fifteen
scoped hook/callback checks pass after the fix, including accepted/pending
duplicates, declined/error retries, error visibility and noncancellable stages.
TypeScript and the production frontend build pass. Cancellation IPC is controlled;
no native task or UI is driven. Temporary probe source is removed, with no new
retained test or dependency. Published 0.54 packaging is unchanged.

## Shared modpack-import progress after 0.54.0

The importer consumes the application's readonly task map instead of subscribing
to native progress events itself. Matching server and task-kind ownership recovers
existing pack-install progress when reopening an import. A recognized active task
blocks duplicate work; a missing server leaves the dialog closable. Once installation
is acknowledged, a failed library refresh can be retried without creating another
server or reinstalling, even if the shared task snapshot still reports activity.

A baseline component check reproduces missing resumed progress. Forty-two scoped
ES/EN effect and callback checks cover task ownership, active/failed recovery,
missing servers, optional file selection, new-server association and read-only
refresh recovery. TypeScript and the production frontend build pass. Boundaries
are controlled; no real file import, native UI or consent change is exercised.
Temporary probe sources are removed; no retained test or dependency is added.
Published 0.54 packaging is unchanged; source awaits a later packaging milestone.

## Shared update-task progress after 0.54.0

Loader and Minecraft update panels consume the application-owned task stream,
which already combines native events with snapshot recovery. Opening settings
during a loader update now shows its existing task instead of waiting for a new
panel-local event. Both panels require the task input and filter server/kind
ownership. Their local display buffers still retain outcomes and allow dismissal.
The sole settings caller supplies the same shared source to both panels.

Two redundant native-event subscriptions are removed. Retry controls also follow
the surrounding busy/startup state, matching their action guards. A baseline
component/effect case reproduces the missing active loader task. Scoped ES/EN
checks cover recovered/live/terminal/dismissed tasks, foreign ownership, zero
child subscriptions and a confirmed loader update followed by failed list refresh.
Read-only recovery repeats the refresh without reinstalling. TypeScript and the
Windows frontend build pass. IPC, events and refresh boundaries are controlled;
no native update or UI is driven. Temporary probes are removed with no retained
test, translation or dependency added. Published 0.54 packaging is unchanged.

## Searchable, paged world inventory after 0.54.0

World inventory follows the existing 50-row paging pattern used by other local
lists. Search matches names without case/accent differences, preserves native
ordering and identities, and returns to the first page. Counts and empty-filter
recovery distinguish a filtered list from a server with no worlds. Paging closes
context menus and focuses the result status; pages clamp when inventory shrinks.
Small inventories keep their compact heading without a search field.

Export/duplicate drafts retain their existing state. File operations disable
search/paging while pending, and menu visibility follows the displayed rows.
Existing row actions, safeguards and combined-export defaults remain connected.
The search uses existing input styling and wrapping layout, without new colors.

A baseline production-component tree contains all 253 fixture rows; the changed
tree contains at most 50 live rows per page. Scoped ES/EN callbacks check page
bounds, focus requests, accent search, filter recovery, unchanged ordering/source
data, export/duplicate payloads, busy/read-error states and inventory shrinkage.
TypeScript and the Windows frontend build pass. IPC, motion, inventory and focus
are controlled; no native UI or actual file operation is exercised. Native full
inventory reading is unchanged; this is not a RAM/FPS benchmark, and transient
motion ghosts may overlap live rows. Temporary probes are removed, adding no
retained test or dependency. Four English messages are added. The published 0.54
installer is unchanged; these source changes await the next packaging milestone.

## Published Windows 0.54.0

[GitHub preview](https://github.com/Hakaelll/TCHLauncher/releases/tag/v0.54.0) groups the 0.52–0.54 milestones. Its
source tag resolves to packaged commit `1f4f7a795471f61d3f3eb7c6395ac95df8a858c1`.
Exactly one EXE asset is published; its reported size and SHA-256 match the local
verified installer. No checksum asset is uploaded. Local retention keeps 0.53/0.54.
Unsigned preview status, scoped validation and maintainer-owned native checks
remain explicit; publication does not close the full product objective.

## Changes packaged for Windows 0.54.0

Unsigned Windows x64 NSIS installer from `1f4f7a795471f61d3f3eb7c6395ac95df8a858c1`.
The package groups the 0.52–0.54 milestones. TypeScript, the Windows frontend,
five retained frontend tests, optimized offline packaging, versions and embedded
icons pass. The installer is 6,881,081 bytes. Source-specific evidence below
remains scoped; native UI and new Minecraft sessions are not repeated for packaging.

Protected cleanup removes 3,693,826,194 logical bytes of generated files while
retained 0.53/0.54 installers and the executable keep their hashes. The obsolete
0.52 installer is separately verified and removed. Checksums stay local; grouped
publication is prepared separately against the exact packaged source and EXE.

## Shared motion work reduction after 0.53.0

The navigation indicator retains its existing state object when an observer
reports identical geometry. List motion captures snapshot widths during its
existing row pass instead of querying and traversing those rows a second time.
Timing, easing, row ordering and the reduced-motion preference are unchanged.

Actual component/hook probes compare the committed baseline and changed source.
An initial measure plus two identical observer notifications produces one state
change instead of three; each list pass makes one row query instead of two.
These are controlled operation counts, not measured FPS, CPU or RAM improvements.
Checks also cover all indicator axes, changed/missing targets, observer cleanup,
reordering/removal, captured ghost widths, animation cancellation and zero list
geometry/cloning under reduced motion. TypeScript and the Windows frontend build
pass. Temporary probes are removed, adding no retained test or dependency.
Native animation appearance remains maintainer validation; installers are unchanged.

## Console drafts during pending sends after 0.53.0

Command acknowledgements clear the input only if it has not been edited or
navigated since submission. Retyping the same text, changing whitespace or
recalling history while a previous send is pending no longer loses that work.
Acknowledged commands still enter the bounded session history. When its oldest
entry is evicted, the active recall position moves with the retained entries
and navigation back to the unsent draft remains available.

An actual callback case reproduces a retyped command being cleared by an older
acknowledgement. Scoped ES/EN cases cover unchanged/new/retyped text, whitespace,
recall drafts, full-history eviction, duplicate sends, failed/negative replies
and disposal. TypeScript and the Windows frontend build pass. Hooks and send
replies are controlled; no Minecraft command or native keyboard interaction is
driven. Temporary probes are removed with no retained test, translation or
dependency added. Existing installers and the published release are unchanged.

## Crash-restart observation warnings after 0.53.0

Crash-restart subscription failures have their own notice instead of sharing the
status-read error. A successful initial/manual read no longer erases the warning
that automatic updates are unavailable. A failed read retains its own details
alongside the refresh guidance. Existing controls and restart policy are unchanged.

An actual component/effect case reproduces the lost warning when subscription
failure precedes read success. Scoped ES/EN checks cover both completion orders,
read success/failure, manual refresh outcomes, language switching, and late results
after disposal. TypeScript and the Windows frontend build pass. Subscription and
read boundaries are controlled; native Tauri delivery and real crash recovery are
not exercised. Temporary probes are removed with no retained test, translation or
dependency added. Existing installers and the published release are unchanged.

## JVM verification working directory after 0.53.0

Custom JVM option validation now runs Java in the server directory, matching the
real launch. Previously, options such as a relative GC log path were evaluated
in the launcher's directory and could reject a valid server profile or create
files beside the application. Only the child command's directory changes;
plain runtime version inspection keeps its previous behavior.

A real Java 21 case reproduces rejection of a log destination that exists only
inside an isolated server folder. After the fix, the same case writes the log in
that folder and persists the valid profile. Invalid arguments preserve the saved
profile, a missing server directory fails, ordinary runtime inspection succeeds,
and the process-wide working directory remains unchanged. The fixture root has
spaces. The native test target compiles; no Minecraft session or UI is exercised.
Both temporary libraries and the temporary case are removed, existing Java and
local installer hashes are unchanged, and no retained test or dependency is added.

## Real custom Java and JVM lifecycle after 0.53.0

Current native engine code runs Vanilla 1.21.4 with an existing external Java 21
selected through production runtime inspection/candidate validation and persisted
with the storage owner. No managed Java is downloaded. Java 17 and a corrupt
executable are rejected while the saved valid path remains unchanged.

A checked custom JVM profile writes a real GC log during startup. An invalid VM
option is rejected without replacing that profile. The Java path and arguments
survive reload/restart; restoring the recommended profile produces another clean
session without writing to the custom GC log. All three sessions reach readiness,
answer a loopback Minecraft status request and console command, then stop cleanly.

The run uses a fresh temporary library, seed 42, loopback binding, the official
SHA-1-verified Vanilla download and a copy of existing explicit EULA consent.
The test event collector replaces Tauri presentation. Temporary world data and
the temporary test are removed; existing Java executable and retained 0.52/0.53
package hashes are unchanged. No retained test or dependency is added. This
verifies native functions and Java/Minecraft behavior, not the native file picker,
Tauri command reservation, other loaders, UI interaction or Internet gameplay.

## Storage and retention message localization after 0.53.0

Storage measurements use the existing translated message for skipped symbolic
links. Backup cleanup keeps the removed count and warning messages as structured
notice data, translating each at render time instead of joining a Spanish result.
Changing language updates the result without repeating cleanup, and unknown
warning text or paths remain verbatim. One English result message is added.

An actual component-tree check reproduces the Spanish storage fragment in English.
Sixty-two scoped storage/retention component and callback checks cover ES/EN,
zero/nonzero excluded links, removed counts, translated and external warnings,
language changes after acknowledgement and failed post-cleanup list refresh.
TypeScript and the Windows frontend build pass. Hooks and IPC are controlled;
no actual disk scan, backup deletion or native UI is driven. Temporary probes
are removed, adding no retained test or dependency. Existing installers and the
published release remain unchanged.

## NAT-PMP actions during background reads after 0.53.0

Periodic status reads no longer own the mutation lock. Previously, a visible
enabled control could silently ignore discovery, opening or removal while a
background read was pending. An action now invalidates that older observation
and owns the lock until its operation and status reconciliation finish. Stale
read completion cannot clear the action lock or publish a stale error.

An actual component/effect/callback probe reproduces the ignored discovery click.
Seventy-two scoped checks cover ES/EN discovery, confirmed opening and removal,
older read success/failure, immediate duplicate actions, reconciliation and
removal feedback. TypeScript and the Windows frontend build pass. Polling and IPC
are controlled; no real router port, native UI, firewall setting or Internet
reachability is tested. Temporary probes are removed, with no retained test or
dependency added. Local 0.53 and the published 0.51 release remain unchanged.

## Changes packaged in local 0.53.0

Unsigned Windows x64 NSIS installer from `f58a04a79ad4cf247be88b2d7a8c934f38750d17`.
The package groups visible-series memory scaling, empty-discovery recovery and
ZIP/icon cleanup described below. TypeScript, the Windows frontend, five retained
frontend tests, optimized offline packaging, versions and embedded icons pass.
The installer is 6,883,055 bytes. No native UI or new Minecraft session is run.

Protected cleanup removes 1,916,188,833 logical bytes of generated files while
retained 0.52/0.53 installers and the executable keep their hashes. The obsolete
0.51 local installer is separately verified and removed. Checksums stay local.
GitHub remains at 0.51; publication cadence and manual-validation ownership are
unchanged. Package byte counts do not measure runtime memory or gameplay latency.

## Visible memory chart scaling after 0.52.0

The memory chart scales from its visible series. Hiding resident process RAM now
lets the heap line use its own range; hiding heap excludes its peaks as well.
The same typed series selection controls scaling, inspection and line visibility.
Both-series defaults, configured-memory fallback and CPU scaling are preserved.

A production-helper check reproduces the hidden-series scaling error. Thirty
scoped helper/component-tree/callback checks cover ES/EN, each visibility choice,
line coordinates, invalid/missing samples, process ownership and CPU defaults.
TypeScript, the Windows frontend build and five retained frontend tests pass.
Temporary probes are removed; no retained test or dependency is added. These
checks do not exercise native pointer/keyboard rendering or measure performance.
The local installer and published release are unchanged.

## ZIP dependency reduction after 0.52.0

ZIP uses explicit features matching its previous default compression codecs and
time support, with unused AES/password support disabled. Application readers do
not supply archive passwords. The locked Windows registry graph drops from 342
to 335 packages without added packages or version changes. Third-party notices
and their inventory are regenerated with no missing local license texts.

A targeted native test passes before and after the feature change using the same
Stored, Deflate, Bzip2, Zstd and Xz modpack fixtures. Production inspection checks
manifest metadata, selected override paths and exact extracted bytes. The same
AES-encrypted fixture is rejected without a password in both configurations.
Deflate64, LZMA and time features remain enabled but are not separately exercised
by these fixtures. Temporary probes and fixtures are removed; no retained test is
added. This measures dependency removal, not installed size, RAM or performance.
No installer or GitHub release is generated for this source change.

## Empty discovery search recovery after 0.52.0

Broadening an empty content search keeps both the submitted query and any current
text being edited. It removes the category restriction, includes other Minecraft
versions and returns to the first page without changing content kind or sorting.
The action appears only when a restriction can actually be widened, respects the
existing lock state, and returns focus/scroll to the result heading before its
button disappears. Fully widened searches instead suggest another name/filter.

An actual callback reproduces the previous query erasure. Forty-eight scoped
ES/EN component/callback/React SSR checks cover all five content kinds, category
and version combinations, preserved draft/submitted text, kind/sort preservation,
page reset, disabled/loading/nonempty states, heading focus requests and absent
focus targets. TypeScript and the Windows frontend build pass. Hooks, reads and
focus are controlled; no Modrinth request or native UI is driven. Temporary probes
are removed, adding no retained test, translation or dependency. The local 0.52
installer and published 0.51 release are unchanged.

## Windows icon-source cleanup after 0.52.0

Remove 13 unused macOS/Store/icon derivatives, totaling 3,413,415 logical bytes.
The configured Windows ICO and three PNGs retain their original hashes. Native
Windows resources and NSIS explicitly use the retained ICO; the unused ICNS entry
is removed from bundle configuration. The source emblem/cutout and Git history
remain available. The icon wrapper stages the full Tauri derivative set but copies
only the four configured Windows outputs; Python bytecode directories are ignored.
Deferred platform regeneration is documented rather than treated as active work.

A real installed-CLI generation into an isolated folder produces exactly those
four outputs with the expected PNG dimensions and ICO type. The generated ICO's
small/large pixels match the existing 0.52 executable and installer. Temporary
staging/probe outputs are removed. This verifies the generator and current package
icon identity, not a new application build or native UI. The 0.52 setup is unchanged.

A read-only inventory of regular checkout files, excluding reparse points, reports
about 159 MB before this derivative cleanup. Dependencies, retained executables
and Git objects dominate the remaining bytes. This is not allocated disk usage,
application-data size or a like-for-like percentage against the old Explorer
screenshot. No history, third-party license, server library or retained installer
is deleted. No dependency or retained test is added.

## Changes packaged in local 0.52.0

Unsigned Windows x64 NSIS installer from
`6c756089dee67355a3ad5047933315c0de31d32a`. Five retained frontend tests, TypeScript,
Windows frontend, optimized offline packaging, executable/setup versions and
embedded icons pass. The installer is 6,910,025 bytes. The release groups
scheduling validation, file-error advice, shared cached-image safety, skin-request
retirement and version-aware gameplay choices described below. No dependency or
retained test is added. Native interaction and gameplay are not repeated.

Protected cleanup removes 3,715,392,282 logical bytes of generated files, with
both retained setup hashes and the executable unchanged. The obsolete local 0.50
setup is separately verified and removed; 0.51/0.52 remain. These are file lengths,
not allocated-disk savings. No server library is changed. GitHub remains at 0.51;
no release or checksum asset is published for this local milestone.

## Version-aware gameplay choices after 0.51.0

Game-mode and difficulty option values now follow the same known 1.x version
boundary as the native editor defaults: numeric before 1.13, named from 1.13.
The imported current value remains visible as an explicit current-value option
when it uses another representation. Merely reading the form does not rewrite
it. Unknown version IDs retain the existing input-representation fallback, and
known pre-1.8 versions still omit spectator mode.

An actual component/change-handler probe reproduces modern numeric input emitting
another numeric choice and legacy named input emitting a named choice before the
change. A scoped ES/EN matrix then checks known old/current/prerelease versions,
unknown IDs, numeric/named imported values, exact changed-field payloads,
preserved original/other values, current-value options and spectator visibility.
TypeScript and the Windows frontend build pass. These are component/React SSR
checks, not native picker interaction or new Minecraft gameplay sessions.
Temporary probes are removed, adding no retained test, string or dependency.
The published 0.51 installer is unchanged.

## Skin-request registry lifetime after 0.51.0

The skin owner no longer retains an idle synchronization entry for every UUID
consulted during the application session. Each request removes completed entries
without an active failure backoff, plus expired failures. In-flight references
keep their lock identity. A shared five-minute duration governs both failure
retention and retry backoff; recent failures are not discarded just to trim memory.

A temporary native case reproduces 32 completed cached-profile entries remaining
before the change. It passes afterward with idle retirement, exact cached PNG
responses, preserved active-lock identity, retained recent failures, expired
failure release, backoff returning no head when appropriate, and four concurrent
cached requests. PNGs are real temporary files; failure timestamps/references are
controlled. The full native test target compiles. The fixture/probe is removed,
adding no retained test or dependency. No Java server or native UI is driven.

This fixes persistent registry growth across completed requests. It does not set
a strict entry cap, measure application RAM/latency, evict disk images or reduce
Minecraft's configured heap. The published 0.51 installer is unchanged.

## Cached-image validation and publication after 0.51.0

Skin heads and content icons now share bounded cached-PNG reads and temporary
publication. Cache directories must resolve to their expected canonical paths;
non-regular entries are rejected. Invalid, oversized or over-dimensioned PNGs are
cache misses. Files are written through a uniquely owned temporary file, synced,
and passed through checked replacement. A short shared I/O lock prevents this
process's concurrent cache writers from racing replacement. Existing paths,
namespace limits, skin freshness/backoff and remote-source policies remain intact.

A temporary native case reproduces both old public icon/head calls following
Windows junctions and returning invalid PNG bytes from outside their cache folder.
The corrected public calls reject those real junctions before network work.
The same case verifies missing/corrupt/oversized PNG handling, valid content/head
round trips, replacement, concurrent complete-image writes, rejected non-file
paths, preserved fixture bytes and absence of temporary-write leftovers. The full
native test target compiles. Five ES/EN native messages, TypeScript and the
Windows frontend build pass. Temporary code, junctions and fixture folders are
removed without changing server libraries or adding retained tests/dependencies.

These checks do not measure cache performance, drive native UI, contact Mojang or
Modrinth, simulate power loss or prove defenses against adversarial concurrent
filesystem swaps. Total disk-cache eviction is not added in this change. The
published 0.51 installer is unchanged.

## Windows filesystem recovery advice after 0.51.0

General action, server start/restart/runtime and failed task notices now share
recovery guidance for terminal Rust `os error` suffixes: Win32 39/112 indicate
insufficient disk space, 5 denied access, and 32 a file in use. The mapping follows
[Microsoft's Win32 error reference](https://learn.microsoft.com/en-us/windows/win32/debug/system-error-codes--0-499-)
and is checked against this computer's Win32Exception messages. Guidance asks the
user to check operation state before retrying; it does not repeat an operation,
change permissions or remove files. Unknown/unrecognized messages retain their
existing operation-specific advice. Original technical payloads stay verbatim.

The actual general error render first shows generic-only recovery for a disk-full
message. Sixty-six scoped React SSR checks then cover ES/EN, all three notice
families, exact JSON detail preservation, trailing whitespace and conservative
fallback for unknown codes, lookalike filenames and nonterminal markers.
TypeScript and the Windows frontend build pass. This does not force filesystem
failures, drive native UI or establish broader OS-error classification. Temporary
probes are removed, adding no retained tests or dependency. Linux error numbers
require separate interpretation when that deferred platform resumes. The published
0.51 installer is unchanged.

## Scheduled local-time validation after 0.51.0

The task form rejects local dates/times that JavaScript would silently normalize,
including a skipped daylight-saving hour. It also mirrors the native five-year
scheduling bound. Date and repeat-interval errors are associated with their fields,
focus the field needing correction, and clear on edits or modal dismissal.
Native task timestamps and scheduling behavior remain unchanged.

An actual submit-callback probe reproduces 02:30 becoming 03:30 in Madrid before
the change. Thirty-four scoped ES/EN checks then cover Madrid/New York DST gaps,
UTC, a repeated fall-back hour, valid/invalid leap dates, future/range validation,
repeat intervals, exact submitted timestamps, focus requests and field error markup.
TypeScript and the Windows frontend build pass. Native/focus boundaries are
controlled; this does not automate the Windows date picker or screen reader.
Repeated local times retain the platform's default occurrence. Temporary probes
are removed, adding no retained tests or dependency. The 0.51 release is unchanged.

## Published Windows 0.51.0

[Preview release](https://github.com/Hakaelll/TCHLauncher/releases/tag/v0.51.0) groups the
0.49, 0.50 and 0.51 milestones. Its only asset is the unsigned x64 EXE, built from
`7fc678d40a4f7302845f996b7474ebd531b1a64b`. The tag, remote asset digest and
6,900,811-byte size match the verified local package. TypeScript, five retained
frontend tests, Windows frontend, optimized offline NSIS packaging, executable/setup
versions and embedded icons pass. This does not add native interaction/gameplay
acceptance beyond the scoped evidence below.

Protected cleanup removes 3,714,008,336 logical bytes of generated files, with
both retained setup hashes and the executable unchanged. The old local 0.49 setup
is separately checked and removed; 0.50/0.51 remain. Checksums stay local. File
length totals do not measure allocated-disk savings. No server library is changed.

## Independent storage measurement after 0.50.0

Process metrics no longer wait for directory traversal. The size cache schedules
one background scan per server cell and returns its existing sample or an
explicitly absent initial measurement. Filesystem work runs outside the cell
lock. Cached timestamps and successful/error TTLs remain unchanged. Path changes
hide a previous directory's sample; invalidation detaches any older worker.
The storage meter distinguishes pending measurement from a reported read error.

One temporary native case passes with real temporary files, background publication,
active/stopped TTL transitions, stale values during refresh, directory changes,
invalidation and failed-read recovery. Controlled pending cells also verify
repeated reads retain pending/known data instead of running another traversal.
The full native test target compiles. Eight ES/EN actual meter render checks,
TypeScript and the Windows frontend build pass. Temporary probes are removed,
adding no retained tests or dependencies. No Java server or native UI is driven.

This removes filesystem traversal from the process-metric response path. It is
not a measured claim about large-library latency, peak RAM or gameplay networking.
The 0.50 installer and published 0.48 release are unchanged.

## Streaming retention review after 0.50.0

Retention preview no longer collects and globally sorts every directory entry
in a backup. The sorted walker processes entries as it visits them; individual
directory sorting still buffers siblings. Existing metadata, size, symbolic-link
and location checks remain in the traversal. No dependency or setting changes.
The fingerprint is internal to reviewed plans and is not a file-content checksum.

A temporary native case passes before and after the change using three nested
backup trees and a real SQLite library. It verifies exact byte totals and oldest
removal order, protected/newest preservation, repeatable review fingerprints,
refusal after a reviewed file changes, successful prune/database reconciliation,
preserved newest bytes and disabled retention. The full native test target
compiles; the probe is removed afterward and no retained test is added.

This establishes the scoped behavior and removal of the whole-tree collection.
It does not measure peak RAM, latency or behavior on a large production library.
The existing 0.50 installer and published 0.48 release are unchanged.

## Stop-backup preference recovery after 0.50.0

The native setter now returns the exact idle preference committed by its SQL
statement. The panel acknowledges that response without a second read that could
misreport a successful save. Manual refresh recovers failed initial reads and
retains the last known value on subsequent failures. Mutation stays disabled
until the preference is confirmed. Listener failures expose manual-refresh
instructions; disposed callbacks and repeated writes are ignored.

A controlled callback probe reproduces the previous saved-but-unchecked result.
Nine current component/reader checks cover acknowledgement without another read,
retry, preserved data, duplicate writes, disposal, the current change callback
and ES/EN listener-failure guidance. TypeScript, the Windows frontend build and
five retained frontend tests pass. These checks do not automate the native UI.
A focused native case also passes against a temporary SQLite library: returned
acknowledgements match persisted values, changing preference clears old error
metadata, and copying rejects mutation without changing the saved preference.
The full native test target compiles; the temporary case is removed afterward.

The change is not included in the existing 0.50 installer. No release is
published, no dependency is added and temporary probes add no retained tests.

## Windows startup review after 0.50.0

The generated 0.50 NSIS script removes the same current-user Run value used by
native autostart and excludes update mode from that cleanup. No custom uninstall
hook is needed for this behavior. Runtime launch validation reads the local Java
version and cached requirement without a catalog lookup in that function.

This is read-only source/generated-script evidence. It does not replace an actual
registry transition, new silent uninstall or offline Minecraft session. Installed
preferences are unchanged, and no test, dependency or runtime behavior is added.

## Changes packaged in local 0.50.0

Unsigned Windows x64 NSIS installer from
`abfe08a164bc911cb75f4a082955c7caaee87ec8`. TypeScript, Windows frontend,
optimized offline packaging, executable/setup versions and embedded icons pass.
The installer is 6,900,234 bytes. GitHub remains at 0.48.0; no new release or
checksum asset is published. Local retention keeps the verified 0.49/0.50 setups.
Protected cleanup removes 3,713,878,156 logical bytes of generated cache, with both
setup hashes and the executable hash unchanged. Allocated-disk savings are not
measured. Native interaction and gameplay are not repeated for this package.

Download tasks reuse successful HTTP clients through separate catalog, transfer
and strict modpack profiles. Task IDs, cancellation, progress and byte limits
remain task-owned. Builder failures are not cached. Redirect policies, user agents
and timeouts are unchanged; the modpack profile retains its origin allowlist.

One focused native loopback case reproduces four independent connections before
the change and passes with two shared connections afterward, one per profile.
It also checks independent cancellation, verified transfer bytes, preserved old
targets on hash mismatch and partial-file cleanup. The full native test target
compiles. Temporary probe/include files are removed, adding no retained tests or
dependencies. This proves controlled connection reuse, not a remote HTTPS,
latency, RAM, native UI or Minecraft-network improvement.

The strict modpack builder also reproduces two per-file connections before caching
and passes with one shared loopback connection afterward. Its native case verifies
SHA-512, per-file byte limits, cancellation isolation, preserved old destinations,
partial cleanup and refusal to follow a forbidden redirect. Initial/redirect URL
validation is unchanged, and the original builder tokens compare equal apart from
whitespace. The standard catalog/transfer factory is unchanged. The temporary
probe/include is removed; no retained case or dependency is added. This is a local
transport check, not remote modpack-provider or native interaction acceptance.

Local library, player, installed/disabled content and backup searches now share
the accent/case normalization used by settings, command search and menu typeahead.
For example, `montana` finds `Montaña` and `decoracion` finds `Decoración`.
Matching never rewrites names, merges identities or changes source files. Console
substring search and outgoing Modrinth queries retain their existing behavior.

Actual filters and memo callbacks reproduce inconsistent matching before the
change and pass 18 scoped cases afterward, including composed/decomposed accents,
different identities, unchanged source data and preserved filters. The five
retained frontend tests, TypeScript and the Windows frontend build pass. Native
interaction remains maintainer validation. No dependency or retained test is added.

CSS cleanup removes eight declarations shadowed by later root rules with the same
selector, property and importance. These cover six old colors plus the runtime
mark's obsolete background and corner radius. Empty rules are removed too. The
remaining declaration order/context and final root property values compare equal
before/after, and the Windows frontend build passes. No dependency or retained
test is added. This is source cleanup, not a measured rendering-speed improvement
or native visual validation.

Console command, clipboard and export failures now carry an error kind rather
than sharing plain success notices. Errors use the theme's danger color and
`role="alert"`; confirmations retain `role="status"`. Minecraft lines and copied
content remain verbatim, and exporting paths still render unchanged.

Actual callbacks/notice JSX reproduce ordinary-status errors before the change
and pass 12 scoped English/Spanish command/copy/export success/failure cases after
it. TypeScript and the Windows frontend build pass. Clipboard and API boundaries
are controlled; native interaction remains maintainer validation. No retained
test or dependency is added, and no new installer is built for this change.

The background task panel can hide all completed results in one action. Active,
cancelled and failed tasks remain visible; existing task-ID checks protect a new
operation that replaces an old completed result before the click. Clearing returns
focus to the panel heading and never cancels or retries native work.

Actual panel/callback checks pass eight cases across English/Spanish, including
mixed outcomes, empty/non-completed lists, stale task replacement and focus return.
TypeScript and the Windows frontend build pass. Hooks, portal and focus boundaries
are controlled; native popover interaction remains maintainer validation. No
dependency or retained test is added. The local 0.49.0 installer predates this change.

## Changes packaged in local 0.49.0

Unsigned Windows x64 NSIS installer from
`daac81c7d11068ce06d792d2d002ed586d34598d`. TypeScript, Windows frontend,
optimized offline packaging, executable/setup versions and embedded icons pass.
The installer is 6,901,187 bytes and remains local; GitHub stays at 0.48.0.
Local retention keeps 0.48/0.49 after checking all setup hashes. Protected cache
cleanup, initially deferred by active compiler processes, now removes
1,923,146,108 logical bytes. Both retained setup hashes and the executable hash
remain unchanged. This measures file lengths, not allocated-disk savings.

RAM and CPU plots now share the inspected timestamp for pointer and keyboard
navigation. A missing reading keeps the selected time and shows a dash instead of
substituting the latest value. Escape returns both plots to recent readings;
server/process/range changes cannot inherit a selection from another context.
Repeated pointer movement within one sample reuses the inspection state.

Actual component/callback checks reproduce independent timestamps before the
change and pass afterward for shared inspection, missing CPU readings, Escape,
pointer leave, context changes and unchanged-state identity in English/Spanish.
All 32 scoped cases, TypeScript and the Windows frontend build pass. Hooks and
pointer bounds are controlled; native visual/keyboard validation remains with the
maintainer. No dependency or retained test is added, and sampling is unchanged.

The maintainer confirms on October 31, 2026 that the old reports from the other
computer are already resolved. Those reports are closed on user confirmation and
must not remain implementation blockers. Pending manual validation belongs to
the maintainer and does not stop independent improvements.

Modrinth pagination now moves focus and the viewport to the results status when
Previous or Next is requested, following the installed-content list pattern.
The status remains mounted during loading, so keyboard focus has a stable target.
Actual paging callbacks reproduce the missing focus/scroll before the change and
pass afterward for both directions and an absent ref. TypeScript and the Windows
frontend build pass. Focus/scroll boundaries are controlled; native interaction
remains maintainer validation. No retained test or dependency is added.

Players and pre-import file inventories now follow the same paging behavior.
Players focus the result count; import inventories focus their labeled file list.
Eight actual callback cases cover Previous/Next and absent refs, reproducing the
previous missing movement and passing after the change. TypeScript and the Windows
frontend build pass. Focus/scroll boundaries remain controlled; no native keyboard
or screen-reader result is claimed. Filtering and server operations are unchanged.

Startup preferences now translate both cached-data notices after a read failure.
The English strings already existed; two JSX branches bypassed `t`. A focused
render check reproduces both Spanish leaks before the fix and passes afterward
across English/Spanish switching and all four cached-data combinations. TypeScript
and the Windows frontend build pass. This checks actual component markup with
controlled read snapshots, not native startup or IPC. No retained test is added.
The published 0.48.0 installer predates this two-line fix.

The same JSX review found untranslated notes in the JVM preview before installation
and the operator-permission conflict summary. Both now use existing translations.
Actual paragraph renders reproduce the leaks before the fix and pass afterward
in English/Spanish with both conditional branches. TypeScript and the Windows
frontend build pass. Player action labels found by the scan already translate in
`PlayerRow` and need no change. The scan checks exact existing catalog matches;
it does not prove complete localization, native Java or permission operations.

## Verification after packaging 0.48.0

The current-source Paper 1.21.4 scenario also passes. Three sessions check
readiness, status, commands and graceful stops, then backup and exact world
restoration. It exports populated Overworld, Nether and End folders and imports
them into a second Paper instance. All 45 copied file hashes match before boot;
read-only block checks find the three saved blocks after boot. The source and
export remain unchanged. All five sessions return real TPS readings.

The isolated library and consent copy are removed, and both package hashes are
unchanged. No new test case or production change is added. These checks use the
engine's test event collector, not Tauri interaction or a Minecraft client.

The existing real Vanilla case now configures hard difficulty and creative mode
through the native settings save before any world exists. Minecraft's initial
startup reports CREATIVE; read-only seed/difficulty queries confirm 42 and Hard.
The same case retains three clean sessions, backup, byte-identical world restore
and restored startup. It passes with actual Java/Minecraft 1.21.4 in a fresh
loopback-only library. Existing consent is reused without changing user settings.
Owned processes exit, temporary data is removed and package hashes are unchanged.

Only the existing cfg(test) scenario expands; all nine real scenarios remain and
no production code changes. The event collector replaces the Tauri UI, so native
interaction, other versions/engines and client behavior are not established.

## Changes packaged in 0.48.0

After packaging, the existing Vanilla lifecycle scenario passes against the same
application source: actual managed Java/Minecraft 1.21.4 downloads, three ready
sessions with status and command responses, three graceful stops, tracked backup,
byte-identical world restoration before boot and successful restored startup.
It reuses the current explicit EULA record in a fresh loopback-only temporary
library. Owned processes exit and temporary data is removed; user servers and
settings are not written. Installer/executable hashes remain unchanged. The
event collector is a test adapter, so this is not Tauri interaction, multiplayer
gameplay, router validation or evidence about the friend's workload. No retained
test case, application code or package changed.

Local unsigned Windows x64 NSIS package from `382aca1b4b645c0952cc5c485d33e4186dc5d95e`.
TypeScript, frontend, optimized offline packaging, executable/setup versions and
embedded icons pass. The installer is 6,904,819 bytes and is published as the
0.48.0 Windows preview, fulfilling the maintainer's request to release the
batch-closing version. GitHub contains exactly one EXE with the verified digest;
no checksum file is attached. Native interaction and gameplay were not repeated
for this package.

LAN and NAT-PMP clipboard feedback belongs to the exact address and most recent
copy request. A changed address cannot inherit Copied or a clipboard error from
the former value. Copy retries clear only copy feedback; public-IP and router
operation failures remain visible. Disposed replies are ignored, and the existing
LAN acknowledgement timer remains. Actual component checks reproduce both former
stale labels and cover delayed replies, failures, request order, disposal and
ES/EN markup with controlled clipboard/network/hook/timer boundaries. TypeScript
and the Windows frontend build pass; OS clipboard/router use was not exercised.

Download reads retain the response buffer instead of cloning every chunk into a
second owned vector. Existing byte limits, checksums, cancellation and publication
remain in the same consumers. Production transfers against a loopback HTTP server
pass SHA-1/256/512, no-hash, mismatch, cancellation and partial-cleanup checks.
The first fixture run exposed an inherited nonblocking Windows socket; explicitly
making the accepted test socket blocking fixes that fixture race. No production
failure was suppressed. Full native test compilation passes, including JSON/text
buffer consumers. The temporary check was removed; retained tests and dependencies
are unchanged. This does not measure CPU/RAM gains or exercise HTTPS/native UI.

Console following skips animation when already at its target. The paused-view
counter counts matching arrivals directly instead of allocating another filtered
array. Actual prior/current effect checks reproduce the idle frame and preserve
normal/reduced-motion scrolling, immediate large jumps, pause and cleanup. Twenty-
four count combinations preserve server, clear and disk-history boundaries.
DOM/RAF boundaries are controlled; this is not a native scrolling or CPU/RAM
benchmark. TypeScript and the Windows frontend production build pass.

UPnP and NAT-PMP distinguish status-read failures from operation failures.
Successful polling clears a recovered read error without hiding a failed
discovery/open/close operation. Cached details remain visible, but an unreadable
status is labeled unverified and disables new openings and pending confirmation.
The confirmation dialog also shows the read failure. Removal remains available.
Controlled checks reproduce both former sticky errors and cover recovery,
preserved operation failures, opening guards, confirmation feedback, unmount and
ES/EN markup. TypeScript and the Windows frontend build pass. No physical router
was queried or changed by these checks.

Initial numeric values and chart lines no longer schedule animation frames for
unchanged geometry or values. A numeric value also recovers immediately from a
non-finite starting value instead of propagating it through interpolation.
Changed readings retain their existing animation duration and retargeting.

Reduced-motion lists skip geometry reads and DOM cloning, and clear retained
positions and snapshots. Re-enabling normal motion rebuilds tracking on the next
list change. Controlled component/hook checks reproduce the old redundant work
and cover changed targets, interruptions, invalid values and preference changes.
DOM, hook storage and animation scheduling are controlled; operation counts do
not establish a measured CPU/RAM improvement or native visual acceptance.
TypeScript and the Windows frontend production build pass. These changes are included in the local 0.48 installer.

## Changes packaged in 0.47.0

Windows x64 NSIS build from `db647c2554e0db8cdf493af278a5938c4684dc59`, published as
an unsigned preview with exactly one EXE asset. The remote tag, asset size and
digest match the local package. TypeScript, frontend, optimized offline native
packaging, executable/setup versions and embedded icons pass. No native
interaction or live Minecraft session was repeated for this package.

Combined content plans now preload explicit root versions before traversing
dependencies. A dependency without an exact version uses the reviewed root
selection instead of independently selecting the latest release. Previously,
visiting that dependency first could select a stable release and then reject a
valid plan containing a selected beta. Exact conflicts still fail.
Preloaded versions are reused, including canonical project IDs when a root uses
a slug. Version identity, compatibility, files and graph limits remain checked.
The isolated production-resolver check reproduces the ordering failure and
covers root order, aliases, cycles, conflicts, invalid compatibility/identity and
the root-count bound. Catalog, compatibility and artifact boundaries use local
fixtures; it does not install mods or establish gameplay behavior. The full
offline native library check passes. The temporary probe source and binaries
were removed, without adding retained tests or application dependencies.

The shared progress panel now mounts its cancellation state by task UUID. A new
task no longer inherits a pending cancellation or cancellation error from the
previous task, including a late reply to the previous cancellation request.
This applies to every caller, including the update and modpack panels that do
not supply their own React key. Progress updates within one task retain state.
An isolated component check reproduces the previous leak and covers accepted,
refused and rejected replies, new-task error clearing and ES/EN React markup.
Hook storage/remount semantics and cancellation IPC are controlled. TypeScript
and the Windows frontend build pass; native downloads were not exercised.

Read errors now use the application-copy translator in console history, player
reads, Java/runtime reads and Modrinth availability. The cached-runtime marker
also follows the selected language. Invalid player names/reasons and the crash
retry limit return localized validation text from their existing helpers.

Post-restore refresh state now distinguishes loading from per-section failures.
Properties, backups, EULA consent and library errors remain separate strings until
rendering, so changing the language after a failure updates each section and
detail. Retry performs the same four reads and never repeats restoration. A
successful retry clears only its server entry; stale revisions and consent reads
for another selected server do not replace the current result.

Actual old JSX/helper checks reproduce untranslated text. Checks of the changed
fragments, validation helpers, App refresh callback and notice component cover
ES/EN, all four failure sections, retry and stale replies. The actual console row
and runtime-path fragment retain original text. Read/React boundaries are
controlled; these are not native interaction or restore-operation checks. These
source changes pass TypeScript and the Windows frontend production build and are
packaged in the published 0.47 installer.

## Changes packaged in 0.46.0

The English catalog adds 843 application-owned native messages across seven passes:
startup/JVM, players/permissions, properties, import, basic networking, backups,
retention, worlds, recovery, router diagnostics, content management, engine
installation/updates, startup, task states and diagnostic reports. Existing
translations are retained and scheduler help uses its existing English action
label. Reviewed native copy now
lives in `src/locales/native-en.ts` and is included by the general catalog.

Actual locale checks reproduce the original Spanish fallback and failures hidden
behind a stored `Error: ` prefix. A general UI name template could also partially
translate a whole diagnostic. The prefix now uses only reviewed native entries
and patterns; exact complete-message translations take precedence. An unknown
`Error: `-prefixed path ending in `copia` remains unchanged. The shared matcher reuses compiled
patterns. The helper's return type widens translated strings while preserving
non-string values, without the previous generic cast.

Checks cover every added message in ES/EN with formatted arguments, stored error
wrappers, unknown details and non-string identity. Catalog regrouping preserves
values, reverse aliases and general lookup results for generated examples from
all entries. TypeScript and the Windows frontend build pass; Vite reports its
main-chunk size advisory. These are locale/helper checks, not native interaction,
backup recovery or gameplay checks. No runtime performance gain is claimed.

The network pass adds 93 UPnP/NAT-PMP diagnostics and moves eleven existing
Minecraft-query translations into the native catalog without changing their values
or reverse aliases. The NAT-PMP timeout now suggests manual forwarding instead of
the removed tunnel feature. Actual locale checks include stored error wrappers
and preserve unknown protocol/vendor details. Router discovery, opening, renewal
and cleanup logic are unchanged; no physical-router interaction was performed.

The content pass adds 142 messages for Modrinth plans, JAR/descriptor checks,
content transactions and mrpack validation, and moves nineteen existing entries
into the native catalog without changing their translations. Loader-specific
compatibility messages avoid splitting a mod name that itself contains the
Spanish compatibility wording. Locale checks retain project names, paths and
version ranges in formatted arguments.

Fabric load-list and dependency-review notes previously arrived as one joined
string, which prevented independent translation. The native audit now sends
`logNotes`, an array, and the summary translates each note before joining them.
The actual summary paragraph renders the expected Spanish/English text for
complete and incomplete review fixtures; the old joined representation reproduces
the translation gap. TypeScript, the Windows frontend build and the full offline
native library check pass with the updated contract. This does not confirm native
interaction, mod loading or gameplay behavior.

The engine pass adds 146 messages for Fabric, NeoForge, Paper, managed Java and
version changes, preserving fifty existing translations. A further 108 messages
cover import identity, startup registration, publication/recovery, memory counters,
scheduled/crash-restart status and player skin validation, preserving another
twenty-seven translations. Actual locale checks cover ES/EN, stored error wrappers,
multiline technical details, unchanged catalog values and reverse aliases. No
engine installation, OS startup, account login or scheduler execution was repeated
for these catalog-only changes.

All remaining static candidates in the initial heuristic inventory have been
reviewed. SQL statements remain untranslated; their application-owned status
values are handled separately. This inventory is not a complete census. Nested
application errors in fourteen reviewed wrappers now have explicit argument metadata.
The old helper reproduces untranslated nested bodies in eight initial fixtures for attempt failures, content
migration, retention, recovery, catalog lookup, dependency review and partial player
commands. Only application-error arguments are translated, to a maximum depth of
eight; project names, destination lists and raw NeoForge installer output remain
data. The general translator retains its lookup priority and explicit-argument
behavior. Three existing messages move into the native catalog without changing
their values or reverse aliases.

Actual locale checks cover nested messages, ES/EN switching, opaque details,
bounded depth and compatibility with the existing catalog. Joined error lists and
native interaction still needs review. A separate Rust token reader inspects
4,415 string literals, including raw strings and escaped values, and finds another
117 application-copy entries beyond the first inventory. Checks cover JVM option
feedback, progress labels, player-file errors, import and restart outcomes. Six
additional wrappers identify their application-error arguments while preserving
raw Java output and rollback paths. Existing translations and generated general
lookup examples remain unchanged. TypeScript and the frontend build pass.

The remaining heuristic candidates are manually excluded: an English source
comment, an already-English process-session error, three English test messages,
the Windows reserved filename `CON`, a raw launcher console line and a router
fallback for platforms outside the current Windows scope. This is source-copy
evidence, not proof of every native UI path. Five file-picker commands now receive
the current ES/EN value at invocation. The existing native language enum selects
image, Java, player-list and export labels; no second preference store is added.
Actual API checks cover fifteen calls across ES/EN/ES transitions and preserve
server IDs, with the IPC boundary substituted. Native dialogs are not opened.
These changes are packaged in the published 0.46.0 Windows preview.

Stop-and-exit failures are now returned as separate server ID/name and cause
records rather than a newline-joined paragraph. Stop errors, timeouts and backup
errors retain their own data. The dialog lists each server and translates its
detail independently, including names that match application-copy keys. An empty
failure list permits exit; any failure releases the close guard and allows retry.
All stop requests still precede observation, and only cleanly stopped servers get
their configured safety copy.

The actual Rust shutdown function passes mixed failures, all-success, empty-target
and delayed-clean cases. Its serialized output is used directly by the actual
React error component in ES/EN checks. Production App callback checks cover retry,
successful exit request, IPC rejection, hide, duplicate invocation and a late
reply after unmount. These checks control process/window/API boundaries; they do
not exercise the installed Windows dialog or a live server shutdown. TypeScript,
the Windows frontend build and the full offline native library check pass. The
failure list is keyboard-focusable for scrolling inside its bounded panel.

The optimized offline NSIS build completes from source commit
`2007f9b7815edf050846844190efa4f401611410`. The 6,897,697-byte installer and
executable report version 0.46.0, and both embedded icons match the logo. GitHub
release 0.46.0 contains exactly one EXE asset, with its server-reported digest and
size matching the local artifact and its tag pointing to the packaged commit.
The installer remains unsigned. No installer lifecycle, native interaction or
Minecraft gameplay session is repeated for this version.

## Workspace maintenance after 0.45.0

Removed the unused 0.32 commit-message draft, ten ignored localization-migration
scratch files from 0.25 and two empty scratch directories. The discarded Cargo
metadata is not an input to the current legal generator, which runs Cargo metadata
itself. Current translations, license resources, executable and both retained
installers have unchanged hashes. The regular-file inventory is cross-checked
with independent directory enumeration without following redirected entries;
its logical-byte count is not an allocated-disk measurement. No application source
or package changed, so no build, gameplay check or release was repeated.

## Changes packaged in 0.45.0

These changes are included in the unpublished local 0.45.0 Windows installer.
TypeScript/frontend and optimized offline NSIS packaging pass. Executable/setup
versions and embedded icons are verified; the build is unsigned. Package source:
`121c1f3643d50e609027261e91c36f7bd4dfe47f`. No native interaction, installer lifecycle
or Minecraft session was repeated for this milestone. The published preview from
this thread remains 0.44.0; the normal next publication window is 0.47/0.48.

Size-format callers now import the existing formatter directly instead of routing
through the visual server component. The pass-through export is removed; the
formatter and displayed values are unchanged. TypeScript and the Windows frontend
production build pass. This cleanup makes no measured application-size or runtime-performance claim.

Shared server/world/content action menus now support timed type-ahead: prefixes,
repeated-letter cycling and accent-insensitive matching skip disabled actions.
Arrow/Home/End navigation resets the prefix; composition and modified shortcuts
are left alone. Typing only moves focus and never executes an action. Focused
production callback checks cover Spanish/English labels, timeout reset, disabled
and empty lists, existing navigation and Escape. Hook effects/DOM focus and native
assistive-technology interaction are outside that isolated check. TypeScript and
the Windows frontend production build pass.

The shared properties reader now recognizes colon/whitespace separators, escaped
keys and backslash-continued logical records. Its writer replaces the complete
changed record and preserves unrelated original bytes, including mixed line
endings and an unterminated final line. Trailing continuation markers are isolated
before adding or replacing records so the next setting is not absorbed. A
colon-separated port fails before the fix; actual `read`/`parse`/`prepare` checks now pass separators, comments, duplicate
precedence, continued keys/values, no-op byte equality, seed/world Unicode,
validation and unchanged source files. Short malformed Unicode escapes are no
longer silently decoded. Other values retain the editor's existing raw-escape
representation; this is not full `java.util.Properties` conformance. The grammar
reference is [Java's Properties documentation](https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/Properties.html#load(java.io.Reader)).
Full offline native `cargo check --lib` passes. Two sandbox builds first failed
with MSVC LNK1114/access-denied while replacing a dependency archive; the same
check passed outside the sandbox. No UI/client outcome is claimed.

Console batches now pause React publication while the document reports hidden.
The existing 1,000-line ring keeps only the latest entries, then resumes normal
batch publication on visibility return. Reset/disposal invalidate cancelled
callbacks; switching servers still clears queued lines. The actual App effect
fails the hidden-publication check before the change and passes afterward, with
retained order/arrival identities and visibility-listener cleanup. Pure buffer
checks also cover overflow, repeated resume and stale-timer races. Browser
visibility/timers are substituted in that check; Windows tray behavior and an
actual CPU/RAM saving are not established. TypeScript and the Windows frontend
production build pass. Native server logging, scheduling and state events are
unchanged; no network/gameplay improvement is claimed.

Memory reports now discard the former process session before applying timestamp
or resident-memory availability checks. A new session, PID or uptime reset cannot
leave the previous session visible after a clock rollback or an unavailable first
RAM reading. Missing RAM alone still preserves samples from the same session and
does not allocate a history for a never-sampled server. The App also clears only
the starting server's chart history, avoiding mixtures when a PID is reused.
Production recording/callback probes reproduce both failures before correction
and pass after it; native recording is extracted with model/serialization
boundaries substituted. TypeScript, the Windows frontend build and full offline
native Cargo check pass. No actual clock change, Windows PID reuse or Minecraft
restart is exercised by those probes. Heap collection remains read-only; this
fix does not establish a memory leak, lower Java RAM use or solve Internet stalls.

Fabric schema-1 environment parsing now follows the supported string contract:
missing/empty means universal, client/server names are case-insensitive, and
arrays or other JSON types are rejected. This removes the previous array union
inference and false rejection of accepted string variants. The reference is
[Fabric Loader 0.19.5's metadata parser](https://github.com/FabricMC/fabric-loader/blob/0.19.5/src/main/java/net/fabricmc/loader/impl/metadata/V1ModMetadataParser.java).
A focused native test fails before on an accepted environment array, then passes
string/default/invalid-type cases and actual JAR metadata inspection. Synthetic
provider/consumer/client JARs also pass the existing dependency review: the alias
satisfies the consumer and the client-only parent is excluded. This is metadata
inspection, not Minecraft loading or a real backpack-mod check. The temporary
unit and fixtures are removed; retained test counts are unchanged. Three existing
environment/client-only messages now translate in ES/EN through the actual locale
module. TypeScript and the Windows frontend production build pass. No GitHub release is generated for this milestone.

## Changes packaged in 0.44.0

The changes below are included in the published 0.44.0 Windows preview.
Verification limits remain attached to each observation.
The reference audit also removed the unused single-file diagnostic wrapper.


Backup history now reuses the shared 50-row window and existing pagination
controls. Search/reason changes reset the page, shrinking lists clamp it and
page changes request focus on the translated result count. Restore receives the
original selected record; busy/read-error/running-server guards stay in place.
Focused ES/EN React markup, production callbacks/filtering and page helper checks
pass, as do TypeScript and the frontend build. Policy/icon/motion presentation
was outside that isolated check; native interaction remains manual. This is not
a measured RAM/speed improvement.

CPU charts now default to a labeled automatic percentage axis, rounded in five
percentage-point steps with a 5% floor and 100% ceiling. A pressed-state control
switches back to a fixed 0–100% axis. Sample values, peaks and collection remain
unchanged. Actual domain/helper, ES/EN chart markup and toggle checks pass with
TypeScript/frontend builds; animation/icon presentation and native layout remain
outside that check. Process collection was reviewed and already targets only the
Java PID, with bounded session entries; no runtime performance gain is claimed.

Automatic folder-size cache success lifetimes are now five minutes with an
active process and one minute while stopped; errors retry after fifteen seconds.
Path changes and explicit invalidation still force a read, and detailed manual
storage inspection is unchanged. The meter labels the active/stopped cadence
beside the actual sample time. Actual cache code passes deterministic age and
counted metadata-boundary checks; ES/EN meter markup, TypeScript/frontend build
and full offline native Cargo check pass. The standalone cache probe substitutes
the clock/measurement boundaries; it is not a directory or gameplay benchmark.
No claim is made that the reported Internet stalls are resolved.

Quiet content-log updates now reuse a frozen empty diagnostic list, allowing
unchanged inventory rows to retain their memoized result. Empty diagnostics skip
all per-file pattern construction. Production-function checks cover identity,
constructor counts, actual Paper associations, unrelated names and preserved
static inventory warnings; TypeScript/frontend build pass. This is deterministic
work avoidance, not a measured CPU/RAM reduction or native interaction check.

Import preview and configuration now share the existing supported properties
reader. `!` comments no longer become editable settings; duplicate keys use their
last effective value before preview validation. Saving omits unchanged values,
retaining their original escapes/duplicate entries. Preparation reads one snapshot,
returns it verbatim for no-op changes and keeps CRLF when editing CRLF files.
Actual native read/preview/prepare checks reproduce the prior comment and
preservation failures, then pass with private values omitted, unchanged source
files and invalid changed ports/booleans/keys/seeds still rejected. The native test
target compiles; no UI/server-boot or full Java-properties grammar claim is made.

World names now share the existing seed string escaping codec for `level-name`
reads and writes. A native fixture with accented/CJK folder names confirms import
preview and active-world detection agree. Real selection creates a safety backup,
preserves the previous properties and world bytes, and performs no extra backup
when the selected world is already active. Unrelated seed/MOTD escapes remain
verbatim. The native test target passes; fixtures contain minimal NBT and do not
establish a Minecraft boot, UI outcome or complete properties grammar support.

## Source after the local 0.42.0 installer

These changes are included in the local 0.43.0 installer and were absent from
0.42.0. Version 0.43.0 is now a published Windows preview; 0.42.0 remains local.
Remaining native interaction and
host-specific checks belong to the maintainer and do not block implementation.
Focused check receipts are local under `artifacts/`; they are not a claim that
all Windows interactions or Minecraft gameplay have passed.

| Commit | Change | Recorded check and limit |
| --- | --- | --- |
| `27ed69b` | Restore minimized main windows before focusing them. | Native Cargo check; tray interaction remains manual. |
| `5701519` | Paginate disabled content in groups of 24. | ES/EN component markup, page clamping and focus/scroll requests; native interaction untested. |
| `5446776` | Poll task snapshots every 15 seconds while hidden and three while visible. | Actual effect and scheduler, stale replies and error/disposal paths; no measured CPU/RAM improvement. |
| `63b93a5` | Scope public-IP/copy feedback to the current server; replace old copy timers. | Actual handlers reproduce the old stale reply and pass after correction; no native clipboard/router exercise. |
| `b185c88` | Record eight native copy/manage import cases. | SQLite and exact fixture file equality for renamed Paper/Fabric, legacy Vanilla and explicitly confirmed unknown launchers; synthetic metadata JARs, no Minecraft boot or script execution. |
| `85cc28b` | Align shared server-name length validation with HTML UTF-16 limits. | Actual Rust function compiled standalone; Unicode, bounds, blanks, controls and trimming pass; no full packaging/native entry check. |
| `3175e1c` | Show new matching lines while console following is paused. | Actual callbacks, arrival helpers and ES/EN control markup; native scrolling untested. |
| `cd45f49` | Distinguish failed/cancelled/interrupted one-off tasks from Completed. | Production helper before/after in ES/EN with active/recurring priorities; scheduling and persistence unchanged. |

Discovery icon loading now uses the existing four-worker page queue instead of
one independent request per card. Completed icons survive reopening the current
page. Leaving the page stops new work and ignores late replies; already-started
IPC calls can finish. Actual effect checks compare 24 simultaneous old card
requests with four workers, including missing URLs, failures and reopening.
This is a deterministic API-boundary check, not a network/RAM benchmark or proof
that the reported game-connection stalls are resolved. Native use remains manual.

Relevant frontend changes passed TypeScript and production builds. Temporary
probe sources and generated caches were removed; retained regression tests were
not expanded. Packaged changes through 0.43.0 are in its
[release notes](../release-notes/RELEASE_NOTES_v0.43.0.md).

## Earlier records

- [Console, sidebar and transfer refinements after 0.29.0](OPERATIONS_AFTER_0.29.0.md)
- [World and installed-content action refinement after 0.26.0](WORLD_ACTIONS_AFTER_0.26.0.md)
- [Performance work after 0.26.0](PERFORMANCE_AFTER_0.26.0.md)

- [0.31.0](DEVELOPMENT_NOTES_v0.31.0.md)
- [0.30.0](DEVELOPMENT_NOTES_v0.30.0.md)
- [0.29.0](DEVELOPMENT_NOTES_v0.29.0.md)
- [0.28.0](DEVELOPMENT_NOTES_v0.28.0.md)
- [0.27.0](DEVELOPMENT_NOTES_v0.27.0.md)
- [0.26.0](DEVELOPMENT_NOTES_v0.26.0.md)
- [0.25.0](DEVELOPMENT_NOTES_v0.25.0.md)
- [0.24.0](DEVELOPMENT_NOTES_v0.24.0.md)
- [0.23.0](DEVELOPMENT_NOTES_v0.23.0.md)
- [0.22.0](DEVELOPMENT_NOTES_v0.22.0.md)

[Back to the project](../../README.md)
