# TCHLauncher 0.26.0

## Share worlds and content

- Export a selected world with its mods or plugins in one ZIP, with optional `config` and `defaultconfigs` folders.
- Include matching Nether and End world folders for Paper, preserve disabled files, and provide an English restoration guide plus SHA-256 file manifest.
- Keep the existing world-only folder export. Exports require a stopped server and a destination outside its folder.
- Stream compression with progress and cancellation. Reject redirected paths and detected file changes; preserve the previous ZIP until publication.
- Refine world cards and the inline export choices with End-themed surfaces, pixel artwork, keyboard focus and responsive spacing. Prevent stale results from populating another server's world panel.

A sharing export is not a full server backup. It excludes Java, launcher startup settings, `server.properties`, EULA files and logs. World player data and selected content configurations remain included; review them and each project's redistribution terms before sharing.

## Language and documentation

- Add Spanish and English application languages in Settings → General, applied immediately and persisted locally.
- Translate interface controls, accessibility labels, workflows, known task stages and common launcher diagnostics. Format dates and numbers for the selected language.
- Preserve names, commands, paths, Minecraft logs and third-party descriptions. Unrecognized diagnostics retain their original text.
- Rewrite README and SECURITY in English, move release/development records into `docs/`, and add document indexes.
- License project source under MIT. Preserve dependency and artwork notices separately, including Monocraft's OFL and the NSIS installer notices. Include collected third-party license texts and a versioned source inventory in the Windows package.

## Fabric dependency review

- Correct recognized numeric comparisons for Unicode decimal digits, Java integer overflow and UTF-16 behavior.
- Keep runtime-dependent Sinhala Lith/Myanmar Tai Laing core digits unverified because Java 8 and modern Java interpret them differently.
- Compare recognized native verdicts with the official Fabric API: 1,263 comparisons in a 1,497-case fixture passed independently on Java 8, 21 and 25. This does not instantiate Fabric Loader or run Minecraft.

## Verification

- All 78 frontend tests and 230 native offline tests passed; 28 condition-dependent native tests were excluded from the offline suite.
- Four new export regressions cover archive contents and hashes, optional configuration, Paper dimensions, cancellation, previous-file preservation, unsafe destinations and detected same-size external edits.
- Localization checks cover persistence, storage failure, immediate static-label changes, placeholders and preservation of user data.
- TypeScript and production frontend compilation passed. Vite reports an initial JavaScript chunk above its 500 kB advisory threshold; startup time has not been benchmarked.
- Native visual/keyboard review, complete gameplay and physical router checks remain pending. Linux remains deferred.
- No user server was started, EULA accepted, commit made or GitHub release published during this iteration.

## Windows package

- Installer: `TCHLauncher_0.26.0_x64-setup.exe` (6,631,757 bytes; 6.32 MiB).
- Executable and installer file/product versions: 0.26.0. Embedded icons match the supplied logo.
- Project license, third-party notices, collected texts and source inventory are included as `licenses/` resources. Their copied files match the source hashes, and the NSIS script includes them.
- SHA-256: `5b2a37358a0a0fcc6ff4f11c124c646b782cbd4443971933f22a5616696aafa6`.
- Built locally with NSIS x64; not installed or published during verification.
