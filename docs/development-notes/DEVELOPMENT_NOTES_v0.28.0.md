# Development notes — 0.28.0

## Content state management

- Add an explicit disabled inventory alongside the existing active JAR list.
  Logical filenames retain `.jar`; physical files use `.jar.disabled`. Existing
  manifests, configuration folders and project/version identity are preserved.
- Run writes under the stopped-server reservation and registered file task. Save
  a full safety backup, prepare a separate content folder and publish through the
  existing recoverable directory journal. Cancellation is accepted before final
  publication and leaves the previous content state intact.
- Reject stale hashes, destination collisions, unsafe names, client-only Fabric
  reactivation and duplicate active descriptor identities. Disabled artifacts
  participate in review fingerprints and cannot be silently reinstalled or
  updated as an independent copy of the same managed project.
- Reuse managed Modrinth dependency declarations and static Fabric presence/range
  evidence. Separate known blocking declarations from optional conflict warnings.
  Compare the staged Fabric result with the original inventory so that the change
  cannot introduce a newly detected blocking dependency issue. This is not a full
  Fabric candidate solver or a check of every local plugin's dependencies.
- Add direct Disable and Enable actions, filtered disabled rows, scoped operation
  ownership, real progress/cancellation and read-only recovery after task failure.
  No extra modal interrupts this reversible action. Follow-up read failures do not
  repeat an acknowledged mutation or disguise it as a failed write.

## Interface finish

The mod inspector separates metadata, declared dependencies and related console
messages. The warning action opens evidence directly. View controls and semantic
SVG marks use the existing End/Obsidian/Nether material language. English and
Spanish labels remain available. Cached icons avoid redundant state publications.

## Checks and limits

- Native offline suite: 239 passed, 28 ignored. Seven new content regressions cover
  round trips, manifest preservation, managed disabled project identity, stale
  hashes and collisions, dependency removal, duplicate IDs, client-only content,
  review invalidation and cancellation during backup/content preparation.
- Existing Fabric presence suite: six passed, including partial metadata,
  overrides and nested providers. Known incomplete metadata remains incomplete.
- Frontend: 84 regressions passed; TypeScript passed. Final packaging runs the
  production frontend build again.
- Native UI/keyboard interaction, actual Minecraft loading and gameplay remain
  unverified. A successful file operation is not evidence of mod behavior in a
  world. Physical router tests and internet-stutter diagnosis remain outstanding.
- Windows only. Linux remains deferred. No new external dependency or relay.

## Package

- Installer: `TCHLauncher_0.28.0_x64-setup.exe` (6,672,733 bytes; 6.36 MiB).
- SHA-256: `1878b08e590cb22f002f729b5506b46feea48f881f184fffb621caa10c3d8d0b`.
- Installer and executable file/product versions are 0.28.0. Embedded logo pixels
  match the supplied icon. All four copied license resources match their source
  hashes and are included in the generated NSIS script.
- Final TypeScript/production frontend build and Windows NSIS packaging completed.
  A checksum file is adjacent to the installer. Generated locally; not installed,
  committed, pushed or published by this workflow.
