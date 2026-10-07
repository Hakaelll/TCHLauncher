# TCHLauncher

<img src="src/assets/launcher-logo.png" width="88" height="88" alt="TCHLauncher emblem">

**Your Minecraft servers. Your computer. One desktop workspace.**

TCHLauncher is a Windows desktop manager for local Minecraft Java servers. Create or import a server, manage its worlds and content, watch the console and performance, and keep backups in one Minecraft-inspired workspace.

Built with **Tauri 2, Rust, React, TypeScript and SQLite**.

[Downloads](https://github.com/Hakaelll/TCHLauncher/releases) · [Release notes](docs/release-notes/README.md) · [Roadmap](NEXT_TODOS.md) · [Security policy](SECURITY.md)

## Project status

Latest development milestone: **0.30.0** — [changes and verification](docs/release-notes/RELEASE_NOTES_v0.30.0.md).

TCHLauncher is a **Windows preview** under active development. Features below are implemented in the application, with automated checks where documented. Native visual and keyboard review, complete gameplay scenarios and physical router tests remain outstanding. See the [roadmap](NEXT_TODOS.md) for the verification boundaries.

Linux is deferred in [LINUX_LATER.md](LINUX_LATER.md). Minecraft client launching and Microsoft account integration are outside the current scope.

## Features

| Area | Available functionality |
| --- | --- |
| Servers | Vanilla, Paper, Fabric and NeoForge; create a server or import an existing folder by copy or in place |
| Setup | Version selection, world seed, game settings and resources before first launch; managed Eclipse Temurin runtimes |
| Process control | Startup tracking, console commands, graceful shutdown and a separately confirmed forced stop |
| Worlds | Import, duplicate, select, delete and export; share a world and its mods or plugins in one ZIP |
| Content | Modrinth discovery for mods/plugins, modpacks, resource packs and shaders; compatibility labels, installed and disabled inventories, reversible enable/disable actions and log-related warnings |
| Players | Skin heads when a verified profile is available; whitelist, operator and ban management |
| Console | Searchable, colored log labels, commands and log export |
| Performance | Simultaneous process RAM and CPU charts; supported JVM heap data and Paper metrics where available |
| Backups | Manual and scheduled copies, retention settings, restore and optional copies after clean shutdown |
| Networking | Local connection details, automatic router discovery and temporary UPnP or NAT-PMP mappings when supported |
| Appearance | End, Obsidian and Nether themes; Minecraft-inspired typography, motion preferences, Spanish and English |

Resource packs and shaders are **client content**. Listing them in discovery does not install them as server mods. Modpacks use the `.mrpack` review and import flow to create an instance. Metadata and log checks help identify problems; they cannot guarantee that every mod works inside a world.

## Getting started on Windows

1. Download a Windows installer from [Releases](https://github.com/Hakaelll/TCHLauncher/releases), when available, and install it.
2. Choose **New server** or **Import server**. Keep a separate copy of an existing server before importing it in place.
3. Choose the Minecraft version, server type, world and resource limits.
4. Read and explicitly accept the Minecraft EULA at the first download. The application remembers that acceptance for subsequent setup.
5. Let TCHLauncher install its managed Java runtime and server files, then start the server.
6. Use the local address for players on your network. Internet access requires a suitable router configuration and firewall; an accepted port mapping alone does not verify outside connectivity.

Managed Java is stored in the application library. It does not replace your system Java installation. Custom Java can be selected explicitly in server settings.

### Export a world with its content

Stop the server, open **Worlds**, select **Export**, and choose **World and mods** or **World and plugins**. You can include `config` and `defaultconfigs`. Choose a ZIP destination outside the server folder.

The archive contains the selected world, server content, an English restoration guide and a manifest with file hashes. Paper exports also include matching Nether and End world folders when present. Disabled content remains disabled. Plugin folders retain their own configurations.

This is a sharing export, not a complete server backup: Java, launcher startup settings, `server.properties`, EULA files and logs are excluded. World player data and selected configurations are included, so review the archive before sharing it. Use **Backups** for a full server copy.

### Change the language

Open **Settings → General → Application language** and choose **Español** or **English**. The choice applies immediately and is remembered on the computer.

Server names, world names, paths, commands, Minecraft logs and third-party descriptions retain their original text. Unrecognized diagnostics also retain their original wording.

## Development

### Requirements

- Windows x64 with Microsoft Edge WebView2.
- Rust with the MSVC toolchain and Visual Studio C++ build tools.
- Node.js 24 and pnpm.
- The [Tauri Windows prerequisites](https://v2.tauri.app/start/prerequisites/).

```sh
pnpm install --frozen-lockfile
pnpm tauri dev
```

### Checks and packaging

```sh
pnpm test
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml --offline --lib
pnpm tauri build --bundles nsis -- --offline
```

Offline Cargo commands require dependencies to have been fetched previously. Some integration tests are ignored by default because they require downloads, managed Java, external services or explicit test fixtures; their setup is documented beside the tests and in [fixture documentation](src-tauri/tests/fixtures/README.md). Automated checks do not accept the Minecraft EULA on the user's behalf.

NSIS installers are generated under `src-tauri/target/release/bundle/nsis/`. Build outputs are ignored by Git; generating an installer locally does not publish a GitHub release.

To check the embedded application and installer icons from PowerShell:

```powershell
./scripts/verify_windows_icons.ps1 -Installer 'C:/path/to/TCHLauncher_VERSION_x64-setup.exe'
```

### Data and trust boundaries

Server files and local application data remain on your computer. Destructive file operations require the server to be stopped and use per-server operation reservations. Importing in place uses your existing folder directly.

Published download hashes are checked when supplied. Mods, plugins, custom Java and Java agents execute code with the permissions of your user account: TCHLauncher is not a sandbox for them.

Router discovery does not change the firewall or bypass CGNAT. Opening a temporary mapping requires an explicit action. Bore and Playit are not integrated.

Process RAM is distinct from JVM heap usage. Rising process RAM alone does not establish a memory leak. See [SECURITY.md](SECURITY.md) for other limits and reporting guidance.

## Documentation

| Document | Purpose |
| --- | --- |
| [Architecture](ARCHITECTURE.md) | Native services, data flow and implementation decisions |
| [Visual system](design.md) | Minecraft/End aesthetic, tokens and interaction guidance |
| [Roadmap and verification](NEXT_TODOS.md) | Remaining work and known verification limits |
| [Release notes](docs/release-notes/README.md) | Versioned user-facing changes |
| [Development notes](docs/development-notes/README.md) | Engineering records and test evidence |
| [Security](SECURITY.md) | Trust boundaries and vulnerability reporting |
| [Third-party notices](THIRD_PARTY_NOTICES.md) | Dependency licenses and artwork provenance |
| [Legal inventory](docs/legal/README.md) | Collected license texts and their sources |

## Contributing and bug reports

Include the application version, Windows version, Minecraft version, server type, reproduction steps and relevant **redacted** logs. Mention whether the server was created or imported and which mods or plugins are involved. Never post credentials or private player data.

Preserve the documented file and process safeguards, add meaningful regression checks for behavioral changes, and use English commit messages and release notes. Please discuss large changes before implementing them.

## License and acknowledgements

TCHLauncher source code is licensed under the [MIT License](LICENSE). Third-party dependencies and assets retain their own licenses and notices; see [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and the [collected license texts](docs/legal/THIRD_PARTY_LICENSES.txt).

Monocraft is distributed unmodified under the SIL Open Font License 1.1. Artwork provenance is documented separately. Downloaded Minecraft, Java and community content have their own terms.

TCHLauncher is an independent project, not an official Minecraft product and not approved by or associated with Mojang or Microsoft. Minecraft server use remains subject to its applicable EULA.
