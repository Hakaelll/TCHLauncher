# TCHLauncher visual system

## Approved direction, 0.6.4

The user's desktop collage supersedes the original grayscale ChatGPT composition and the earlier TFT cards. The latest color preference is anthracite: primary actions, selection and navigation use `#303335`. Olive and copper are no longer action accents. State and log category colors remain semantic, with explicit labels.

- Three columns: a dark server image rail, a light navigation column and a white workspace. Collapsing removes the navigation column while keeping server images and accessible tooltips. Small widths use the rail alone.
- The opening library displays actual servers, image covers, name, platform, state, saved RAM and storage target. Search and grid/list controls work; create and import remain available when empty. No invented players or metrics are shown.
- Server overview uses its selected image as a wide cover, real status and measured statistics. Existing images can fall back to their Minecraft icon; newly selected files preserve the original image for larger surfaces alongside the 64×64 icon.
- Seven sections are in the second navigation column; selection has one persistent moving indicator. The rail retains right-click menus and pin/delete actions.
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
