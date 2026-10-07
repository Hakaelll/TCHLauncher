# TCHLauncher 0.15.0 - Temporary router access and server visibility

## Temporary internet access

- Add NAT-PMP as an alternative when UPnP discovery does not find a compatible router. Detect the default gateway or enter a router address manually in Connection.
- Require confirmation before requesting a temporary TCP mapping. Display the actual assigned external port and address, rather than assuming the requested port was granted.
- Renew the lease while the server remains running at the same local address and port. Attempt removal after stop or launcher exit, retain interrupted requests and failed cleanup for retry, and avoid simultaneous UPnP/NAT-PMP mappings for the same server.
- NAT-PMP requires router support and does not bypass CGNAT or alter the Windows firewall.
- Add an account-free Bore TCP tunnel directly in Connection, with explicit activation, a copyable assigned address, connection-state monitoring and cleanup after server stop or application exit. Forward traffic to the confirmed local target without installing a separate executable or changing authentication. Friends use the public address in Minecraft Java; no Bore account or client is required.
- Bore depends on bore.pub availability and forwards traffic through that service. No Playit login or integration is used.

## Storage, worlds and performance

- Show measured storage usage for worlds, mods/plugins, logs, other files and backups, plus actual disk availability and the measurement time. Persist a configurable warning threshold without imposing a disk quota.
- Track world duplication, export, primary-world changes and deletion with measured copy progress and cancellation before publication. Preserve source files, existing destinations and completed safety backups; protect the active world from deletion.
- Add an explicit Paper TPS query with 1/5/15-minute readings, timestamps and low-rate feedback. Accept recognized responses only from the current process within a bounded request window; expire stale readings instead of estimating TPS from CPU usage.

## Verification boundaries

- All 151 local native tests and 24 frontend tests pass. Nineteen external integration tests remain excluded from the offline suite. TypeScript and the production frontend build pass.
- An explicit bore.pub integration test passed: a temporary local echo listener received traffic through its assigned public endpoint and returned a reply. No Minecraft server or user files were exposed by the test.
- Local Bore tests cover bidirectional TCP forwarding, closure of control/player connections, invalid handshakes and remote disconnect detection.
- Local NAT-PMP tests cover packet validation, loopback UDP exchanges, lease renewal, assigned-port handling, interrupted replies and cleanup retries.
- Physical router compatibility, native gateway detection, Minecraft client connections, real Paper TPS responses and native visual interaction still require manual verification.
- Imported authentication settings and player data are preserved. This release does not migrate historical offline/proxy UUIDs.
- Linux distribution and the remaining original application requirements are still in progress.

## Windows preview

- Installer: `TCHLauncher_0.15.0_x64-setup.exe` (5.94 MiB, 6,232,154 bytes).
- Native application product/file version: `0.15.0`.
- SHA-256: `d3207f00c278197866dd737886f5fea2605f2eb8be2afac239234ec0add7a295`.
- Small and large embedded application/installer icons match `icons/icon.ico` using `scripts/verify_windows_icons.ps1`.
- The installer and adjacent `.sha256` file are in `src-tauri/target/release/bundle/nsis/`; the 0.14.0 preview is preserved.
