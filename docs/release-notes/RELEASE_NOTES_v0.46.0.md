# TCHLauncher 0.46.0

Windows x64 preview. The installer is unsigned.

## Changes since 0.44.0

- Expanded English coverage for native installation, content management,
  networking, tasks and recovery messages. Nested application errors translate
  separately while names, paths and external technical output remain unchanged.
- Native image, Java, player-list and diagnostic export pickers receive the
  current language when opened.
- Closing several servers shows separate stop, timeout and backup failures for
  each server. A failed close remains retryable; successful stops still get their
  configured safety copy.
- Console publication pauses while the application document is hidden, retaining
  the latest bounded buffer for visibility return.
- Memory reports reset at process-session boundaries, including missing-RAM
  samples and startup, to avoid mixing former sessions.
- Imported properties support colon/whitespace separators, escaped keys and
  continued logical records. Saves preserve unrelated source bytes.
- Action menus support typing names, repeated letters and accent-insensitive
  matching while skipping disabled actions.
- Fabric environment metadata normalizes supported defaults and case variants,
  and rejects invalid non-string values. Shared formatting imports are simplified.

## Validation and limits

Source checks exercise locale functions, ES/EN rendering, actual API/callback
logic and the production shutdown function with controlled boundaries. The
shutdown check's serialized failures feed the actual error component. Earlier
native checks cover property handling, memory recording and Fabric metadata.
See the development notes for their scope.

Native Windows interaction, installer lifecycle and live Minecraft sessions are
not repeated for this version. Manual validation remains with the maintainer.
This release does not establish fixes for Internet stalls, mod-specific world
behavior, rising Java RAM or imported Minehut account identity.

No application dependency or external service is added. Only the EXE installer
is attached to the GitHub release; checksums remain local.
