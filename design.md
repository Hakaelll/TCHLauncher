# TCHLauncher - End / Obsidian visual system (0.15.1)

## Authority and use

### Operation and startup refinement

Startup uses a connected three-phase track for Java, world and ready. Only real
observed phases advance the track; no estimated percentage is shown. Completed
phases use a crisp check mark and success color; current phases use the theme
accent. Phase changes have short color transitions with reduced-motion support.
Downloads use inset continuous rails with actual bytes, speed and percentage.
Task headings separate their title from a small semantic status mark; completion,
cancellation and errors keep explicit text. Controls wrap on narrow windows.
Operations own these styles in operations.css rather than duplicate overrides.
Native visual review remains manual.

### Backup history refinement

Backup history precedes stop-copy and retention preferences. The primary create
control shares a wrapping header with refresh. Search and reason filtering form
one compact toolbar; visible totals come from the current list. Backup rows use
an authored square file mark, readable date, reason, saved engine/version and a
separate size/action column. The newest recorded date receives a small labeled
accent mark; it does not claim integrity or freshness of a failed refresh.
Narrow layouts place the description above size and restore. Restore keeps the
existing confirmation, stale-read and stopped-server guards. List movement uses
the shared interrupted/reduced-motion handling; restore arrows move two pixels
on hover. Empty and no-match states offer clear recovery. Native visual review
remains manual. Backup-only styles live in the deferred backups.css entry.

### Performance instruments

RAM and CPU remain simultaneous sampled plots in standard desktop windows, with
one shared 30-second / one-minute / two-minute range control. Memory fits measured
RAM and heap with visible axis values, rather than treating Xmx as a resident RAM
limit. Keep zero as the baseline and label automatic scaling. Real process and
heap series have independent pressed legend controls; at least one available
series remains visible. Preserve gaps, unknown values and square single-sample
markers. Pointer/keyboard sample readings sit above the plot, without covering it.
Use quiet square framing, five restrained measurement guides and non-scaling line
strokes. Plot height follows the window. Java interpretation/report explanations
use disclosures; response/TPS and process/disk facts form separate reading groups.
No new polling or ambient animation accompanies this layout. Native visual review
remains manual.

### Catalog composition refinement

Modrinth discovery uses four raised category selectors with flat pixel SVG marks,
concise role labels and a persistent selected state. The search and native filters
share one workbench. Project artwork occupies a larger inset slot; names, authors,
descriptions and tags precede a separate compatibility/action footer. Status uses
a small square and explicit text rather than a thick colored stripe. Download
counts remain actual catalog values. Search loading shows static, decorative
placeholders alongside the existing announced status, with no simulated progress
or continuous shimmer. Category icons move two pixels on hover/focus and respect
reduced motion. Two/three-column results reflow to one column on small windows;
all catalog controls and installation reviews retain their existing behavior.
Native appearance and keyboard verification remain manual.

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

## Connection and inventory monitoring (0.18.0)

Connection groups the LAN address and its facts before an always-visible internet section. UPnP and NAT-PMP discovery run automatically when a valid LAN interface is observed; explicit confirmation still owns all port creation. Retry and advanced gateway selection remain available. There is no Bore panel or production provider/IPC. Minecraft status response belongs to Performance. A historical crash disclosure appears only after a real saved failure has been loaded.

Overview uses one inventory-style metric strip: authored pixel icons, display typography, square inset segments and theme/semantic colors. Player segments measure occupied slots; memory segments show measured used heap relative to the configured Xmx, separately from resident RAM. CPU segments measure process CPU. Missing values are labelled and never rendered as measured zero. Uptime remains elapsed text with no invented gauge.

Performance keeps RAM and CPU together. Square framing, pixel icons, orthogonal sampled lines and square hover points match the controls; dashed heap remains distinct from resident RAM. Lines preserve sampling gaps. Download, copy and storage bars retain native progress semantics, real totals and cancellation, with square segmented fills. Motion remains brief and respects reduced-motion preferences. Native visual review remains the user's manual check.

## Mod inventory and chart refinement (0.19.0)

Use square item cards, inset icon wells, restrained category tags and a release status footer in Modrinth. Keep category, sort and version filters visible, preserving browsing after installation. Color signals are paired with text: compatible, experimental, unavailable and unverified. Local inventory separates descriptor warnings from historical Fabric startup-list evidence.

RAM and CPU remain simultaneous orthogonal plots, with lightly shaded measured areas, three quiet horizontal guides, compact peak readings and a dashed heap legend. Remove the duplicate segmented meters from these plot cards. Pointer or keyboard sample inspection shows measured values and time; gaps and unavailable readings remain explicit. Keep network controls visible, placing lengthy instructions in optional help or the confirmation dialog.

