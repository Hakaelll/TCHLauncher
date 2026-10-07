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

## Fabric version predicate probe

`FabricPredicateProbe.java` calls the official Fabric API's `Version.parse` and
`VersionPredicate.parse` for a batch of Base64-encoded fixture values. It does not
instantiate Fabric Loader, launch Minecraft, install mods or accept the EULA.
Only recognized native verdicts are required to match the independent oracle;
unknown native cases remain unverified rather than being declared incompatible.

Download Fabric Loader 0.19.5 from its official Maven repository into ignored
`artifacts/fabric-predicates` and verify the published checksum before using it.
The development artifact's SHA-256 was
`93044e4dd46de5d8136701292f05e868da096d2c9fddb4793e4fdbcc63efc695`.
Neither this JAR nor the compiler is shipped as an application dependency.

```powershell
javac --release 8 -cp artifacts/fabric-predicates/fabric-loader-0.19.5.jar -d artifacts/fabric-predicates/classes src-tauri/tests/fixtures/FabricPredicateProbe.java
$env:TCH_TEST_JAVA = 'C:/path/to/java.exe'
$env:TCH_TEST_FABRIC_CLASSPATH = (Resolve-Path artifacts/fabric-predicates/classes).Path + ';' + (Resolve-Path artifacts/fabric-predicates/fabric-loader-0.19.5.jar).Path
cargo test --manifest-path src-tauri/Cargo.toml --offline --lib fabric_version::tests::recognized_predicates_match_the_real_fabric_api -- --ignored --nocapture
```

The numeric-component table uses BMP decimal-digit zero points from the official
[Unicode 15 data](https://www.unicode.org/Public/15.0.0/ucd/UnicodeData.txt).
Supplementary-plane digits are not numeric components because Java's integer
parser uses UTF-16 `char` digits. Sinhala Lith and Myanmar Tai Laing decimal
digits differ between Java 8 and modern Java; since the static audit does not
establish the runtime's Unicode version, those core values return no verdict.
Build metadata remains excluded from numeric comparisons.
