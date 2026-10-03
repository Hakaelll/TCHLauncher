# Arquitectura

`src/` contiene la interfaz React y un cliente tipado de comandos Tauri. `src-tauri/src/` contiene módulos de almacenamiento, instalación, procesos, propiedades, jugadores y copias. Los comandos del backend validan y leen de disco; el frontend no decide por sí mismo si un servidor ha arrancado.

El registro de servidores y copias está en SQLite (`database/tchlauncher.sqlite`). Cada servidor tiene UUID y directorio propio bajo `servers/<uuid>`. Java gestionado está en `runtimes/` y las copias en `backups/<uuid>/`. El objetivo de espacio se guarda por servidor y se migra con un valor predeterminado de 20 GB para registros anteriores. Mide la carpeta del servidor, sin incluir las copias. `server.properties`, `eula.txt`, mundos y logs siguen siendo archivos ordinarios.

Las imágenes se decodifican con límites de tamaño y memoria, se recortan a 64 × 64 y se publican como PNG en `server-icon.png` dentro del servidor. Launcher y Minecraft comparten el mismo archivo. Los archivos antiguos `server-images/<uuid>` se migran antes del inicio; no se modifica la imagen original del usuario. Las operaciones requieren el servidor detenido.

`skin.rs` obtiene perfiles por UUID desde Mojang, valida que la textura procede de `textures.minecraft.net` y recorta rostro y sombrero a una cabeza de 8 × 8. Cachea PNG durante 24 horas y limita respuestas y decodificación. Un fallo conserva una caché previa o una inicial; no inventa skins. Las consultas se ejecutan fuera del hilo de interfaz.

La migración SQLite 2 añade `pinned`, con valor falso para registros anteriores. La lista ordena los fijados primero y persiste la elección.

La instalación consulta el catálogo oficial de Mojang y la API de PaperMC. La versión de Java procede de los metadatos de Minecraft. El runtime se descarga desde Eclipse Temurin, buscando JRE y después JDK de la misma versión mayor (incluido Java 16 para Minecraft 1.17.1), se verifica mediante SHA-256 y se extrae en un directorio temporal antes de moverlo a su destino. El jar Vanilla se verifica con SHA-1; Paper utiliza SHA-256 cuando la respuesta del servicio lo proporciona. No se acepta EULA automáticamente.

`ProcessManager` mantiene los procesos activos y emite eventos Tauri de estado y consola. Conserva el último estado terminal durante la sesión para que un fallo siga visible después de retirar el proceso. Envía `stop` por stdin durante inicio o ejecución y fuerza la terminación solo tras 30 segundos; las peticiones repetidas de parada se ignoran. La salida de inicio solo puede promover un proceso que aún esté en `starting`. Si se cierra la ventana con servidores activos, el usuario puede cancelarlo, detenerlos o mantener la aplicación en la bandeja del sistema. Al iniciar la aplicación, el estado se reconstruye a partir de los procesos bajo control de esta instancia; no se recuperan procesos externos u huérfanos.

`get_metrics` toma muestras del proceso Java con `sysinfo` en una tarea bloqueante separada: RAM física del proceso, CPU normalizada sobre los núcleos del equipo y contadores acumulados de lectura/escritura. Cada respuesta incluye servidor, PID y hora de muestreo; la primera lectura de CPU puede no estar disponible. El tamaño de la carpeta se cachea durante 60 segundos. La interfaz consulta aproximadamente cada tres segundos sin solapar consultas y conserva un historial de dos minutos por servidor en `App`. `telemetry.ts` descarta respuestas tardías, reinicia el historial cuando cambia el proceso, calcula tasas con el tiempo real entre muestras y separa huecos mayores de 15 segundos. El historial no se guarda en disco y no mide TPS.

Las propiedades se releen al cambiar el estado del servidor, porque Minecraft puede generar valores durante el primer inicio. Las respuestas antiguas se descartan y las modificaciones pendientes del usuario tienen prioridad hasta guardar. `Modal.tsx` presenta los diálogos mediante un portal y la capa superior nativa, con foco inicial, cierre por Escape cuando procede y restauración del foco sin desplazar el contenido.

