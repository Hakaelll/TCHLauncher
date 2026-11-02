# TCHLauncher: Windows scope closure

The original Windows manager scope is delivered in
[0.59.0](https://github.com/Hakaelll/TCHLauncher/releases/tag/v0.59.0).
On November 2, 2026, the maintainer confirmed the pending manual checks with:
"tu sigue las verificaciones estan hechas". This closes the final acceptance
checklist. It is maintainer-reported acceptance, not an additional agent-run test.
No required implementation or acceptance task remains open in the current scope.

## Evidence and boundaries

- Packaged source: `926a9d608c806274f8237e105ec0460bc3f897db`. Later changes are
  documentation only. GitHub's source tag and sole EXE asset match the verified
  local package. The installer is unsigned; no checksum asset is uploaded.
- Real isolated Vanilla, Paper, Fabric and NeoForge engine runs cover installation,
  managed Java, readiness, commands, graceful stops and backup/restore. Current
  0.59 Vanilla checks pass three sessions, byte-identical world restoration and
  restored-world boot through the changed process engine.
- Recorded checks also cover simultaneous servers, Minecraft upgrade rollback,
  Paper world dimensions, Fabric world/content exports, a controlled mrpack,
  custom Java/JVM profiles and interrupted world publication. Each retains its
  documented scope; these are not universal content or hardware guarantees.
- Five retained frontend cases, TypeScript, production frontend, optimized offline
  NSIS packaging, executable/setup versions and embedded icons pass for 0.59.
- The maintainer has accepted the visual design and now confirms completion of
  the pending native Windows, accessibility, installer and applicable network
  acceptance checklist. No individual device, protocol or assistive-technology
  result is invented from that general confirmation.
- The earlier other-computer incidents were already resolved by maintainer
  confirmation. They are not outstanding work.
- Local retention keeps 0.58/0.59. Guarded cleanup removed 4,317,254,952 logical
  bytes of generated files while preserving the verified packages and executable.
  File lengths are not allocated disk usage or runtime RAM measurements.

Implementation details and scoped results remain in
[Architecture](ARCHITECTURE.md), [development notes](docs/development-notes/README.md)
and [release notes](docs/release-notes/README.md). The longer pre-acceptance roadmap
and its original evidence remain available in Git history at `179b0aa`.

## Accepted areas

`DONE` combines implemented functionality, the recorded checks and final maintainer
acceptance. It does not claim that every native interaction was automated.

| ID | Status | Area |
| --- | --- | --- |
| CORE-01 | DONE | Multiple servers and persistence |
| CORE-02 | DONE | Managed downloads and Java |
| CORE-03 | DONE | Start, console, stop and restart |
| CORE-04 | DONE | Properties and capacity |
| CORE-05 | DONE | Backup and restore |
| BACKUP-STOP-01 | DONE | Copy after clean stop |
| CORE-06 | DONE | Close with active processes |
| CORE-07 | DONE | Measured telemetry |
| CRASH-DIAG-01 | DONE | Crash evidence |
| CORE-08 | DONE | Download cancellation and retries |
| JVM-01 | DONE | JVM settings |
| JAVA-01 | DONE | Custom runtime |
| LOAD-01 | DONE | Fabric |
| LOAD-02 | DONE | NeoForge |
| MC-VERSIONS-01 | DONE | Version catalog and experimental opt-in |
| MC-UPDATE-01 | DONE | Minecraft upgrades and rollback |
| MOD-01 | DONE | Mods and Paper plugins |
| PACK-01 | DONE | Modpack import |
| IMPORT-01 | DONE | Existing server import |
| IMPORT-02 | DONE | World import during creation |
| PLAYER-01 | DONE | Players and permissions |
| WORLD-01 | DONE | World operations and combined export |
| NET-01 | DONE | LAN connection |
| NET-TUNNEL-01 | DEFERRED | External tunnels |
| STORAGE-USAGE-01 | DONE | Storage readings and warnings |
| NET-02 | DONE | Internet and router mapping |
| TASK-01 | DONE | Local scheduling and autostart |
| UX-01 | DONE | Visual design acceptance |
| UX-ACCESS-01 | DONE | Keyboard and accessibility |
| PLAYER-BANS-01 | DONE | Ban metadata |
| RAM-ADVICE-01 | DONE | Local memory guidance |
| UX-TASKS-01 | DONE | Background operations |
| UX-MOTION-01 | DONE | Motion preferences |
| UX-KEY-01 | DONE | Commands and search shortcuts |
| APP-SETTINGS-01 | DONE | Application preferences |
| I18N-NATIVE-01 | DONE | English native messages |
| APP-LOG-01 | DONE | Launcher diagnostics |
| UX-02 | DONE | Server image |
| UX-03 | DONE | Automatic sidebar |
| UX-04 | DONE | Creation and settings experience |
| UX-05 | DONE | Deletion confirmation |
| REL-01 | DONE | Windows distribution |
| REL-02 | DONE | Current installer |

## Explicitly deferred or withdrawn

- Linux is deferred in [LINUX_LATER.md](LINUX_LATER.md).
- Fabric client instances and Microsoft sign-in were withdrawn for now.
- External tunnels were withdrawn: Bore removed, Playit rejected.

Further improvements are optional future work. New defects should be tracked as
specific reports rather than reopening the completed closing checklist.
