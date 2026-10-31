# TCHLauncher 0.57.0

Windows x64 preview, grouping the 0.55, 0.56 and 0.57 milestones.
Published Windows preview. The installer is unsigned; only the EXE is attached.

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

## Windows package

Packaged source: `9c7bd682b25768994b46b5ae8cb7d7ce6cdf037b`.
Five retained frontend tests, TypeScript, the production frontend and optimized
offline NSIS packaging pass. Executable/setup versions and embedded icons match.
The installer is 6,895,327 bytes. Local retention keeps 0.56/0.57, with checksums
kept locally. Protected cleanup removes 2,151,068,244 logical bytes of generated
files and verifies unchanged executable/installer hashes. These are file lengths,
not allocated-disk space or runtime memory measurements.
