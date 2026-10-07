# Unreleased

Accumulated work through Windows 0.28.0 is packaged locally and recorded in
[release notes](RELEASE_NOTES_v0.28.0.md) and
[development notes](../development-notes/DEVELOPMENT_NOTES_v0.28.0.md).

## Interface refinements after 0.27.0

- Separate file details, declared dependencies and related console messages in
  the mod/plugin inspector, keeping long evidence out of the metadata view.
- Open related messages directly from the warning action and return to file
  details when refreshed diagnostics remove the selected evidence view.
- Add square SVG status marks and grid/list controls, refine error framing and
  preserve the End, Obsidian and Nether palettes and reduced-motion preferences.
- Skip redundant icon state updates when a cached icon has not changed.

These source changes and reversible content state management are recorded in
[0.28.0](RELEASE_NOTES_v0.28.0.md), whose installer has been generated and verified.
They are not included in the older 0.27.0 installer.
Native visual/keyboard review,
complete gameplay, physical router checks and measured real-session RAM/internet
latency improvements remain pending. Native enable/disable interaction and actual
Minecraft loading still require review.
Linux is deferred; see [the later-work plan](../../LINUX_LATER.md).
