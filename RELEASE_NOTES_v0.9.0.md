# TCHLauncher 0.9.0 — Windows preview

## Minecraft and loader updates

- Review stable Minecraft upgrades with official release chronology, compatible Java selection and exact engine releases.
- Resolve compatible managed mods/plugins and required dependencies together, with reviewed replacements and changelogs.
- Create a safety backup and prepare downloads in an isolated instance before publishing Minecraft, Java and launch paths with recovery journals.
- Update Fabric/NeoForge loaders independently for the current Minecraft version, including identifiable imported launchers.
- Offer startup verification after a Minecraft upgrade. Readiness is established by the native Minecraft startup state, not process creation; failed or unconfirmed starts link to the console and retain the safety backup.

## Interface and player identities

- Refine first-server creation, folder import and modpack import choices with responsive action placement.
- Group overview metrics with clearer process/configuration context and emphasize the selectable LAN address.
- Preserve anthracite controls, semantic colors and reduced-motion behavior.
- Avoid choosing stale offline UUIDs when imported history contains multiple records for an authenticated player's name. Keep distinct history records intact.
- Normalize Windows Java classloading paths for imported and prepared loaders.

## Verification

- 108 local Rust tests passed; 16 external-resource tests are excluded from the offline suite.
- 13 frontend logic tests, strict TypeScript compilation and the production Vite build passed.
- Separate official-download checks exercised Minecraft upgrade/restore cycles with Java 21 for Vanilla, Fabric, NeoForge and Paper.
- Additional Fabric/Lithium and NeoForge/FerriteCore migration cycles checked content hashes, restored inventory and backup counts.
- Loader update/restore cycles also passed for managed and imported Fabric/NeoForge layouts.
- Live Java checks stopped at EULA refusal: no agreement was accepted and no world was generated. They do not prove client gameplay, world conversion or native visual quality.

## Preview limits

Native interaction/startup-verification flow, Minecraft client/gameplay, world conversion, Windows installation/login startup and Linux integration still need validation. Navigating away from the update panel stops its startup observation; it does not stop the server. Public-network automation, reliable TPS telemetry, additional legacy/custom loaders and distribution signing remain unfinished.

Manual or modified content remains unverified and is preserved with explicit warnings. Restoring a different Minecraft version clears Java selection until a compatible runtime is prepared or selected. A startup failure never silently restores worlds, since doing so would discard subsequent changes.
