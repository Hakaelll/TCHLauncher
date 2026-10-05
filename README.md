# TCHLauncher

Gestor local de servidores Minecraft para Windows y Linux, construido con Tauri, Rust, React y TypeScript. El código actual es una base funcional en desarrollo; [NEXT_TODOS.md](NEXT_TODOS.md) registra lo pendiente.

## Vista previa actual: 0.18.0

[Instalador Windows 0.18.0](src-tauri/target/release/bundle/nsis/TCHLauncher_0.18.0_x64-setup.exe) - [Release notes (English)](RELEASE_NOTES_v0.18.0.md).

Temas End, Obsidiana y Nether en Ajustes, debajo de Inicio automático. La elección se recuerda entre sesiones. End usa negro/gris y morado; la tipografía Minecraft aparece en más controles y puede cambiarse por la de lectura. Logo sin fondo cuadrado, inicio con ilustraciones voxel y panel Java reorganizado. [design.md](design.md) describe el sistema visual. La comprobación visual nativa queda a cargo del usuario, como solicitó.

La 0.18.0 reorganiza Conexión con acceso desde internet siempre visible y detección automática UPnP/NAT-PMP. Retira Bore y mueve la respuesta de Minecraft a Rendimiento. Métricas, gráficas y barras de carga adoptan el estilo de inventario de Minecraft; el historial de fallos solo aparece cuando existe uno. Conserva el diagnóstico de memoria, copias y controles de operadores. La apertura TCP sigue siendo temporal y confirmada, con renovación y retirada de aperturas propias. La comprobación con router físico sigue pendiente; no verifica accesibilidad externa ni cambia el firewall.

## Ejecutar

Requisitos de desarrollo: Rust, Node.js, pnpm y los prerrequisitos de Tauri 2 para el sistema operativo.

```sh
pnpm install
pnpm tauri dev
```

Para compilar el frontend: `pnpm build`. Para las pruebas Rust: `cargo test --manifest-path src-tauri/Cargo.toml`. Las pruebas de telemetría se ejecutan con `pnpm test` (Node.js 24).

## Flujo disponible

