# TCHLauncher 0.24.0

## World creation and configuration

- Choose an optional world seed in the creation wizard, including exact large signed numbers or text. Review it before creating the server.
- Configure game mode, difficulty and core rules before the first server start. Existing file values and unknown properties are preserved; settings are applied on the next start.
- Refine the End-themed world step and integrate navigation actions into the form.

## Mods and application behavior

- Combine inventory state/source filters, search secondary declared IDs and use clearer file slots, counts and empty states.
- Review required Fabric dependency presence across aliases and nested JARs. Show missing providers or incomplete metadata without claiming version compatibility or gameplay success.
- Separate completed content installation/removal from failed inventory synchronization; protect against duplicate dispatch and late replies.
- Improve startup preference recovery and coordinated graceful application shutdown, preserving the close choice on failures.

## Verification and scope

- 218 offline native tests and all 74 frontend tests passed; 25 external/runtime/helper integrations were excluded. TypeScript and production build passed.
- Linux is deferred by request; Windows is the active platform. See LINUX_LATER.md.
- Native visual/keyboard review, actual Minecraft gameplay, router access and OS-session startup remain pending. No EULA was accepted, no user server was started or stopped, and the installer was not installed during verification.
- Version-predicate resolution for local Fabric metadata is not implemented; presence checks do not replace Fabric Loader.

## Windows package

- Installer: `TCHLauncher_0.24.0_x64-setup.exe` (6,499,032 bytes).
- Native file/product version: 0.24.0. Executable and installer icons match the supplied golden logo.
- SHA-256: `483a5b3e7607eb3bd63c0c6d7f75e90040c8dcc2e1249b6c034c5e26a43df2f0`.
- Built locally with NSIS x64. Not installed, committed, pushed or published.
