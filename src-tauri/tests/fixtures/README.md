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

Run one scenario at a time on Windows. Dependencies must already be available to
Cargo; Minecraft, loaders, Java and selected content can still require Internet
access. Supply a copy of an existing explicit Minecraft EULA acceptance record.
The checks validate that record and never accept the EULA themselves.

```powershell
$env:TCH_TEST_CONSENT_RECORD = 'C:/path/to/existing-consent.json'
cargo test --manifest-path src-tauri/Cargo.toml --offline --lib process::lifecycle_tests::vanilla_ready_command_stop_restart_backup_restore -- --ignored --exact --nocapture
```

Most scenarios default to Minecraft 1.21.4. `TCH_TEST_MINECRAFT` overrides the
version for the shared lifecycle fixture, but scenario-specific upgrades and
content can have their own compatibility requirements. Read the chosen scenario
before overriding it. Fixtures attempt to stop owned processes and remove their
temporary directories on exit, including failures. Capture command output when
investigating a failed scenario.

These checks do not prove native UI, client gameplay, physical router support or
Internet connection quality. The old standalone Java predicate/properties probes
were retired with the large test-suite reduction.
