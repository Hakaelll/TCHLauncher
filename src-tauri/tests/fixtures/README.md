# Real Minecraft integration checks

The nine opt-in scenarios in `src/process_lifecycle_test.rs` exercise real Java
processes, isolated server directories, commands, graceful stops, backups and
restores. They also cover simultaneous servers, plugin/mod enablement, world
conversion rollback and a world-plus-content archive round trip.

The standard Paper lifecycle additionally populates Overworld, Nether and End
with distinct blocks, exports their folders and imports the reviewed family into
a second Paper instance. It checks all copied file hashes before boot, then uses
Minecraft commands to verify the three saved blocks after boot without placing
new blocks in the target. Source and exported folders must remain unchanged.
This extends an existing scenario without adding another retained test case.

The standard Fabric lifecycle also imports a small generated `.mrpack` into a
second instance with real Lithium downloads and the exact source loader version.
It checks file selection, client exclusion, override precedence, local settings,
receipt/hash equality and a real ready/command/clean-stop session. This is a
controlled fixture, not broad third-party modpack compatibility.

Run one scenario at a time on Windows. Dependencies must already be available to
Cargo; Minecraft, loaders, Java and selected content can still require Internet
access. Supply a copy of an existing explicit Minecraft EULA acceptance record.
The checks validate that record and never accept the EULA themselves.

The Vanilla lifecycle also saves hard difficulty and creative mode through the
native settings owner before the first world exists. It checks Minecraft's
startup game-mode announcement and read-only seed/difficulty replies for seed 42
and Hard. No console setter supplies those expected values. The existing three
sessions and backup/restore checks still run; this adds no scenario.

```powershell
$env:TCH_TEST_CONSENT_RECORD = 'C:/path/to/existing-consent.json'
cargo test --manifest-path src-tauri/Cargo.toml --offline --lib process::lifecycle_tests::vanilla_ready_command_stop_restart_backup_restore -- --ignored --exact --nocapture
```

The NeoForge scenario also passed with `TCH_TEST_MINECRAFT=1.20.1`, selecting
historical 1.20.1-47.1.106 and managed Temurin 17. It verifies three real server
sessions, backup/restore, level.dat and MOTD equality, and restored boot. This
adds no test case and does not include additional mods, native UI or a client.

Most scenarios default to Minecraft 1.21.4. `TCH_TEST_MINECRAFT` overrides the
version for the shared lifecycle fixture, but scenario-specific upgrades and
content can have their own compatibility requirements. Read the chosen scenario
before overriding it. Fixtures attempt to stop owned processes and remove their
temporary directories on exit, including failures. Capture command output when
investigating a failed scenario.

These checks do not prove native UI, client gameplay, physical router support or
Internet connection quality. The old standalone Java predicate/properties probes
were retired with the large test-suite reduction.

## World publication process termination

The compact native safety case
`world_publication::tests::interrupted_family_publication_and_cleanup_recover_idempotently`
also starts the current test executable as a child process. It publishes a
synthetic three-folder world family through the production publication code,
signals a durable boundary, and waits. The parent terminates only its owned child
and waits for exit before recovery. It checks all eight boundaries: prepared plan,
three renames, committed plan and three ownership-marker cleanup steps.

Recovery must preserve each dimension's bytes, remove staging folders/markers,
clear the journal, succeed again without changes and remove temporary fixtures.
The original unwind/conflict/copy checks remain in the same retained case; no
additional test case was added. A child guard terminates and waits on failure,
and the wait for a boundary has a 15-second deadline.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --offline --lib world_publication::tests::interrupted_family_publication_and_cleanup_recover_idempotently -- --exact --nocapture
```

This scenario needs no Java, Minecraft download or EULA acceptance. It verifies
Windows process termination after the durable plan exists. It does not simulate
power loss, a native application crash, or termination before journal creation.
