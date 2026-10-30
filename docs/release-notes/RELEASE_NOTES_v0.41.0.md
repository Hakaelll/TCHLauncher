# TCHLauncher 0.41.0

## Player and capacity details

- Authentication guidance now flags local-type UUIDs observed among connected
  players when the saved configuration indicates direct authentication. Historical
  known players do not trigger this warning. UUID shape is a diagnostic hint;
  it does not prove authentication or repair an account's inventory or skin.
- Creation, modpack import and server settings show an advisory starting maximum
  heap based on local physical memory, server type and configured player slots.
  Recommendations leave a host reserve and never change settings automatically.
  They are starting points, not measured requirements or memory-leak fixes.
- Player and IP bans show labeled reason, source, original date and expiry details.
  Permanent and unrecorded expiry values are distinguished. User-entered reasons
  and original date strings remain unchanged.

## Windows packaging

- The desktop build generates only the Rust library used by its executable.
  Unused C-compatible static and dynamic library outputs are no longer requested.
  This removes unnecessary build artifacts; no installer-size, runtime-memory or
  build-speed reduction is claimed.
- The accepted Minecraft/End visual design remains in place.

## Verification and limits

Focused rule and ES/EN component checks passed for memory guidance, session
identity warnings and ban details. These checks do not establish results on the
affected player's remote server or replace native interaction and gameplay.

This is an unsigned Windows x64 preview. Linux and Minecraft client launching
remain deferred. Only the installer is attached to the GitHub release; checksum
files remain local.

TypeScript, the Windows frontend production build, the five retained frontend
data-lifecycle tests and the optimized native NSIS build passed. The generated
application library artifacts are rlib/rmeta, with no C-compatible library output.
Executable and installer report version 0.41.0 and their embedded icons match the
application emblem. The installer is 6,867,051 bytes. Native interaction and
gameplay were not exercised for this build.
