# TCHLauncher v0.3.0

This release improves how local servers are presented and managed.

## What's new

- The sidebar can be expanded or collapsed. In its compact mode it shows each server's image and status, with the name available on hover or keyboard focus. The preference is saved.
- A more visual overview shows server status, live metrics, resource usage, local address, recent activity, and backups.
- Players and Settings have clearer layouts that give the numbers and server image more space.
- The console now shows time, level, and message, with search, filters, output follow, visible-line copying, and full-log export.
- Short transitions and animated status feedback during installation, startup, and shutdown respect the reduced-motion preference.
- Log loading and message classification were corrected.

## Verification

- The frontend built and the automated Rust tests passed.
- Desktop visual review and installation of the Windows package remain for manual verification.

## Known limitations

- The Windows installer is not digitally signed.
- Java and server downloads require internet. Servers and their data run and remain local.
- Fabric, NeoForge, mods, plugins, world and modpack imports, public networking, and scheduled tasks were still pending in this version.
