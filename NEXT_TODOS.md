EXPORT-LOCALE-18 — 0.26.0: selected world plus mods/plugins ZIP export, optional shared configurations, Paper dimensions, cancellation and staged publication; refined world cards and scoped operation completions. English/Spanish UI choice persists in Settings; names, commands, logs and unknown diagnostics remain verbatim. README/SECURITY rewritten in English, historical notes moved under docs/, MIT project license and collected dependency notices added to Windows resources. 78 frontend and 230 native tests pass; 28 condition-dependent native tests ignored. Fabric API oracle passed 1,263 recognized comparisons on Java 8/21/25 separately. Native visual/keyboard/gameplay/router checks remain open. Windows NSIS installer generated locally (6,631,757 bytes); native versions, icons and copied license resources verified. SHA-256 recorded in [release notes](docs/release-notes/RELEASE_NOTES_v0.26.0.md). No commit, push or GitHub publication.

CONTENT-DIVISIONS-17 — 0.25.0: four discovery types, validated native facets and project URLs, existing .mrpack review entry, client/server role labels, stale-result suppression and installed-card refinements. Redundant wizard next-step text removed. Fabric predicate review added for main-mod declarations with conservative nested/override limits. 225 offline native and 74 frontend tests passed; public Modrinth search integration passed without archive downloads. Native visual/keyboard/browser dispatch and gameplay still pending. English commit messages requested for subsequent iterations.

0.25.0 Windows NSIS installer generated: 6,514,214 bytes; native version and executable/installer icons verified; SHA-256 in release notes and adjacent checksum file. Final category refinement uses official per-type tags, including texture resolution; ten focused content tests pass after that refinement. Not installed, committed, pushed or published.

LIFECYCLE-14 — SOURCE CHANGES after 0.23.0: graceful stop requests dispatched to all active servers before a shared deadline; errors aggregated without skipping clean-stop copies. Native exit checks operations/processes before cleanup and after waiting, restoring the user choice if a previously admitted start produced active Java. Ordinary reservations/imports reject shutdown; internal cleanup-copy reservation retained. Close dialog preserves local errors and duplicate-dispatch guards, keeping acknowledged exit requests busy. Three batch regressions, two process and two post-stop backup tests, 72 frontend tests and TypeScript/Vite pass. Native window/tray/IPC race, real Minecraft stop and router cleanup review remain pending. No new installer.

**Alcance activo: Windows. Linux aplazado por petición del usuario; ver [LINUX_LATER.md](LINUX_LATER.md). Las referencias históricas a Linux no son tareas activas.**

STARTUP-UI-13 — SOURCE CHANGES after 0.23.0: independent OS/server preference reads and retries, last-known/error states, acknowledged snapshots invalidating earlier reads, scoped duplicate/disposal guards and catalogue refresh. Scheduled-task creation closes after native acknowledgement and reports follow-up list failure separately; integer interval validation added. 72 frontend tests and TypeScript/Vite pass; three native autostart and nine scheduler tests pass. Actual OS-session start, native visual/keyboard and IPC behavior remain pending. No additional installer.

Release milestone — 0.23.0 generated locally: accumulated post-0.22 runtime/JVM/operator/update recovery, deferred sections, historical NeoForge integration and library browsing polish are included. Earlier source-only entries below are chronological development records. 70 frontend and 208 offline native tests pass; real historical NeoForge installation/import boot to EULA refusal passes separately. Windows installer/version/icons verified. Native visual/keyboard assessment, gameplay, external router access and Linux packaging remain open.

# Trabajo pendiente

SEED-VERIFY-16 — SOURCE after 0.24.0: unchanged imported seed escapes retained during unrelated property edits, including previously decoded escaped control characters. Seven focused properties tests pass; independent Java Properties integration passes with managed Java 8, 21 and 25 for large numbers, Unicode, spaces and literal escapes. No Minecraft process, EULA acceptance or world generation. Actual seeded-world gameplay and native visual/keyboard review remain pending; existing 0.24.0 installer does not contain this small follow-up.

NEOFORGE-LEGACY-12 — INTEGRATED IN SOURCE after 0.22.0: official historical catalog, installer/profile/library routing, imported forgeVersion identity and numeric updates. Real isolated 1.20.1-47.1.106 installation and copied import booted with managed Temurin 17 to EULA refusal, preserved properties and created no world. Modern catalog-confirmed prerelease layout/import preserved. Gameplay/mod compatibility and native selector review remain pending; no new installer.


PLAYER-UI-11 — REVIEW, después de 0.22.0: borradores de permisos OP conservados durante polling, conflicto de valores nativos bloquea escritura y permite cargar actuales explícitamente. Payload usa baseline original; guardado confirmado resetea borrador y guardas evitan doble envío/completado obsoleto. Acciones generales separan modificación confirmada del fallo posterior de lista; retry solo de lectura. Lista blanca detenida refresca propiedades después de confirmación, sin volver falso el resultado de escritura. 68 pruebas frontend, seis pruebas nativas de jugadores (incluidos permisos obsoletos y conservación de campos/identidades/bytes), TypeScript/Vite aprobados; revisión nativa visual/teclado, carreras IPC y comandos reales pendientes. Sin nuevo instalador.


UPDATE-UI-10 — REVIEW, después de 0.22.0: actualización de loader confirmada separada del refresco posterior; diálogo cierra tras confirmación nativa, aviso de destino y warnings conservados, retry solo de metadatos. Revisiones/escrituras con guarda de operación y componente; eventos tardíos tras cleanup ignorados. Catálogo Minecraft distingue fallo de vacío y reintenta; acciones rechazan doble envío y servidor no detenido. Seis pruebas nativas loader (publicación, rollback, interrupciones, importados y fingerprint) y TypeScript aprobados; prueba con descargas/Java ignorada, interacción asíncrona nativa, teclado, apariencia y juego real pendientes. Sin nuevo instalador.


PERF-UI-09 — REVIEW, después de 0.22.0: Mundos, Mods/Plugins, Rendimiento y Ajustes cargan al entrar, con carga local y frontera de error/reintento por sección. Biblioteca, controles de proceso/consola/creación y entradas de modal conservadas. TypeScript/Vite aprobados; manifiesto confirma cuatro entradas dinámicas sin imports iniciales de paneles. JS inicial 501,30 → 437,31 kB, gzip 148,35 → 130,96 kB; no equivale a tiempo de arranque/memoria medidos. Pendientes primer acceso nativo, recuperación tras fallo de módulo, teclado y apariencia. Sin nuevo instalador.