## Installed mod inventory

Active and disabled content have distinct inventory views. Disable is a direct,
reversible file operation from the context menu; Enable is visible in the disabled
list. No extra confirmation interrupts this reversible action. Native stopped-
server guards, safety copies, staged publication and known dependency checks own
the change. Disabled files retain their original project metadata and use the
normal `.jar.disabled` suffix. Loading/read errors never become an empty list;
completed changes remain acknowledged if a subsequent inventory refresh fails.

The file inspector uses three ordinary view controls: Details, declared
Dependencies and related Messages. Only views with actual data are offered.
The warning action opens related messages directly; refreshed diagnostics fall
back to Details if the selected view becomes empty. Metadata, dependency ranges,
warnings and console recovery actions remain available. Square SVG status marks
and view icons use the existing theme. Error evidence uses a one-pixel frame and
a semantic label instead of a thick side stripe.

Use responsive square item cards with 64-pixel icon slots, readable version/origin metadata, compact semantic status and separate detail/version/remove actions. Red and amber always include a text label and evidence. Put long filenames, declared requirements and related console messages in the detail dialog; keep search and attention/source filters visible. Never imply a JAR in the inventory proves world behavior or blame a mod from a generic log mention. Keep item order stable while diagnostics update. Use short opacity entrance and a two-pixel hover lift on the icon only; respect system/application reduced motion. Preserve filters during audit refreshes.
# Installed content refinements

Installed mods/plugins offer inventory cards and a compact presentation. Keep file icons, explicit diagnostic labels and semantic red/amber states in both views. Provide name/problem ordering and direct console navigation without treating a log association as a root-cause diagnosis. State transitions use existing motion tokens and respect reduced motion; compact rows reflow on narrow windows.

## Creation version controls

Show Minecraft versions in a bounded native list with scrolling and a catalog count. Keep keyboard selection and genuine catalog boundaries explicit, including historical NeoForge 1.20.1 and the modern catalog from 1.20.2. Platform choices use a four-option native radio group with raised theme surfaces, checked text/radio state and visible keyboard focus. No ambient animation or new accent palette is introduced. File details expose dependency declarations in a named inspector view with text-backed kinds and preserved range alternatives.

## Creation and import finish

Creation uses five raised inventory slots with explicit current-step semantics,
numbered upcoming steps and checkmarks on preceding steps. The form has a distinct
content surface with navigation actions integrated into the form. The configuration preview uses the
existing transparent End-island illustration as decoration, separate from the
chosen server image; it disappears at narrower widths without hiding form fields.
World and import-location choices retain native radio behavior, selected material
and visible focus. Focus follows actual step changes, including going back to the
first step. Existing navigation motion remains authoritative; no ambient animation
or additional image downloads are introduced. Import paths wrap rather than being
truncated. Native visual assessment remains manual.

## Existing property controls

Place a compact properties-read status above capacity and property editing. Explicitly
distinguish loading, unavailable, loaded and unsaved states. Hide these controls before
the first successful read; retain but disable last-known controls after a read failure.
Retry uses a normal themed button, and automatic reads never replace an unsaved draft.
RAM/storage edits also retain their current values during metadata refresh. Keep Java,
task and maintenance actions independent of properties loading. Acknowledged writes
remain successful when a subsequent server read fails; restoration may replace a draft
only after the native manager has actually restored the files.

Settings separates identity, rules, simulation limits, generation and access.
Headings retain display lettering; descriptions and values use the reading face.
Each group reports its available option count. Only actual property keys receive
controls, preventing removed version-specific settings from appearing by default.
Unknown existing choice/boolean values remain visible and editable. Generation
help distinguishes settings for a new world from modifying an existing world.
The advanced editor remains available for properties without dedicated controls.

## Player action form

Player management presents the selected action, actual entered target and a short
effect preview before sending. Named action buttons replace generic Apply; blocking
and kick controls use the existing destructive palette. OP/IP-block effects are
explicit without adding confirmation dialogs for reversible commands. Use native
fields, associated inline errors and first-invalid-field focus. A busy fieldset
prevents edits during submission. Preparing an action while stopped remains possible,
but sending requires a running server. Command-sent feedback never implies Minecraft
has accepted or completed the action. Dense identity/IP text stays readable.

## Backup retention and restore details

The backup list distinguishes loading, unavailable, valid-empty and last-known data.
Keep last-known rows visible after a failed refresh, label them as last-known and
disable restore while the list is refreshing or unavailable. Recent copies on Overview
share these status semantics. Backup dates use accent lettering with tabular numerals;
size and metadata stay readable. A restoration acknowledged by the native manager
closes the destructive dialog. Its later data synchronization lives in an inline,
server-specific attention strip with a read-only retry, protecting settings until
fresh properties/server metadata are available. Do not offer to repeat restoration
because a follow-up query failed.

