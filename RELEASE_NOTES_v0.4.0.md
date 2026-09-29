# TCHLauncher v0.4.0

This release adds a guided setup flow and local world management, while refining the server library and settings.

## What's new

- Create a server through five focused steps: identity, Minecraft version, world, capacity, and final review. The live summary stays visible throughout setup.
- Start with a new world or copy an existing Minecraft Java world into the server without changing its source folder.
- Manage worlds from a dedicated tab: import, duplicate, export, open the folder, choose the active world, or delete a secondary world. Changing the active world or deleting a world first creates a server backup.
- Settings now groups server properties by purpose, with clearer descriptions and controls. A server can be removed from Settings after a confirmation that explains which files will be deleted. Imported folders registered in place remain on disk.
- The server library no longer scrolls sideways when the sidebar is expanded or collapsed. The unnecessary icon above the sidebar has been removed, and stray scrollbars in navigation have been hidden.

## Reliability and verification

- File operations require a stopped server. World imports validate `level.dat`, and unsafe names and symbolic links are rejected.
- Frontend build and Rust tests passed. The Windows installer was generated locally; installation and visual review on a separate machine remain to be done.

## Known limitations

- The Windows installer is not digitally signed.
- Fabric, NeoForge, mod and plugin management, modpacks, public networking, and scheduled tasks are still planned.
