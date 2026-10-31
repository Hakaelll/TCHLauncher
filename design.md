# TCHLauncher visual system

## Authority and source files

The current direction is Minecraft and the End: charcoal surfaces, violet accents,
Monocraft where legible, raised rectangular controls and the user's golden voxel
emblem. These later user instructions replace the original light, achromatic
reference. Keep the accepted appearance while improving clarity and interaction.

This document defines current design rules. Iteration history and scoped checks
belong in [development notes](docs/development-notes/README.md), release contents in
[release notes](docs/release-notes/README.md), and remaining acceptance in
[NEXT_TODOS.md](NEXT_TODOS.md). Visual acceptance does not prove keyboard,
assistive-technology, router or gameplay behavior.

| Owner | Responsibility |
| --- | --- |
| [end.css](src/end.css) | Theme tokens, Minecraft materials and shared controls |
| [motion.tsx](src/motion.tsx) | Shared motion timing and behavior |
| [appearance.ts](src/appearance.ts) | Validated theme, font and motion preferences |
| [Asset guide](src/assets/README.md) | Emblem, artwork origins and icon generation |
| [Architecture](ARCHITECTURE.md) | Native authority, persistence and recovery contracts |

## Themes and typography

Use existing role tokens rather than new per-component colors.

| Role | End, default | Obsidian | Nether |
| --- | --- | --- | --- |
| Canvas | `#131217` | `#141518` | `#191414` |
| Surface | `#1c1b22` | `#1e2025` | `#231c1c` |
| Raised surface | `#282532` | `#292b30` | `#302526` |
| Primary text | `#efeaf5` | `#eeeff4` | `#f4eeea` |
| Supporting text | `#bdb5ca` | `#b9beca` | `#cbbab4` |
| Accent | `#baa2ee` | `#c2c7d2` | `#f1ab95` |
| Highlight | `#dfcbff` | `#eeeeff` | `#ffd3c2` |

Success, warning, destructive and console colors keep their meaning across themes.
Pair color with text or a distinct mark. Error evidence uses a thin frame and a
label, not a thick decorative stripe. Keep contrast readable on every surface.

Bundle unmodified Monocraft 4.2.1 under its SIL Open Font License. Use it for
headings, server names, navigation, action labels and short property labels.
Disable ligatures and synthetic bold. Use the reading face for explanations,
inputs and long data; use ordinary monospace for commands, UUIDs, logs and paths.
Keep the reading-font preference fully functional. Current desktop body text is
14 px; the main heading is 24 px. Long text wraps without hiding identifiers.

Appearance preferences use `tch-appearance-v1`, with validated End/Minecraft/system
motion defaults before the first React paint. Storage failure keeps the selected
appearance usable and reports that saving failed. System reduced motion always
wins. Changing theme does not animate the entire page or steal focus.

## Shell, controls and navigation

- Use one compact server rail. Expand on hover or keyboard focus over the workspace,
  keeping icons and main content stationary. Context menus keep the rail open.
  Do not restore the manual collapse button or a second server sidebar.
- Compact rail actions use bare icons with 44 px hit areas. Reveal labels when
  expanded. Server sections use one horizontal wrapping bar and moving indicator.
- Keep the shell's scrolling boundary and avoid horizontal page overflow. Support
  the target 1280×720, 1366×768 and 1920×1080 layouts with wrapped controls and paths.
- Rectangular controls have raised upper and recessed lower edges. Resting controls
  stay quiet; hover/focus use theme illumination. Avoid persistent glow, pill
  geometry and an extra icon box around every action. Destructive actions use
  the shared coral palette.
- Preserve visible focus, associated labels, native radio behavior, checked/pressed
  states and readable disabled reasons. Use text when an icon would be ambiguous.
- Use one action hierarchy per view. Put uncommon actions in the shared overflow
  menu, available through its button or right click. Keep it within the viewport,
  with scrolling, enabled-item arrow/Home/End navigation, typeahead, Escape focus
  return, outside dismissal and ordinary Tab behavior.
- Keep confirmations viewport-centered despite page scrolling. Busy operations
  cannot disappear because a pending exit animation finishes.

The transparent golden emblem keeps its dark central disk. UI, favicon, executable,
installer and uninstaller use that identity without a black square backdrop or
inversion. Do not reuse a selected server image as a stretched workspace banner.

