# TCHLauncher 0.48.0

Windows x64 preview. The installer is unsigned.
Packaged source commit: `382aca1b4b645c0952cc5c485d33e4186dc5d95e`.
Published at the maintainer's request to release the version closing this batch.
Only the EXE is attached; checksums remain local.

## Changes since 0.47.0

- Initial numeric values and chart lines skip unchanged animation frames.
  Reduced-motion lists avoid geometry reads and cloned snapshots. Changed
  readings retain transitions and interruption handling.
- Console following skips scroll animation when already at the target. Pending
  lines are counted without allocating another filtered list.
- Download reads reuse response buffers instead of making a second owned copy
  of every chunk. Byte limits, verification, cancellation and staged publication
  remain in the existing consumers.
- UPnP and NAT-PMP status-read errors clear after recovery without hiding failed
  operations. Unverified status blocks new port openings and pending confirmation;
  known mappings remain removable.
- LAN and NAT-PMP copy feedback belongs to the actual address and newest request.
  A changed address cannot inherit Copied or a clipboard error from the previous
  address. Public-IP and router-operation errors remain independent.
- The design guide consolidates current rules and removes duplicated iteration
  narration. The roadmap separates implemented paths from remaining acceptance.

## Validation and limits

Scoped component/effect checks reproduce the previous idle animation work,
sticky router errors and stale Copied labels. Controlled hooks, DOM, timers,
clipboard and network-status boundaries cover recovery, disposal, interruptions
and Spanish/English feedback.

Production download transfers against a loopback HTTP server pass SHA-1/256/512,
no-hash, mismatch, cancellation and partial-cleanup checks. The accepted test socket
uses blocking mode to prevent its original Windows fixture race. Full native test
compilation passes; temporary checks were removed and retained tests are unchanged.
TypeScript, Windows frontend, optimized offline NSIS packaging, executable/setup
versions and embedded-icon checks pass. The installer is 6,904,819 bytes.

After packaging, the existing current-source Vanilla 1.21.4 scenario passes real
managed downloads and three ready/status/command/graceful-stop sessions, followed
by tracked backup, exact world restoration and restored startup. It uses an
isolated loopback library and an existing explicit EULA record; temporary data
is removed and the installer/executable remain unchanged. Test events replace
the Tauri UI.

The existing Paper 1.21.4 scenario also passes backup/restore and a populated
three-dimension folder export/import. All 45 copied file hashes match before
boot, three saved blocks are verified after boot and source/export stay unchanged.
Five real sessions return TPS readings. Isolated data is removed and package
hashes remain unchanged. The Vanilla scenario additionally verifies seed 42,
hard difficulty and creative mode saved before the first world exists.

No measured CPU/RAM improvement, native scrolling/clipboard interaction, physical
router result or new multiplayer gameplay is claimed. Affected-host Internet
stalls, Java RAM growth, mod behavior and imported account identity remain open.
No dependency or external service is added.
