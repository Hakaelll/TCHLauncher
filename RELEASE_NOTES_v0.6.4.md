# TCHLauncher 0.6.4

## Desktop redesign

- Introduce a dark server image rail, a light section navigator and a clean white workspace based on the supplied desktop reference.
- Use anthracite for primary controls and selection; keep semantic colors for server states, errors and console categories.
- Add a server library with functional search, grid/list views, cover images and contextual actions.
- Present create/import choices directly in the empty library.
- Preserve original server artwork for large covers while generating the same 64×64 icon used by Minecraft; publish both with rollback on failure.
- Refresh artwork after a change and clean up managed cover files when deleting a server.
- Restyle the console with an anthracite reading surface and distinct category labels.

## Motion

- Centralize duration and easing tokens for both JavaScript and CSS.
- Move persistent selection indicators between servers, sections and wizard steps.
- Add directional wizard transitions, list repositioning and coordinated removal.
- Interpolate metric values and chart lines briefly without inventing missing data.
- Add restrained menu, tooltip, dialog and control feedback.
- Smooth short console scroll updates and stop following when the user interacts with the log.
- Respect reduced motion and remove generic animations applied to every section.

## JVM profiles

- Add per-server recommended and custom JVM profiles with a launch argument preview.
- Store profiles in SQLite and apply them to start/restart.
- Validate options with the installed Java before saving custom profiles; rejected settings preserve the previous profile.
- Keep RAM and loader entrypoint arguments under launcher control; preserve spaces by passing options directly to Java.
- Allow restoring defaults. The recommended HotSpot profile uses G1 and parallel reference processing, documented in the [Java command reference](https://docs.oracle.com/en/java/javase/21/docs/specs/man/java.html).

## Validation

- Frontend type checking and production build.
- 35 local Rust regression tests, including original artwork preservation and Minecraft icon generation.
- Real Java 21 checks for recommended/custom profiles, persistence and invalid-option rejection; recommended flags also checked with Java 8.
- 6 telemetry tests pass, including unavailable samples and sampling gaps.
- Desktop visuals, accessibility traversal, installation, Minecraft client connection and frame timing remain manual checks.

Original application requirements including NeoForge, mod management and other pending modules remain tracked in NEXT_TODOS.md.

## Windows installer

- File: `TCHLauncher_0.6.4_x64-setup.exe`
- Size: 4,881,998 bytes
- SHA-256: `B62BAE662E71B2070148D1988912768C807E82CCC16912C940FA54BAAC5EFB55`
