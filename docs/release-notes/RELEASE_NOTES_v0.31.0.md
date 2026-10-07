# TCHLauncher 0.31.0

## Players and whitelist

- Raised Minecraft-style category tabs, more visible skin heads and quieter row
  hover/focus feedback. The connected-player summary uses current session data.
- Review every imported whitelist name with search, paging and new/already-allowed
  labels instead of an eight-name excerpt. Keep the review when switching categories.
- Guard file selection and submission against duplicate work and late completions.
- Distinguish successfully sent requests from a failed subsequent list refresh.
- Send bounded whitelist batches from a worker tied to one server session. Validate
  all commands first and report the preceding write count if dispatch fails.

## Java and console reliability

- Continue reading stdout/stderr after malformed UTF-8 or interrupted reads.
- Bound each source line to 64 KiB, explicitly mark truncation and drain its
  remainder without creating additional log records. Minecraft's own logs are unchanged.
- Wait for confirmed process exit before releasing ownership after a monitor error.
- Retain monitor failures in diagnostic history even with an eventual zero exit code;
  a later successful run does not discard the previous failure.
- Reject empty/control-character commands and preserve extension-command slashes.

## Cleanup

Remove unused avatar and old whitelist-preview styling. After verifying the new
package, remove 61 obsolete generated installer/checksum files, freeing 195.31 MiB.
Keep 0.30.0 for fallback and regenerate the checksum list for the two retained packages.
Current design, legal files, Linux deferral and release history remain useful records.

## Verification and limits

251 offline native tests passed; 29 conditional entries were ignored. All 98
frontend tests and TypeScript passed. The installed Java 21 diagnostic integration
passed separately before packaging. The command pipe regression received 1,000
requests in order from an owned test child; it is not a Minecraft gameplay test.

Native appearance, keyboard, live whitelist acceptance, full Minecraft gameplay
and physical-router checks remain pending. Imported Minehut account behavior,
internet stutters and the reported Fabric RAM growth remain unconfirmed on their
real host. Linux remains deferred. This release does not complete the original goal.

## Windows package

Installer: `TCHLauncher_0.31.0_x64-setup.exe` (6,693,727 bytes / 6.38 MiB).
The Tauri/NSIS build completed successfully. Executable and installer file/product
versions are 0.31.0; both embedded icons match the supplied golden logo. All four
license destinations are present in the generated NSIS script.

SHA-256: `0fdb47606d8059363110b1bda0be19acd2b86a234b6f15d457c3358ef5fff361`.

Generated locally with an adjacent checksum file. Installation and publication
were not performed.
