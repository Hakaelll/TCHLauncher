# Unreleased

Accumulated work through Windows 0.27.0 is packaged locally and recorded in
[release notes](RELEASE_NOTES_v0.27.0.md) and
[development notes](../development-notes/DEVELOPMENT_NOTES_v0.27.0.md).

## Interface refinements after 0.27.0

- Separate file details, declared dependencies and related console messages in
  the mod/plugin inspector, keeping long evidence out of the metadata view.
- Open related messages directly from the warning action and return to file
  details when refreshed diagnostics remove the selected evidence view.
- Add square SVG status marks and grid/list controls, refine error framing and
  preserve the End, Obsidian and Nether palettes and reduced-motion preferences.
- Skip redundant icon state updates when a cached icon has not changed.

These source changes are not included in the existing 0.27.0 installer.
Native visual/keyboard review,
complete gameplay, physical router checks and measured real-session RAM/internet
latency improvements remain pending. Content enable/disable is not implemented.
Linux is deferred; see [the later-work plan](../../LINUX_LATER.md).
