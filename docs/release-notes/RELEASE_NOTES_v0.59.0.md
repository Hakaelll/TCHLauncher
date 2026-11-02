# TCHLauncher 0.59.0

Prepared for publication. The installer is unsigned; only the EXE will be attached.
This release closes the current implementation batch and includes 0.58.0.

## Changes

- Windows retains each assigned Java process in a separate Job Object. If the
  launcher exits abruptly, Windows terminates the assigned process instead of
  leaving it running without launcher control. Normal stop still sends Minecraft's
  graceful stop command; keeping the launcher in the tray retains ownership.
- Includes the [0.58 fixes](RELEASE_NOTES_v0.58.0.md) for creation/import pickers,
  event recovery, storage actions, player search and selected-language feedback.

## Validation and limits

The actual Windows containment helper passes an isolated owner-termination case,
an unrelated-process isolation check and a normal child exit. The previous spawn
path reproduces the surviving child. A real Vanilla 1.21.4 server with managed
Java 21 passes three ready/status/command/graceful-stop sessions, byte-identical
world restoration and a restored-world boot through the changed process engine.
Pre-start seed, difficulty and game-mode checks also pass in that existing case.

The owner-termination check uses harmless fixture processes. The Vanilla case
does not exercise native windows or client gameplay. Assignment follows process
creation; interruption during that initial interval is unverified. Emergency
termination cannot guarantee a clean world save. Manual Windows interaction,
accessibility and physical-router checks remain maintainer-owned. No runtime
memory or Internet-latency improvement is claimed.

## Windows package

Packaged source: `926a9d608c806274f8237e105ec0460bc3f897db`.
Five retained frontend tests, TypeScript, production frontend and optimized offline
NSIS packaging pass. Executable/setup versions and embedded icons match. The
unsigned installer is 6,894,189 bytes. Local retention keeps 0.58/0.59 and removes
the hash-verified 0.57 setup; its published GitHub asset is unaffected. Guarded
cleanup removes 4,317,254,952 logical bytes of generated build files while retaining
the verified installers and executable. These file lengths are not allocated-disk
or runtime memory measurements. Checksums stay local.
