# TCHLauncher 0.53.0

Windows x64 preview prepared for local packaging. This version is not published
on GitHub; the published preview remains 0.51.0.

## Changes since 0.52.0

- Memory charts scale to visible series. Hide resident RAM to inspect heap usage
  without its line being flattened by hidden process-memory peaks.
- Broadening an empty Modrinth search preserves the search text, content kind and
  sort order. It removes category/version restrictions, returns to the first page
  and restores focus to the results heading.
- ZIP support removes unused password-encryption dependencies while retaining all
  previous compression codecs and time support. Seven registry packages leave
  the Windows dependency graph; no package version is upgraded.
- Windows icon generation retains only the four configured outputs. Thirteen
  unused platform derivatives are removed from the checkout. The golden emblem
  and configured Windows icons keep their existing content.

## Validation and limits

Scoped production helper, component-tree, callback and React SSR checks cover
chart scaling and empty-search recovery in Spanish and English. A native
before/after case inspects the same Stored, Deflate, Bzip2, Zstd and Xz modpacks,
reads exact override bytes and rejects the same encrypted archive without a
password. Other retained codecs are not separately exercised by those fixtures.
Third-party notices are regenerated with no missing local license texts.

Icon regeneration produces the four expected files with unchanged hashes.
Temporary checks add no retained tests or dependencies. Native UI, new Minecraft
sessions, installed size and runtime memory or latency are not established by
these checks. Manual validation remains maintainer-owned and nonblocking.
