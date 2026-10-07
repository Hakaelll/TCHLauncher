# TCHLauncher 0.28.0

## Mods and plugins

- Disable a mod or plugin from its inventory actions without deleting its files
  or configuration. Find and enable it again in the new **Disabled** view.
- Each change saves a server backup and runs only while the server is stopped.
  Real copy progress and cancellation are available before final publication.
- Reject destination collisions and stale files. Check known managed dependencies
  and supported Fabric declarations before applying the prepared change.
- Keep disabled files out of the active inventory and prevent an update from
  silently installing a second copy of a disabled managed project.

## Interface

- Separate file details, declared dependencies and related console messages in
  the mod inspector. Open related messages directly from **View warning**.
- Refine the inventory view controls, square status icons and error framing while
  keeping the Minecraft/End style, themes and motion preferences.

## Verification

239 native offline tests and 84 frontend regressions passed. TypeScript passed.
Native visual/keyboard review and complete Minecraft gameplay remain pending.
Dependency checks are limited by available metadata and do not prove that a mod
works in a world or cover every loader candidate selection and local plugin rule.
No measured improvement to real internet latency is claimed.

## Windows package

Installer: `TCHLauncher_0.28.0_x64-setup.exe` (6.36 MiB). File/product versions,
embedded logo and bundled license resources were checked. Generated locally,
not installed or published. Linux remains deferred.

SHA-256: `1878b08e590cb22f002f729b5506b46feea48f881f184fffb621caa10c3d8d0b`.
