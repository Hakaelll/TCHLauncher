# TCHLauncher - End / Obsidian visual system (0.15.1)

## Authority and use

The user's latest references replace the green Tableland structure with black and
dark gray surfaces, End-like violet accents, Minecraft lettering and Minehut-like
rectangular controls. This is an operating desktop application for real local
servers. Explicit user requests outrank previous design decisions.

One server rail stays compact by default and expands on hover or keyboard focus.
It opens over the workspace without shifting content or icon targets; no manual
collapse button remains. Context menus keep it open until dismissed. Server sections belong to a horizontal,
wrapping workspace bar. There is no repeated server-image banner. The user reviews
native appearance manually; no computer use or delegated agents are used.

## Themes and persistence

The default **End** theme uses canvas `#131217`, surface `#1c1b22`, raised surface
`#282532`, ink `#efeaf5`, supporting text `#bdb5ca`, accent `#baa2ee` and highlight
`#dfcbff`. This keeps the workspace neutral and reserves violet for interaction.

**Obsidian** uses charcoal canvas `#141518`, neutral surfaces and graphite/silver
accents. **Nether** uses canvas `#191414`, warm dark surfaces and ember accent
`#f1ab95`. Semantic success, warning, destructive and log colors retain their
meaning in every theme. The theme covers the library, server workspace, input,
slider, console, charts, menu, disclosure and confirmation surfaces.

Ajustes is below Inicio automatico in the rail. It opens an ordinary page with a
native radio group, palette previews and immediate application. Preferences live
under `tch-appearance-v1` in local storage, are validated on load and are applied
before React's first paint. Invalid values fall back to End. Storage errors keep
the selected appearance usable and report that it could not be saved.

## Type and controls

Bundle unmodified Monocraft 4.2.1 under the SIL Open Font License. Disable ligatures
and synthetic bold. Use its pixel lettering for headings, server names, section
navigation, rail actions, buttons, property labels, capacity labels, player tabs and
disclosures. Keep reading fonts for explanations, input and long data. Commands,
logs, UUIDs and executable paths use conventional monospace where appropriate.

A reading-font preference changes display/control lettering without changing
information, layout or functionality. The Minecraft option is the default.

Rectangular controls have a raised upper edge and recessed lower edge. Resting
controls are quiet; hover/focus illuminate them from the selected theme. The section
bar uses the same raised selected treatment and one moving indicator. Primary
controls avoid constant glow and pill geometry. Destructive controls remain coral.
Compact rail actions use bare icons with a 44px-high hit area, without resting
rectangular frames. Hover and selected states use the theme's violet wash. Labels
fade and slide into the expanded rail after a 45ms delay while its width eases
over 280ms. Collapse takes 190ms, with labels fading out in 90ms. Icons and the
workspace remain stationary; reduced motion removes the transition. Expanded
actions include labels.

## Composition

- Library: one larger create action beside two compact folder/modpack import actions.
  Dedicated transparent voxel illustrations depict an End island, a world-map chest
  and a modpack crate. They are decoration for real actions, not fabricated server
  images. Narrow layouts turn the choices into compact rows. Populated libraries
  keep search, grid/list view, artwork, actual states and context menus.
- Runtime: padded panel with a grouped Java identity and actual compatibility status.
  Required Java is adjacent to the actions. Full executable location and custom Java
  use independent disclosures. Existing reinstall, recheck and verified custom
  runtime selection remain connected to the native manager.
- Properties: distinct group headings and row separators, immediate slider tracking
  plus numeric input, descriptions adjacent to controls.
- Console: a deeper surface and distinct INFO/SYSTEM/PLAYER/CHAT/COMMAND/WARN/ERROR
  labels. Output stays selectable; no individual log-row animation.
- Brand: retain the user's original logo. Use a transparent exterior cutout with the
  dark central disk retained for UI, native icons and favicon. No inversion or square
  black backdrop. Asset sources and prompts are in src/assets/README.md.

## Motion and accessibility

Central motion tokens remain authoritative. A single short clipped library entrance
and small illustration displacement support discovery; no repeating ambient motion.
Keep moving selection, list continuity, chart interpolation and numeric updates.
Respect reduced motion in CSS and JavaScript. Theme selection does not animate the
whole page or steal focus. Native radio arrow navigation, focus outlines, semantic
labels and status text remain available.

The shell owns scrolling and prevents horizontal page movement. Native protected
confirmations stay centered in the viewport independently of page scroll.

## Verification boundaries

TypeScript, production build, preference persistence/failure tests, existing manager
regressions and Windows packaging are executable checks. Token contrast covers all
three palettes and log tags. No native screenshot or gameplay verification is claimed.
The broader original specification remains in development; see NEXT_TODOS.md.

## Application preferences

Ajustes uses one main surface and a horizontal, wrapping category bar: General,
Java, Almacenamiento, Red, Copias, Avanzado and Información. It introduces no extra
sidebar. General retains immediate theme/font/motion preferences and real startup
controls. Java/network/backup/task controls clearly identify the selected server.
Storage exposes actual native locations and folder actions; Information uses the
native binary version, OS/architecture and persisted EULA record. Diagnostics and
shortcuts live in Advanced. Native appearance and keyboard review remain manual.

## Observed startup path

Overview places a compact Java / Mundo / Listo ordered path between the current
server state and its metrics. Native events alone advance phases; loading a world
and preparing its spawn have distinct status text. No generated percent, elapsed
time heuristic, pulsing decoration or synthetic Ready transition is used. A server
that does not emit recognized intermediate events may jump directly to Ready.
Current, completed and pending steps use text and native list semantics as well as
theme accents; color/border transitions use the existing 160 ms motion tokens and
respect system/application reduced motion. Labels wrap on narrow windows. The
provided actual Paper log passes parsing; native appearance remains manual.
