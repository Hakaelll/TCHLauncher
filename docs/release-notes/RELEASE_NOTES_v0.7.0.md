# TCHLauncher 0.7.0

## Mods and plugins

- Add a Modrinth browser with project artwork, search, pagination and compatible version selection.
- Keep Fabric/NeoForge mods and Paper plugins in separate server sections and directories; support Bukkit/Spigot plugin releases compatible with Paper.
- Review selected versions, required dependencies, optional dependencies, declared conflicts and available changelogs before installation.
- Resolve transitive dependencies without loops, reject conflicting pinned versions and validate the Minecraft version, loader and server environment.
- List real installed JARs and identify files added or changed outside the launcher.
- Verify downloads with SHA-512, expected sizes and platform-specific JAR descriptors before activation.
- Install through a staged directory switch; reject stale reviews and recover interrupted switches before starting the server.
- Review individual or combined compatible updates and create a server backup before replacing installed content.
- Require confirmation before removing content, preserve configuration directories and save a backup first.
- Preserve explicit beta/alpha selections when a bulk update would otherwise downgrade them.

## Desktop refinement

- Replace stacked navigation with one collapsible server sidebar and horizontal server sections.
- Use the supplied emblem, remove the repeated overview banner and keep restrained semantic and metric color accents.
- Retain server image shortcuts, names on hover, pin/delete context actions and reduced-motion support.

## NeoForge and Java

- Add stable NeoForge releases, verified official libraries and offline installer preparation with cached Mojang mappings.
- Launch Java directly with validated module/classpath arguments and preserve the actual server directory for worlds and configuration.
- Normalize Windows paths for Java class loading and rebase libraries when importing a copied server.
- Stop the installer process on cancellation or timeout.

## EULA

- Request explicit acceptance before the first download and remember the user's decision locally for subsequent servers and starts.

## Verification and remaining work

- TypeScript checking, frontend production compilation, 48 local Rust tests and six telemetry tests pass.
- The real Modrinth search and icon download test passes; the Windows x64 NSIS installer builds successfully.
- Live tests install and update Lithium for Fabric 1.21.4, preserve existing user files, create backups and remove content with recovery available.
- A live test installs LuckPerms into a Paper server's plugins directory without mixing in mods.
- Local tests cover compatibility, dependency cycles/conflicts, SHA-512 downloads, stale reviews and interrupted directory recovery.
- The integrations use isolated test servers and do not execute downloaded mods/plugins or accept the Minecraft EULA automatically.
- Native visual review and Minecraft client connections remain manual. Linux integration, modpack import and the remaining application features are still in development.
