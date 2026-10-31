# TCHLauncher 0.45.0

Unpublished local Windows x64 preview. This build is not a GitHub release.
Packaged source commit: `121c1f3643d50e609027261e91c36f7bd4dfe47f`.

## Changes

- Console publication pauses when the document is hidden. The existing bounded
  buffer retains the latest 1,000 lines and resumes on visibility return. Native
  server logs and scheduled work continue independently.
- Memory reports discard former process sessions before timestamp or missing-RAM
  checks. Starting a server clears its chart history, preventing mixtures when a
  process identifier is reused.
- Imported properties support colon/whitespace separators, escaped keys and
  continued logical records. Saving preserves unrelated source bytes and prevents
  continuation markers from consuming the next setting.
- Shared server, world and content menus support typing action names, repeated
  letters and accent-insensitive matching while skipping disabled actions.
- Fabric environment metadata uses the supported string contract. Accepted
  default/case variants are normalized; arrays and other invalid types are
  rejected. Related environment/client-only messages are localized in English.
- Size formatting imports use their shared utility directly.

## Verification and limits

The source checks cover production console-buffer/effect callbacks, memory-session
recording, startup callbacks, keyboard handlers and property read/save preparation.
Their substituted browser/model boundaries are documented in the development
notes. A focused native test also reads synthetic JAR metadata and checks alias
providers and client-only exclusion. It does not load a real gameplay mod.

The local package passes TypeScript, the Windows frontend production build and
optimized offline native/NSIS packaging. Executable and setup versions are 0.45.0,
and their embedded icons match the application logo. Checksums remain local.

No new dependency or external service was added. The installer remains unsigned.
No native interaction, installer lifecycle or Minecraft session was repeated for
this version. The maintainer owns remaining manual validation. This preview does
not establish lower Java RAM use or fixes for Internet stalls, the reported missing
mods or Minehut account identity. Full Java-properties conformance is not claimed.
