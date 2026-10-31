# TCHLauncher 0.55.0

Local Windows x64 preview. This milestone is not published on GitHub.
Packaging verification is pending. The installer will be unsigned.

## Changes since 0.54.0

- World inventory supports accent-insensitive search and pages of 50 worlds.
  Paging preserves world identities and action drafts, closes obsolete menus and
  returns focus to the result count. File operations keep paging/search locked.
- Minecraft and loader update panels recover existing progress from the shared
  task stream when settings opens. Modpack imports use the same source, retain
  installed outcomes and retry library refreshes without repeating installation.
- Download cancellation rejects duplicate requests before the button updates.
  Declined or failed requests remain retryable.
- Active UPnP openings offer a copyable external address with selectable fallback
  when clipboard access fails. Pending, closing, attention and expired mappings
  do not offer a shareable endpoint. Router confirmation does not prove Internet
  reachability.
- Scheduled-task creation atomically enforces the 100-task limit per server.
  Automatic-event warnings stay visible after successful changes or manual reads
  when the event subscription remains unavailable.

## Validation and limits

Scoped component/effect/callback checks cover shared task recovery, acknowledged
installation recovery, search/paging, cancellation, copy feedback and scheduler
notices. A native probe compiles the actual scheduler module against temporary
SQLite with a minimal storage adapter: 32 concurrent requests from 99 tasks now
save exactly 100, where the baseline saved 131. Independent-server capacity,
removal/re-addition, validation and task metadata also pass.

These checks do not establish rendered visual quality, native Windows interaction,
physical-router behavior or a new Minecraft client session. Existing End themes,
tokens, artwork and navigation remain in use. No new dependencies or retained
tests are added. Remaining manual checks stay maintainer-owned and nonblocking.
