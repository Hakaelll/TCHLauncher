# Performance work after 0.26.0

## Scope

Reduce launcher work during gameplay without adding dependencies, relays, services or changing Minecraft network/JVM settings blindly. The user reports internet-only connection stutters without a visible CPU/RAM spike. No trace from that remote host is available, so the actual cause remains undetermined.

## Changes

- Buffer console events in a fixed 1,000-entry ring and publish at most once per 100 ms during a continuous burst. Preserve ordering and the same newest 1,000-line limit. Reset on server selection, reject late callbacks and release pending references on disposal. Native server state/startup events are still delivered immediately; on-disk logs remain unchanged.
- Poll full player files only while a running/starting/stopping server's Overview or Players view is visible. Preserve loaded data when changing polling cadence, allow explicit refresh and refresh on terminal process events.
- Use a single non-overlapping metrics observation loop: every three seconds while visible, every fifteen seconds when the document reports hidden, and an immediate refresh on return. Pause automatic player reads while hidden; retain manual refresh, server execution, scheduled tasks and router maintenance. Avoid duplicate initial metrics reads and skip automatic player reads during an existing read.
- Parse console severity and bounded continuation blocks once per log snapshot for the installed-content inventory, then match explicit file/ID/name references against only qualifying diagnostics. Index audit checks by filename rather than repeatedly searching them. No global cache or additional retained log history is introduced; visual states and matching rules remain unchanged.
- Run player file parsing on the blocking worker pool instead of the Tauri command thread.
- Replace the process-wide PID history and shared sysinfo process table with a session-scoped collector. Each entry observes only its current PID, replaces its counters when either launch token or PID changes, and is removed when stopped metrics are read. A 32-server LRU bound evicts older observations. The first CPU sample after a restart or eviction remains unavailable until a baseline exists, including when Windows reuses a PID. This bounds retained process observations, not Minecraft heap or all launcher memory.
- Refresh only CPU, memory and disk counters for the selected process. Read system RAM without constructing a full process inventory.
- Separate routing/configuration inspection from TCP probing. UPnP/NAT-PMP maintenance verifies the running process and configured target without connecting to Minecraft every two seconds. Opening a mapping still validates the listener; gateway ownership, lease renewal and cleanup remain enforced.

## Verification

All 84 frontend tests and 232 native offline tests passed; 28 condition-dependent native tests were ignored. TypeScript and production Vite compilation passed. The sandbox blocked Tauri resource compilation and pnpm dependency resolution; the bounded offline build/test commands passed outside it. No user runtime or system network setting was changed.

Two visibility regressions verify hidden cadence, foreground refresh, in-flight exclusivity, disposal and late callbacks. Native minimized/tray visibility signaling and actual RAM/ping effects remain unverified.

Two new log-buffer regressions verify 20,000-line bursts, wraparound/order, subsequent batches, reselection, late timers and disposal. The network regression uses a real local TCP listener and verifies twenty configuration/maintenance target reads accept no connections. Existing router renewal/ownership/cleanup and player-file tests remain passing.

A one-burst synthetic comparison on this machine retained identical output:

| Scenario | Old array updates | Batched publications | Buffer processing time |
| --- | --- | --- | --- |
| 20,000 fixture log lines | 20,000 | 1 | Old 41.49 ms; batched 1.34 ms |

This measures buffer operations, not a React-render, gameplay, resident RAM or internet-ping benchmark. Entries arriving across separate 100 ms windows produce separate publications. The result must not be presented as a measured RAM reduction or a fix for the reported connection stutters.

An additional synthetic inventory comparison used 100 installed files and 1,000 log rows (50 error rows, the rest INFO), ten warm-up runs and thirty measured runs per implementation. The captured previous implementation averaged 25.35 ms per inventory analysis; shared parsing averaged 1.04 ms and returned identical evidence. This is a JavaScript diagnostic-analysis benchmark, not native WebView rendering, resident memory or gameplay latency. Two additional regressions cover multi-file associations, misleading chat/INFO, reusable snapshots and new-session isolation.

Two native collector regressions use a real current-process sample to verify baseline reset with the same PID and a new launch token, stopped-observation removal, and a 1,000-restart / 32-entry eviction scenario. This proves bounded counter retention, not a measured reduction in total application RSS.

## Remaining work

Measure the real native launcher and server together during a representative session, verify console/player interactions manually, and capture evidence during an internet stutter. Direct Minecraft traffic does not pass through TCHLauncher. Physical router, upload congestion, Wi-Fi/link behavior and Java pauses have not been ruled out by slow CPU/RAM samples.

These changes are source-only. The existing 0.26.0 installer does not include them. No new release, installer, commit, push or publication was produced in this iteration.
