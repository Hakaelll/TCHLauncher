# Licenses and third-party inventory

TCHLauncher's own source code is licensed under the [MIT License](../../LICENSE). This does not replace third-party licenses or establish ownership of supplied artwork.

- [Third-party notices and artwork provenance](../../THIRD_PARTY_NOTICES.md)
- [Collected original license texts](THIRD_PARTY_LICENSES.txt)
- [Dependency inventory, versions, source archives and notice hashes](dependency-licenses.json)

The inventory covers the locked Windows Cargo dependency graph, including build and transitive dependencies, the frontend runtime dependencies, Monocraft and the NSIS installer notices. It is intentionally broader than the libraries embedded in the final executable. Development tools, the Rust standard library and operating-system components are not exhaustively inventoried here.

License files are collected from installed package sources. Where a package omits standalone license files, copies from its upstream repository or canonical SPDX license texts are retained under `upstream/`, with provenance in `source.json`. Canonical texts do not invent missing copyright statements. Source archives in the inventory identify the exact package version, including unchanged MPL-2.0 dependencies such as `selectors` and `cssparser`; their source and license notices remain available from those links.

Regenerate after dependency changes, with locked packages already installed and cached:

```powershell
python ./scripts/generate_third_party_notices.py
```

Review the result and any missing texts before distribution. The inventory is not an independent legal audit or a blanket determination of license compatibility. Downloaded Java, Minecraft and community content retain their own terms and are not relicensed by TCHLauncher.

The Windows installer includes the project license, third-party notices, collected texts and dependency inventory under its `licenses` resource directory.
