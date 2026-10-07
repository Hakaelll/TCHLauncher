# Development notes — 0.26.0

## World/content export

`world_bundle.rs` streams selected world files and server content into a staged deflated ZIP. Paper sibling dimension folders are included when present. `tch-export.json` records the Minecraft version, platform, folder selection and per-file SHA-256. Java and launcher/server startup configuration are excluded.

The native command retains the old world-only export as its default for existing callers. The new UI explicitly requests a ZIP, then chooses optional shared configurations. Server reservations and task-registry lifetime cover the native operation. Paths, file types and Windows reparse points are checked; size/modified-time checks reject detected external changes, but do not provide a filesystem snapshot against arbitrary outside writers. A scope guard removes staging files on failures or unwinding. Existing destinations are kept until publication; new Windows destinations use a non-replacing MoveFileExW operation.

The world panel scopes asynchronous completions to the selected server, prevents duplicate operations and distinguishes mutation acknowledgement from subsequent list refresh failures. Updated cards and export radios use existing theme and motion tokens, native form controls and keyboard focus. Native visual review remains outstanding.

## Localization

Spanish source messages serve as stable catalogue keys in `src/locales/en.ts`; indexed placeholders preserve input data. `src/locale.ts` owns the persisted language, subscriptions, formatting and known diagnostic translation. `useLanguage` triggers rendering when the choice changes. Static UI metadata is read through a lazy translation proxy; API objects and user data are never wrapped. Application strings are translated explicitly at rendering boundaries. Unknown diagnostics, logs and external descriptions retain their original text.

Four regression tests verify persistence and blocked storage, static labels and stable identifiers, stored notices and known native messages, placeholder preservation and core English workflows. Existing frontend behavior tests remain passing. No frontend dependency was added.

## Licensing and documentation

MIT applies to project source. Third-party notices are separate. `generate_third_party_notices.py` collects the locked Windows Cargo graph, frontend runtime packages, Monocraft, bundled SQLite's original dedication and NSIS notices. The inventory includes build dependencies and is not a final-binary bill of materials or an independent legal audit. Upstream/canonical fallback text provenance is retained under `docs/legal/upstream/`. Current output: 349 component records, 616 notice text sections, no missing local texts.

The project license, notices, aggregate text and dependency inventory are bundled as Windows application resources. README and SECURITY were rewritten in English. Historical release/development records moved to dedicated folders; local Markdown links were checked after the move.

## Fabric follow-up

BMP Unicode decimal components retain Java i32 and UTF-16 behavior. Runtime-dependent Sinhala Lith/Myanmar Tai Laing core values remain unknown; build metadata is not compared. An independent Fabric 0.19.5 API oracle passed 1,263 recognized comparisons among 1,497 fixture cases on Java 8, 21 and 25. The Java 25 signed-JAR probe required execution outside the filesystem sandbox; its earlier security-provider load error did not establish a corrupt runtime. No Minecraft or loader instance was started.

## Verification

- 78 frontend tests passed.
- 230 native tests passed; 28 external/runtime/helper tests ignored by the standard offline invocation.
- TypeScript and Vite production build passed; initial chunk size warning remains advisory, not a launch benchmark.
- Native desktop visual/keyboard checks, gameplay, physical router tests and the friend's mod/RAM case remain unresolved.
- Linux is deferred. No commit, push, installation or publication is performed.

## Local Windows package

NSIS x64 build completed: 6,631,757 bytes. Native file/product versions are
0.26.0, both embedded icons match the expected logo, and all four copied license
resources match source hashes. Their NSIS inclusion was checked in the generated
installer script. SHA-256: `5b2a37358a0a0fcc6ff4f11c124c646b782cbd4443971933f22a5616696aafa6`.
The installer was not run, committed, pushed or published.