JVM-UI-08 — REVIEW, después de 0.22.0: etiqueta no confirmada antes de leer, recarga explícita bloqueada con borrador y vista previa vinculada al servidor/runtime/RAM/arranque/argumentos efectivos. Guardado requiere vista previa actual y evita doble envío; confirmación solo tras persistencia nativa. Materiales End y foco de radios. Dos regresiones frontend de contexto, TypeScript/Vite y dos pruebas nativas JVM aprobadas. Prueba adicional con Temurin 21 real aprueba perfiles válidos, rechaza argumento inválido y conserva perfil previo. Prueba real de selección/runtime también aprueba compatibilidad y rechazo sin sustituir selección anterior. No se inicia Minecraft ni se acepta EULA. Revisión nativa visual/teclado pendiente; sin nuevo instalador.


RUNTIME-UI-07 — REVIEW, después de 0.22.0: lectura de Java por selección con reintento y rechazo de respuestas antiguas; fallo de consulta nunca conserva «Listo» del dato anterior. Runtime ausente, inválido, incompatible y compatibilidad desconocida separados. Cambio de Java confirmado se distingue del fallo posterior al refrescar metadatos; guardas de operación/vida del componente. Cuatro pruebas frontend, TypeScript/Vite, dos pruebas nativas runtime y prueba adicional con Temurin 21 real aprobados; revisión nativa visual/teclado y ciclo real Minecraft pendientes. Sin nuevo instalador.


RELEASE-0.22.0 — GENERATED locally: installer x64 NSIS de 6,495,003 bytes, recursos de versión 0.22.0 e iconos del ejecutable/instalador comprobados. Incluye los bloques de desarrollo siguientes que históricamente figuraban sin empaquetar después de 0.21.0. 60 pruebas frontend y 204 nativas aprobadas, 25 externas/runtime/helper ignoradas. No instalado, commit, push ni publicación. El objetivo completo sigue activo; pendientes juego real, revisión nativa visual/teclado, routers físicos y Linux.


READ-UI-06 — REVIEW: consola y jugadores con lecturas independientes, reintento por servidor y rechazo de respuestas antiguas; últimos jugadores retenidos y marcados al fallar. Refinamiento End de avisos, indicadores e inventario de jugadores. 60 pruebas frontend y 204 pruebas Rust aprobadas; 25 externas/runtime/helper ignoradas. TypeScript/Vite aprobados. Revisión nativa visual/teclado y juego real pendientes.


SETTINGS-UI-05 — REVIEW, después de 0.21.0: lectura propia de propiedades con carga/error/reintento, edición y guardado protegidos antes de leer o tras fallo; respuestas antiguas invalidadas al editar/guardar y borrador conservado durante eventos. Capacidad RAM/espacio conserva cambios locales ante refresco de metadatos. Restauración reemplaza propiedades por el mismo propietario después de confirmación nativa. Cuatro regresiones frontend asíncronas y TypeScript/Vite/whitespace aprobados. Pendientes pruebas nativas de interacción, capacidad, teclado y apariencia. Sin nuevo instalador.

BACKUP-UI-04 — REVIEW, después de 0.21.0: carga de copias independiente de métricas/logs, propietario único de lecturas por selección y última respuesta, reintento y distinción vacío/error/últimos datos. Restauración confirmada separada de consultas posteriores; sincronización por servidor con retry de lectura y Configuración protegida hasta actualizar datos. Respuestas de propiedades/EULA anteriores a restaurar descartadas. Tres regresiones frontend asíncronas, TypeScript/Vite y whitespace aprobados. Pendientes interacción nativa, revisión visual/teclado y ciclo con mundos reales. Sin nuevo instalador.

BACKUP-UI-03 — REVIEW, después de 0.21.0: retención reintentable, límites validados, foco de error y borrador conservado al actualizar; limpieza confirmada separada del fallo posterior de lista y callback aislado por servidor. Materiales End, versiones en revisión y detalles de restauración adaptados a ventanas estrechas. Dos pruebas frontend y seis pruebas nativas de retención aprobadas; TypeScript/Vite aprobados. Pendientes revisión visual/teclado, carreras IPC nativas y restauración con Minecraft real. La lectura compartida y la separación de restauración frente a consultas posteriores se abordaron en BACKUP-UI-04; queda validación nativa. Sin nuevo instalador.

TASK-UI-02 — REVIEW, después de 0.21.0: tareas con materiales End, próxima fecha y resultado separados, recarga manual y respuestas antiguas aisladas. Reinicio automático con lectura reintentable, cambios locales conservados durante actualizaciones y guardado confirmado separado del fallo de consulta posterior. Validación entera y foco de errores conectados. Cuatro pruebas frontend de presentación/política y cinco pruebas nativas de reinicio aprobadas; TypeScript/Vite y whitespace aprobados. Pendientes revisión nativa visual/teclado, carreras asíncronas de IPC en escritorio y reinicio de Minecraft real. Sin nuevo instalador.

Petición añadida: diagnosticar crecimiento de RAM en Fabric 1.21.11 con unos 30 mods, Xms 1 GB / Xmx 12 GB, observado también en Windows en el ordenador del amigo. Después de 0.16.0: lectura HotSpot de heap usado/reservado y eventos del recolector, junto a RAM residente y curva de heap discontinua. Parser y lectura real de un proceso Java 21 propio aprobados fuera del aislamiento; falta comprobar esa partida, retención tras GC y posibles mods causantes. Exportación nativa JSON de hasta 30 minutos observados con el servidor seleccionado, acotada a 901 muestras y 32 servidores; sesión real, huecos conservados, heap ausente explícito y configuración actual distinguida de límites JVM medidos. Guardado nativo sin logs/rutas/comandos; dos regresiones de aislamiento, límites y publicación aprobadas. La cifra de proceso por sí sola no prueba una fuga; no se fuerza GC ni se cambian parámetros sin evidencia.

CLIENT-FABRIC-01 — DEFERRED por petición explícita del usuario: instancias cliente Fabric y autenticación Microsoft aparcadas. Solo se consultaron metadatos públicos; no se instaló el cliente, no se añadió UI ni autenticación. La pregunta anterior sobre Client ID queda sin efecto. No forma parte del trabajo activo mientras el usuario no lo retome.

Después de 0.16.0: el espacio del Resumen usa lecturas comprobadas y caché independiente por servidor/ruta (60 s para datos, 15 s para errores), conserva CPU/RAM si falla la lectura y distingue carpeta vacía de medición no disponible. Hora de medición, enlaces omitidos y error local conectados a Configuración. Tres pruebas de almacenamiento con archivos reales aprobadas; TypeScript/Vite aprobados. Pendientes permisos de carpeta no legible y revisión visual nativa; no incluido en el instalador 0.16.0.

Estados: `DONE` solo para flujos conectados al backend y verificados por pruebas; `REVIEW` para implementación pendiente de validación integral.

