# TCHLauncher 0.10.0 — Minecraft / End visual refresh

## Visual changes

- Replace the light workspace with an End-inspired dark interface, using the supplied
  Tableland palette: layered obsidian/teal surfaces, mint library headings, lavender
  creation details and readable semantic status colors.
- Introduce Minehut-inspired rectangular controls with raised edges. Buttons rest
  dark, illuminate on hover/focus and depress on activation. Destructive actions
  use a separate coral treatment.
- Bundle [Monocraft 4.2.1](https://github.com/IdreesInc/Monocraft/releases/tag/v4.2.1)
  locally for headings and principal actions. Form input, dense navigation, logs and
  measurements retain readable system or conventional monospace fonts. Spanish
  diacritics and punctuation are included; the complete SIL license ships with the app.
- Use the supplied golden voxel emblem for the application, desktop/tray icon and
  browser favicon. UI-sized assets are generated from the original supplied image;
  the original is retained in the repository.
- Redesign the first-server choices with original block geometry, framed surfaces
  and explicit create/folder-import/modpack-import actions. Add actual server/online
  counts to populated libraries and block placeholders for servers without artwork.
- Apply the same material language to fields, sliders, property groups, runtime
  panels, graphs, context menus and protected confirmations. Console tags retain
  distinct INFO, SYSTEM, PLAYER, CHAT, COMMAND, WARN and ERROR colors.
- Preserve one collapsible server rail and horizontal workspace sections. The
  repeated server-image banner remains removed. Fixed confirmation centering,
  existing keyboard focus management and reduced-motion support are retained.

## Manager changes included since 0.9.0

- Add explicit opt-in experimental Minecraft catalogs for Vanilla and Fabric.
  Stable releases remain the default; Paper and NeoForge remain on their stable
  catalogs. Experimental instances are labeled and the world-format risk is explained.
- Tighten readiness detection: only genuine server-thread completion messages mark
  Minecraft ready; player chat, plugin text and incomplete completion messages do not.

## Verification

- TypeScript and production Vite build: passed.
- Frontend behavior: 13 tests passed, including telemetry, startup observation and
  player identity selection.
- Offline Rust suite: 110 tests passed; 18 tests requiring external resources were
  intentionally excluded from that run.
- Palette: 24 main text/semantic combinations passed 4.5:1 contrast; minimum 5.23:1.
- Actual bundled font: Spanish accents, ñ and inverted punctuation verified in cmap.
- Earlier isolated live checks installed Vanilla/Fabric `26.4-snapshot-2` with Java
  25 up to EULA refusal, without accepting the agreement or generating a world.
- Windows NSIS installer generated from this worktree.

## Scope and remaining checks

Existing creation, import, loaders, runtime management, content reviews, worlds,
backups, scheduling and server controls remain wired to the native manager. This
release does not migrate player UUIDs or change imported server authentication settings.

The user requested manual visual assessment and prohibited computer use; no browser
automation, native screenshot inspection or Minecraft client gameplay check was used.
Linux packaging, external network reachability, automatic UPnP/NAT-PMP port mappings
and trustworthy TPS telemetry remain development work. This is a Windows preview,
not a declaration that the full original specification is complete.
