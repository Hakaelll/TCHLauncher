# TCHLauncher 0.25.0

## Content discovery and inventory

- Browse distinct Mods/Plugins, Modpacks, Textures and Shaders divisions in Modrinth, with search, categories, ordering and Minecraft-version filtering.
- Explore client content without applying the server loader filter. Published game-version metadata is distinguished from verified server compatibility.
- Open official publication pages in the default Windows browser. Import downloaded .mrpack files through the existing review flow to create a new server. Texture/shader client installation is not included.
- Refine installed item cards with larger icon slots, clearer version metadata and grouped actions. Preserve compact layout, filters, diagnostic evidence and console navigation.
- Add brief theme-aware category transitions that respect reduced motion. Hide stale results immediately when changing content type.

## Creation and configuration

- Remove the redundant “Next step” copy from the creation footer.
- Preserve unchanged imported world seed escapes when editing unrelated settings.

## Fabric diagnostics

- Review recognized required version ranges against installed candidates and show declared breaks/conflicts between main mods. Unknown syntax, incomplete metadata and dependency overrides do not become negative compatibility evidence.
- This review does not replace Fabric Loader's solver or verify built-in runtime/Minecraft/loader constraints and nested candidate selection. It does not prove mod behavior inside a world.

## Verification

- 225 native offline tests and all 74 frontend tests passed. TypeScript and the production frontend build passed.
- Real Modrinth metadata searches passed for modpacks, resource packs and shaders; no content archives were downloaded.
- Seed decoding was independently verified with Java 8, 21 and 25; no Minecraft process or world was created.
- Native visual/keyboard/browser-dispatch review, gameplay and physical router verification remain pending. Linux remains deferred.
- No user server was started, no EULA accepted, and no commit, push or publication performed.

## Windows package

- Installer: `TCHLauncher_0.25.0_x64-setup.exe` (6,514,214 bytes).
- Executable and installer file/product versions: 0.25.0. Embedded icons match the supplied logo.
- SHA-256: `ada058c56c1e1daaefef3a90bf7f223764e75befbf2b1118fff81b5bb497cd9f`.
- Built locally with NSIS x64; not installed during verification.
