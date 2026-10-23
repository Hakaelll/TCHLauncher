# TCHLauncher 0.38.0

## Players and access

- Grant OP to a known player by selected UUID while the server is stopped. Review
  the permission level and full-server bypass before confirming. Existing operator
  UUIDs keep their permissions; same-name accounts remain separate.
- Keep the native acknowledgement when a follow-up read fails, prevent overlapping
  access submissions, and block inventory-based controls during unconfirmed reads.
- Share the bounded access-file writer for operator edits, whitelist and bans,
  preserving unknown fields, unrelated identities and original-byte safety copies.
- Translate player result counts into English.

## Tasks and keyboard actions

- Task failures and cancellations identify the actual operation, including Java,
  modpack, content and engine updates. Failed downloads keep their last attempt.
- Show recovery advice for every task kind beside the original technical details.
- Open a server folder from Ctrl+K without changing the workspace selection.
- Restart running servers from Ctrl+K. Commands are checked against the current
  server state and active operations immediately before execution.

## Verification and limits

Focused native access-file checks and temporary task-result checks passed.
TypeScript and actual React rendering checks passed for operator review and task
errors in English/Spanish. Command-generation checks cover search, server identity,
active operations and changed states. No dependency or retained test case was added;
production task handling has less duplicated code. Broad real-engine scenarios
were not repeated for this release.

This is an unsigned Windows x64 preview. Native interaction, clean installation,
upgrade/uninstall, actual Minecraft permissions/gameplay and physical-router
Internet use remain unverified. The full original manager objective remains open.
Linux and Minecraft client launching remain deferred.

Only the Windows installer is attached to the GitHub release. Checksum files stay
local as requested.

Installer build completed successfully: `TCHLauncher_0.38.0_x64-setup.exe`, 6,835,851 bytes,
version 0.38.0. Windows production frontend and optimized native builds passed.
