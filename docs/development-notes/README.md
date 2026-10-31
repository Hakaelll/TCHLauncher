# Development notes

Engineering records, test evidence and pending native checks.

## Unreleased changes after 0.48.0

Startup preferences now translate both cached-data notices after a read failure.
The English strings already existed; two JSX branches bypassed `t`. A focused
render check reproduces both Spanish leaks before the fix and passes afterward
across English/Spanish switching and all four cached-data combinations. TypeScript
and the Windows frontend build pass. This checks actual component markup with
controlled read snapshots, not native startup or IPC. No retained test is added.
The published 0.48.0 installer predates this two-line fix.

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
