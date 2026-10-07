# World actions after 0.26.0

The world inventory now keeps Export and a named overflow button visible. All
previous operations remain connected: activate, duplicate, export, full-server
backup, open folder and delete. The context menu is also available from a world
row's right-click; inline form/input context menus retain their native behavior.

A shared ActionMenu backs server and world actions using the existing theme and
motion CSS. It clamps to the viewport, skips disabled actions in keyboard navigation,
restores focus on Escape, allows Tab to leave, dismisses on outside pointer/viewport
scroll/resize, and supports internal menu scrolling. Parent renders do not reset
focus. Server settings, pinning and deletion callbacks retain their original guards.

World file mutations stay disabled while running or busy. The primary world still
cannot be deleted here. Existing backup-before-change and destructive confirmations
remain. Confirm buttons also react if the server starts while the dialog is open.
Export and duplicate inline forms focus their first field after opening.

TypeScript and production compilation passed; all 84 frontend regressions passed.
The previous native suite passed 232 tests with 28 condition-dependent tests ignored;
no backend change was made for this UI refinement. These tests do not prove native
visual layout, pointer behavior or keyboard interaction; manual review remains open.
No new installer, release, commit or publication was generated. The 0.26.0 installer
does not contain this refinement or the subsequent performance changes.


## Installed mod/plugin actions

Installed cards and compact rows now keep Details/Notice and direct Console
navigation visible. Compatible-version review, managed-project page opening and
removal use the same named overflow button/right-click menu as worlds and servers.
Local files do not offer a fabricated Modrinth page or version lookup. Removal still
opens the existing confirmation and native backup/expected-hash workflow.

A menu belongs to its filename and observed hash. It disappears when that file
changes, leaves the filtered results, or the user switches to discovery. Modrinth
opening uses the existing validated native project command; failures are reported
in the selected manager and late outcomes cannot populate another server's view.
The new label is localized in English and Spanish.

TypeScript, all 84 frontend regressions and production Vite build passed after this
extension. Native keyboard/context-menu/browser dispatch and visual assessment
remain unverified. Enable/disable content is still a separate unfinished operation;
this menu does not expose an inactive placeholder for it.


## Packaging follow-up

The source-only statements above describe their original iterations. This work is
now included in the locally generated Windows 0.27.0 milestone; see its
[development record](DEVELOPMENT_NOTES_v0.27.0.md). Native/gameplay verification
limits remain. The older 0.26.0 installer remains unchanged.
