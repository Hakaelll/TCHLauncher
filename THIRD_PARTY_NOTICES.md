# Third-party assets

## Monocraft

- Version: 4.2.1, regular TTF, distributed unmodified.
- Author: Idrees Hassan.
- Source: https://github.com/IdreesInc/Monocraft/releases/tag/v4.2.1
- Copyright (c) 2022, Idrees Hassan.
- License: SIL Open Font License 1.1. Full license bundled in
  `public/fonts/OFL-Monocraft.txt` and copied into the application assets.

Monocraft is a fan-created font inspired by Minecraft's UI. It does not contain
font files from the original game and is not affiliated with Mojang or Microsoft.

## Launcher emblem

The gold voxel emblem was supplied by the user for the application logo and icon.
The source image is preserved in `src/assets/launcher-emblem.png`. Desktop and UI icon
sizes are generated with Tauri's icon tooling from a transparent exterior cutout.
The built-in imagegen tool produced the cutout while retaining the dark central disk.
The original supplied image is preserved. See `src/assets/README.md` for saved paths
and the exact edit prompt.

## Block glyphs

The launcher's action and placeholder block glyphs are original SVG geometry in
`src/MinecraftGlyph.tsx`. No Minecraft game texture files are bundled.

## Inventory illustrations

The transparent End island, world-map chest and modpack crate are generated artwork
created with the built-in imagegen tool for the launcher. Production PNGs and source
outputs are retained in `src/assets/`; prompts and mode are in its README. These are
Minecraft-inspired decorative assets, not extracted official game textures.