| ID | Prioridad | Estado | Trabajo | Criterio de aceptación |
| --- | --- | --- | --- | --- |
| CORE-01 | Alta | REVIEW | Base Tauri, SQLite, servidores con UUID | Compila y registra múltiples instancias de forma persistente. |
| CORE-02 | Alta | REVIEW | Descarga Vanilla/Paper y Java | Crear e instalar en Windows y Linux con hashes comprobados. |
| CORE-03 | Alta | REVIEW | Inicio, consola y parada | Después de 0.16.0: recorrido Java/Mundo/Listo observado desde logs nativos, carga/zona inicial diferenciadas, sin porcentaje ficticio ni completar por 100% de spawn. Progresión por proceso y rechazo de métricas anteriores al reinicio; eventos en tiempo real por sesión además de métricas; dos pruebas nativas, dos frontend y lectura explícita del log Paper del usuario aprobadas, TypeScript/Vite aprobados. Revisión nativa del arranque visual pendiente. Iniciar servidor real, detectar `Done`, enviar comando, detener durante el inicio y comprobar fallo persistente tras salir el proceso. |
| CORE-04 | Alta | REVIEW | Propiedades y memoria | Guardado/relectura conectados. Corregidos nombres con puntos (query.port/RCON); regresión de guardado completo conserva valores/autenticación/comentarios y rechaza inyección. Incluido en 0.15.1. Revisión nativa pendiente. |
| CORE-05 | Alta | REVIEW | Backups y restore | Copias con versión, motor, arranque relativo, capacidad y JVM; restauración con snapshot previo y recuperación tras interrupción. Retención por cantidad/espacio con activación explícita, revisión manual, protección de copia reciente/origen de restore y rollback SQLite implementada; seis pruebas de retención aprobadas. Después de 0.15.1: motivos explícitos y versión/motor de metadatos visibles en listado/confirmación; compatibilidad antigua sin inventar datos, errores por copia y lectura acotada/rutas verificadas, doce pruebas de backup aprobadas. Programador local de copias conectado; pendientes revisión visual nativa y restauración con Java/mundos reales. |
| BACKUP-STOP-01 | Alta | REVIEW | Copia automática al detener | Opción persistente desactivada por defecto; requiere salida correcta de una sesión que llegó a running. Observación de parada manual/consola, copia antes de iniciar/reiniciar y espera al detener/salir conectadas; reserva de archivos, retención, progreso/cancelación y deduplicación durable por sesión. Recuperación registra interrupción sin repetir. Dos pruebas con archivos/SQLite cubren exclusiones, copia real, cancelación, fallo, recuperación y nueva sesión. Verificar Java/Minecraft y cierre nativo. Después de 0.15.1. |
| CORE-06 | Alta | REVIEW | Cierre con procesos activos | Ofrecer detener, mantener en segundo plano o cancelar el cierre. |
| CORE-07 | Alta | REVIEW | Estado y métricas completas | RAM, CPU, historial de dos minutos, I/O, PID y jugadores conectados implementados. Después de 0.15.1: tiempo de arranque del proceso actual medido desde lanzar Java hasta el primer aviso válido de listo; valor ausente antes de listo/al salir, reinicio sin arrastrar valores. Conectado a Rendimiento; TypeScript/Vite aprobados, verificar con Minecraft nativo. Pruebas de telemetría aprobadas; Consulta explícita de TPS Paper conectada al proceso actual y Rendimiento, con medias 1/5/15 min, validación de respuesta, plazo y caducidad; dos pruebas locales aprobadas. Verificar el comando con Paper/Java real y fuentes fiables para otros motores. Incluido en el preview 0.15.0. |
| CRASH-DIAG-01 | Alta | REVIEW | Diagnóstico local de cierres | Salida acotada de una sola sesión; informe atómico con código, parada solicitada y drenaje, reglas Java/memoria/puerto/mods/plugins y archivos modificados recientes separados. Resumen con evidencia desplegable y acciones conectado. Dos pruebas de reglas/persistencia, etiquetas y prueba real Java 21 de clase no compatible seguida de proceso correcto aprobadas; captura stderr/log, códigos reales, estados y sesiones separadas comprobados sin Minecraft/EULA/mundo. TypeScript/build aprobados. Validar Minecraft y escritorio. Incluido en 0.15.1. Después: último fallo separado y consulta histórica explícita con fecha tras reiniciar launcher o arranque correcto; tres pruebas de diagnóstico y Java real aprobados, sin prueba de relanzamiento del escritorio. |
| CORE-08 | Alta | REVIEW | Descargas cancelables y reintentos | Cancelación, limpieza de parciales, reintentos transitorios y nuevo intento desde UI implementados; verificar interacción nativa. No hay reanudación por HTTP Range. |
| JVM-01 | Alta | REVIEW | Argumentos JVM | Perfiles, vista previa, validación, persistencia y arranque conectados; comprobar con servidor real y Linux. |
| JAVA-01 | Alta | REVIEW | Runtime personalizado | Selección nativa, vista previa y verificación real implementadas; comprobar interacción en escritorio y Linux. |
| LOAD-01 | Alta | REVIEW | Fabric | Catálogo compatible, instalación verificada y arranque hasta EULA implementados; Actualización de loader conectada con selección estable, copia y publicación recuperable; actualización 0.16.10→0.19.5 y restauración a 0.16.10 comprobadas con Java 21 hasta EULA, conservando Java/JVM/configuración. También verificado el ciclo con formato externo importado. Identificación de Fabric clásico y unattended por metadatos, y NeoForge por argumentos, con regresiones locales. Pendientes formatos legacy/custom, ciclo real unattended Fabric, conexión con mods, revisión nativa y Linux. |
| LOAD-02 | Alta | REVIEW | NeoForge | Catálogo estable, instalación verificada, arranque e importación conectados; comprobados con Minecraft 1.21.4 y 1.21.11 y Java 21 hasta EULA, incluida una copia importada. Actualización de loader conectada con copia previa, generación aislada y recuperación; actualización 21.4.157→21.4.158 y restauración a 21.4.157 comprobadas con Java 21 hasta EULA; rutas canónicas de Windows corregidas y cubiertas por regresión. También verificado el ciclo con formato externo importado. Identificación de Fabric clásico y unattended por metadatos, y NeoForge por argumentos, con regresiones locales. Pendientes formatos legacy/custom, ciclo real unattended Fabric, conexión con mods, revisión nativa y Linux. |
| MC-VERSIONS-01 | Media | REVIEW | Catálogo experimental explícito | Switch desactivado por defecto en creación Vanilla/Fabric; tipos de Mojang conservados, snapshots etiquetados y selección estable al desactivarlo. Fabric exige coincidencia con su catálogo y Mojang; Paper/NeoForge rechazan el modo experimental. Regresión de filtros/orden aprobada; catálogos oficiales y metadatos de servidor de 26.4-snapshot-2 comprobados en vivo. Instalación y arranque de 26.4-snapshot-2 en Vanilla y Fabric con Java 25 hasta rechazo de EULA aprobados; propiedades y ausencia de aceptación/mundo comprobadas. Conexión/juego y revisión nativa pendientes. No incluido en el instalador 0.9.0 ya generado. |
| MC-UPDATE-01 | Alta | IN PROGRESS | Cambio de versión de Minecraft | Revisión nativa de releases por fecha, Java, catálogo del motor y compatibilidad de archivos por SHA-512 añadida. Distingue archivos manuales/modificados y metadatos no disponibles; detecta cambios posteriores en configuración y arranque. Publicación de carpeta/versión/Java con token SQLite y recuperación añadida; preparación de Paper estable con checksum y respeto del Java personalizado añadida. Flujo nativo conectado a revisión, confirmación de avisos, copia previa, preparación separada y publicación. Cuatro regresiones de publicación/restauración local aprobadas. Interfaz en Configuración conectada con revisión por archivo, Java/motor, avisos, confirmación y progreso cancelable. Ciclos reales Vanilla, Fabric y NeoForge 1.21.4→1.21.5→1.21.4 con Java 21 hasta EULA aprobados, sin aceptarla. Paper 1.21.4→1.21.8→1.21.4 también aprobado con Java 21; 1.21.5 se rechazó correctamente por no tener build STABLE. Migración de proyectos gestionados por plan conjunto con dependencias exactas, revisión de cambios y changelogs implementada; ciclos con Lithium/Fabric y FerriteCore/NeoForge aprobados hasta EULA, verificando hashes, inventario restaurado y copias. Regresión de versiones con artefacto idéntico aprobada. Inicio opcional desde la revisión y comprobación por estado nativo hasta tres minutos conectados, con cuatro regresiones frontend; salir del panel interrumpe observación sin parar Java. Pendientes validación nativa del flujo, conexión/juego con mundos/mods, revisión visual y Linux. Incluido en el instalador preview 0.9.0. |
| MOD-01 | Alta | REVIEW | Mods y plugins | Navegador, inventario real, versiones, dependencias, instalación con SHA-512, actualizaciones individuales/conjuntas y retirada confirmada conectados. Lithium instalado/actualizado/retirado con backups y LuckPerms instalado en Paper en pruebas reales aisladas. Pendientes prueba visual nativa, conexión desde Minecraft y mods NeoForge en ejecución. |
| PACK-01 | Media | REVIEW | Importación de modpacks | Asistente `.mrpack`, selección de archivos, loader exacto, instalación aislada, publicación con SQLite y recuperación tras interrupción conectados. Paquetes Fabric/Lithium 1.21.4 y NeoForge/FerriteCore 1.21.11 probados con Java hasta EULA sin aceptarla. Pendientes revisión visual nativa, conexión desde Minecraft, packs complejos de terceros y Linux. Forge/Quilt se explican como no compatibles. |
| IMPORT-01 | Alta | REVIEW | Importar servidor existente | Elegir carpeta, inspeccionar `.jar`, administrar o copiar sin tocar origen. |
| IMPORT-02 | Media | REVIEW | Importar mundo | El wizard copia un mundo elegido al crear el servidor; verificar con mundos reales de distintas versiones. |
| PLAYER-01 | Media | REVIEW | Jugadores, whitelist, bans y OP | Lectura de archivos reales y comandos en vivo implementados. Después de 0.15.1: editor detenido para nivel 0–4 y bypass de límite de operadores existentes, UUID conservado, campos desconocidos/otros operadores intactos, copia original sincronizada y rechazo de cambios obsoletos/UUID repetido/JSON dañado. Cinco pruebas de jugadores aprobadas: eventos de conexión unificados con el mensaje limpio de consola, soporte ANSI/Paper y rechazo de chat/comandos que imitan entradas/salidas; lectura de logs reales coincide con eventos en vivo. TypeScript/Vite aprobados para el editor; verificar permisos/conexiones con Vanilla/Paper en ejecución y teclado/escritorio. |
| WORLD-01 | Media | REVIEW | Gestión de mundos | Listado, tamaño, imagen, importación, exportación, duplicado, cambio, copia, apertura y borrado implementados. Duplicado/exportación conectados al registro con bytes medidos, cancelación antes de publicar, validación y conservación del origen/destinos existentes; Cambio de principal/borrado también registrados con copia medida, cancelación antes de mutación, revalidación y conservación de copias completadas. Cinco pruebas locales de mundos aprobadas. Revisión nativa y mundos reales pendientes. Seguimiento incluido en el preview 0.15.0. |
| NET-01 | Alta | REVIEW | Red LAN y puerto local | Detección de dirección local, copia y comprobación TCP implementadas; verificar con varias interfaces y redes reales. |
| NET-TUNNEL-01 | Media | DEFERRED | Bore retirado | Por petición del usuario en 0.18.0: retirados panel, API, comandos, proveedor y mantenimiento de producción. Código de protocolo/lifecycle solo bajo cfg(test); sin túnel alternativo. Red local, UPnP/NAT-PMP y guía manual conservados. |
| STORAGE-USAGE-01 | Media | REVIEW | Medición y avisos de almacenamiento | Desglose real de mundos, contenido, logs, otros archivos y copias; espacio libre nativo y umbral porcentual persistente con slider/entrada, aplicado también al resumen. Fallos de lectura no se muestran como mediciones completas. Dos pruebas nativas, TypeScript y build aprobados. Revisión visual pendiente; sin cuota ni borrado automático. Incluido en el preview 0.15.0. |
| NET-02 | Alta | REVIEW | Red pública | UPnP conectado: detección explícita, apertura TCP temporal confirmada, propiedad durable, comprobación, renovación y retirada al detener/cerrar, con diez pruebas locales. Verificar router físico y cierre nativo. NAT-PMP conectado con detección de puerta de enlace, confirmación, apertura TCP temporal, puerto externo real, persistencia previa, renovación y retirada/reintento; tres pruebas locales aprobadas. Aislamiento de registros dañados/fallos individuales añadido tras 0.15.1; dos regresiones nuevas cubren renovación, cierre y reintento con SQLite real sin router. Verificar router real y alcance desde internet. IP pública opcional y guía manual disponibles. |
| TASK-01 | Media | REVIEW | Programador local | Tareas persistentes puntuales/periódicas para copia, inicio, reinicio y parada; copia con parada/reinicio explícita, próxima fecha, pausa y resultado. Claims SQLite, recuperación sin repetir y propietario exclusivo implementados. Reinicio tras crash configurable conectado, desactivado por defecto: espera, límite de intentos, reset tras 10 minutos en running, parada solicitada excluida, cancelación y defer por operación ocupada. Arranque de aplicación con sistema y selección explícita de servidores conectados; pruebas de preferencias y entradas temporales aprobadas. Después de 0.15.1: fallos inesperados de workers registrados y plazas liberadas por propietario incluso al fallar; validación de tareas persistidas antes de reclamar/reactivar, aislamiento de filas inválidas y listado seguro con retirada; nueve pruebas de scheduler aprobadas, con SQLite/hilo real, fechas extremas y sin repetición puntual. Pendientes inicio de sesión real y prueba nativa/Java/Linux. |
| UX-01 | Alta | TODO | QA de escritorio | Prueba manual 1280×720 y 1920×1080, teclado y lector de pantalla. |
| UX-TASKS-01 | Media | IN PROGRESS | Seguimiento global de operaciones | Panel no bloqueante en la barra con instalaciones/Java/contenido/modpacks/actualizaciones reales, cancelación por task ID y navegación al servidor. Consulta nativa de tareas actuales, reconciliación cada tres segundos sin solapamiento y protección de eventos nuevos; resultados recientes solo durante la sesión. Nueve pruebas nativas de descarga y cuatro de seguimiento, TypeScript y build aprobados. Copias manuales añadidas con bytes reales, cancelación antes de publicar y límite de publicación protegido; diecinueve pruebas nativas relacionadas con copias aprobadas (una integración externa excluida). Pendientes revisión nativa de popover/teclado y revisión del flujo de retirada de contenido. Retirada de mods/plugins conectada al registro, copia previa y preparación de contenido medidas por separado, cancelación antes de publicar y conservación de copia completada; regresión local de las cuatro fases aprobada. Importación de servidores conectada con copia medida y cancelación antes de publicar; registro de carpeta externa sin porcentaje inventado. Prueba local de finalización, cancelación, fallo SQLite, conservación del origen y limpieza del registro aprobada; revisión nativa pendiente. Importación de mundos conectada con bytes reales, cancelación antes de publicar, validación del mundo preparado y limpieza de parciales; tres pruebas de mundos aprobadas, revisión nativa pendiente. Copias programadas y previas a Minecraft/loaders/contenido conectadas después del preview 0.13.0; dos pruebas de registro/copia programada y una del límite de publicación aprobadas. Parada/reinicio de Java por el programador y revisión nativa siguen pendientes. Restauración y su copia previa añadidas con medidas separadas y cancelación antes de publicar; servidor actual y copia previa conservados en regresión. Suite completa: 128 pruebas nativas y 24 frontend aprobadas; 18 integraciones externas excluidas. Incluido en el preview 0.13.0. |
| UX-MOTION-01 | Media | REVIEW | Preferencia de movimiento | Opción persistente en Ajustes aplicada antes del primer render, CSS y animaciones de navegación/diálogos/consola/métricas; respeta siempre la reducción del sistema. Preferencias anteriores conservadas; cuatro regresiones de persistencia, datos inválidos, almacenamiento bloqueado y prioridad del sistema aprobadas, además de TypeScript y build. Revisión visual nativa pendiente. Incluido en el preview 0.13.0. |
| UX-KEY-01 | Media | REVIEW | Acciones y atajos | Ctrl+K busca acciones y servidores, con teclado y restricciones del estado nativo para inicio/parada/copia; Ctrl+N crea y Ctrl+F enfoca la búsqueda actual sin interferir con diálogos de confirmación. Tres regresiones de acciones/búsqueda, TypeScript y build aprobados. Revisión nativa de teclado pendiente. Incluido en el preview 0.13.0. |
| APP-SETTINGS-01 | Media | REVIEW | Preferencias por categorías | General, Java, Almacenamiento, Red, Copias, Avanzado e Información conectados a los controles existentes en una superficie principal con barra horizontal. Selección explícita de servidor para Java/JVM/red/retención/tareas; ubicaciones nativas y apertura de carpetas, versión del binario y aceptación EULA leídas del backend. Dos pruebas de ubicaciones/validación aprobadas; TypeScript y build aprobados. Revisión visual/teclado/apertura nativa pendiente; cambio/migración de ubicación de biblioteca no implementado. Incluido en el preview 0.14.0. |
| APP-LOG-01 | Media | REVIEW | Diagnóstico del launcher | Registro JSONL interno de inicio/cierre, estados de proceso y fallo de mantenimiento del router; info/warn/error, UUID válido y versión, sin comandos ni texto arbitrario. Rotación de dos archivos hasta 2 MB; botón nativo de apertura en Ajustes. Dos pruebas de escritura/rotación, TypeScript y build aprobados. Apertura de carpeta y revisión nativa pendientes. Incluido en el preview 0.13.0. |
| UX-02 | Media | REVIEW | Imagen de servidor | Elegir, validar, guardar, cambiar y quitar imagen; comprobarlo visualmente en la aplicación nativa. |
| UX-03 | Media | REVIEW | Barra lateral automática y rediseño | Reducida por defecto, ampliación por hover/foco de teclado sobre el contenido sin desplazar iconos/página, etiquetas animadas, menú contextual conservado y accesos sin marcos; botón manual retirado. TypeScript y build aprobados. Verificar modos, foco, escalas, interacción y movimiento en Windows. Cambio automático incluido en el preview 0.13.0. |
| UX-04 | Media | REVIEW | Asistente y configuración | Verificar los cinco pasos, la revisión final, los grupos de propiedades y el comportamiento con teclado. |
| UX-05 | Media | REVIEW | Eliminación de servidor | Confirmar en escritorio el diálogo centrado desde Configuración desplazada, restauración de foco, borrado administrado y conservación de carpetas externas importadas. |
| REL-01 | Alta | TODO | Distribución | Iconos, firmas y paquetes de Windows/Linux. |
| REL-02 | Media | REVIEW | Instalador Windows v0.8.0 | NSIS x64 generado y checksum SHA-256 guardado; 86 pruebas Rust locales y seis de telemetría aprobadas. Falta instalación, prueba de inicio de sesión, acciones programadas con Java, conexión desde Minecraft y revisión visual manual. |

