# TCHLauncher 0.23.0 — Library polish and clearer management

## Interface and browsing

- Refine the library with square End materials, clearer capacity/action groups, pinned markers, larger search/view controls and responsive list rows.
- Add online/error/pinned filters, combined search, name/recent ordering and a filtered-empty reset. Preserve native ordering by default and keep separate servers with equal names.
- Group import actions and enable the existing context menu from server cards. Retain creation, artwork, themes, bundled Minecraft lettering and reduced-motion preferences.
- Load Worlds, Content, Performance and Application Settings on demand, with local loading/error states and retry. The initial script is 439.80 kB (131.13 kB gzip), compared with 501.30 kB (148.35 kB gzip) before deferred sections; this is bundle-size evidence, not a native launch benchmark.

## Runtime, settings and acknowledged actions

- Separate checking, unavailable, missing, incompatible and confirmed Java states. Preserve scoped runtime selection and successful writes when later metadata queries fail.
- Protect JVM drafts and require a current launch preview matching Java, RAM, platform/version and effective arguments before saving.
- Preserve operator drafts during polling, show concurrent permission changes and require an explicit reload before overwriting them.
- Distinguish completed loader/player actions from follow-up read failures. Offer read-only recovery instead of repeating a completed mutation.
- Add explicit Minecraft catalog retry and guards for obsolete or duplicate update/review requests.

## Historical NeoForge

- Integrate the official historical Minecraft 1.20.1 catalog and net.neoforged:forge installer/profile/library layout alongside modern NeoForge.
- Identify imported historical forgeVersion arguments and compare historical loader updates numerically. Preserve catalog-confirmed modern prerelease layouts/imports.
- This does not certify universal Forge mod compatibility.

## Verification and limitations

- All 70 frontend behavioral tests and 208 offline native tests pass; 25 external/runtime/helper checks are excluded from the general native run. TypeScript and production Vite build pass.
- A separately executed real NeoForge 1.20.1-47.1.106 installation and copied import both boot with managed Temurin 17 to EULA refusal, preserve properties and create no world. No EULA was accepted.
- Separate real Temurin 21 checks validate JVM profiles, reject invalid arguments and preserve an existing Java selection on incompatible replacement.
- Native visual/keyboard review, actual Minecraft gameplay, physical router/external access and Linux packaging remain pending.
- Generated locally; not installed, committed, pushed or published to GitHub.

## Windows installer

- Artifact: `src-tauri/target/release/bundle/nsis/TCHLauncher_0.23.0_x64-setup.exe` (6,485,834 bytes).
- SHA-256: `a04c848886d700dd3904faf3be631319b524519c60096b77bff678b486d28cf1`.
- Windows x64 NSIS packaging succeeds; executable version resources report 0.23.0. Executable and installer icons match the supplied logo.
