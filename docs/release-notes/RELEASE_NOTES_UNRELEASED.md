# Unreleased

Accumulated development through Windows 0.26.0 is packaged locally and recorded in
[release notes](RELEASE_NOTES_v0.26.0.md) and
[development notes](../development-notes/DEVELOPMENT_NOTES_v0.26.0.md).

Native visual/keyboard review, complete gameplay and physical router checks remain pending.
Linux is deferred; see [the later-work plan](../../LINUX_LATER.md).

## Source changes after 0.26.0

- Group installed mod/plugin version review, Modrinth pages and removal in contextual actions; retain visible details and console evidence. Close obsolete menus when the file hash/filter/view changes.
- Simplify world rows to Export and contextual actions while retaining all operations, confirmations and running-server restrictions. Share viewport menu handling with server actions and preserve focus across ordinary refreshes. Manual native visual/keyboard review remains pending.

- Batch console updates in a bounded buffer while preserving the newest 1,000 lines and immediate native lifecycle events.
- Avoid repeated player file reads outside the active Overview/Players sections, preserve cached reads when changing cadence, and move parsing off the command thread.
- Slow metrics observations when the window reports hidden, pause periodic player reads, refresh on return and prevent overlapping automatic reads.
- Share console diagnostic parsing across installed mod/plugin cards, retaining explicit-reference matching and chat/INFO exclusions.
- Bound retained process counters to 32 observed servers and reset CPU baselines for each launch session, including reused PIDs.
- Collect only required process and system memory counters.
- Stop routine router maintenance from repeatedly opening TCP connections to Minecraft; keep explicit opening validation, ownership checks, renewal and cleanup.
- All 84 frontend and 232 native offline tests passed; TypeScript and production compilation passed. The internet-stutter cause and real-session RAM/latency improvements remain unverified.
- These changes are not included in the existing 0.26.0 installer. No additional package was generated.

See [performance evidence](../development-notes/PERFORMANCE_AFTER_0.26.0.md).
