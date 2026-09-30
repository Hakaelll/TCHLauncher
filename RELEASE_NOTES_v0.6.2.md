# TCHLauncher 0.6.2

## Fabric server support

- Add Fabric to the guided creation flow and server import.
- Load the official compatible Minecraft catalog when Fabric is selected; prevent advancement while loading and discard stale platform responses.
- Select a stable Fabric Loader and verify all libraries with official SHA-256 checksums.
- Download the matching Mojang server with SHA-1 verification through the existing cancellable download manager.
- Generate a launcher with relative library paths and publish the complete installation as one generation before updating the database.
- Preserve server worlds, properties and artwork during setup.
- Recognize Fabric launchers in conventional folders and TCHLauncher generations during import; prepare Java without replacing imported JARs.
- Launch managed Fabric servers with the matching game JAR and hide Java console windows on Windows.

## Validation

- Frontend type checking and production build.
- 32 local Rust tests, including Maven path validation, stable-loader selection and JAR manifest folding.
- A live integration test installs Minecraft 1.21.4 with Fabric Loader 0.19.5 and Java 21, reaches the EULA gate without accepting it, then checks import and Java preparation.
- Joining from a Minecraft client and testing a modded world remain manual checks.

NeoForge, mod management and loader updates remain in progress under the original application scope.

## Windows installer

- File: `TCHLauncher_0.6.2_x64-setup.exe`
- Size: 4,827,893 bytes
- SHA-256: `718714BFD0B3ECE10EAB198F9BF0DEFC80637CEF07981194D3910748960C2B5F`