Revisión 0.6.1: descargas cancelables y con reintento, Temurin JDK 16 como alternativa al JRE ausente, perfiles de skins, icono Minecraft compartido, fijados persistentes, menú contextual y gráficas simultáneas implementados. Prueba visual manual pendiente; no se usaron computer use ni subagentes.

Revisión 0.6.2: Fabric añadido al asistente y a importación. Bibliotecas verificadas por SHA-256, Minecraft por SHA-1, generación atómica y arranque sin descargas externas ocultas. La comprobación con Java real alcanza la EULA sin aceptarla; conexión desde el juego, mods y actualizaciones siguen pendientes.

Revisión 0.6.3: Java personalizado con vista previa, confirmación, verificación real y requisitos oficiales cacheados. Prueba con Java 21 aprobada; revisión nativa del selector y compatibilidad en Linux pendientes.

Revisión 0.6.4: biblioteca con búsqueda y vistas, rail oscuro y navegación lateral clara, controles antracita según la última preferencia, imágenes originales y sistema de motion implementados. Perfiles JVM recomendados/personalizados conectados al arranque. La revisión visual nativa y la medición de fluidez están pendientes; no se usaron subagentes ni computer use.

Desarrollo posterior a 0.6.4: una sola sidebar plegable, secciones horizontales, emblema proporcionado y banner eliminado. NeoForge usa bibliotecas verificadas y el instalador oficial offline con mappings de Mojang cacheados; prepara generaciones aisladas y arranca Java directamente. La aceptación de EULA se solicita antes de la primera descarga y se recuerda en SQLite para los siguientes servidores, por petición explícita del usuario. Sin incremento de versión ni nuevo instalador hasta reunir cambios significativos.

