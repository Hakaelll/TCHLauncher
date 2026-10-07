# Operations refinements after 0.29.0

These source changes follow the packaged Windows 0.29.0 preview. Its existing
installer does not contain them.

- Selection indicators ignore inert, hidden animation copies. Removal copies
  lose their current-selection attributes; a scoped mutation observer updates
  the indicator when live targets disappear. Both observers disconnect on cleanup.
- Console categories have visible pressed filters and counts for the searched
  in-memory view. The newest 1,000-line retention limit is unchanged.
- Paused following has an explicit jump-to-latest control. Keyboard scrolling
  pauses following without overriding native scroll keys.
- Up/Down recalls up to 50 acknowledged commands from the current server session
  and restores the unsent draft. Recalling never submits a command. Late async
  completions cannot put notices or command history on a different server.
- Transfer bars use a continuous inset rail with pixel relief, separate real
  percentage and transfer readings, and theme accents. Unknown totals remain
  indeterminate; there is no simulated percentage or recurring decorative timer.
- Removed UI_DESIGN.md, which mixed obsolete light/anthracite guidance with the
  current End design, and the obsolete unreleased summary of already shipped work.
  design.md remains authoritative. Release history, legal notices, deferred Linux
  work and implementation evidence are retained.

TypeScript and all 92 frontend tests passed, including four command/count
regressions. Production Vite compilation passed after allowing access to local
pnpm dependencies outside the sandbox. Native layout, pointer and keyboard review
remain manual; these checks do not establish rendered appearance or gameplay.
The native server engine was unchanged and its suite was not rerun.

Suggested commit: `fix: clear stale sidebar selection and refine console progress UI`

## Imported-server authentication diagnostics

The player snapshot now includes read-only Paper/Spigot configuration evidence.
Paper checks settings.bungeecord in spigot.yml and the version-appropriate
Velocity and PROXY protocol settings in paper.yml or config/paper-global.yml.
Modern year-based Minecraft versions also use the current Paper configuration.
Other loaders do not infer proxy support from these files.

Parsing uses the existing yaml-rust2 dependency with byte, event and depth limits.
Duplicate keys, aliases, unsupported types and unreadable files remain unknown.
Nested key segments are compared individually; a literal dotted key cannot
impersonate the nested setting. Only file names, setting paths, booleans and
fixed diagnostic messages cross IPC. Proxy secrets and arbitrary YAML contents
are never returned. Reads do not alter configuration or player data.

The player view separates whitelist policy from authentication configuration,
lists enabled or unknown proxy checks and explains restart requirements. A
proxy-enabled or unknown configuration no longer uses online-mode alone to pick
an account avatar from an ambiguous cache. Spanish and English copy are included.

These are saved-file observations, not confirmation of a running server's
effective configuration, account authentication, plugins/mods, JVM overrides,
proxy routing or skin downloads. The reported Minehut case remains unverified
without the host's configuration and a real reconnect. Inventories and permissions
associated with separate UUIDs remain separate.

Verification: all 94 frontend tests, TypeScript and production Vite compilation
passed. The offline native suite passed 242 tests with 28 conditional tests
ignored. All five final authentication fixtures passed separately, including the
two added checks for configuration locations and YAML work limits.
Native UI inspection and real imported-server reconnection remain pending. No
installer, commit or publication was generated for this addition.

