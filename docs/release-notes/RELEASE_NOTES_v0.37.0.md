# TCHLauncher 0.37.0

## Players and access

- Track UUID observations from server login messages for the current process.
  Connected-player heads, details and search use that identity. Unknown sessions
  show a neutral head; historical same-name accounts remain separate.
- Add known players to the whitelist by selected UUID while stopped, with
  already-authorized status and acknowledgement that survives refresh failures.
- Remove selected operators and banned players by UUID while stopped. Preserve
  unrelated records and retain an original-byte safety copy.
- Unban IPs while stopped with the same safe file writer. Running servers retain
  Minecraft's pardon command. IPv6 spelling variants match the same address.

These features do not change Minecraft authentication or migrate player data.

## Recovery and background work

- Actionable in-app notices identify crashes and failed tasks. Successful tasks
  notify when their workflow is in the background. Duplicate outcomes and old
  startup snapshots are suppressed. Notices respect the current visual theme
  and reduced-motion preference.
- Start, restart and Java request failures show review advice, navigation and
  expandable technical evidence for the affected server.
- A completed modpack import remains completed if refreshing the library fails.
  Its recovery action refreshes data without repeating installation. Duplicate
  submissions are blocked and the completion view replaces the wizard controls.
- Creation/import show the acknowledged native server immediately and continue
  preparation after library-read failures. Application locations/About support
  read-only retry from Settings.

## Verification and limits

The existing native access-file check passed with focused IP coverage. Earlier
checks also exercised all eight durable world-publication boundaries through real
Windows child-process termination, preserving bytes and recovering repeatedly.
This does not prove power-loss recovery or pre-journal termination.

The existing Fabric lifecycle passed a controlled Minecraft 1.21.4 / Fabric 0.20.1
mrpack import with real Lithium downloads and an additional ready/command/clean-stop
session. It checked loader selection, hashes, client/optional exclusions, override
precedence, protected settings and the receipt. Complex third-party packs remain
unverified. The other real-engine scenarios were not rerun for this preview.

TypeScript, Windows production frontend builds and native compilation passed for
the source changes. Recent work used brief focused checks at the user's request;
no retained test case or dependency was added. Native UI, clean install/upgrade,
actual client identities/gameplay and physical-router Internet use remain manual
acceptance gates. The full original manager objective remains open.

## Windows download

Unsigned Windows x64 preview. The installer was built successfully with version
0.37.0 and is 6,810,346 bytes. Clean installation and upgrade
have not been exercised for this build. Linux and client launching remain deferred.

- Installer: `TCHLauncher_0.37.0_x64-setup.exe`
- SHA-256: `0d4f4159c73afd79c0beced6b97531a042ad8a329480defc4f550faeccf42b73`

