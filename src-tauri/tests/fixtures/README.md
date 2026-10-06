# Java properties seed probe

`SeedPropertiesProbe.java` independently reads the properties file with Java's
standard `Properties.load(InputStream)` and returns the seed as UTF-8 Base64.
It also checks that the fixture retains `online-mode=true`.
It does not launch Minecraft, generate a world, accept the EULA or access the network.

Compile the source to an ignored output directory with a JDK:

```powershell
javac --release 8 -d artifacts/java-properties/classes src-tauri/tests/fixtures/SeedPropertiesProbe.java
```

For a Java 8 compiler use `-source 8 -target 8` instead of `--release 8`.
Then select the Java executable to test:

```powershell
$env:TCH_TEST_JAVA = 'C:/path/to/java.exe'
$env:TCH_TEST_PROPERTIES_CLASSES = (Resolve-Path artifacts/java-properties/classes).Path
cargo test --manifest-path src-tauri/Cargo.toml --offline --lib properties::tests::real_java_properties_reads_exact_seed_strings -- --ignored
```

The test writes only its own temporary fixture, executes this probe and compares
the returned bytes with the seed supplied to the native writer. Java 8, 21 and 25
passed independently during development. With only a JRE available, the fixture
was compiled for Java 8 with the official Eclipse ECJ 3.44.0 compiler, downloaded
into ignored artifacts and checked against its published checksum. The compiler
is not a launcher dependency and is not bundled with the application.
