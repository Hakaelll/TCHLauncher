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

## Imported-server authentication diagnostics

The player snapshot now includes read-only Paper/Spigot configuration evidence.
Paper checks settings.bungeecord in spigot.yml and the version-appropriate
Velocity and PROXY protocol settings in paper.yml or config/paper-global.yml.
Modern year-based Minecraft versions also use the current Paper configuration.
Other loaders do not infer proxy support from these files.

Parsing uses the existing yaml-rust2 dependency with byte, event and depth limits.
Duplicate keys, aliases, unsupported types and unreadable files remain unknown.
Nested key segments are compared individually; a literal dotted key cannot
impersonate the nested setting. Only file names, setting paths, booleans and
fixed diagnostic messages cross IPC. Proxy secrets and arbitrary YAML contents
are never returned. Reads do not alter configuration or player data.

The player view separates whitelist policy from authentication configuration,
lists enabled or unknown proxy checks and explains restart requirements. A
proxy-enabled or unknown configuration no longer uses online-mode alone to pick
an account avatar from an ambiguous cache. Spanish and English copy are included.

These are saved-file observations, not confirmation of a running server's
effective configuration, account authentication, plugins/mods, JVM overrides,
proxy routing or skin downloads. The reported Minehut case remains unverified
without the host's configuration and a real reconnect. Inventories and permissions
associated with separate UUIDs remain separate.

Verification: all 94 frontend tests, TypeScript and production Vite compilation
passed. The offline native suite passed 242 tests with 28 conditional tests
ignored. All five final authentication fixtures passed separately, including the
two added checks for configuration locations and YAML work limits.
Native UI inspection and real imported-server reconnection remain pending. No
installer, commit or publication was generated for this addition.

Configuration references: [Paper global configuration](https://docs.papermc.io/paper/reference/global-configuration/)
and [player information forwarding](https://docs.papermc.io/velocity/player-information-forwarding/).

Suggested commit: `feat: diagnose imported server authentication and proxy settings`
