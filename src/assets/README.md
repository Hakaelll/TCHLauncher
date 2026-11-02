# Launcher visual assets

The current logo was generated with the built-in imagegen tool at the user's request. It is a flat black geometric block with stacked hosting layers and a central cutout. The block refers to Minecraft; the layers refer to managed servers. The reusable master has transparent alpha. The black-on-white rendition supplies the UI and native icons so the mark stays visible on dark surfaces. The previous golden emblem remains recoverable from Git history.

`scripts/generate_icon.py` delegates icon-format generation to the installed Tauri CLI using the black-on-white icon rendition. It stages the full derivative set in a temporary `artifacts/` folder and retains only the four configured Windows files: `32x32.png`, `128x128.png`, `128x128@2x.png` and `icon.ico`. The temporary directory is removed afterward; unused platform formats are not kept in the source tree. The Windows build explicitly uses `icons/icon.ico`, tracks icon input changes and assigns the same icon to the NSIS installer and uninstaller. Run `scripts/verify_windows_icons.ps1` after packaging to compare the executable and installer icon pixels with the expected ICO at Windows small/large sizes.

## Final assets

| Saved path | Use |
| --- | --- |
| `src/assets/launcher-logo.png` | Black-on-white logo at 256px; UI and welcome |
| `src/assets/launcher-mark.png` | Black geometric master with transparent background, 1254px |
| `src/assets/launcher-icon-source.png` | Black-on-white rendition for UI and native icons, 1254px |
| `src-tauri/icons/` | Four configured Windows app/tray/installer derivatives |
| `public/launcher-icon.png` | Browser favicon |
| `src/assets/choice-world.png` | Production 256px inventory illustration |
| `src/assets/choice-import.png` | Production 256px inventory illustration |
| `src/assets/choice-pack.png` | Production 256px inventory illustration |

Only the production-size inventory artwork is retained in the current tree. Unused full-size originals were removed to keep the workspace smaller and remain recoverable from Git history. The current logo master is retained for native icon generation; superseded full-size logo files are removed. These illustrations decorate real create/import actions; they do not represent an actual server or bundled game item.

## Regenerating deferred platform formats

The black-on-white icon rendition remains the native icon source. When another platform resumes,
use the installed Tauri CLI to generate the required formats and add only those
to its configuration. The current wrapper intentionally keeps Windows outputs.

```powershell
node node_modules/@tauri-apps/cli/tauri.js icon src/assets/launcher-icon-source.png --output src-tauri/icons --fit contain
```

## Prompts

### Logo: built-in generation and edits, logo-brand

#### Geometric concept

Use case: logo-brand. Design ONE genuinely simple, abstract corporate logo symbol for TCHLauncher, a Minecraft server management application. The concept is ONE modular block made of TWO stacked hosting layers: a cube suggests Minecraft's building blocks, and the two layers suggest hosted servers. Reduce this to a distinctive flat geometric mark, NOT an illustration of a cube or a piece of hardware. Use only three or four broad, carefully aligned angular shapes with deliberate generous negative space. Hint at a compact cubic/hexagonal silhouette through 30/60-degree geometry and TWO horizontal breaks, with a small square negative-space core joining the construction. The result should look like an intentional professional software identity designed on a precise grid, not a generic gaming badge. One solid ink color only: violet #AA88E8. Every filled shape must be exactly this color; transparent negative space. It must also work as a solid black silhouette with the same geometry. No text, letters, initials, slogan, status LEDs, circuitry, rack drawers, grass, dirt, textures, pixels within faces, outlines, shading, gradients, bevels, glow, shadows, 3D rendering, mascot, clouds, background tile or frame. Do not add additional small decorative elements. Balanced optical weight, thick substantial strokes/forms and clean cutouts. Recognizable and crisp at 16 and 32 pixels. Center a single coherent logo with about 12 percent even transparent padding in a square canvas. Actual transparent background. Only the finished flat geometric symbol, no mockup, no presentation sheet, no alternate logos.

#### Black master and edge cleanup

Edit the referenced geometric logo. Keep its EXACT silhouette, arrangement, proportions, square negative-space core, upper diamond shape and stacked angular bands. Recolor ALL filled areas to pure solid BLACK #000000, completely opaque, perfectly uniform, without any shading or purple. Clean up the raster artifacts: remove EVERY stray magenta/purple/blue speckle, mottled pixel and halo from the gaps and exterior. All negative space, including the square core, horizontal separations and surrounding canvas, must be genuinely transparent with clean crisp antialiased edges only. This is a precise corporate logo color change and edge cleanup, not a redesign. No new marks, outlines, borders, shadows, gradients, texture, text, white fill or background square. Keep original square canvas and optical centering. A professional, simple black geometric symbol on true transparent alpha.

#### Black-on-white rendition

Make a clean presentation version of this EXACT black geometric logo. Change ONLY the transparent background to pure opaque white #FFFFFF, keeping the existing black mark, negative-space cutouts, exact shape, margins, proportions and alignment. The mark must remain solid black with clean professional edges. No redesign, no extra geometry, no text, no shadows, no gradients, no texture, no frame. A black geometric corporate logo on a plain white square. This is simply the white-background preview so the black mark is clearly visible.

### world: built-in generation, stylized-concept

Use case: stylized-concept. Asset type: decorative inventory illustration for a Minecraft server launcher action. A small floating Minecraft End island built from pale end-stone voxel blocks, one tall dark obsidian pillar and two branching purple chorus plants. A few lavender crystalline block details. Clearly Minecraft End inspired, quiet and beautiful, not a plain cube. Render a polished miniature voxel object in a consistent three-quarter isometric view, with crisp square blocks and fine pixel material details, restrained purple illumination, charcoal shadows and pale stone highlights. Transparent background with true alpha, object centered and fully visible with 10 percent even transparent padding. The asset must read cleanly at 120px; strong silhouette, no environment backdrop, no floor plane, no frame, no UI, no text, no watermark, no blurry halo, no extra loose objects outside the silhouette.

### import: built-in generation, stylized-concept

Use case: stylized-concept. Asset type: decorative inventory illustration for a Minecraft server launcher action. A small open Minecraft style obsidian and dark oak voxel chest with violet metal trim, holding a rolled pale paper world map with a few tiny pixel markings, no readable letters. Clearly suggests bringing an existing world into a launcher. Render a polished miniature voxel object in a consistent three-quarter isometric view, with crisp square blocks and fine pixel material details, restrained purple illumination, charcoal shadows and pale stone highlights. Transparent background with true alpha, object centered and fully visible with 10 percent even transparent padding. The asset must read cleanly at 120px; strong silhouette, no environment backdrop, no floor plane, no frame, no UI, no text, no watermark, no blurry halo, no extra loose objects outside the silhouette.

### pack: built-in generation, stylized-concept

Use case: stylized-concept. Asset type: decorative inventory illustration for a Minecraft server launcher action. A small Minecraft style dark voxel storage crate containing three distinct purple and lavender glowing cubic crystals and an obsidian puzzle-piece shaped block. Clearly suggests an assembled modpack. No readable text. Render a polished miniature voxel object in a consistent three-quarter isometric view, with crisp square blocks and fine pixel material details, restrained purple illumination, charcoal shadows and pale stone highlights. Transparent background with true alpha, object centered and fully visible with 10 percent even transparent padding. The asset must read cleanly at 120px; strong silhouette, no environment backdrop, no floor plane, no frame, no UI, no text, no watermark, no blurry halo, no extra loose objects outside the silhouette.

