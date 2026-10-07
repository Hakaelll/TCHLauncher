# TCHLauncher 0.18.0 — Connection and Minecraft-style monitoring

## Connection

- Remove Bore from the interface and production backend, including its commands, provider and maintenance. No replacement tunnel or account requirement is introduced.
- Discover UPnP and NAT-PMP automatically when a valid LAN interface is available. Keep explicit retries and advanced gateway selection. Port creation still requires confirmation and a running server; UPnP refreshes discovery when confirming an opening.
- Keep internet access visible beside its router controls and manual-forwarding guidance. Retain LAN address copying, optional public-IP lookup, temporary lease renewal and cleanup.
- Move Minecraft status response to Performance. Show historical failure review only when a saved failure exists.

## Minecraft interface

- Redesign Overview metrics as inventory compartments with authored pixel icons, Minecraft typography and measured segmented bars for players, heap and CPU. Keep resident RAM distinct from used heap; uptime has no fabricated gauge.
- Give the simultaneous memory/CPU charts square inventory frames, pixel icons, orthogonal sampled lines and square hover points. Preserve dashed heap, unavailable values, sampling gaps and reduced motion.
- Style download, installation, copying and storage progress with square segmented native bars. Measured totals, transfer speed and cancellation are retained.

## Verification

- TypeScript, the production frontend build and all 27 frontend tests pass.
- Ten UPnP and three NAT-PMP manager regressions pass, covering confirmed leases, destination ownership, renewal, cleanup and failures. These do not constitute a physical-router or UI interaction test.
- Windows NSIS x64 packaging completes; native file/product version is `0.18.0`. Embedded small/large icons match the provided logo in both application and installer.
- Bore relay and RPC strings are absent from the production application/frontend artifacts.

## Windows preview

- Installer: `TCHLauncher_0.18.0_x64-setup.exe` — 6,323,452 bytes (6.03 MiB).
- SHA-256: `fcb65159da3c4a7c29588d304968d3b19807b9062a86d794a2b36d535a12a0a1`.
- The installer and adjacent `.sha256` file are in `src-tauri/target/release/bundle/nsis/`. Earlier installers remain available.
 Native appearance, physical routers and gameplay remain manual checks. Automatic router detection does not bypass CGNAT or confirm internet reachability. Fabric client instances and Microsoft authentication remain deferred.