Retention uses a compact disclosure with explicit enabled/unavailable text and End
material edges. Keep limits together on a deep surface, with named actions, associated
errors and first-invalid-field focus. Empty limit fields mean unlimited; enabled
automatic deletion requires at least one actual limit. Refresh retains unsaved edits.
Review lists show native copy dates, sizes and recorded versions with readable text,
not synthetic previews. Acknowledged deletion closes its confirmation before list
refresh; a later read failure remains separate. Restoration metadata uses a deep
surface and changes from two columns to stacked labels at narrow widths.

## Scheduled tasks

Use raised inventory rows and pixel clock slots for scheduled work. Put the current
state beside the action, the next date beneath it and the recorded result in its own
deep surface. Errors and interruptions use explicit labels as well as semantic color.
Start requests never imply readiness. Keep neutral paused rows readable, manual refresh
available after read failure, and an empty state pointing to the scheduling action.
Automatic restart settings share these materials. At narrow widths, keep row controls
under the content and wrap headings; do not add decorative progress meters.

Automatic restart refresh lives in a compact footer with inline unsaved/saved feedback.
Initial read errors show an unavailable state and retain a retry action. Observed
policy changes update a clean form, never an unsaved draft. An acknowledged write
and a failed subsequent read are distinct; hide previous attempt data until a fresh
snapshot is available. Integer errors stay beside their fields with focus on the
first invalid control. These states use existing semantic colors and reading text.

## Player list navigation

Keep category counts based on the full native snapshot. Search the chosen category
by name, UUID or IP; paginate matching entries in their original order, 50 per page.
Do not merge offline/account identities with the same name. Search resets the page,
and a shrinking snapshot clamps it to an existing page. An unavailable snapshot is
labelled separately from a valid empty list. Search, result count and page buttons
use the existing form and Minecraft control tokens. List motion tracks visible rows
rather than serializing the entire history on each search keystroke.


### Independent read feedback

Console history and player lists use selected-server read ownership and compact
inline feedback. Preserve live output and last-known players after read failures;
never describe an unavailable snapshot as empty or authenticated. Overview marks
stale player totals. Reading errors use a warning inventory marker, readable body
text and an explicit retry; no animated loader or modal. Player totals use squared
End slots, raised edges and tabular values with the bundled display font.


### Runtime observation versus persistence

Never show a retained ready Java result as current after an observation failure.
Use a labelled warning state and retain executable/version details as a last read.
Unknown compatibility means only the version was verified. Acknowledged custom
runtime changes close confirmation and keep an inline success message even if
later server metadata reads fail. Error detail and a direct check action remain
visible; status markers never rely on color alone.


### JVM profile review

Do not label the profile Recommended/Custom before successful reading. Keep reload
separate from editing and disable it while a draft is unsaved. Launch previews
belong to the selected server, runtime, saved RAM and effective arguments; obsolete
previews are neither displayed nor sufficient to enable saving. Radio choices use
square End surfaces, explicit selection and visible focus. Preview command text
wraps in a deep reading surface. Save acknowledgement remains inline.


### Deferred sections

Load secondary section code when opened. The shell stays interactive. During load,
show only the section name and a short status in an End reading surface; do not
invent a percentage or animate a shimmer. A failure has a local retry and optional
technical detail. Keep modal entry/focus behavior synchronous. Navigation keeps
its existing motion and reduced-motion preference; chunk arrival adds no second
entrance animation.


### Update acknowledgement and catalog recovery

Close a loader confirmation after native publication succeeds. Keep a readable
installed-target acknowledgement and native warnings; metadata refresh failures
get a separate, read-only recovery action. Do not offer to repeat publication as
recovery for a failed read. Version catalogs distinguish unavailable from truly
empty, with a direct retry inside review. Keep loading, publication and subsequent
Minecraft startup observation distinct; success never certifies gameplay mods.


### Operator drafts and action follow-up

Polling must not overwrite unsaved operator permissions. A changed native baseline
uses a labelled conflict with current values and an explicit load-current action;
editing remains blocked until reviewed. Keep UUIDs independent. Generic mutation
acknowledgement and subsequent read failures have distinct feedback, with a read-
only recovery action. Avoid suggesting repeated commands to recover stale data.

## Library browsing refinement

