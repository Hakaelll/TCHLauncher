# Launcher visual assets

These assets were produced with the built-in imagegen tool, then integrated into this workspace. Tauri icon tooling generates bounded UI sizes without cropping the objects. The user's original logo remains at `src/assets/launcher-emblem.png`.

`scripts/generate_icon.py` delegates icon-format generation to the installed Tauri CLI using the transparent full-size emblem. It does not redraw the former letter-T mark. The Windows build explicitly uses `icons/icon.ico`, tracks icon input changes and assigns the same icon to the NSIS installer and uninstaller. Run `scripts/verify_windows_icons.ps1` after packaging to compare the executable and installer icon pixels with the expected ICO at Windows small/large sizes.

## Final assets

| Saved path | Use |
| --- | --- |
| `src/assets/launcher-logo.png` | Transparent logo at 256px; UI and welcome |
| `src/assets/launcher-emblem-cutout.png` | Selected transparent edit, full resolution |
| `src-tauri/icons/` | Native app/tray icon derivatives |
| `public/launcher-icon.png` | Browser favicon |
| `src/assets/choice-world.png` | Production 256px inventory illustration |
| `src/assets/choice-import.png` | Production 256px inventory illustration |
| `src/assets/choice-pack.png` | Production 256px inventory illustration |

Full-size generated inventory artwork is retained in `src/assets/choice-world-original.png`, `choice-import-original.png` and `choice-pack-original.png`. These illustrations are decoration for real create/import actions. They do not represent an actual server or bundled game item.

## Prompts

### Logo: built-in edit, background-extraction

Use case: background-extraction. Asset type: TCHLauncher application logo and Windows app icon. Input image 1 is the edit target, not a style reference. Remove ONLY the exterior black background around the existing golden voxel sun emblem and in the gaps between its rays, producing true transparent alpha. Preserve the golden blocks, all ray shapes and tips, lighting, colors, monogram and the opaque dark obsidian circular center exactly as supplied. The central black disk is part of the logo and must remain opaque. Center the full emblem in a square canvas with a small even transparent margin, about 3% on each side; keep every ray tip fully visible. No square backdrop, no halo painted into a black background, no redesign, no new text, no new symbols, no border, no extra objects. Clean sharp cutout suitable for use at 48px while faithfully preserving the original logo.

### world: built-in generation, stylized-concept

Use case: stylized-concept. Asset type: decorative inventory illustration for a Minecraft server launcher action. A small floating Minecraft End island built from pale end-stone voxel blocks, one tall dark obsidian pillar and two branching purple chorus plants. A few lavender crystalline block details. Clearly Minecraft End inspired, quiet and beautiful, not a plain cube. Render a polished miniature voxel object in a consistent three-quarter isometric view, with crisp square blocks and fine pixel material details, restrained purple illumination, charcoal shadows and pale stone highlights. Transparent background with true alpha, object centered and fully visible with 10 percent even transparent padding. The asset must read cleanly at 120px; strong silhouette, no environment backdrop, no floor plane, no frame, no UI, no text, no watermark, no blurry halo, no extra loose objects outside the silhouette.

### import: built-in generation, stylized-concept

Use case: stylized-concept. Asset type: decorative inventory illustration for a Minecraft server launcher action. A small open Minecraft style obsidian and dark oak voxel chest with violet metal trim, holding a rolled pale paper world map with a few tiny pixel markings, no readable letters. Clearly suggests bringing an existing world into a launcher. Render a polished miniature voxel object in a consistent three-quarter isometric view, with crisp square blocks and fine pixel material details, restrained purple illumination, charcoal shadows and pale stone highlights. Transparent background with true alpha, object centered and fully visible with 10 percent even transparent padding. The asset must read cleanly at 120px; strong silhouette, no environment backdrop, no floor plane, no frame, no UI, no text, no watermark, no blurry halo, no extra loose objects outside the silhouette.

### pack: built-in generation, stylized-concept

Use case: stylized-concept. Asset type: decorative inventory illustration for a Minecraft server launcher action. A small Minecraft style dark voxel storage crate containing three distinct purple and lavender glowing cubic crystals and an obsidian puzzle-piece shaped block. Clearly suggests an assembled modpack. No readable text. Render a polished miniature voxel object in a consistent three-quarter isometric view, with crisp square blocks and fine pixel material details, restrained purple illumination, charcoal shadows and pale stone highlights. Transparent background with true alpha, object centered and fully visible with 10 percent even transparent padding. The asset must read cleanly at 120px; strong silhouette, no environment backdrop, no floor plane, no frame, no UI, no text, no watermark, no blurry halo, no extra loose objects outside the silhouette.

