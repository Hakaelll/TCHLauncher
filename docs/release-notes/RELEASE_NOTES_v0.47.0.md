# TCHLauncher 0.47.0

Windows x64 preview. The installer is unsigned.
Published at the maintainer's explicit request, with only the EXE attached.
Packaged source commit: `db647c2554e0db8cdf493af278a5938c4684dc59`.

## Changes since 0.46.0

- Combined mod plans respect explicitly selected versions throughout dependency
  traversal. The order of files no longer rejects a valid plan because an
  earlier dependency independently chose another release. Selected betas remain
  selected, while real version conflicts still stop installation.
- Each task owns its progress-panel cancellation state. Starting another task
  clears the previous cancellation request/error, including late responses.
- Refresh failures after restoring a backup appear separately for properties,
  backups, EULA consent and the server library. Retrying refresh reads data again
  without repeating restoration.
- More application read errors and validation messages follow the selected
  Spanish or English language. Raw console text and executable paths are retained.

## Validation and limits

Isolated checks reproduce the dependency-ordering and cancellation-state bugs.
They exercise production graph/callback code with controlled catalog, artifact,
IPC and React-state boundaries. The native library check passes. Earlier scoped
checks cover restore refresh retries and ES/EN error rendering. TypeScript, the Windows frontend build, optimized offline NSIS packaging,
executable/setup version checks and embedded-icon verification pass.
The installer is 6,899,503 bytes.

Native Windows interaction, installer lifecycle and live Minecraft sessions are
not repeated for this version. Manual validation remains with the maintainer.
This release does not establish fixes for Internet stalls, mod-specific world
behavior, rising Java RAM or imported Minehut account identity.

No application dependency or external service is added. Only the EXE installer
is attached to the GitHub release; checksums remain local.
