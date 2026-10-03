# Current visual direction - Minecraft End themes (0.11.0)

The latest brief uses black/dark gray structure and End violet accents. End is the default, with persistent Obsidian and Nether alternatives in Ajustes below Inicio automatico. Pixel lettering extends to navigation and controls, with a reading-font preference. The logo has a transparent exterior; library artwork and the Java panel have been rebuilt. The current specification is [design.md](design.md); asset prompts are in [src/assets/README.md](src/assets/README.md).

TypeScript/build, 16 frontend behaviors, 110 local native regressions and 160 text/semantic contrast combinations passed. Native appearance and keyboard review remain manual by the user's request. The historical sections below are superseded where they conflict with the current specification.

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

## Modpack import

A viewport-centered native dialog follows three steps: choose a local package, review server files and optional selections, then set capacity. Minecraft and loader versions are read from the archive; unsupported engines explain the limitation before proceeding. Protected files remain unchecked and locked. Sliders accompany numeric inputs, consent is requested only if it is not remembered, and transfer progress provides cancellation. Step transitions reuse the navigation motion and reduced-motion settings. Failed imports can retry the same new server without creating another instance. Native visual and keyboard review remain pending.

## Backup retention controls

The Copies section exposes retention in a collapsed details panel with its saved enabled/disabled state. Automatic deletion requires an explicit checkbox and Save action; count and GB limits accept an empty value for no limit. Existing copies use a separate review action and a viewport-centered modal listing dates, measured sizes and before/after storage. Confirmation uses the destructive color, preserves keyboard focus through the shared Modal, and displays stale-review or cleanup failures in place. Native visual review remains pending.

## Scheduled tasks

Settings groups local tasks in a dedicated section. Rows show the action, next time, interval, state and last result; pause/activate and remove are separate from server deletion. A centered native modal collects the action, local date/time and optional interval. The backup with stop/restart action explicitly explains the player interruption; the stopped-only action explains omission while running. The panel states that the launcher must remain open and how overdue tasks are handled. Existing reduced motion, focus restoration and responsive wrapping apply. Native visual and keyboard review remains pending.

## Crash restart policy

A separate group beneath scheduled tasks offers an explicit opt-in toggle, retry delay and attempt limit. The status shows the saved enabled state, attempts used, next retry and last outcome. Saving resets attempts; a separate reset action uses the saved policy. Copy explains requested-stop exclusion and the ten-minute healthy reset. Editing drafts does not hide or change the saved execution state. Native visual review remains pending.

## Global startup preferences

The rail's settings button opens a viewport-centered startup dialog even with an empty library. Application login startup and selected-server startup are separate groups. Server images identify selectable rows, persisted results appear beneath them and asynchronous preference changes hold the dialog open until finished. The global switch explains tray startup; server selection explains that existing Java and consent are required. Development builds explain why their transient executable is not registered. Responsive rail labels follow the compact-navigation behavior. Native visual review remains pending.

## Loader update review

Settings offers one stopped-server loader update entry for installed Fabric/NeoForge instances. Review shows the installed version, the unchanged Minecraft version and newer stable choices. The explicit confirmation describes the safety copy and preservation of server content. The shared viewport-centered modal holds download progress and cancellation, with in-place errors and a fresh review action on retry. Native visual review remains pending.

### Library and overview refinement

The empty library offers three actual entry points: creation, folder import and mrpack import. Each choice places its action at the lower edge; small windows use compact horizontal rows. The search/layout toolbar appears once there are servers to browse. Anthracite remains the primary control color; blue, teal and violet distinguish existing operations without a decorative banner.

Overview metrics share one bordered strip with real readings and explicit process/configuration context. Resident process memory is not represented as heap utilization. The LAN address remains selectable and copyable in a dedicated surface. Choice arrows move only on interaction, with reduced-motion preferences respected. TypeScript compilation is verified; native visual review remains pending under the user's no-computer-use constraint.
