# TCHLauncher 0.58.0

Local Windows x64 preview. This milestone is not published on GitHub.
GitHub retains preview 0.57.0; the usual publication cadence remains unchanged.

## Changes since 0.57.0

- Creation and existing-server import prevent overlapping image/world/folder
  pickers and ignore results after leaving the view. Creation waits for selection
  before changing steps or submitting. Cancellation and failed selection permit
  retry without replacing the chosen data.
- Stop-backup, automatic-start results and server artwork retry failed event
  registrations. Recovery refreshes missed data through existing readers.
- Storage measurements and warning saves share immediate operation ownership,
  preserving acknowledged settings and recovery after failure.
- Player search processes only the active tab. Ban presentation records and
  membership indexes are built only where used, preserving paging and identities.
- Minecraft-update blockers, new-world summaries and accessible creation-step
  labels follow the selected language. Technical details and imported names remain
  unchanged.

## Validation and limits

Scoped actual callback/effect/expression checks cover selection, cancellation,
recovery, paging, acknowledgement and ES/EN behavior with controlled native
boundaries. Opaque shared-theme text roles pass 168 token contrast comparisons;
the accepted palette is unchanged. Actual NAT-PMP transport passes owned loopback
UDP exchanges without contacting or changing the router.

These checks do not establish native dialog/keyboard behavior, full accessibility,
physical-router operation or Minecraft client gameplay. No measured runtime RAM,
CPU, frame-rate or Internet-latency improvement is claimed. Remaining manual
validation stays maintainer-owned and nonblocking. No retained test or dependency
is added.

Packaging evidence will be recorded after the Windows build finishes.
