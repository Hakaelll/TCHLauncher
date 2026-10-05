# TCHLauncher 0.19.0 — Mod discovery and monitoring refinement

## Mods and plugins

- Filter Modrinth by category, sorting and server Minecraft version, with an option to include other versions. Keep the server loader fixed. Show checked compatible releases, beta/alpha labels, unavailable versions and request failures before opening the install review.
- Preserve the search, filters and page after installation and when switching to installed content. Refresh installed badges without leaving discovery. Keep reviewed versions, required dependencies, SHA-512 verification, stopped-server mutation, backups and stale-review protection.
- Review JAR descriptors without executing them. Report malformed/wrong-loader files, Fabric client-only mods and duplicate IDs. Show Fabric names, versions and declared Minecraft ranges; local dependency ranges are not fully resolved.
- Compare Fabric IDs with a complete, bounded startup list in the last server log. Report historical presence separately from inventory existence; missing, partial or stale evidence remains unconfirmed. Legacy Fabric and non-Fabric descriptor details have explicit limits.
- Reject modern Fabric client-only or malformed descriptors when publishing a reviewed Modrinth download. Preserve existing local files and configurations.

## Interface

- Redesign discovery as inventory item cards with visible filters, category tags and text-backed release states.
- Remove the extra segmented bars from memory/CPU plots. Keep simultaneous charts with orthogonal measured lines, restrained area shading, compact peaks and a dashed heap legend. Support pointer and keyboard sample inspection while preserving measured gaps and unavailable values.
- Remove long permanent explanations below router controls. Keep internet controls visible and put detailed guidance in optional help and the existing port-opening confirmation.

## Verification

- TypeScript, the production frontend build and all 29 frontend tests pass.
- The complete offline native suite passes 179 tests; 23 external/runtime integration tests are excluded from that run.
- Three explicit live Modrinth integrations pass: search/icon retrieval, release-level availability (including an unavailable Minecraft version), and real install/update with verified hashes, preserved files and a safety backup. They run in isolated temporary folders without executing Java or mods. The network sandbox blocked the first catalog attempt; the explicit permitted external-network run passed.
- Native visual review and the reported multiplayer world are not exercised by these tests.
- Three focused descriptor/log regressions pass again after the final metadata validation change.
- Windows NSIS x64 packaging completes. Native file/product version is `0.19.0`; the embedded small/large logo pixels match the supplied icon in both executable and installer.

## Windows preview

- Installer: `TCHLauncher_0.19.0_x64-setup.exe` — 6,334,294 bytes (6.04 MiB).
- SHA-256: `f5d8cd6eb8e894dd675827c9eb4aa7f6f9939de5c4919c9d4aecd5c78c043404`.
- Installer and adjacent `.sha256` file: `src-tauri/target/release/bundle/nsis/`.
- No commit, push or GitHub publication was performed.

## References

- [Modrinth search API](https://docs.modrinth.com/api/operations/searchprojects/): category/version facets and sort options.
- [Fabric descriptor specification](https://wiki.fabricmc.net/documentation:fabric_mod_json_spec): mod IDs, environments and declared dependency ranges.

## Limits

The exact missing backpack/structure projects and the friend's startup log were not provided. This release adds checks but does not claim to have diagnosed their world or prove in-game mod behavior. Native visual review, Minecraft gameplay and physical router checks remain manual. Fabric client instances and Microsoft authentication remain deferred.

## Suggested commit

Improve mod compatibility checks and discovery while refining Minecraft monitoring

- Add bounded JAR audits and historical Fabric loading evidence.
- Preserve browsing after installs and expose Modrinth filters and release availability.
- Simplify network guidance and performance charts without removing server controls.
