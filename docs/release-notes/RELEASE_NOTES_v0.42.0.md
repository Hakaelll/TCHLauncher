# TCHLauncher 0.42.0

Local Windows development build. No GitHub release is published for this version;
0.40.0 remains the latest published preview.

## Settings

- Rename a server from Name and image. The native command persists its local
  display name without changing its ID, folder, Minecraft MOTD or other settings.
  A failed library refresh does not undo the acknowledged name in the interface.
- Discard unsaved properties, memory and storage targets without writing files or
  restarting the server. Properties return to the last read/acknowledged snapshot.
- Save reveals the first changed known numeric property outside its field bounds,
  including fields hidden by search. Feedback supports Spanish and English.
  Unchanged imported values remain intact; native validation stays authoritative.

## Console, content and worlds

- Copy view uses the displayed message instead of duplicating raw log headers.
  Original log export remains unchanged.
- Installed-plugin diagnostics recognize explicit Paper event and task exception
  references. Names must match; chat, commands and INFO text remain excluded.
  Related messages retain their original severity and console navigation.
- Paper world/content exports can omit plugin configuration/data folders while
  retaining top-level enabled and disabled JARs. The checked default preserves
  complete plugin folders. The archive guide describes the selected behavior.

## Maintenance

- Remove 18 exact shadowed CSS declarations while preserving final scoped values.
- Add a PowerShell cache cleaner with WhatIf preview, build-process checks and
  redirected-path rejection. It preserves release executables, installers, source
  assets and installed project dependencies.

## Verification

Isolated native SQLite/file checks cover rename persistence and unchanged server
files. Real ZIP generation from synthetic files covers both Paper configuration
choices, exact membership, manifest hashes and unchanged originals. Focused
frontend checks cover acknowledgement failures, saved/discard state, numeric
validation, ES/EN markup, console copying and Paper references. Browser/API
boundaries were simulated where recorded; no native interaction or client gameplay
is claimed for this build. The maintainer owns the remaining manual validation.

This remains an unsigned Windows x64 preview. Linux, client launching and external
tunnels remain deferred. No dependency was added.

TypeScript, the Windows frontend production build and optimized NSIS packaging
passed. Executable and installer report 0.42.0 and their embedded icons match the
application emblem. The installer is 6,860,850 bytes. Only the local installer was
generated; no GitHub release or asset upload was performed.