Gestor de contenido en desarrollo: consulta Modrinth desde el motor nativo; distingue Paper de loaders de mods y comprueba la versión específica antes de construir un plan. Resuelve dependencias obligatorias transitivas, evita ciclos, conserva las opcionales para revisión y rechaza conflictos declarados, versiones fijadas contradictorias, archivos ambiguos y nombres inseguros. SHA-512 añadido al motor de descargas. El gestor instala y actualiza mediante generaciones temporales, revisión nativa con caducidad, hashes SHA-512 y recuperación de cambios de carpeta interrumpidos. Retirada confirmada y actualizaciones crean backups. Inventario y navegador están conectados; queda revisión visual manual y carga real dentro del juego.

Release 0.7.0: instalador Windows solicitado al terminar el gestor. Incluye gestor Modrinth, NeoForge, EULA persistente y navegación revisada. El objetivo completo sigue activo: modpacks, red pública, tareas programadas, actualizaciones del loader, validación integral y Linux siguen pendientes.

Revisión 0.8.0: importación mrpack, backup con metadatos/restauración recuperable, retención y scheduler con crash recovery e inicio explícito integrados. 86 pruebas Rust locales y seis de telemetría aprobadas. Inicio de sesión real, acciones de Java y revisión visual nativa pendientes; las pruebas no activaron el inicio con el sistema en este equipo.

