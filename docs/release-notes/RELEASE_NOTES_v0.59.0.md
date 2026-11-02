# TCHLauncher 0.59.0

Windows x64 preview in preparation. GitHub still publishes 0.57.0.

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
path reproduces the surviving child. Offline native compilation passes.

These checks use harmless real Windows processes, not Minecraft gameplay or native
window interaction. Assignment follows process creation; interruption during that
initial interval is unverified. Emergency termination cannot guarantee a clean
world save. Manual Windows interaction, accessibility and physical-router checks
remain maintainer-owned. No runtime memory or Internet-latency improvement is
claimed. Package and real-engine results remain pending until recorded.
