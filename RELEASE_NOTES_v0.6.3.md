# TCHLauncher 0.6.3

## Custom Java runtimes

- Add an advanced Java executable picker to each server's settings.
- Inspect the actual executable and compare its version with the Java major required by official Minecraft metadata.
- Preview the verified version and path before confirming the change; incompatible selections preserve the current configuration.
- Recheck the executable when confirming and before starting or restarting a server.
- Cache the official requirement for offline validation. Existing servers without cached metadata report version verification without claiming compatibility.
- Keep managed Temurin available when switching back from an external runtime.
- Run executable checks outside the UI thread, with a five-second timeout and hidden Windows console.

## Validation

- Frontend type checking and production build.
- 33 local Rust regression tests and a real Java 21 integration check covering compatible selection, rejection, persistence and launch validation.
- Desktop layout, installation and joining from Minecraft remain manual checks.

The existing console uses distinct colored tags for Info, System, Player, Chat, Command, Warning and Error messages.

## Windows installer

- File: `TCHLauncher_0.6.3_x64-setup.exe`
- Size: 4,858,671 bytes
- SHA-256: `B7719852D60A409C67ACA6324F034D0EB07C90B32FAEC71CF6830B06C108ACB8`
