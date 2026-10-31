# Development notes

Engineering records, test evidence and pending native checks.

## Source after local 0.45.0

The English catalog adds 726 application-owned native messages across six passes:
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
application errors in eight reviewed wrappers now have explicit argument metadata.
The old helper reproduces untranslated nested bodies for attempt failures, content
migration, retention, recovery, catalog lookup, dependency review and partial player
commands. Only application-error arguments are translated, to a maximum depth of
eight; project names, destination lists and raw NeoForge installer output remain
data. The general translator retains its lookup priority and explicit-argument
behavior. Three existing messages move into the native catalog without changing
their values or reverse aliases.

Actual locale checks cover nested messages, ES/EN switching, opaque details,
bounded depth and compatibility with the existing catalog. Joined error lists and
copy outside the initial inventory still need review. This source postdates the
local 0.45 installer. No version or release is generated for these changes.

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