Release 0.9.0: instalador NSIS x64 generado (5.60 MiB), con SHA-256 junto al ejecutable. Incluye actualizaciones revisadas de Minecraft/loaders, migración conjunta de contenido, comprobación opcional del arranque y refinamiento de biblioteca/resumen. Suite offline: 108 pruebas Rust aprobadas, 16 externas omitidas; 13 pruebas frontend aprobadas. No se acepta EULA ni se genera mundo en las integraciones automatizadas. Pendientes comprobación visual nativa, arranque desde la revisión, juego real y Linux.

## Visual refresh 0.10.0

The current direction is Minecraft / End with the supplied Tableland palette, Minehut-like controls, local Monocraft and the user-provided gold voxel emblem. See design.md and docs/release-notes/RELEASE_NOTES_v0.10.0.md. Frontend production compilation, 13 frontend behaviors, 110 offline Rust regressions, token contrast and font glyph coverage passed. Native visual/keyboard review and client gameplay remain unverified by request; this does not complete the original goal. Network mapping work remains pending.

## Theme and appearance refresh 0.11.0

End now defaults to charcoal and violet. Ajustes below Inicio automatico offers End,
Obsidian and Nether with persisted selection and a Minecraft/reading-font choice.
Logo cutout, collapsed rail controls, section tabs, library artwork and Java panel
have been revised. Six theme/font combinations round-trip through preferences;
corrupt values and denied storage are covered. TypeScript/Vite, 16 frontend tests,
110 offline Rust regressions and 160 palette/log contrast pairs passed. Native visual
and keyboard review remain manual. This milestone does not complete the full goal.

The Windows NSIS 0.11.0 installer was generated (5.54 MiB), with matching native
product/file version and a SHA-256 sidecar. Existing installers remain available.

## Verification checkpoint 0.14.0

Windows installer generated with verified native version and matching embedded icons.
The complete offline suite passes 135 native tests; 18 external integrations remain
excluded. All 24 frontend tests, TypeScript and the production build pass. Application
settings and tracked scheduled/safety copies, imports and content removal are included.
Native visual/keyboard/folder review, real scheduled Java resume, gameplay, physical
router, TPS and Linux verification remain open; this checkpoint is not completion of
the original specification. See docs/release-notes/RELEASE_NOTES_v0.14.0.md for the exact checksum.

## Verification checkpoint 0.15.0

Windows NSIS installer generated with verified native version and matching embedded icons.
The offline suite passes 151 native tests with 19 external integrations excluded; all
24 frontend tests, TypeScript and Vite pass. The public Bore echo integration also
passes when run explicitly outside the network sandbox. A focused tunnel maintenance
regression passes after the final liveness change. NAT-PMP, storage measurements,
world operations and Paper TPS queries are included alongside the account-free tunnel.
Native UI, physical router, Minecraft gameplay, real TPS responses and Linux checks
remain open. See docs/release-notes/RELEASE_NOTES_v0.15.0.md for the installer checksum. The original
application goal remains in progress.

## Verification checkpoint 0.15.1

Configuration saving accepts standard dotted property names and preserves existing
values and account verification. All 155 offline native tests and 24 frontend tests
pass; 20 resource/runtime integration tests remain excluded from that run. An
explicit Java 21 diagnostic integration also passes with actual failure/success
process exits and persisted per-session reports, without Minecraft/EULA/worlds.
Windows packaging, native 0.15.1 version and embedded icons are verified; see
docs/release-notes/RELEASE_NOTES_v0.15.1.md for the checksum. Native desktop and Minecraft gameplay
verification remain open; the full application objective remains in progress.

## Verification checkpoint 0.16.0

Windows NSIS x64 preview generated: 6,323,459 bytes, native product/file version
0.16.0 and matching embedded application/installer icons. SHA-256:
`f5098d6288e6742c8d2396eb0e198d40e46a94ec3ecf9d18080c48ea5ce0d81e`.
All 166 offline native tests and 24 frontend tests pass; 20 integration tests remain
excluded from the offline run. The explicit Java 21 diagnostic failure/success
integration also passes without Minecraft/EULA/worlds. TypeScript and Vite pass.
Automatic stop copies, recorded backup metadata, historical failures, scheduler
hardening, isolated NAT-PMP maintenance, startup duration and operator/player controls
are included. Native desktop, real scheduled Java resume, Minecraft gameplay and
permission effects, stop-copy/shutdown behavior, physical routers and Linux remain
open. The full original goal remains active. See docs/release-notes/RELEASE_NOTES_v0.16.0.md.

