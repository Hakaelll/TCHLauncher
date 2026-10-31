# TCHLauncher 0.53.0

Windows x64 preview, built locally. The installer is unsigned. This version is
not published on GitHub; the published preview remains 0.51.0.

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

Packaged source: `f58a04a79ad4cf247be88b2d7a8c934f38750d17`. TypeScript, five retained frontend
tests, Windows frontend, optimized offline NSIS packaging, executable/setup
versions and embedded icons pass. The installer is 6,883,055 bytes, compared with
6,910,025 bytes for 0.52. These packages include several changes; this
is not an isolated dependency-size or performance benchmark.

Protected cleanup removes 1,916,188,833 logical bytes of generated files, preserving
the 0.52/0.53 setup hashes and current executable. The obsolete local 0.51 setup
is separately verified and removed. Checksums remain local and nothing is
published to GitHub Releases. Native interaction and gameplay are not repeated.