Configuration references: [Paper global configuration](https://docs.papermc.io/paper/reference/global-configuration/)
and [player information forwarding](https://docs.papermc.io/velocity/player-information-forwarding/).

Suggested commit: `feat: diagnose imported server authentication and proxy settings`

## Process supervision history after 0.30.0

Source follow-up; the existing 0.30.0 installer is unchanged. Monitor cleanup
retains process ownership until termination and waiting confirm an exit. A
supervision failure is not a clean stop, even if the eventual exit code is zero.
The diagnostic writer and history reader now share that failure classification,
including the older single-file fallback. A later successful session preserves
the last failure. Combined monitor, forced-stop and server diagnoses stay within
the reader's six-diagnosis limit, retaining the monitor and forced-stop entries.

Verification: four process tests passed (two conditional fixtures ignored),
including real owned-child cleanup and zero-code terminal reporting; five
diagnostic persistence/rule tests passed. The combined-diagnosis limit and
historical failure after a clean session are covered. These checks do not prove
Minecraft gameplay, native window behavior or router connectivity.

Suggested commit: `fix: preserve Java monitor failures in diagnostic history`

## Bounded resilient Java output after 0.30.0

Source follow-up; the existing installer is unchanged. Both stdout and stderr
now decode malformed UTF-8 with replacement characters and continue reading.
Previously BufRead::lines stopped the pump on an invalid byte sequence, so a
later readiness message could be lost. Interrupted reads retry; valid multibyte
text, blank lines, CRLF and unterminated final lines retain their boundaries.

Each source line retains at most 64 KiB before decoding. Excess bytes are drained
until the newline, without being published as additional log records. The
retained line explicitly indicates truncation. This bounds line-buffer memory
and avoids retaining an arbitrary-size single-line payload. The bounded buffer
is reused; no timer, polling or dependency was added. Very large lines are
deliberately truncated in launcher logs as well as the UI; Minecraft's own log
files remain untouched. This is not a measured overall RAM or network improvement.

Verification: seven process tests passed with two conditional fixtures ignored,
including invalid bytes followed by a real-format parsed startup event, a 4 MiB
line followed by readiness, invalid-byte expansion, fragmented Unicode and
interrupted reads. The conditional Java 21 process diagnostic test passed
separately using the installed managed runtime: incompatible test class, stderr,
exit codes, session isolation, persisted failure and subsequent successful
process. It did not run Minecraft, accept EULA or create a world. The reported
server's startup issue still needs a real reproduction to identify its cause.

Suggested commit: `fix: keep Java output readable after malformed or oversized lines`

## Whitelist import review after 0.30.0

Source follow-up; the current installer is unchanged. The former eight-name
preview is now a searchable, paged review of every validated imported name,
with explicit new/already-allowed labels and counts. Each page contains at most
50 rows; the inset scrolling list is keyboard focusable with a visible End
accent focus outline. The review keeps its file, query and page when switching
player categories. Switching away from the server/view discards the local review.

File selection and submission use one local synchronous operation guard and
ignore late completions after disposal. Submission captures the pending names,
clears the review only after native acknowledgement, and distinguishes a later
player-list refresh failure from command-send failure. Names already present
are excluded using the existing case-insensitive rule. Missing/stale player
snapshots block submission; Minecraft still handles account/name resolution
through live whitelist commands. This does not import offline UUID mappings or
prove Minecraft accepted each queued command. Spanish/English copy and row
action translations are included; names and filenames remain verbatim.

Verification: TypeScript and production Vite compilation passed; eleven related
locale/player identity/input/paging tests passed. Final copy/focus adjustments
passed TypeScript and the four locale tests. Native file-picker, keyboard,
appearance and live whitelist acceptance remain pending manual review.

Suggested commit: `feat: improve whitelist import review and harden Java output`

## Session-bound whitelist command dispatch after 0.30.0

Source follow-up; the existing installer is unchanged. Whitelist command writes
now run in a blocking worker under the existing running-server operation
reservation, instead of a synchronous Tauri command. The worker validates the
whole bounded batch before writing and holds one process/session reference.
A changed session is rejected rather than sending remaining requests to a
replacement process. Each request checks the running state, and a stop or pipe
failure reports the count of preceding writes. Counts acknowledge pipe writes,
not Minecraft account resolution or successful whitelist additions.

Single console commands reject whitespace/slash-only input and control characters.
Exactly one leading slash is removed, preserving the additional slash used by
commands such as WorldEdit's //wand. UTF-8 text is retained and the existing
512-byte command and 1,000-player limits remain in effect. No additional service,
timer or network connection was added. Worker dispatch does not imply a timeout
for a blocked OS pipe or a measured reduction in gameplay latency.

Verification: eight process tests passed, with three conditional fixture entries
ignored. The command test starts a real owned hidden child, drains its stdout,
checks all 1,000 received requests in order plus Unicode and slash handling, and
proves invalid batches, wrong sessions and stopped states wrote no commands.
Minecraft command acceptance, native interaction and router behavior remain open.

Suggested commit: `fix: dispatch whitelist commands off the UI thread and bind them to a session`