Release 0.17.0: instalador Windows NSIS x64 generado a petición del usuario, con progreso nativo de arranque, consulta SLP, heap y exportación de memoria y almacenamiento comprobado. 175 pruebas nativas offline y 27 frontend aprobadas; 22 integraciones externas excluidas en esta ejecución. TypeScript/Vite, versión del ejecutable e iconos de aplicación/instalador comprobados. SHA-256 y notas en inglés en docs/release-notes/RELEASE_NOTES_v0.17.0.md. Cliente Fabric/Microsoft sigue aparcado; prueba de instalación/escritorio, partida del amigo, Minecraft/router reales y distribución Linux siguen pendientes. Objetivo completo activo.

Petición 0.18.0: Conexión reorganizada con acceso de internet visible y detección automática por interfaz LAN, reintento y router avanzado. Confirmación/revalidación nativa para aperturas conservadas; UPnP se vuelve a detectar al confirmar para evitar caducidad de la caché. Bore retirado de producción. Respuesta Minecraft movida a Rendimiento; historial de fallos no aparece si no existe informe. Indicadores con iconos pixel, medidores reales de plazas/heap/CPU, gráficas ortogonales y barras nativas segmentadas; fuente de lectura y reducción de movimiento conservadas. Pendientes router físico y revisión visual manual; instalador Windows NSIS 0.18.0 generado y verificado (versión/iconos/SHA-256). TypeScript, Vite y 27 pruebas frontend aprobados; 10 UPnP y 3 NAT-PMP aprobados. Sin computer use ni subagentes. El objetivo completo sigue activo.

Petición 0.19.0: revisión de JARs/descriptor Fabric (cliente, cargador, JSON, IDs duplicados), presencia histórica en lista Fabric completa de latest.log sin certificar partida; incompatibilidad por versión/loader/entorno visible en Modrinth antes de revisar instalación, filtros categoría/orden/otras versiones, búsqueda/página conservadas tras instalar. Gráficas sin barras duplicadas, área de muestras, leyendas y navegación de teclado; conexión con ayuda opcional y controles visibles. El caso de mochilas/estructuras de la partida del amigo no se puede identificar sin nombres/log concreto. Comprobaciones de juego, router físico y visuales nativas siguen pendientes. El objetivo completo permanece activo. Sin subagentes, computer use, commit ni publicación.

Verificación 0.19.0: 179 pruebas nativas offline y 29 frontend aprobadas; 23 integraciones excluidas de la suite offline. Catálogo/icono, disponibilidad por publicación y descarga/instalación/actualización real con copia pasan en ejecuciones explícitas con red externa, sin ejecutar mods ni Java. Tres regresiones descriptor/log pasan de nuevo tras la última validación. TypeScript/Vite aprobados. NSIS x64 0.19.0 generado, versiones nativas e iconos comprobados; 6.04 MiB y SHA-256 en docs/release-notes/RELEASE_NOTES_v0.19.0.md. El caso del amigo, revisión visual nativa/router/juego y Linux siguen pendientes. Objetivo completo activo.

## 0.20.0 — installed content and related diagnostics

- Subsequent development: card/compact presentation, alphabetical/problem ordering, direct attention review and console actions implemented; raw NeoForge/Log4j headers now support related-file diagnostics. Eight attribution regressions plus TypeScript/production build pass. Not packaged; native layout and real mod failure review remain pending.
- Inventory cards with local declared Fabric icons, text-backed loading/attention states, search/source filters, detailed evidence and filtered-console navigation are connected to real native audits and console events.
- Explicit diagnostic references and bounded loader continuations associate messages with files without claiming causation. Manual review still needs a real Fabric/NeoForge/Paper error, long names and keyboard/reduced-motion at 1280×720 and 1920×1080. The original friend's world has not been diagnosed.
- Missing or unsupported local icons use the authored inventory glyph. Runtime console associations remain limited to the available recent log window; Paper/NeoForge IDs are not inferred from Java package names.

- Verificación local de 0.20.0: TypeScript/build y 36 pruebas frontend aprobados; 180 pruebas nativas aprobadas, 23 integraciones externas excluidas. Siete regresiones cubren asociación de diagnósticos y una nueva regresión nativa los iconos declarados del JAR.

- Instalador NSIS 0.20.0 generado con el frontend final: 6.338.750 bytes, versión nativa/iconos verificados y SHA-256 registrado en docs/release-notes/RELEASE_NOTES_v0.20.0.md. No se instaló ni se abrió la aplicación para QA visual.

## Development after 0.20.0 — truthful save reporting

- Original sections 25/28 and requested visual mod diagnostics: NeoForge TOML identity/secondary IDs/Minecraft expression and declared PNG logo now read without executing JARs; duplicate secondary IDs and console attribution connected. Nine attribution tests, five descriptor/icon and five content-storage cases pass; TypeScript/build pass. Paper YAML metadata completed subsequently; plugin icons are not a standard descriptor field. NeoForge gameplay and native visual review remain open. Included in 0.21.0.
- Forced-exit reason now persists across launcher restarts, including a zero OS exit code; old report files remain readable. Diagnostic findings exclude structured INFO discussions. Four report tests and two process cases pass, including owned-child termination and its durable report; TypeScript/production build pass. Native Minecraft/visual gates remain open; included in 0.21.0.
- Original sections 15/16: removed automatic kill at 30 seconds; stopping-only, session-bound force-stop preview/confirmation added. Forced exits cannot create clean-stop backups or automatic crash retries, and restart checks successful terminal state. Owned native-child regression passes; actual Minecraft slow-save shutdown and native confirmation UX remain pending. Included in 0.21.0.
- Sections 18/19: distinguish observed save start, dimension completion, whole-save notification and failure; show Overview state and explicit failure evidence, avoiding false successful-save timestamps from `Saving chunks`. Pending/incomplete saves stay unconfirmed on exit and old-session evidence clears on observed startup.
- Derive console levels from real header/exception formats instead of generic keyword mentions. Bound recent-log reads to the observed extent and exclude incomplete final records.
- TypeScript/build, 39 frontend tests and the ten final focused frontend cases pass. Sixteen related native cases and a separate read-only startup integration with the supplied real Paper log pass. Actual save/gameplay and native visual verification remain pending. Included in the requested 0.21.0 installer.


## Requested Windows preview 0.21.0

Paper descriptor identity/API parsing now uses bounded streaming YAML, with scalar aliases, duplicate/invalid identity rejection and no plugin execution. Six descriptor/icon regressions pass; a real reviewed LuckPerms installation integration passes in an isolated plugins folder without Java or EULA acceptance. NeoForge secondary IDs/icons, compact inventory and related diagnostic refinements, observed-save reporting and explicit session-bound force close are included.