## Motion

Use the shared tokens: 90 ms instant, 120 ms interaction, 160 ms micro,
200 ms standard, 240 ms navigation and 300 ms large. The rail retains its existing
280 ms expansion, 190 ms collapse and delayed label reveal. Do not add ambient
loops, shimmers, extra entrance effects when chunks arrive or individual log-row
animation.

Transitions explain selection, navigation, real progress or list continuity.
Changed numbers and sampled lines may interpolate; unchanged initial values and
geometry do not need animation frames. Interruption retargets from the displayed
value. Invalid numeric starts recover without propagating non-finite values.
Reduced-motion lists skip measurement/cloning and release retained snapshots.

## Read, write and progress states

Keep these distinctions throughout the application:

- Unread, loading, unavailable, valid-empty and last-known data are different.
  Retain useful known rows after read failure, label them stale and provide a
  read-only retry. Do not permit mutations based on unconfirmed inventory.
- Native acknowledgement establishes a completed write. A later refresh failure
  must not turn it into a failed operation or invite repeating the mutation.
  Keep the acknowledgement and refresh recovery separate, scoped to its server.
- Polling does not replace unsaved drafts. Conflicting baselines require an
  explicit load-current decision. Obsolete previews cannot enable saving.
- Long operations show real stages and measured bytes, totals and speed. Use a
  continuous inset progress rail; never invent a percentage or duplicate it with
  decorative segmented bars. Each task owns its cancellation and error state.
- Preserve original native evidence behind bounded, keyboard-scrollable technical
  details. Translate application copy separately from names, paths and logs.

Error feedback identifies the server and attempted action, provides the reason
and offers relevant Settings/Console navigation. Navigation does not retry or
reinstall anything. A missing server disables its navigation action.

Background notices use compact End surfaces with a server name and one relevant
action. Notify successful work only outside its visible workflow or while hidden.
Failures and crashes remain dismissible without a timeout. Do not replay historical
snapshots, duplicate outcomes, steal focus or report cancellation as success.

## Library, creation and settings

The empty library offers create, folder import and modpack import through the
existing transparent End-island, world-map chest and crate artwork. Decoration
must not resemble a real server result. Small windows use compact rows.

Populated libraries keep search, state/pinned filters with actual counts,
name/recent ordering and grid/list view. Equal names remain distinct servers.
Context menus work from the item and its menu button. No-match results offer one
reset action, and import actions remain accessible.

Creation uses five numbered steps with current/completed semantics, focus following
actual step changes and navigation integrated into the form. Omit the redundant
next-step caption in the footer. The version list is bounded and scrollable with
keyboard selection and genuine catalog boundaries. Platform choices retain their
four-option native radio group. Never fabricate historical NeoForge support.

Offer an optional seed for a new world and show the exact value in final review.
Existing-world import does not offer a replacement seed. Preview original folders,
whole-family size and destination names before copying; keep source files intact.
Review split Paper families only for a Paper destination.

Server configuration places capacity and gameplay before runtime, upgrades and
automation. Keep its opaque save heading sticky. Search includes localized group
names, labels, descriptions and native property keys. Reveal matching advanced
keys; clearing search restores all fields. Reset property-add/search drafts when
changing server. Preserve unknown existing values and the advanced editor.

Show available pre-start rules without requiring an initial server launch.
Generation help distinguishes new-world configuration from existing-world edits.
Runtime observations label cached executable/version details as the last read
when checking fails. Custom-runtime acknowledgement survives metadata failure.
JVM previews belong to the selected runtime, saved RAM and effective arguments.

Application preferences use a single content surface with General, Java, Storage,
Network, Backups, Advanced and About categories. Server-specific controls name the
selected server. About uses the actual binary version and saved EULA record.
Language changes also update supported native picker and tray labels.

## Server observations and connection

Startup uses Java / World / Ready phases driven only by native events. Preparing
spawn and loading the world have distinct text. Engines may skip unobserved phases;
no timer or estimated percentage establishes readiness.

Overview keeps one inventory-style metric strip with pixel icons and real slot,
heap and CPU readings. Unknown values are not zero. Uptime remains elapsed text.
Console output stays selectable with distinct INFO/SYSTEM/PLAYER/CHAT/COMMAND/WARN/
ERROR labels. Preserve live output if historical reads fail.

