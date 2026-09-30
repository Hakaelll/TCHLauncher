# TCHLauncher visual system

## Current development direction (after 0.6.4)

The user's desktop collage supersedes the original grayscale ChatGPT composition and the earlier TFT cards. The latest color preference is anthracite: primary actions, selection and navigation use `#303335`. Olive and copper are no longer action accents. State and log category colors remain semantic, with explicit labels. Blue, teal and violet distinguish metric icons; neutral structure and controls remain anthracite. Installation progress and context menus no longer retain copper or olive styling.

- One collapsible dark server sidebar and a white workspace. Expanded mode shows server names and versions; collapsed mode keeps images and accessible tooltips. Server sections use a wrapping horizontal navigation within the workspace, avoiding nested sidebars.
- The opening library displays actual servers, image covers, name, platform, state, saved RAM and storage target. Search and grid/list controls work; create and import remain available when empty. No invented players or metrics are shown.
- Server overview uses a compact image in the header, real status and measured statistics. The repeated panorama has been removed. Existing images can fall back to their Minecraft icon; newly selected files preserve the original image for larger surfaces alongside the 64×64 icon.
- Seven sections sit below the server header with one persistent moving selection indicator. The sidebar retains right-click menus and pin/delete actions. The supplied transparent emblem replaces the T monogram, using CSS framing to trim its outer margin and inversion for legibility on the dark surface.
- The five-step creation flow retains validation and a stable summary. Step changes move only the form content by 16px, in the direction of travel.
- Settings use grouped properties, a Java runtime panel and a collapsed JVM editor. Destructive controls retain red and require confirmation.
- The console has an anthracite reading surface and colored category labels. Incoming output scrolls quickly while following is enabled; user scrolling pauses following. Individual log rows are not animated.
- RAM and CPU remain together in the dedicated performance view. Lines and values interpolate briefly; unknown values and gaps remain unknown.

## Motion system

`motion.tsx` owns tokens: 90ms instant, 120ms interaction, 160ms micro, 200ms standard, 240ms navigation and 300ms large. It exports the same values as CSS variables. Standard/enter/exit curves are centralized. A 1200ms cadence is reserved for an indeterminate working state; stable servers never pulse.

Moving indicators preserve selection continuity. FLIP list transitions move retained items, fade removed inert snapshots and coordinate container height. Cards have no lift or hover scale. Menus enter from their origin; dialogs enter at scale .985 and close faster on explicit dismissals. Sliders retain immediate pointer tracking. Native controls retain native menus rather than a second custom dropdown system.

CSS and JavaScript respect reduced motion. Keyboard controls and focus rings remain available. There is no universal page fade or animation on every section.

## Verification status

Type checking, production compilation and backend tests are executable checks. Native screenshots, frame timing and the full keyboard/desktop motion review remain manual because the user requested no computer use. No visual audit or 60 FPS guarantee is claimed without those measurements.

## Content manager

Operate surface: server-specific Mods or Plugins section, installed-file rows and a searchable Modrinth catalog. Project icons carry color; anthracite controls and restrained blue/teal compatibility and action accents preserve the desktop palette. A protected native modal reviews versions, dependency changes and backups before installation. Destructive removal has its own confirmation. Read-only browsing remains available while Java runs; file changes require a stopped server. Existing motion tokens, reduced motion, focus restoration and viewport-centered dialogs apply.
