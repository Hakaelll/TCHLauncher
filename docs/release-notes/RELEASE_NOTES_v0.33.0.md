# TCHLauncher 0.33.0

## Daily use

- Put server capacity and gameplay settings before Java, updates and automation.
- Keep save controls visible while scrolling the settings workspace.
- Search localized property groups, labels/descriptions and native keys, including
  advanced properties. Search preserves draft values and handles accents.
- Refuse duplicate property additions instead of overwriting an existing value;
  show translated key/value validation beside the inputs.
- Share date and number formatting across panels so dates follow the selected
  application language, including backups, tasks and crash history.

## Server reliability

- Read actual Paper TPS responses with repeated Windows console prompts and ANSI
  formatting. Shared normalization preserves raw logs and rejects chat/plugin
  imitations. TPS values expire and reset with their process session.
- Avoid a temporary vector allocation when parsing the three TPS values.
- Reuse the byte formatter for storage totals and avoid temporary flattened/filtered
  memory arrays when calculating chart limits. This removes 26 source bytes across
  the affected frontend modules; it is not a measured application RAM improvement.

## Real engine evidence

Paper 1.21.4 passed TPS queries across restart/restore, graceful Stop while still
Starting and a durable single stop-copy after a ready session. The actual stop-copy
worker preserved level.dat and did not replay the same exit after recovery.

Paper with Chunky installed through the production Modrinth plan/download/hash path
passed three plugin enablements, TPS/status/commands, graceful stops and tracked
backup/restore. The exact installed plugin JAR and level.dat were restored.

Two real Vanilla 1.21.4 instances reached Running simultaneously with distinct PIDs,
ports and directories. Commands and saved logs remained independent. After one
stopped, the other retained its session and answered a new command/status request;
both then stopped gracefully. These tests use isolated temporary libraries and an
existing explicit consent record; they do not accept new terms on the user's behalf.

## Verification boundaries

Native installation, appearance, keyboard/screen-reader use, player connections,
client gameplay and physical routers remain unverified. The imported Minehut,
friend-hosted Fabric RAM/missing-mod and internet-stall reports still need actual
host evidence. Linux and client launching remain deferred. The full original
manager objective remains open; see NEXT_TODOS.md.

## Windows package

Installer: `TCHLauncher_0.33.0_x64-setup.exe` (6,719,027 bytes / 6.41 MiB).
Tauri/NSIS packaging completed. Executable and installer versions are 0.33.0;
both embedded icons match the supplied logo. All four legal resource destinations
are present in the generated NSIS script. The inventory matches 342 locked Windows
Cargo packages and the four installed frontend runtime packages.

251 native tests passed, with 36 conditional integrations ignored. All 105 frontend
tests, TypeScript and the production Windows build passed. The real engine tests
above ran separately. Initial JavaScript is 460.12 kB, not a RAM benchmark.

SHA-256: `e468ab707f4817eb6bd4777116ad9f24cb38c3d48507639158f6e29378978cb6`.

Generated locally with adjacent checksum files; unsigned, not installed or
published as a GitHub release. Version 0.32.0 is retained for fallback; obsolete
0.31.0 generated installer/checksum files are removed after verification.
