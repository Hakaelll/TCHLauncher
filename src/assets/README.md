# Launcher visual assets

The current logo was generated with the built-in imagegen tool at the user's request. It combines a Minecraft-inspired voxel block with two server slots, using charcoal, violet and lavender. Its background has true transparent alpha. The previous golden emblem remains recoverable from Git history.

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

Use case: logo-brand. Asset type: production desktop application icon and logo for TCHLauncher, a local Minecraft Java server manager. Create ONE exceptionally simple, distinctive compact logo: a chunky isometric voxel cube whose front face is also a server rack with exactly two broad horizontal slots and one small square violet status light per slot. The cube silhouette suggests a Minecraft block; the slots immediately communicate hosting/managing a server. No letters, no wordmark, no monogram. Clean vector-like flat geometric artwork with hard straight edges, a subtle stepped pixel corner treatment, thick readable shapes, no intricate pixel texture. Palette: dark charcoal outline #17141f, saturated medium purple #8461cb front, lighter lavender #c8b0fa top, deep violet #50347a side, near-white lavender slots #eee5ff. Strong clear contrast on BOTH light and dark UI. A single unified mark, not separate icons arranged together. Large centered symbol fills about 86 percent of a square canvas with even safe transparent margins. All of symbol inside canvas. Actual transparent background, no square backdrop, no ground shadow, no glow, no gradients, no gold, no sun rays, no realistic 3D rendering, no tiny decoration, no official Minecraft logos or Creeper faces. Design must stay identifiable as a server/block at 24 and 32 pixels. Deliver only the single finished icon.

### world: built-in generation, stylized-concept

Use case: stylized-concept. Asset type: decorative inventory illustration for a Minecraft server launcher action. A small floating Minecraft End island built from pale end-stone voxel blocks, one tall dark obsidian pillar and two branching purple chorus plants. A few lavender crystalline block details. Clearly Minecraft End inspired, quiet and beautiful, not a plain cube. Render a polished miniature voxel object in a consistent three-quarter isometric view, with crisp square blocks and fine pixel material details, restrained purple illumination, charcoal shadows and pale stone highlights. Transparent background with true alpha, object centered and fully visible with 10 percent even transparent padding. The asset must read cleanly at 120px; strong silhouette, no environment backdrop, no floor plane, no frame, no UI, no text, no watermark, no blurry halo, no extra loose objects outside the silhouette.

### import: built-in generation, stylized-concept

Use case: stylized-concept. Asset type: decorative inventory illustration for a Minecraft server launcher action. A small open Minecraft style obsidian and dark oak voxel chest with violet metal trim, holding a rolled pale paper world map with a few tiny pixel markings, no readable letters. Clearly suggests bringing an existing world into a launcher. Render a polished miniature voxel object in a consistent three-quarter isometric view, with crisp square blocks and fine pixel material details, restrained purple illumination, charcoal shadows and pale stone highlights. Transparent background with true alpha, object centered and fully visible with 10 percent even transparent padding. The asset must read cleanly at 120px; strong silhouette, no environment backdrop, no floor plane, no frame, no UI, no text, no watermark, no blurry halo, no extra loose objects outside the silhouette.

### pack: built-in generation, stylized-concept

Use case: stylized-concept. Asset type: decorative inventory illustration for a Minecraft server launcher action. A small Minecraft style dark voxel storage crate containing three distinct purple and lavender glowing cubic crystals and an obsidian puzzle-piece shaped block. Clearly suggests an assembled modpack. No readable text. Render a polished miniature voxel object in a consistent three-quarter isometric view, with crisp square blocks and fine pixel material details, restrained purple illumination, charcoal shadows and pale stone highlights. Transparent background with true alpha, object centered and fully visible with 10 percent even transparent padding. The asset must read cleanly at 120px; strong silhouette, no environment backdrop, no floor plane, no frame, no UI, no text, no watermark, no blurry halo, no extra loose objects outside the silhouette.

