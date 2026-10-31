# TCHLauncher 0.54.0

Windows x64 preview. The installer is unsigned.
Prepared for publication after the 0.52, 0.53 and 0.54 version milestones.
Only the EXE installer will be attached; checksums stay local.

## Changes since 0.51.0

- Console input preserves new edits and recalled drafts while an earlier command
  is being sent. Confirmed commands still enter session history, including when
  the history reaches its limit.
- Custom JVM validation uses the server working directory, matching startup.
  Valid options with relative file paths no longer fail because they were checked
  beside the launcher. Invalid profiles preserve the saved configuration.
- NAT-PMP discovery, opening and removal controls remain responsive during
  background status reads. Superseded observations cannot unlock an active action.
- Crash-restart notices keep automatic-update failures separate from status-read
  errors. Refreshing successfully no longer hides a missing event subscription.
- Memory charts scale to visible series so heap usage remains readable when
  resident process RAM is hidden. Broader empty Modrinth searches retain their
  query, content type and sort order.
- Scheduling identifies invalid local dates, including skipped daylight-saving
  hours. Game-mode and difficulty choices follow known version formats while
  preserving imported values until edited.
- Storage exclusions and backup-cleanup results follow the selected language.
  Recognized Windows file failures show recovery guidance alongside their
  original technical details.
- Cached skin/content images use bounded PNG validation and checked temporary
  publication. Completed skin requests can release idle registry entries while
  active requests and failure backoff remain protected.
- Shared motion avoids repeated state updates for unchanged indicator geometry
  and captures list snapshot widths in one pass. Timings and reduced-motion
  behavior remain unchanged.
- Unused ZIP password-encryption dependencies and platform icon derivatives are
  removed. Compression codecs, third-party notices and configured Windows icons
  are preserved; dependency versions are not upgraded.

## Validation and limits

Scoped ES/EN component, callback, rendering and native checks cover the changes
above. Real Java 21 and Vanilla 1.21.4 complete three isolated ready/status/command/
clean-stop sessions with an external custom runtime, persisted JVM arguments and
restored recommended settings. A separate real-Java regression checks relative
JVM output paths, rejected profiles and unchanged process-wide working directory.
Native archive cases preserve the same compressed modpacks before/after ZIP
feature reduction. Temporary probes add no retained tests or dependencies.

Remaining native interaction, physical-router cleanup, keyboard/accessibility
and broader client validation are maintainer-owned. These scoped checks do not
establish FPS, RAM or Internet-latency improvements. No native UI automation is
used. Earlier affected-host issues remain closed by maintainer confirmation.

Packaging evidence will be recorded after the installer is built.