Keep the current End imagery and creation/import actions. Populated libraries expose
All, Online, Errors and Pinned as text-backed filter buttons with native counts;
search combines with the selected filter. Explicit name/recent ordering uses copied
rows, retaining native library order by default and keeping equal-name servers distinct.
Square, quiet card footers separate capacity metadata from the open/menu targets;
pinned servers retain a compact icon and text. An empty filtered result offers one
reset action. Import actions remain visible below the inventory. Context menus work
from the card as well as its menu button. Stable item keys preserve existing list
continuity; hover arrow movement is short, non-repeating and respects reduced motion.
Keyboard controls and narrow layouts remain available; native visual review is pending.

## Startup preference recovery

Application autostart and individual server startup remain independent sections.
Each exposes its own read/retry state; a failed read retains last-known values with
explicit text and disables edits until confirmed data is available. Publish only
native acknowledged settings and invalidate pre-write reads. A failed follow-up
query does not turn a persisted preference into a failed write. Server rows use
square icon slots, legible names and Selected/Manual text beside native checkboxes.
Scheduling closes creation on native acknowledgement and reports list recovery
separately; no animation claims a task has executed. Native visual review is pending.

## Application close feedback

Keep one protected viewport-centred confirmation for background, graceful stop/exit
and cancellation. Aggregate shutdown failures remain inside this dialog, with
readable line breaks and a bounded scrolling reading area, instead of dismissing
the dialog and placing errors behind it. Requests disable all choices until their
native outcome; an acknowledged exit remains busy through cleanup. No generated
progress, automatic force kill or success animation replaces observed process
state. Native window/tray/focus review remains pending.

## Installed inventory workbench

Keep log-state filters independent from Modrinth/local origin, with real counts,
pressed-state text and result totals. Search includes all declared IDs; equal
names remain separate files. Group ordering and card/compact presentation with
search rather than another navigation bar. Neutral slots frame local icons;
warning and error colors belong to evidence, not inferred gameplay compatibility.
Explain temporarily disabled file mutations once above the inventory. Details
and log inspection remain available. Reduced-motion behavior and existing themes
remain in force; native visual review is still pending.

## World setup and continuation

World generation owns an optional seed field beneath the new/existing choices.
Use the reading face for seed text and exact numerals; show its full value in the
final review with safe wrapping. Existing-world import does not offer a new seed.
Continuation shares the form material and spacing, with a next-step label and
paired previous/continue controls; it is not a detached dark strip. Default rules
are editable before Java's first start, retaining real imported values. Native
visual and keyboard review remain pending.

## Content divisions (0.25.0)

Modrinth discovery has four labelled square controls: Mods (Plugins on Paper),
Modpacks, Textures and Shaders. Selected materials, authored pixel glyphs and short
color/background transitions use existing theme tokens and reduced-motion rules.
Changing type clears incompatible category and pagination filters; stale results
are hidden before rendering a new type. Shader/texture results describe published
game versions rather than server compatibility and open official publication pages
in the default Windows browser. Packs lead to the existing .mrpack import review
for a new server. No client launcher is added.

Installed item cards give the real mod icon a larger inset slot, separate version
metadata, explicit diagnostic status and a grouped action footer. Compact view,
filters, read failures and console evidence remain available. The wizard footer
omits the redundant next-step text; the step navigation above remains authoritative.
Native appearance and keyboard review remain pending.


## World action hierarchy

Keep Export and one labelled overflow trigger visible beside each world. Group
primary-world selection, duplication, full-server backup, folder opening and deletion
in the existing End context-menu material, available by right click or the button.
Hide primary-world deletion/selection; disable file mutations while running or busy.
Confirm destructive actions with the existing viewport dialog. Arrow/Home/End keys
navigate enabled actions; Escape restores the trigger, Tab leaves the menu, and
outside interaction dismisses it. Keep menus inside the viewport, allow internal
scrolling, and avoid refocusing on ordinary parent data refresh. Inline duplication
and export focus their first field. Native visual/keyboard validation remains manual.


## Installed content action hierarchy

Keep Details/Notice and a related Console action visible. Use the shared End overflow
menu for compatible-version review, View on Modrinth (managed projects only),
disabling and removal. Keep the removal confirmation and backup workflow. Context menus belong
to the observed filename/hash and close on a changed file, filtering it away or
switching to discovery. Read failures/running state still block file mutations;
read-only details and publication inspection remain available. New copy is localized.
Native review remains manual.


## World inventory feedback

Keep read failures distinct from a genuine empty world list. Retain last-known
rows with an unavailable label and retry, and block file mutations while reads
are pending or failed. Manual Refresh worlds remains beside Import world; retain
readable feedback and no fictitious progress. Post-operation read failure does
not revoke the native acknowledgement. Native validation remains manual.
