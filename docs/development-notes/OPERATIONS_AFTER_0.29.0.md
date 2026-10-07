# Operations refinements after 0.29.0

These source changes follow the packaged Windows 0.29.0 preview. Its existing
installer does not contain them.

- Selection indicators ignore inert, hidden animation copies. Removal copies
  lose their current-selection attributes; a scoped mutation observer updates
  the indicator when live targets disappear. Both observers disconnect on cleanup.
- Console categories have visible pressed filters and counts for the searched
  in-memory view. The newest 1,000-line retention limit is unchanged.
- Paused following has an explicit jump-to-latest control. Keyboard scrolling
  pauses following without overriding native scroll keys.
- Up/Down recalls up to 50 acknowledged commands from the current server session
  and restores the unsent draft. Recalling never submits a command. Late async
  completions cannot put notices or command history on a different server.
- Transfer bars use a continuous inset rail with pixel relief, separate real
  percentage and transfer readings, and theme accents. Unknown totals remain
  indeterminate; there is no simulated percentage or recurring decorative timer.
- Removed UI_DESIGN.md, which mixed obsolete light/anthracite guidance with the
  current End design, and the obsolete unreleased summary of already shipped work.
  design.md remains authoritative. Release history, legal notices, deferred Linux
  work and implementation evidence are retained.

TypeScript and all 92 frontend tests passed, including four command/count
regressions. Production Vite compilation passed after allowing access to local
pnpm dependencies outside the sandbox. Native layout, pointer and keyboard review
remain manual; these checks do not establish rendered appearance or gameplay.
The native server engine was unchanged and its suite was not rerun.

Suggested commit: `fix: clear stale sidebar selection and refine console progress UI`