`log.rs` añade una marca UTC a cada línea capturada, clasifica el nivel y algunos eventos operativos, y lee un tramo reciente para la Consola y el Resumen. Acepta líneas antiguas sin marca. La exportación copia el log completo a una ruta elegida por el usuario sin modificar el original. Limpiar la vista de Consola es un filtro local y conserva el historial y los eventos del Resumen.

La sección Jugadores lee los archivos Vanilla que Minecraft mantiene para jugadores conocidos, lista blanca, operadores y bloqueos. Las modificaciones con el servidor en marcha se envían por la consola, y los conectados se derivan de los eventos de entrada y salida del proceso gestionado. Tras reiniciar la aplicación, la lista de conectados se reconstruye a partir de nuevos eventos.

Las copias se hacen con el servidor detenido y se activan por intercambio de directorios. La restauración crea antes una copia del estado actual.

La importación inspecciona una carpeta existente mediante un selector nativo. Puede registrarla en su ubicación original o copiarla a la biblioteca. El `.jar` importado permanece intacto; se prepara únicamente el runtime Java compatible.

`world.rs` identifica carpetas de mundo inmediatas mediante `level.dat` comprimido y datos NBT básicos. Puede importar, duplicar y exportar a través de copias preparadas en directorios temporales y renombradas al finalizar. Rechaza nombres de ruta inseguros y enlaces simbólicos. Cambiar el mundo principal actualiza `level-name` después de guardar una copia; borrar también crea antes una copia y nunca permite eliminar el mundo activo. La creación de un servidor puede importar un mundo elegido a `world` antes de instalarlo. Las operaciones de archivos reservan el servidor y bloquean su inicio hasta terminar.

