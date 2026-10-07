# TCHLauncher 0.30.0

## Console and task progress

- Filter log categories with visible controls, colors and searched-view counts.
- Pause following and explicitly jump back to the latest lines.
- Recall the last 50 acknowledged commands with Up/Down; restore the unsent draft
  when leaving history. Recalling a command never sends it.
- Clear the view without depending on server clocks or timestamp formats. New
  live lines remain visible, and reloading history preserves occurrence counts
  without resurrecting hidden disk history. Showing history restores the view.
- Replace densely segmented transfer bars with a continuous pixel rail and
  separate actual percentage, bytes and speed. Unknown totals stay indeterminate.
- Remove the stale sidebar selection marker after deleting the final server.

## Minecraft identity

- Show saved authentication configuration separately from whitelist policy.
- Detect enabled BungeeCord, Velocity and PROXY protocol settings in the relevant
  Paper/Spigot files, including old Paper layouts.
- Identify unreadable or ambiguous settings without treating them as disabled.
- List the files and values to review, explain restart requirements and retain
  separate UUID inventories and permissions. Proxy secrets are not exposed.
- Preserve conservative avatar selection when proxy configuration is unknown or
  an imported cache has multiple identities for a name.

## Documentation

Remove the obsolete duplicate visual specification and the unreleased summary
of previously shipped changes. Keep current design, release history and legal
documentation.

## Verification and limits

98 frontend regressions, TypeScript and production Vite compilation passed.
The native suite passed 242 tests with 28 conditional tests ignored. All five
final authentication fixtures passed separately, including the two added later.

Native visual/keyboard review and real Minecraft gameplay remain pending. Saved
configuration is not proof of the running server's effective authentication.
The reported Minehut account issue and internet stutters still need evidence
from their real host. Linux remains deferred.

## Windows package

Packaging verification pending. This document does not yet confirm an installer.