Release gate: 188 native tests passed, 24 external/runtime/helper tests excluded; all 41 frontend tests passed. TypeScript/Vite and NSIS x64 packaging passed. Product/file versions are 0.21.0 and both embedded icons match the logo. Installer: 6,440,897 bytes; SHA-256 `56950b2ff5e0cb3e25050fd4e06ff37742e73d0ff23045e3aa9733806077528e`. English release notes: docs/release-notes/RELEASE_NOTES_v0.21.0.md. No install, native visual QA, subagents, computer use, commit, push or publication. Actual gameplay, slow-save shutdown, the friend's missing mods/RAM case, router and Linux checks remain open. Full original objective remains in progress.

## Development after 0.21.0

User clarified NeoForge selection opens and reaches Minecraft 1.20 but cannot scroll to earlier releases. Official modern NeoForge catalog has eleven stable families; NeoForge does not exist for 1.19 or older, and legacy 1.20.1 has separate coordinates not yet integrated. Creation now uses a visible bounded native list plus catalog count and scope explanation, and four native radio choices with End-themed selection/focus relief. Native visual/scroll/keyboard verification remains pending, not claimed fixed by browser automation.

Fabric file details now show required/recommended/suggested/conflicting/breaking declarations with OR alternatives, bounded native metadata validation and no invented compatibility verdict. Real-JAR regression covers kinds, serialized identity/ranges, malformed shapes, ID/length/count limits and preservation of original bytes. Seven descriptor tests, five storage tests, nine frontend attribution tests, TypeScript and production build pass. No new installer/commit/publication; 0.21.0 remains the delivered preview. Full original goal remains in progress. Next catalog review should address Paper version availability at selection time, since its current source still uses the general stable Mojang list.

## Paper catalog correction after 0.21.0

Original sections 7/8/35: Paper creation no longer offers every Mojang release. Native `paper_catalog.rs` queries official paginated STABLE availability with current public-schema filtering and SHA-256/server-download evidence, then intersects official Mojang releases without chronological truncation. Malformed/partial pages, repeated cursors and query limits fail visibly. Installation retains exact-version REST revalidation. Two local regression cases and a separate live application-path catalog/REST evidence check pass, including old Minecraft 1.8.8. Experimental opt-in/chronology regression also passes. No Java, EULA acceptance, gameplay or native UI checks; no installer, commit or publication. Full goal remains active. The earlier noted Paper availability gap is now implemented; native selection and real boot/play verification remain open.
# Current verification follow-up

- Manually review creation steps, world/location radio focus and narrow layouts in
  the native Windows app. Review the End preview and existing reduced-motion option.
- Exercise renamed Paper/Fabric and NeoForge import folders with real startup in
  the native app. Bounded metadata/import regressions pass; they do not establish
  that every legacy or custom launcher format is identified.
- Current import identity and creation finish changes are source-only after the
  generated 0.21.0 installer; include them in the next requested significant release.
- Original section 13: additional real-property controls and changed-value native
  validation are implemented. Four file-based property regressions pass, including
  preservation of legacy and unknown values. Review the native group layout and
  verify version-specific behavior in a real stopped/save/start session. Properties
  absent from server.properties are not inserted automatically by this UI.
- Combined settings save now owns one stopped-server reservation and recoverable
  file/SQLite publication. Eight file/database scenarios pass, including injected
  database failure, four interruption phases, real filesystem rename failure,
  read-only files and preservation of orphaned copies. Review native save feedback and
  an actual application interruption; do not equate fixture recovery with that gate.
- Player action form now includes target/effect preview, named actions, associated
  errors and first-invalid focus. Five native player regressions and two frontend
  name/reason boundary cases pass. Native keyboard review, IP failure feedback and
  actual server command outcomes remain pending; command dispatch is not completion.
- Player lists now provide category search and 50-row pagination without merging
  identities or reducing category totals. Six native player regressions and two
  frontend search/paging cases pass. File reads are bounded before/during loading,
  and malformed data is preserved. Review search/page keyboard focus and a real
  large usercache in the native app; special-file behavior on Linux remains untested.

## Inventory refinement after 0.23.0

- Source-only: independent state/origin filters, secondary-ID search, grouped tools,
  explicit lock explanation and refined End inventory/empty state. All 74 frontend
  tests, TypeScript and Vite pass. Native narrow layout, keyboard focus, all themes
  and actual mod error review remain pending. Installer unchanged.
- Linux container verification route prepared but not executed: Docker engine
  remained unavailable after startup. PowerShell syntax passes; do not count this
  as a Linux test/package/runtime gate. Retry only when the Linux engine is ready.

## Fabric dependency presence

- Implemented read-only required-ID checks with aliases/nested JARs and top-level
  file attribution. Four archive/filesystem cases pass. Partial metadata/budgets
  or loader overrides suppress negative absence conclusions. Client-only files
  do not supply server dependencies. Optional declarations remain optional.
- Version predicates, candidate selection and real gameplay remain unverified;
  presence alone is not compatibility. TypeScript/production build pass.
- Source-only after 0.23.0. Linux deferred; see LINUX_LATER.md.

## Windows 0.24.0 preparation

- New-world seed input/review and native string transport are connected. New
  creation writes game rules before Java; editor reads merge missing core fields
  while retaining real/unknown values without writing. Settings save uses existing
  recoverable publication. Seed fixtures include large signed numbers, Unicode,
  backslashes, spaces and injection rejection; creation proves no world/Java/EULA.
- Six property and one real creation/storage regression pass. Full native suite:
  218 passed, 25 external/runtime/helper checks excluded. All 74 frontend tests,
  TypeScript and Vite pass. Native visual/keyboard and actual seeded generation
  remain pending; static persistence does not prove gameplay.
- Content confirmations now acknowledge completed writes before refreshing, with
  read-only recovery, scoped inventory reads and late/duplicate dispatch guards.
  Inventory failures are not empty lists. Actual native IPC interaction pending.
- Packaging is running; mark release generated only after installer/version/icon
  and checksum evidence. Windows only; Linux remains deferred in LINUX_LATER.md.

## Windows 0.24.0 generated

NSIS x64 installer generated: 6,499,032 bytes; SHA-256
`483a5b3e7607eb3bd63c0c6d7f75e90040c8dcc2e1249b6c034c5e26a43df2f0`.
Native file/product version 0.24.0 and executable/installer golden-logo icons pass.
Includes accumulated post-0.23 changes above. Installer not installed; no commit,
push or publication. Historical source-only notes are superseded by this packaging
entry. Native UI/keyboard, seeded-world/gameplay and physical-router/OS-session
checks remain pending. Full Windows objective remains active; Linux deferred.
