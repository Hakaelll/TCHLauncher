# TCHLauncher v0.4.1

This maintenance release fixes confirmation dialogs and adds a graceful server restart action.

## Changes

- Server deletion and world confirmation dialogs now cover the entire application window, including the sidebar. They remain centered in the visible window even when the settings page is scrolled to the bottom.
- After deleting a server, the main view returns to the top so the result message is visible.
- A running server can be restarted from its header. TCHLauncher sends the normal stop command, waits for the process to exit, and then starts it again. The existing forced-stop timeout remains the last resort.

## Verification

- TypeScript compilation and 17 Rust tests passed.
- The Windows installer was built locally. Visual review and installation on a separate machine remain to be done.

## Known limitations

- The Windows installer is not digitally signed.
- Fabric, NeoForge, mod and plugin management, modpacks, public networking, and scheduled tasks are still planned.