1. Crear un servidor Vanilla, Paper, Fabric o NeoForge con un asistente de cinco pasos: identidad, versión, mundo, capacidad y revisión. Se elige una versión release del catálogo de Mojang y, opcionalmente, un mundo existente.
2. Descargar un runtime Temurin local y el servidor desde Mojang o PaperMC. Los artefactos con hash publicado se verifican antes de usarse.
3. Leer y aceptar explícitamente la EULA de Minecraft.
4. Iniciar el servidor, ver la salida, enviar comandos y detenerlo de forma ordenada, también durante el inicio.
5. Editar `server.properties` y memoria con el servidor detenido.
6. Crear y restaurar copias de seguridad locales con el servidor detenido.
7. Importar una carpeta existente, copiándola a la biblioteca o administrándola en su ubicación original, y preparar Java sin sustituir su `.jar`.
8. Elegir una imagen PNG, JPG o WebP para un servidor nuevo, importado o ya existente. Se adapta a `server-icon.png` de 64 × 64 y se muestra tanto en el launcher como en la lista de Minecraft.
9. Ajustar RAM, jugadores y objetivo de espacio escribiendo un valor o moviendo un deslizador. El objetivo de espacio compara el tamaño de la carpeta del servidor y avisa al superarlo; no limita el disco.
10. Consultar jugadores conocidos y conectados, lista blanca, operadores y bloqueos. Con el servidor en marcha, enviar comandos para gestionar permisos, expulsiones y bloqueos. Importar una lista blanca `.txt` o `whitelist.json` con vista previa.
11. Buscar y filtrar la consola por nivel, copiar las líneas visibles y exportar el log completo. El Resumen muestra eventos recientes detectados en la salida.
12. Reducir la barra lateral a una columna de imágenes de servidores, con nombre al pasar el cursor o enfocar con teclado. La elección se recuerda entre sesiones.
13. Gestionar mundos locales: importar una copia, duplicar, exportar, cambiar el mundo principal, abrir su carpeta y eliminarlo con copia previa. La pestaña Mundos requiere detener el servidor para operaciones de archivos.
14. Eliminar un servidor detenido desde Configuración con una confirmación que muestra el alcance. Las carpetas importadas administradas en su ubicación original permanecen en el disco.
15. Reiniciar un servidor en marcha: se envía `stop`, se espera al cierre del proceso y se vuelve a iniciar. La parada forzada permanece como último recurso.
16. Ver la dirección LAN y el estado TCP del puerto configurado; copiar la dirección local. La consulta de IP pública mediante [ipify](https://www.ipify.org/) es opcional y no implica que el puerto sea accesible desde internet.
17. Seguir RAM y CPU simultáneamente en la pestaña Rendimiento con gráficas de dos minutos que se conserva al cambiar de sección. Consultar valores actuales y picos, tasas de lectura/escritura, PID y heap máximo. Los valores proceden del proceso Java y no representan TPS.

Los datos se almacenan en el directorio de datos de la aplicación del sistema. No hay cuentas ni servicio cloud propio. Algunas descargas requieren acceso a internet.

## Probar un servidor real en v0.6.0

Crear un servidor Vanilla, instalarlo, leer y aceptar su EULA e iniciarlo. La Consola permite seguir el arranque; el estado pasa a En marcha cuando Minecraft termina de cargar el mundo. Abre Minecraft Java Edition con la misma versión y conecta a la dirección LAN mostrada; desde el equipo anfitrión puedes usar `localhost:<puerto configurado>`. Comprueba jugadores, métricas y comandos y después detén el servidor antes de modificar propiedades o hacer una copia.

Paper también está disponible si publica un build estable para la versión elegida. El selector utiliza el catálogo de releases de Mojang. La v0.6.0 ha pasado compilación, 21 pruebas Rust y 6 de telemetría; la instalación nativa, el ciclo completo con Java y la conexión desde el juego siguen pendientes de prueba manual.

## Generar un instalador de Windows

Vista previa actual: [instalador Windows x64 0.9.0](src-tauri/target/release/bundle/nsis/TCHLauncher_0.9.0_x64-setup.exe), con checksum SHA-256 junto al archivo. [Release notes 0.9.0](RELEASE_NOTES_v0.9.0.md) en inglés y [mensaje de commit](COMMIT_MESSAGE.txt). Incluye modpacks, restauración/retención, scheduler, recuperación tras fallos e inicio explícito; revisión visual y comprobación de juego/inicio de sesión pendientes.

```sh
pnpm tauri build --bundles nsis
```

El instalador actual de Windows x64 es `src-tauri/target/release/bundle/nsis/TCHLauncher_0.8.0_x64-setup.exe`, con checksum SHA-256 junto al archivo. El binario y el instalador no están firmados digitalmente. Los cambios y las limitaciones se describen en inglés en [RELEASE_NOTES_v0.8.0.md](RELEASE_NOTES_v0.8.0.md). Los cambios posteriores están en [RELEASE_NOTES_UNRELEASED.md](RELEASE_NOTES_UNRELEASED.md).

## Estado del proyecto

Consulta [ARCHITECTURE.md](ARCHITECTURE.md) para la estructura, [design.md](design.md) para la referencia visual aportada y [NEXT_TODOS.md](NEXT_TODOS.md) para funciones pendientes. Fabric, NeoForge, mods, plugins, importación de modpacks, copias y automatizaciones están conectados. La red pública automática, Linux y varias comprobaciones de escritorio y juego siguen pendientes; consulta las limitaciones y evidencias de cada función más abajo.

### Cambios de 0.6.1

- Entrada con opciones visuales inspiradas en aumentos de TFT, acentos de color y movimiento breve.
- Menú de clic derecho para fijar/desfijar servidores, abrir configuración y solicitar eliminación confirmada.
- Cabezas de skins reales por UUID, con caché local y alternativa cuando no están disponibles.
- Etiquetas de consola coloreadas y autocompletado de WebView2 desactivado.
- Descargas cancelables con progreso, velocidad, reintentos y limpieza de parciales.
- Gestión visible de Java y fallback de JRE a JDK de la misma versión para Temurin 16.

Validación 0.6.1: 30 pruebas Rust locales, 6 de telemetría y las 2 comprobaciones con servicios oficiales aprobadas. Las comprobaciones con servicios oficiales son optativas: `cargo test --manifest-path src-tauri/Cargo.toml installs_real_minecraft_1171_and_temurin16_without_accepting_eula -- --ignored --nocapture`. Usan carpetas temporales y no aceptan la EULA. La prueba Fabric arranca Minecraft hasta el aviso de EULA y comprueba que no se haya creado un mundo. La revisión visual y la conexión desde un cliente Minecraft requieren prueba manual.

### Cambios de 0.6.2

Fabric aparece en creación e importación. Su catálogo muestra versiones de Minecraft compatibles, y la instalación elige un loader estable con bibliotecas verificadas y publicación atómica. La importación mantiene los archivos existentes, incluidas generaciones creadas por TCHLauncher. La comprobación real usa Minecraft 1.21.4 y Fabric Loader 0.19.5 y llega al aviso de EULA sin aceptarla. En 0.6.2, NeoForge, gestión de mods y actualización de loaders seguían pendientes; su estado posterior se describe más abajo.

### Java personalizado

En Configuración, abre **Java personalizado** y elige `bin/java.exe` (Windows) o `bin/java` (Linux). La aplicación verifica la versión requerida por Minecraft y muestra la ruta antes de confirmar. Puedes volver a Temurin gestionado desde el mismo panel. La comprobación de arranque usa los requisitos oficiales guardados, sin modificar el Java global.

Prueba dirigida con Java 21 instalado: define `TCH_TEST_JAVA` con la ruta del ejecutable y ejecuta `cargo test --manifest-path src-tauri/Cargo.toml verifies_real_custom_java_without_changing_an_incompatible_selection -- --ignored`.

### Biblioteca y perfiles JVM (0.6.4)

La biblioteca permite buscar y alternar tarjetas/lista. La navegación del servidor está junto al rail oscuro, que conserva los menús contextuales. Para imágenes grandes de servidores anteriores, vuelve a elegir el archivo original: su icono anterior ya estaba reducido a 64 × 64. En Configuración, abre Argumentos JVM para elegir el perfil recomendado o introducir una opción por línea. Guarda RAM antes de revisar la vista previa; los cambios de perfil se aplican en el siguiente inicio.


### En desarrollo, sin nueva release

La navegación ahora usa una sola barra de servidores plegable y las secciones dentro del área principal. El banner repetido del resumen se ha retirado y el emblema proporcionado sustituye a la T. La aceptación de EULA se solicita antes de la primera descarga y se recuerda en este equipo para los siguientes servidores; crear una instancia no registra consentimiento por sí solo.

NeoForge dispone de catálogo estable compatible, instalación en una generación aislada y arranque mediante argumentos modulares de Java. Los mundos, mods y configuración siguen en el directorio del servidor. El instalador y las bibliotecas se verifican; los mappings oficiales se añaden a la caché del instalador para preparar offline. No se ejecutan los scripts run.bat o run.sh ni se aplican automáticamente argumentos de user_jvm_args.txt; utiliza el editor JVM del launcher.

Comprobación dirigida: `cargo test --manifest-path src-tauri/Cargo.toml installs_neoforge_and_imports_without_accepting_eula -- --ignored --nocapture`. Descarga archivos oficiales y arranca la instalación original y su copia hasta el rechazo de EULA, sin aceptarla. Si defines TCH_TEST_JAVA con un Java 21 instalado, la prueba copia ese runtime a su carpeta temporal; no modifica la instalación original. La conexión desde Minecraft con mods, la actualización de loader y Linux siguen pendientes.

## Mods and plugins in 0.7.0

Fabric and NeoForge servers show Mods; Paper servers show Plugins, including compatible Bukkit/Spigot releases. Browse Modrinth, review the selected version and dependency changes, then install while the server is stopped. Downloads are verified with SHA-512 and staged before switching directories. Individual and combined updates create a backup; removal requires confirmation and also saves a backup. Files added outside the launcher remain visible and are preserved, but their dependencies cannot be inferred automatically.

Release notes: [0.7.0](RELEASE_NOTES_v0.7.0.md). Native visual review and in-game connections remain manual.

## Modpack import (0.8.0 preview)

Choose **Importar un modpack .mrpack** in the server library. The wizard reviews server files and optional content, then creates a new instance with the package's exact Minecraft and Fabric/NeoForge loader versions. Client files are excluded; server properties, EULA and the chosen server icon are preserved. Unsupported engines produce an explanation before installation.

Downloads are hash-checked and size-limited. Installation prepares a separate directory, records the directory switch and commits launch paths in SQLite afterward. Startup recovery restores the initial files before a database commit, or retains the published instance after a commit. Cancelling keeps the source archive and initial server files. Imported package content is visible in Mods as externally managed files; automatic project/dependency tracking is not inferred from filenames.

Live integration test: `cargo test --manifest-path src-tauri/Cargo.toml installs_real_pack_and_starts_published_instance -- --ignored --nocapture`. The default tests Fabric 0.16.10 / Minecraft 1.21.4 with Lithium. Set `TCH_TEST_PACK_PLATFORM=neoforge` for NeoForge 21.11.45 / Minecraft 1.21.11 with FerriteCore. Tests install official files in an isolated library and start Java only to the EULA refusal; they never accept it or create a world. Native visual review and in-game connections remain manual. These features are not included in the 0.7.1 installer.

## Backup restoration (0.8.0 preview)

New backups include launcher metadata alongside the server files. Restoring them restores the Minecraft/platform versions, launcher generation, memory/storage targets and JVM settings in one database transaction. The current folder is backed up before it is replaced. A restore journal recovers an interrupted directory switch on the next launch and keeps the restored instance after a confirmed database commit.

If the Minecraft version changes, prepare its Java runtime again before starting. The global EULA choice remains remembered. Older backups without launcher metadata use detected Minecraft information where available; an unidentifiable old installed server needs inspection through server import rather than assuming its version. Real gameplay restoration checks remain pending. These changes are included in the 0.8.0 preview; they are not in the 0.7.1 installer.

### Backup retention (0.8.0 preview)

In **Copias → Retención de copias**, enable automatic cleanup and save a maximum count, a storage limit, or both. Retention is disabled by default. Saving applies the policy to future successful backups; **Revisar limpieza de copias** previews deletion of existing copies and requires confirmation. Cleanup only handles registered copies in the server's owned backup library.

The newest copy is always preserved. During restoration, the selected source and the newly created safety copy are protected. Protected copies can exceed the configured limits; the preview explains this. File sizes are measured from disk, and a changed review must be repeated. A cleanup failure retains the newly saved copy and reports a warning; interrupted cleanup is recovered at startup. Local tests cover pruning, stale reviews, database rollback, interrupted recovery and restoration with retention enabled. Native visual review remains pending.

### Local scheduled tasks (0.8.0 preview)

In **Configuración → Tareas programadas**, choose a first execution time and an optional interval in minutes. Available actions are start, restart, stop, backup while stopped, and backup with an explicit stop/restart interruption. The second backup option resumes a previously running server even if copying failed, and reports any restart failure. An already stopped server stays stopped after its backup. Startup validates the installed runtime and remembered EULA choice; tasks do not install missing runtimes silently.

Tasks persist in SQLite and run while the launcher is open, including when its window is hidden. Overdue tasks execute once when the application opens; missed interval ticks do not accumulate. Paused periodic tasks resume with a future execution time. Each row shows its next execution and last result. A durable claim prevents duplicate occurrences and serializes tasks for one server. Interrupted occurrences are recorded without replaying them, and an exclusive library lock prevents another launcher instance from resetting the running tasks.

Local tests cover scheduling, pauses, concurrent claims, interruption recovery, ownership and a real filesystem backup with retention. Native scheduled stop/restart, Minecraft gameplay and Linux checks remain pending. Application autostart and selected-server startup are available in the preview; actual login startup still needs verification. These changes are included in the 0.8.0 preview; they are not in the 0.7.1 installer.

### Crash recovery (0.8.0 preview)

**Configuración → Tareas programadas → Reinicio tras un fallo** exposes an opt-in per-server policy. Set a delay of 5–3600 seconds and a maximum of 1–10 attempts. Saving the policy resets its counter and ignores any crash that happened before the preference was enabled. Requested stops, including a forced shutdown after the stop timeout, do not trigger automatic restart.

A failed process exit schedules one delayed retry. Startup preparation failures retry within the same budget; another quick process crash continues that budget. Ten minutes in the actual running state resets it, excluding time spent starting up. An occupied server defers the retry without consuming an attempt. Newer process/configuration changes cancel stale attempts, and closing the launcher cancels pending retries without replaying them at startup. The panel shows attempts used, next retry time and the last result. Saving cannot race an active server operation.

SQLite policy/claim/retry tests pass; native Java crash, user-stop and desktop interaction checks remain pending. These changes are included in the 0.8.0 preview; they are not in the 0.7.1 installer.

### Application and selected-server startup (0.8.0 preview)

Open **Inicio automático** from the settings button at the bottom of the server rail. Application startup and server selection are separate explicit preferences, both disabled by default. Application startup registers the current installed executable for the current user and opens the launcher in the tray. Development builds cannot register their executable. Existing registrations pointing to another installation can be updated from the dialog.

Each selected server starts whenever the launcher opens, using the same runtime, EULA, icon synchronization and operation checks as manual startup. Results are retained in the dialog; a failed startup shows the launcher window. Selecting a server does not start it immediately, install Java, download its files or accept an agreement.

Windows uses the current user's [Run registration](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys). Linux uses the user's [desktop-session autostart directory](https://specifications.freedesktop.org/autostart/latest/) and [Exec quoting rules](https://specifications.freedesktop.org/desktop-entry/latest/exec-variables.html), with the AppImage path where available. Only the launcher's owned entry is changed; foreign desktop files are preserved. Local checks cover persisted selection, Windows command encoding and temporary desktop-entry replacement/removal. No test enabled startup on this computer. Actual login startup, native server startup and Linux integration remain pending.

### Loader updates (development source after 0.8.0)

For installed Fabric/NeoForge servers with managed metadata or an identifiable imported launcher, **Configuración → Fabric Loader / NeoForge → Buscar actualizaciones** reviews newer stable releases for the same Minecraft version. Choose a version and confirm **Crear copia y actualizar**. Preparation uses the configured Java runtime, preserves worlds/mods/properties, and switches launch paths only after a separate generation is complete. A recovery journal retains the previous generation on publication failure or an interrupted database commit. The backup can restore the old launcher settings.

Local tests cover publication, preserved world/configuration, database rollback, interruption recovery and protection of the active generation. A live official-download/Java check updates Fabric 0.16.10 to 0.19.5 on Minecraft 1.21.4, boots it to EULA refusal, restores its safety backup and boots 0.16.10 again. Java selection, JVM arguments and configuration are preserved. The equivalent NeoForge check updates 21.4.157 to 21.4.158 and restores 21.4.157 with the same Java 21 runtime. Both updated and restored servers reach EULA refusal; no agreement is accepted and no world is created. Windows canonical launch paths are covered by a regression test. Native interaction, gameplay and Linux checks remain pending. Imported NeoForge argument files and classic Fabric launchers with local classpath libraries are identified without launcher-owned metadata. Their update/backup/restore cycles have also passed isolated official-download/Java checks. Embedded unattended Fabric launchers are identified through install.properties with local regression coverage; their full live update cycle remains pending. Legacy shaded/custom launchers without sufficient identity data still need additional support. Cancellation during the safety backup completes once that backup finishes. This feature is not in the 0.8.0 installer.

Run the isolated integration with `TCH_TEST_JAVA` pointing to Java 21: `cargo test --manifest-path src-tauri/Cargo.toml updates_real_loader_and_restores_previous_generation -- --ignored --nocapture`. Fabric is the default; set `TCH_TEST_UPDATE_PLATFORM=neoforge` for NeoForge. Set `TCH_TEST_UPDATE_IMPORTED=1` to copy the verified engine into a conventional external-server layout, import it without managed metadata and run the same cycle. The test selects newer stable official releases, so the exact target can change with the catalog. Failed runs retain their temporary directory for diagnostics.

Imported identity follows the official [Fabric server installer](https://github.com/FabricMC/fabric-installer/blob/master/src/main/java/net/fabricmc/installer/server/ServerInstaller.java) and [unattended launcher](https://github.com/FabricMC/fabric-installer/blob/master/src/main/java/net/fabricmc/installer/ServerLauncher.java) formats. Version detection checks Minecraft compatibility, referenced local files and contradictory metadata. SHA-512 review fingerprints include the Fabric loader JAR, so changing it invalidates a pending review. Windows verbatim filesystem paths are converted only when passed to Java, including UNC paths; persisted paths and ownership validation retain their original form.

### Minecraft version changes (development source after 0.8.0)

Settings provides stable version selection, Java/engine review and installed mod/plugin compatibility checks backed by SHA-512. Verified managed projects are resolved together against the target Minecraft version, including required dependencies and exact-version conflicts. The dialog shows the proposed replacements, additions and changelogs before confirmation. Manual or modified files remain unverified and require acknowledgement; unavailable compatible releases block migration.

A safety backup precedes isolated content/engine preparation. Directory, Minecraft version, launch path and Java selection are published with SQLite recovery. Official-download Java 21 checks passed upgrade/restore cycles for Vanilla, Fabric and NeoForge 1.21.4→1.21.5→1.21.4 and Paper 1.21.4→1.21.8→1.21.4. Additional migration cycles with Lithium on Fabric and FerriteCore on NeoForge passed, checking reviewed content hashes, restored inventory and backup counts. All boots stopped at EULA refusal without accepting terms or generating worlds. Paper 1.21.5 correctly failed review because no STABLE build was available.

The update dialog can start the server and observe native readiness for up to three minutes. A failure or unconfirmed start links to the console; leaving this panel stops observation without stopping Java. This flow is included in the 0.9.0 Windows preview. Native interaction, client gameplay/world conversion and Linux checks remain pending.

Run the isolated integration with Java 21 already installed:

```powershell
$env:TCH_TEST_JAVA = 'C:\path\to\java.exe'
$env:TCH_TEST_UPDATE_PLATFORM = 'fabric' # vanilla, fabric, neoforge or paper
$env:TCH_TEST_UPDATE_CONTENT = 'lithium' # optional managed project
cargo test --manifest-path src-tauri/Cargo.toml minecraft_update::tests::updates_real_minecraft_and_restores_previous_installation -- --ignored --nocapture
```

### Experimental Minecraft catalog (development after the 0.9.0 installer)

In the creation wizard's version step, **Mostrar versiones experimentales** explicitly enables snapshots and prereleases for Vanilla or Fabric. It is off by default, experimental choices are labeled in the selector/final review, and disabling it reloads the stable catalog. Fabric intersects supported game IDs with Mojang's official release/snapshot classification. Paper and NeoForge continue using their stable platform catalogs; the normal update dialog remains restricted to stable upgrades.

The release list now retains all valid official release IDs rather than truncating it to fifty entries. Legacy alpha/beta categories and unsafe IDs are excluded. Separate isolated official-download checks installed 26.4-snapshot-2 on Vanilla and Fabric with Java 25 and reached EULA refusal, preserving properties without accepting terms or generating worlds. Client gameplay and native checkbox interaction still require verification; no new installer was generated for this addition.

The snapshot integration can be rerun with an already installed matching Java executable:

```powershell
$env:TCH_TEST_JAVA = 'C:\path\to\java.exe'
$env:TCH_TEST_SNAPSHOT_PLATFORM = 'fabric' # vanilla or fabric
cargo test --manifest-path src-tauri/Cargo.toml install::tests::installs_snapshot_and_reaches_eula_without_accepting -- --ignored --nocapture
```

By default it selects the latest experimental entry from the official platform catalog. `TCH_TEST_SNAPSHOT` can select a specific catalog-listed snapshot. The configured test Java must match the official major requirement; each run uses a temporary library and never registers consent. Failed runs retain their diagnostic files.
# Windows preview 0.18.0

The current preview reorganizes Connection with automatic router discovery, removes
Bore and adds Minecraft inventory styling to metrics, charts and progress bars. It retains automatic stop
backups, historical failure review and operator editing. See [release notes](RELEASE_NOTES_v0.18.0.md)
for verification limits and the installer checksum.

- **Rendimiento → Exportar diagnóstico:** save up to 30 minutes of native resident/heap/collector samples from the hosting computer. Keep that server selected while playing; missing readings remain explicit. Reports do not prove a memory leak.
- **Rendimiento → Respuesta de Minecraft:** explicitly query the local Java server. This measures its local status exchange, not internet latency or a friend’s connection.

- **Copias → Copia al detener:** enable a per-server automatic copy after a successful
  session that reached Running. It is off by default. Progress/cancellation appears
  in Tareas; retention follows the saved backup policy. A failed or cancelled copy
  does not retry the same session automatically. **Detener y salir** waits for its copy.
- **Jugadores → Operadores → Permisos:** edit an existing operator's level and ability
  to enter a full server while stopped. The UUID remains unchanged. Settings apply
  on the next start; plugins may provide their own permissions. An exact original
  `ops-before-UUID.json` remains in the server folder before replacement.
- **Rendimiento → Tiempo de arranque:** measured from Java launch to the first server
  ready event for the current process; absent until readiness or after process removal.
- **Resumen → Consultar el último fallo guardado:** review the dated last failure
  without confusing it with the current successful session.

Native desktop interaction, real Minecraft gameplay, physical-router behavior and
Linux distribution still require verification. This preview does not complete the
full application specification.