Performance shows RAM and CPU together, with one 30-second/one-minute/two-minute
range control. RAM and heap are distinct, with zero baseline, visible axes and
automatic scaling that does not treat Xmx as a resident-memory limit. Keep sampled
orthogonal lines, gaps, square markers and measured peak values. The current plot
uses five quiet measurement guides. Do not add segmented meters to the plots.
Pointer and keyboard inspection show time and values above the plot. At least one
available memory series remains selected. Response/TPS and process/disk facts form
separate groups; memory interpretation and export help use disclosures.

Connection places the LAN address and its facts before an always-visible internet
section. UPnP/NAT-PMP discovery is automatic on a valid LAN interface; creating a
mapping requires explicit confirmation. Keep gateway selection, manual help and
retries accessible. There is no tunnel-provider panel. Minecraft response belongs
to Performance, and historical crash details appear only for a real recorded crash.
Read recovery clears its own router error without hiding operation failures.
Unconfirmed status blocks new openings; removal of a known mapping stays available.

## Content, players and worlds

Modrinth has four square selectors: Mods, or Plugins on Paper, Modpacks, Textures
and Shaders. Keep category, order and version filters visible. Preserve browsing
after installation. Changing type resets incompatible filters and hides stale
results. Textures/shaders open publication pages as client content, not server
mods. Modpacks lead to reviewed `.mrpack` import into a new server.

Content reviews identify exact releases, dependency changes, optional/conflicting
entries and author changelogs. Mount long changelogs only while their labelled,
scrollable disclosure is open. Keep review, publication and startup observation
separate. A completed pack import shows its server and read-only refresh recovery,
without leaving controls that can repeat installation.

Installed and disabled content remain distinct. Provide cards and compact rows,
search across declared IDs, independent diagnostic/source filters and actual
counts. Page installed results at 24 items and read icons only for the visible
page. Keep list order stable as diagnostics change. File menus belong to filename
and hash and close when their item is removed, changed or filtered away.

Details, declared Dependencies and related Messages are ordinary inspector views;
empty views fall back to Details. Pair red/amber with evidence and Console actions.
A JAR or Fabric log mention does not establish gameplay behavior or root cause.
Disabling is reversible and preserves metadata/configuration; removal retains its
confirmation and safety-copy workflow. Read-only inspection remains available
while file mutations require stopped, confirmed state.

Player groups retain actual counts, category search and 50-row pagination. Keep
same-name UUIDs distinct. Connected heads use the session-observed UUID, with a
neutral fallback if absent. Never substitute a historical same-name profile.
Known-player OP/whitelist actions use the displayed UUID and stopped-server rules,
with explicit level/bypass review. JSON whitelist import reviews exact identities;
TXT name resolution stays separate and requires running Minecraft commands.
Ban metadata preserves reason, source, dates and expiry with readable details.

Keep player action effects and inline validation beside the target. Command-sent
feedback does not imply Minecraft accepted it. Polling must not overwrite operator
permission drafts. Stopped-file actions retain acknowledged UUIDs and safety-copy
filenames even if the following read fails. IP pardon keeps its running command
and stopped-file behavior.

World rows group a Paper primary world with its actual Nether/End folders and
whole-family size. Include the family in duplication, import and both export
modes, including combined world/content export. Export and one overflow trigger
stay visible; primary-world deletion/selection are hidden. Stop-required mutations
and destructive confirmations retain their existing guards. Import/copy/export
fields wrap paths and focus the first field. Read failures do not become empty
lists or offer to repeat acknowledged writes.

Backup history precedes retention and stop-copy preferences. Keep search, reason,
actual saved engine/version, date, size and restore action readable. A newest-date
mark does not establish integrity or freshness. Restoration closes on native
acknowledgement; subsequent properties/backups/EULA/library refresh failures stay
separate with read-only recovery. Retention requires at least one deletion limit
when enabled; empty individual limits mean unlimited.

Scheduled rows show real next dates, result/error/interrupt state and explicit
paused text. Startup/login and server-selection preferences have independent
read/retry feedback. Closing with active servers uses one protected dialog for
background, stop/exit and cancel. Per-server shutdown failures stay inside the
bounded dialog; no automatic force-kill or synthetic success replaces observed
outcomes.
