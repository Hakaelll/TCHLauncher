# TCHLauncher 0.44.0

[Windows x64 preview](https://github.com/Hakaelll/TCHLauncher/releases/tag/v0.44.0).
The installer is unsigned and is the only attached release asset; checksums remain local.
Packaged source commit: `2dfe13f0a7c83a796539d0c5da394326e362327e`.

## Performance views and backup history

- CPU charts use a labeled automatic percentage axis so small changes remain
  readable. A control restores the fixed 0–100% scale. Readings retain their real
  percentage values.
- Backup history uses pages of 50. Search and reason filters cover the complete
  list, reset the page and preserve the original selected backup when restoring.
- Automatic folder-size measurements use a five-minute success cache with an
  active server process and a one-minute cache while stopped. Errors retry after
  fifteen seconds; invalidation and path changes force a read. Manual detailed
  inspection stays available. The meter shows the observation time and cadence.
- Quiet content logs reuse an immutable empty diagnostic result and skip per-mod
  pattern construction. Real error associations and static file warnings remain.

## Imported settings and Unicode worlds

- Import preview and configuration share their supported properties reader.
  Comments beginning with `!` no longer appear as editable properties. Duplicate
  keys use their last effective value for preview validation.
- Saving leaves unchanged values, escapes and duplicate entries alone. A no-op
  preparation returns the original content verbatim; editing a CRLF file retains
  Windows line endings. Changed invalid values are still rejected.
- Escaped Unicode world names are decoded when identifying the active world and
  encoded when changing it. Native fixture checks cover accented/CJK folder names,
  safety backups and repeated selection without duplicate backups. Unrelated seed
  and MOTD values remain intact.

## Verification and remaining checks

Focused production-helper/callback checks and Spanish/English component markup
cover chart scaling, backup pagination, cached folder-size observations and quiet
content diagnostics. Native fixtures cover property parsing/preparation, private
preview fields, invalid changed values, Unicode world selection and backup bytes.
The Windows packaging record covers TypeScript/frontend and optimized native
builds, executable/setup versions and application icons.

Fixtures and isolated component checks do not establish native keyboard/layout
behavior or Minecraft gameplay. Remaining manual validation belongs to the
maintainer. This release does not claim that Internet stalls, RAM growth, missing
mod behavior or imported Minehut account identity are resolved. No runtime CPU,
RAM or network benchmark improvement is claimed, and complete Java-properties
grammar support is not claimed.

No dependency or external service was added. Linux and client launching remain
outside the current Windows scope. Temporary probes and generated build caches
are removed after packaging.
