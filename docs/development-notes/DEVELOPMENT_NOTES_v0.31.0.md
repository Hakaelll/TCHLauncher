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
