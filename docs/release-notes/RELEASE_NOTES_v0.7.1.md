# TCHLauncher 0.7.1

## Fixed

- Fix NeoForge installation for Minecraft 1.21.11 by accepting the official `tsrg.lzma` mappings artifact extension.
- Support modern NeoForge's dedicated server entrypoint while preserving compatibility with older BootstrapLauncher profiles.
- Keep library path validation and reject malformed extensions, unsafe paths and client launch entrypoints.

## Verification

- Installed official NeoForge 21.11.45 for Minecraft 1.21.11 with verified downloads.
- Started the installed server and a copied import with Java 21. Both reached Minecraft's EULA gate without accepting the agreement or creating a world.
- Added regression coverage for compound artifact extensions and modern server launch arguments.
- All 52 local Rust tests and six telemetry tests pass; TypeScript checking and frontend production compilation pass.
- The Windows x64 NSIS installer builds successfully.

This is a targeted Windows bugfix preview. Connecting a Minecraft client and loading gameplay mods remain separate manual checks.
