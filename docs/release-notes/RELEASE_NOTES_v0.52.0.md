# TCHLauncher 0.52.0

Windows x64 preview, prepared as a local build. The installer is unsigned.
This version is not a GitHub release; the published preview remains 0.51.0.

## Changes since 0.51.0

- Task scheduling rejects local dates and hours that would silently shift through
  JavaScript normalization, including skipped daylight-saving hours. Date and
  interval errors identify and focus the affected field. The form also checks
  the native scheduling range before submission.
- General, server and failed-task notices give specific recovery guidance for
  recognized Windows disk-space, access and file-use errors. Original technical
  details remain intact and unknown messages retain their previous guidance.
- Skin-head and content-icon caches validate bounded PNG data, reject redirected
  folders and non-regular entries, and publish complete images through temporary
  files instead of writing through the destination.
- Completed skin requests no longer leave a permanent in-memory entry for every
  consulted UUID. Active requests and the five-minute failure backoff are retained.
- Game-mode and difficulty choices follow the known server version's numeric or
  named format when edited. Imported values remain unchanged until edited;
  unknown version IDs keep the existing representation fallback.

## Validation and limits

Scoped ES/EN component, callback and React SSR checks cover scheduling, failure
advice and gameplay-property choices. Native temporary cases reproduce the prior
redirected/invalid-image reads and skin-request entry accumulation, then check
PNG limits, complete concurrent writes, rejected paths, active-lock identity and
failure-backoff preservation. Five new native cache messages are checked in both
languages. Temporary probes add no retained tests or dependencies.

These checks do not automate native UI, force OS failures, contact profile/icon
providers, run new Minecraft sessions, measure RAM/latency or establish protection
against adversarial concurrent filesystem changes. Disk-cache eviction is not
added and Minecraft heap settings are unchanged. Remaining manual validation is
maintainer-owned and does not block independent work.

Packaging evidence will be recorded after the installer is built.
