# Launcher visual assets

The current logo was generated with the built-in imagegen tool at the user's request. It combines a Minecraft-inspired grass-and-dirt top with an obsidian server body, two inset server bays and amethyst details. Green is confined to the grass; the application keeps its existing End theme. Its background has true transparent alpha. The previous golden emblem remains recoverable from Git history.

`scripts/generate_icon.py` delegates icon-format generation to the installed Tauri CLI using the transparent full-size server-block mark. It stages the full derivative set in a temporary `artifacts/` folder and retains only the four configured Windows files: `32x32.png`, `128x128.png`, `128x128@2x.png` and `icon.ico`. The temporary directory is removed afterward; unused platform formats are not kept in the source tree. The Windows build explicitly uses `icons/icon.ico`, tracks icon input changes and assigns the same icon to the NSIS installer and uninstaller. Run `scripts/verify_windows_icons.ps1` after packaging to compare the executable and installer icon pixels with the expected ICO at Windows small/large sizes.

## Final assets

| Saved path | Use |
| --- | --- |
| `src/assets/launcher-logo.png` | Transparent logo at 256px; UI and welcome |
| `src/assets/launcher-mark.png` | Generated transparent logo master, 1254px |
| `src-tauri/icons/` | Four configured Windows app/tray/installer derivatives |
| `public/launcher-icon.png` | Browser favicon |
| `src/assets/choice-world.png` | Production 256px inventory illustration |
| `src/assets/choice-import.png` | Production 256px inventory illustration |
| `src/assets/choice-pack.png` | Production 256px inventory illustration |

Only the production-size inventory artwork is retained in the current tree. Unused full-size originals were removed to keep the workspace smaller and remain recoverable from Git history. The current logo master is retained for native icon generation; superseded full-size logo files are removed. These illustrations decorate real create/import actions; they do not represent an actual server or bundled game item.

## Regenerating deferred platform formats

The full transparent emblem remains the source. When another platform resumes,
use the installed Tauri CLI to generate the required formats and add only those
to its configuration. The current wrapper intentionally keeps Windows outputs.

```powershell
node node_modules/@tauri-apps/cli/tauri.js icon src/assets/launcher-mark.png --output src-tauri/icons --fit contain
```

## Prompts

### Logo: built-in generation, logo-brand

Use case: logo-brand. Create ONE polished, original application emblem for a Minecraft server manager. It must instantly communicate BOTH Minecraft worlds and hosting a server. Design a compact chunky Minecraft grass-and-dirt block transformed into a premium obsidian server block: unmistakable square voxel grass tiles across the top, a thin brown block-earth layer beneath the grass, and a dark charcoal/obsidian front with TWO inset server bays. Each bay has a broad readable pale-lavender slot, a few chunky ventilation squares, and one small square amethyst-purple status indicator. The visible right side has a restrained stepped amethyst circuit/rune inset, using just a few large geometric blocks. One cohesive strong cube silhouette in tasteful three-quarter isometric perspective. More carefully crafted and distinctive than a generic server-rack clipart: precise voxel construction, thoughtful proportions, tasteful stepped bevels, selective pixel material highlights, polished restrained game-icon shading. Minecraft grass green on the top only, dark obsidian charcoal body, purple/amethyst/lavender accents, earth brown as a thin rim. No green ambient lighting. Strong dark outer silhouette and clear bright interior forms, readable on both dark and light backgrounds. Keep all details large and deliberate so the mark remains identifiable at 32px; not a complex scene or a pile of props. Center the single symbol with 9 percent even transparent padding, completely inside a square canvas. Genuine transparent background. No text, no letters, no initials, no wordmark, no official Minecraft logo, no Creeper face, no sun emblem, no golden rays, no generic rounded-square app background, no cables trailing outside, no ground plane, no cast shadow, no surrounding scenery, no photorealism, no blurry bloom or sparkles. Deliver one final professionally finished Minecraft server-block emblem only, not a sheet of concepts.

### world: built-in generation, stylized-concept

Use case: stylized-concept. Asset type: decorative inventory illustration for a Minecraft server launcher action. A small floating Minecraft End island built from pale end-stone voxel blocks, one tall dark obsidian pillar and two branching purple chorus plants. A few lavender crystalline block details. Clearly Minecraft End inspired, quiet and beautiful, not a plain cube. Render a polished miniature voxel object in a consistent three-quarter isometric view, with crisp square blocks and fine pixel material details, restrained purple illumination, charcoal shadows and pale stone highlights. Transparent background with true alpha, object centered and fully visible with 10 percent even transparent padding. The asset must read cleanly at 120px; strong silhouette, no environment backdrop, no floor plane, no frame, no UI, no text, no watermark, no blurry halo, no extra loose objects outside the silhouette.

### import: built-in generation, stylized-concept

Use case: stylized-concept. Asset type: decorative inventory illustration for a Minecraft server launcher action. A small open Minecraft style obsidian and dark oak voxel chest with violet metal trim, holding a rolled pale paper world map with a few tiny pixel markings, no readable letters. Clearly suggests bringing an existing world into a launcher. Render a polished miniature voxel object in a consistent three-quarter isometric view, with crisp square blocks and fine pixel material details, restrained purple illumination, charcoal shadows and pale stone highlights. Transparent background with true alpha, object centered and fully visible with 10 percent even transparent padding. The asset must read cleanly at 120px; strong silhouette, no environment backdrop, no floor plane, no frame, no UI, no text, no watermark, no blurry halo, no extra loose objects outside the silhouette.

### pack: built-in generation, stylized-concept

Use case: stylized-concept. Asset type: decorative inventory illustration for a Minecraft server launcher action. A small Minecraft style dark voxel storage crate containing three distinct purple and lavender glowing cubic crystals and an obsidian puzzle-piece shaped block. Clearly suggests an assembled modpack. No readable text. Render a polished miniature voxel object in a consistent three-quarter isometric view, with crisp square blocks and fine pixel material details, restrained purple illumination, charcoal shadows and pale stone highlights. Transparent background with true alpha, object centered and fully visible with 10 percent even transparent padding. The asset must read cleanly at 120px; strong silhouette, no environment backdrop, no floor plane, no frame, no UI, no text, no watermark, no blurry halo, no extra loose objects outside the silhouette.

