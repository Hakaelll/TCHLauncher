# Third-party notices


TCHLauncher's own source code is licensed under the [MIT License](LICENSE).
Third-party code and assets retain their respective licenses and copyright notices.

## Software dependencies and installer

The [license inventory](docs/legal/dependency-licenses.json) records locked Windows
Cargo dependencies, including build and transitive packages, frontend runtime
packages, Monocraft and NSIS. Full upstream notices are collected in
[THIRD_PARTY_LICENSES.txt](docs/legal/THIRD_PARTY_LICENSES.txt) and bundled with the
Windows application under `licenses/`. The inventory can include packages that are
not embedded in the final executable.

Unmodified dependencies under MPL-2.0 retain their source and notices. Exact package
source archive links are provided in the inventory. Missing package-local texts are
supplemented with documented upstream or canonical SPDX copies; see
[the inventory methodology](docs/legal/README.md). This is not an independent
license compatibility or security audit.

NSIS installer notices are preserved from its upstream `COPYING` file. Windows,
WebView2 and toolchain components retain their own terms.

## Downloaded software and content

Eclipse Temurin runtimes, Minecraft servers, loaders, mods, plugins, resource packs
and shaders are downloaded separately when requested. Their own licenses and terms
apply. TCHLauncher does not relicense them. Minecraft server downloads require the
user's explicit EULA acceptance. Review each content project's redistribution
terms before sharing a world/content export, and review configurations for secrets.

## Monocraft

- Version: 4.2.1, regular TTF, distributed unmodified.
- Author: Idrees Hassan.
- Source: https://github.com/IdreesInc/Monocraft/releases/tag/v4.2.1
- Copyright (c) 2022, Idrees Hassan.
- License: SIL Open Font License 1.1. Full license bundled in
  `public/fonts/OFL-Monocraft.txt` and copied into the application assets.

Monocraft is a fan-created font inspired by Minecraft's UI. It does not contain
font files from the original game and is not affiliated with Mojang or Microsoft.

## Launcher logo

The grass-and-obsidian server-block mark was generated with the built-in imagegen
tool at the user's request. The transparent master is `src/assets/launcher-mark.png`; Tauri's
icon tooling produces the desktop and UI sizes. The design uses original geometric
artwork rather than extracted Minecraft textures or official game logos. The
superseded user-supplied golden emblem remains recoverable from Git history.
See `src/assets/README.md` for the generation prompt and saved paths.

## Block glyphs

The launcher's action and placeholder block glyphs are original SVG geometry in
`src/MinecraftGlyph.tsx`. No Minecraft game texture files are bundled.

## Inventory illustrations

The transparent End island, world-map chest and modpack crate are generated artwork
created with the built-in imagegen tool for the launcher. Production PNGs and source
outputs are retained in `src/assets/`; prompts and mode are in its README. These are
Minecraft-inspired decorative assets, not extracted official game textures.
