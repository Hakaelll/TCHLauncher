# TCHLauncher 0.43.0

Local Windows development build. This version is not published on GitHub;
0.40.0 remains the published preview.

## Console and scheduled tasks

- While following is paused, the console shows newly arrived lines retained in
  the current filtered view. Disk history and other servers do not inflate it.
- One-off scheduled tasks show Failed, Cancelled or Interrupted when appropriate,
  instead of the generic Completed state. Recurring and running states retain
  their existing priorities.

## Content discovery and inventory

- Disabled files use pages of 24 with Spanish/English navigation, filter resets
  and page clamping when the inventory shrinks.
- Modrinth discovery loads icons through four page workers and retains completed
  icons when reopening that page. Leaving stops new scheduling and discards late
  replies; requests already in progress can finish.

## Window, connection and names

- Restoring the main window from the tray or recovery paths first restores it
  when minimized, then requests focus.
- Public-IP and clipboard replies from a previous server no longer change the
  current connection panel. Repeated copies replace the old confirmation timer;
  duplicate public-IP requests are guarded.
- Native display-name validation matches HTML UTF-16 length limits. Accented and
  CJK names no longer hit a 64-byte limit before reaching the form's limit.
- Task snapshots are checked every three seconds while visible and fifteen while
  hidden, with a refresh on return. Live progress events stay enabled.

## Verification and limits

Focused checks cover production callbacks/helpers, ES/EN markup, queue scheduling,
late replies, name limits and task-state precedence. Native inspection and SQLite
imports also pass eight copy/manage fixture cases for renamed Paper/Fabric,
legacy Vanilla and explicitly confirmed unknown launchers, with exact source and
copied file equality and no script execution. Those fixtures contain metadata
JARs, not executable Minecraft servers.

The Windows packaging record covers the TypeScript/frontend build, optimized
native build, executable/setup versions and application icons. Remaining native
interaction, assistive-technology behavior and affected-host gameplay checks
belong to the maintainer. This build does not establish that reported Internet
stalls, RAM growth, missing mod behavior or Minehut account identity are resolved.
No measured RAM, CPU or network improvement is claimed.

Silent installation of 0.40.0, upgrade to 0.43.0 and uninstallation of 0.43.0
also passed in an isolated project folder. A user-owned fixture survived both
upgrade and uninstall. The original executable/database hashes and exported
registry keys were unchanged after restoration. The application was not launched;
interactive installer choices and gameplay were not exercised by this probe.

No external service or dependency was added. Temporary probes and generated build
caches are removed after packaging; checksums stay local and are not release assets.
