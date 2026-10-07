# Security policy

## Supported scope

Security fixes target the latest Windows preview and current source. There is no commitment to backport fixes to older previews. Linux is deferred. The project has not undergone an independent security audit and does not offer a response-time guarantee.

## Reporting a vulnerability

Use GitHub's **Report a vulnerability** feature if private vulnerability reporting is enabled for this repository. Otherwise, open an issue asking for a private reporting channel, without publishing exploit details or sensitive data. No dedicated security email is currently specified.

Include the affected application version, Windows and Minecraft versions, server type, impact, reproduction steps and a minimal test case where possible. Redact credentials, public addresses you do not want disclosed, player data and private paths. Please avoid testing against servers or routers you do not control.

## Trust boundaries

TCHLauncher manages files and launches server processes as the current user. It is **not a sandbox** for Minecraft, mods, plugins, custom Java executables or Java agents. Those components can access files and the network with your account's permissions.

### Desktop application

The frontend uses bundled assets and communicates with native commands through Tauri IPC. The current content security policy is:

```text
default-src 'self'; connect-src ipc: http://ipc.localhost; img-src 'self' asset: http://asset.localhost data:; style-src 'self' 'unsafe-inline'
```

The main window capability grants `core:default`. Native input validation and command-specific safeguards remain essential; frontend controls alone are not authorization boundaries. See the [Tauri security documentation](https://v2.tauri.app/security/) for platform context.

### Files, downloads and archives

- Native commands validate server names, versions, resource limits, ports and editable properties.
- Server file mutations use per-server reservations and require a stopped server where applicable.
- Downloads use staging files and verify published hashes when available before replacing existing artifacts. Cancellation and bounded transient retries are supported.
- Archive extraction and copy paths are checked to prevent escaping the intended destination; links are rejected in guarded archive flows.
- World/content ZIP exports reject links and Windows reparse points, validate relative names and require a destination outside the server folder. Files are streamed with hashes, and a previous destination is retained until publication.
- Backup and restore flows include recovery safeguards documented in the architecture. External programs editing the same files can still invalidate assumptions; stop external writers before copying or exporting.
- In-place imports use the original folder. The launcher cannot protect that folder from changes made by other programs. Keep independent backups.

Hash verification checks integrity against supplied metadata, not whether an upstream artifact is safe or functions correctly in the game.

### Java and process control

Selecting custom Java requires an explicit executable choice and confirmation. Accepted paths are canonicalized and must reference a file named `java` or `java.exe`. This does not establish trust: inspection executes the selected program with `-version`, using a timeout, hidden Windows console and bounded output.

JVM options are passed as individual arguments, not shell commands. Memory, entrypoint and external argument-file overrides are rejected. Java agents execute user-selected code and must be trusted.

Normal shutdown requests a graceful stop and waits for the process. Forced termination requires separate confirmation and cannot establish a clean world save. Minecraft EULA acceptance requires an explicit user action; remembered acceptance is used for subsequent setup.

### Images, identities and content

Server artwork is limited to 2 MB, 4096 × 4096 dimensions and bounded decode allocation. The Minecraft server icon is derived as a 64 × 64 PNG. Image changes require the server to be stopped.

Skin profiles are resolved by UUID through Mojang services. Texture downloads accept the official texture host without redirects and use bounded responses and image decoding. Offline and online UUIDs are different identities; a name match alone is not proof of account ownership.

Content descriptors, compatibility checks and console warnings are diagnostic aids. A listed mod is not proof that Minecraft loaded it, and a log association is not proof that the named mod caused a failure.

### Networking and privacy

Router discovery is automatic; opening a temporary UPnP or NAT-PMP mapping requires an explicit action. Renewal and removal are attempted while managing the server lifecycle. Router responses do not verify internet reachability, configure Windows Firewall or bypass CGNAT. Bore is not integrated.

Provider requests are made as needed for version metadata, downloads, Modrinth content and player profiles. Public-address discovery may contact ipify. Review provider policies before using these services.

Logs, world data, selected content configuration and exports can contain player identifiers, addresses or sensitive configuration. Review and redact them before sharing. Language switching preserves user-provided names, paths, commands and Minecraft logs; unknown diagnostics retain their original text.

## Remaining verification

Native window, tray and IPC behavior; full Minecraft gameplay and stop/restore scenarios; physical router access; and additional disk interruption scenarios remain open. Automated regression tests do not replace these checks. Dependency notices are an inventory, not a complete vulnerability or license compatibility audit.

See [NEXT_TODOS.md](NEXT_TODOS.md), [ARCHITECTURE.md](ARCHITECTURE.md) and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for implementation details and verification evidence.
