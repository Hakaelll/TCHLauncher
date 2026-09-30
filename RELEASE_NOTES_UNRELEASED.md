# TCHLauncher — Unreleased

Changes through the content manager are included in [0.7.0](RELEASE_NOTES_v0.7.0.md).

The NeoForge Minecraft 1.21.11 installation and startup fix is included in [0.7.1](RELEASE_NOTES_v0.7.1.md).

## In development: modpack import

- Add isolated `.mrpack` file preparation with server overrides, optional-file selection, protected server files, SHA-1/SHA-512 verification and cancellation cleanup.
- Restrict download and redirect hosts and enforce the package's declared file sizes during streaming.
- Support exact Fabric and NeoForge loader requests instead of silently substituting the latest version.
- Verify a real Lithium package download and a Fabric 0.16.10 installation/startup to Minecraft's EULA gate without accepting it.

These backend changes are not included in the 0.7.1 installer. The import wizard, instance publication and interrupted-import recovery are still pending; modpack import is not yet an available application feature.

Remaining development includes modpack import, loader updates, scheduling, public-network features and end-to-end native/Linux verification. These are not shipped as completed features.
