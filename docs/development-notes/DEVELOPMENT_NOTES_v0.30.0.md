# Development notes - 0.30.0

This Windows milestone includes the [operations and authentication refinements](OPERATIONS_AFTER_0.29.0.md).

## Console timeline

Client-only monotonic arrival numbers are assigned once at batch publication,
outside React state updaters. Native log timestamps, levels, messages and disk
files are unchanged. Clear stores an arrival boundary; zero is a valid cleared
boundary for an initially history-only view. A null boundary restores history.

History reconciliation matches per-server timestamp/line occurrences, retaining
live objects and their arrival numbers rather than replacing them with disk
copies. Extra repeated live occurrences are retained. Unknown disk-only rows use
arrival zero and remain hidden until Show history while the view is cleared.
This distinguishes live delivery from explicit disk reads; it does not claim to
recover missed IPC events as new arrivals while a cleared view is active.

Both paths keep the existing 1,000-line limit. Match maps and consumed sets are
bounded and local to reconciliation; there is no additional polling or retained
log cache. Search normalizes its query once per view calculation.

Four new regressions cover clock reversal, missing/legacy timestamps, duplicate
occurrences, refresh after clear, server isolation, unchanged source data and
large bursts/repeated reconciliation. All 98 frontend tests, TypeScript and Vite
production compilation passed. The native evidence is detailed in release notes.

## Package

Offline Windows NSIS packaging is in progress. Do not treat this entry as proof
of installer availability, successful installation or manual UI review.

Suggested commit: `feat: release 0.30 with console and authentication refinements`
