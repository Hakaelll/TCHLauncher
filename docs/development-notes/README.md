# Development notes

Engineering records, test evidence and pending native checks.

## Source after the local 0.42.0 installer

These commits are in source and are not included in the local 0.42.0 installer.
They do not represent a new published release. Remaining native interaction and
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
not expanded. Packaged changes through 0.42.0 are in its
[release notes](../release-notes/RELEASE_NOTES_v0.42.0.md).

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