`network.rs` lee el puerto y la IP de escucha desde `server.properties`, obtiene la dirección IPv4 de la ruta local sin enviar paquetes y comprueba el puerto TCP local. El Resumen presenta la dirección LAN cuando puede determinarse. La consulta de IP pública se realiza solo si el usuario la solicita; utiliza el servicio gratuito [ipify](https://www.ipify.org/) y no comprueba la accesibilidad desde internet.

`upnp.rs` detecta IGD IPv4 mediante SSDP y limita el control HTTP al router privado anunciante, sin redirecciones ni proxy. La apertura TCP requiere confirmación explícita y coincidencia de IP/puerto con lo mostrado; su intención se persiste antes de modificar el router. Las concesiones de una hora se comprueban y renuevan solo con propietario, dirección y servidor en ejecución coincidentes. La parada, cambio de dirección/puerto y cierre activan retirada y reintento durables, conservando los registros tras eliminar un servidor. El cierre espera la limpieza en un hilo de trabajo; las concesiones temporales siguen teniendo caducidad si el router no responde. La interfaz distingue confirmación del router de accesibilidad real desde internet. Diez pruebas locales cubren transporte y ciclo de concesión; verificación con router físico, NAT-PMP y comprobación remota siguen pendientes.

## Limitaciones actuales

La base todavía no gestiona mapeo automático de puertos ni el resto de plataformas y gestores especificados. Se detallan en `NEXT_TODOS.md`.

## Descargas y runtime

`download.rs` usa HTTP asíncrono desde un worker, con cancelación durante conexiones, cuerpos y esperas de reintento. Hay hasta tres intentos para fallos transitorios; errores permanentes y discrepancias de hash no se reintentan automáticamente. Los bytes se escriben en archivos parciales únicos y solo se publican después de verificar el hash. Los parciales se limpian al cancelar o fallar; los artefactos ya verificados se pueden conservar.

Cada evento lleva servidor, tarea, tipo de operación, fase, bytes, total, velocidad y número de intento. El frontend conserva el estado por servidor al navegar. El servidor permanece reservado hasta terminar la tarea. La extracción comprueba cancelación entre entradas y usa un directorio de preparación. Los runtimes compartidos tienen un bloqueo por ruta; reinstalar crea una generación nueva para no sustituir ejecutables usados por otro servidor.

`runtime.rs` verifica `java -version` en un proceso oculto con tiempo límite. Configuración muestra la versión y ruta leídas, con acciones de instalar, reinstalar y verificar. En Windows se desactivan autofill y guardado de contraseñas de WebView2 sobre su hilo UI.

## Fabric

`fabric.rs` consulta el catálogo oficial, selecciona un loader estable compatible y descarga cada biblioteca desde Maven Fabric con SHA-256. Los checksums ausentes en el perfil se consultan en sus archivos oficiales `.sha256`, con tamaño acotado. El servidor Mojang conserva su comprobación SHA-1.

La instalación se prepara bajo `.tch-fabric/.stage-<uuid>` dentro del servidor y se publica como una generación completa. Solo después se actualiza SQLite; un fallo o cancelación limpia la generación sin modificar mundos ni propiedades. El manifiesto del launcher se genera con classpath relativo, líneas plegadas y clase principal leída del loader verificado. `installation.json` conserva versiones y bibliotecas para inspección. No se ejecuta un instalador externo ni se acepta la EULA.

El arranque añade `fabric.gameJarPath` para localizar el JAR Mojang de esa generación y conserva el directorio del servidor como raíz de mundos, mods, propiedades y logs. Las carpetas Fabric convencionales importadas mantienen su launcher y configuración; las generaciones de TCHLauncher también se reconocen y se copian con rutas relativas. El catálogo del asistente descarta respuestas obsoletas al cambiar de plataforma. Las actualizaciones de loader y el gestor de mods están conectados; sus límites actuales y verificaciones se documentan en las secciones siguientes y NEXT_TODOS.md.

## Custom runtime validation (0.6.3)

Each server can select a canonical Java executable through a native picker. Selection inspects `java -version`, checks the exact Java major from official Minecraft metadata and returns a preview. Confirmation repeats verification before persisting the path. Start/restart validate the configured executable in a blocking worker under the server operation reservation. Requirements are cached per Minecraft version; legacy records with no cache remain version-verified without claiming compatibility or forcing network access at launch. Managed installation refreshes the requirement cache.

## JVM and artwork (0.6.4)

JVM profiles live in a separate SQLite table linked to servers with cascading deletion. The same native argument builder supplies preview and launch, including Fabric paths. Custom saves verify the installed Java in a blocking worker under a stopped-server reservation. Artwork publication stages the original and a derived Minecraft icon, backs up previous files and rolls back the pair on failure. Full covers are loaded lazily, refreshed through a server-artwork-changed event and omitted from recurring server-list IPC payloads.

## Reviewed upgrades and experimental catalogs

`loader_identity.rs` detects identifiable Fabric/NeoForge engines from owned metadata or conventional local launcher files. `loader_update.rs` prepares a separate engine generation and commits its launch path through recovery journals and SQLite markers. Java paths are normalized only when passed to Java; filesystem ownership checks retain canonical native paths.

`minecraft_update.rs` reviews official release chronology, Java requirements, exact engine availability and installed content hashes. Managed roots are resolved as a single target-compatible dependency graph through `content.rs`; `content_storage.rs` installs that reviewed plan into a copied instance. `instance_publication.rs` switches adjacent directories and records Minecraft, launcher and Java selection together. Its recovery journal distinguishes committed from uncommitted publication; a prior safety backup remains available. Unknown/manual/modified content is preserved with warnings rather than declared compatible.

The update interface can request the existing native start command after publication. `startupVerification.ts` observes native state for up to three minutes; Java process creation does not imply readiness. Leaving the panel cancels observation without stopping Java. A failed start does not automatically restore worlds. Client gameplay/world conversion and native observation-flow validation remain pending.

Experimental selection is an explicit creation-wizard option, disabled by default. `install.rs` classifies Mojang IDs by their real release/snapshot type and intersects Fabric IDs with the official game catalog. The stable update review continues to reject experimental targets. Paper/NeoForge experimental catalog requests return a readable error. Every installation still resolves official metadata, checks published artifact hashes and verifies the exact Java major.

`log.rs` recognizes readiness only from a complete INFO startup message emitted on the native server thread, including a finite non-negative loading duration. Generic completion text, worker messages, chat and truncated lines do not establish readiness.
