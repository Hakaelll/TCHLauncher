# TCHLauncher 0.57.0

Windows x64 preview, grouping the 0.55, 0.56 and 0.57 milestones.
Prepared for publication. The installer is unsigned; only the EXE will be attached.

## Changes since 0.54.0

- Server and world inventories support search and pages of 50 entries, preserving
  filters, layouts and actions. Library and installed-content name ordering follow
  the selected language.
- Console command history survives navigation for the current app session,
  keeping up to 50 acknowledged commands per server. Search no longer depends on
  the Windows language's lowercasing rules.
- NeoForge discovery, installation and update reviews support annual 26.x
  versions. Deobfuscated Minecraft releases can install without obsolete mappings;
  older releases retain their verified mapping preparation.
- Event subscriptions retry failed registration and reconcile missed state on
  recovery. Late library reads preserve newer process states and acknowledged
  creation, rename or deletion.
- Settings, backup, restore and general actions reject duplicate operations
  immediately. Cancellation remains retryable when declined or unsuccessful.
  Modpack and update panels recover progress from the shared task stream.
- Heap-only charts show heap values and peaks. Unchanged task snapshots and
  backup inventories reuse existing data; superseded CSS declarations are removed.
- Confirmed active UPnP mappings offer a copyable external address. Scheduled
  task creation atomically enforces the per-server limit under concurrent writes.

## Validation and limits

The isolated real Minecraft 26.3 / NeoForge 26.3.0.100 lifecycle passes with managed
Java 25: three status/command/graceful-stop sessions, byte-identical world restore
and a restored-world boot. Parser replay covers 110 annual stable catalog entries.
Focused actual helper/component/callback checks cover inventory paging, locale,
command history, recovery and duplicate-action ownership. Controlled boundaries
do not establish native keyboard, focus, router or rendered UI behavior.

No new Minecraft client session, installed-mod behavior or physical-router test
is claimed. No measured CPU, RAM, frame-rate or Internet-latency improvement is
claimed. Pending manual validation stays maintainer-owned and nonblocking.

Packaging evidence will be recorded after the Windows build finishes.
