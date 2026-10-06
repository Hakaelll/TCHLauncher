# Unreleased

Accumulated development through Windows 0.24.0 is packaged and recorded in
RELEASE_NOTES_v0.24.0.md and DEVELOPMENT_NOTES_v0.24.0.md.

Native visual/keyboard review and actual Minecraft gameplay remain pending.
Linux is deferred by user request; see LINUX_LATER.md.

## Source changes after 0.24.0

- Preserve an unchanged imported world seed, including its original escapes, when saving unrelated settings. Escaped control characters in an existing seed no longer block those edits.
- Independently verified seed decoding with Java 8, 21 and 25 using `java.util.Properties`: signed 64-bit limits, Unicode, spaces, backslashes, literal Unicode escapes, punctuation and an empty seed retain their exact values. Authentication stays enabled in the fixture.
- Seven focused native properties tests pass. The Java integration test passed separately on all three runtimes. These checks do not generate a Minecraft world or verify gameplay. This small follow-up is not included in the existing 0.24.0 installer.
