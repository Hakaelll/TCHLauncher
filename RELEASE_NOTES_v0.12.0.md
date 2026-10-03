# TCHLauncher 0.12.0 - Paper startup and temporary router mappings

## Fixes

- Rebuild the 0.12.0 package with the current gold emblem explicitly embedded in the
  application, NSIS installer and uninstaller. Track icon changes during native builds
  and remove the obsolete letter-T icon generator.
- Recognize Paper's `[HH:mm:ss INFO]` console formatter when Minecraft finishes
  starting, including ANSI-colored output and launcher timestamps. The regression
  test includes the exact startup line from the reported Paper 1.17.1 server.
- Keep console command input available during startup while the Java process is
  alive; stopped and stopping servers cannot submit commands through this control.
- Use the transparent End island artwork on the New Server action in populated
  libraries as well as first setup, with separate grid/list sizing and reduced-motion
  hover behavior.

## Connection

- Add explicit IPv4 UPnP router detection and confirmed temporary TCP opening to
  the Connection section. The confirmation names the local destination and port;
  native validation rejects a destination that changed before confirmation.
- Persist ownership before modifying the router, verify the returned mapping and
  renew only while the server remains running at the matching LAN address and port.
- Attempt removal after server stop, configuration changes or launcher exit.
  Pending cleanup survives restart and server deletion. Existing foreign mappings
  are preserved, and unsupported permanent-only leases are rejected.
- Show the router, external address, last check, confirmed lease expiry and actionable
  failures. Router confirmation is explicitly separate from internet reachability;
  manual port-forwarding guidance and the optional public-IP lookup remain available.

## Verification and limits

- Ten local UPnP tests cover bounded SOAP/HTTP transport, discovery restrictions,
  durable ownership, corrupt records, lost responses, cleanup retries, renewal and
  a server stopping during mutation. They do not change a physical router.
- Sixteen frontend tests pass; TypeScript and production frontend build pass.
- Offline Rust suite: 120 tests pass; 18 external-resource tests are intentionally excluded.
- Physical-router behavior, native exit/keyboard/visual review and Minecraft client
  gameplay still require manual verification. NAT-PMP and external reachability
  checks are not implemented. UPnP does not configure the operating-system firewall.
- Imported servers retain their authentication settings and player data. For direct
  authenticated connections, `online-mode=true` belongs in `server.properties`.
  A setting named `online-mode` in `spigot.yml` does not enable that authentication.

## Windows preview

The NSIS x64 installer and SHA-256 checksum are generated in
`src-tauri/target/release/bundle/nsis/`. Install this preview to test the above changes;
the previous 0.11.0 installer does not contain them.

- Installer: `TCHLauncher_0.12.0_x64-setup.exe` (5.64 MiB, 5,914,554 bytes).
- Native application product/file version: `0.12.0`.
- SHA-256: `1ee73fa06d81d3ca60483fe31c791e3e7061dbf4fa494f8176e4073ccb1302e4`.
