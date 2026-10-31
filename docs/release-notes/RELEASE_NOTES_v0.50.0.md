# TCHLauncher 0.50.0

Windows x64 preview, prepared as a local build. The installer is unsigned.
This version is not a GitHub release; the published preview remains 0.48.0.

## Changes since 0.49.0

- Hide completed background tasks together, keeping failures, cancelled tasks and
  active work visible. Focus returns to the panel heading.
- Console command, copy and export failures use error coloring and alert semantics.
  Copy/export confirmations remain status messages; Minecraft output stays verbatim.
- Local list searches ignore accents and case consistently. `montana` finds
  `Montaña`, while original names and distinct server/player IDs stay intact.
  Settings, command search and menu typeahead use the same comparison function.
- Catalog queries, transfers and strict modpack downloads reuse separate HTTP
  client pools. Cancellation, progress and file byte limits remain task-owned.
  HTTPS, origin/redirect rules, hashes and existing timeout profiles are unchanged.
- Remove eight obsolete CSS declarations overridden by later rules, preserving
  effective colors and declaration order.

## Validation and limits

Scoped callback/markup checks cover task dismissal and console feedback in both
languages. Actual local filters cover accent matching, distinct identities,
unchanged data and preserved filters. The five retained frontend tests,
TypeScript and Windows frontend builds pass.

Focused native loopback checks reproduce separate connections before pooling and
verify reuse afterward. They cover cancellation isolation, SHA-256/SHA-512,
preserved destinations on mismatch, partial-file cleanup, modpack byte limits and
forbidden redirects. Original client configuration and URL guards are compared
against the previous source. No dependency or retained test is added.

These checks use controlled UI boundaries or a local HTTP fixture. They do not
establish remote-provider behavior, native visual/keyboard/screen-reader results,
measured latency/RAM gains or a Minecraft gameplay-network improvement. Manual
validation remains with the maintainer and does not block independent work.
Packaging results will be recorded after the installer is built.
