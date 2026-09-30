# Trabajo pendiente

Estados: `DONE` solo para flujos conectados al backend y verificados por pruebas; `REVIEW` para implementación pendiente de validación integral.

| ID | Prioridad | Estado | Trabajo | Criterio de aceptación |
| --- | --- | --- | --- | --- |
| CORE-01 | Alta | REVIEW | Base Tauri, SQLite, servidores con UUID | Compila y registra múltiples instancias de forma persistente. |
| CORE-02 | Alta | REVIEW | Descarga Vanilla/Paper y Java | Crear e instalar en Windows y Linux con hashes comprobados. |
| CORE-03 | Alta | REVIEW | Inicio, consola y parada | Iniciar servidor real, detectar `Done`, enviar comando, detener durante el inicio y comprobar fallo persistente tras salir el proceso. |
| CORE-04 | Alta | REVIEW | Propiedades y memoria | Guardar y releer cambios con servidor detenido. |
| CORE-05 | Alta | REVIEW | Backups y restore | Copia consistente, snapshot previo y restauración verificable. |
| CORE-06 | Alta | REVIEW | Cierre con procesos activos | Ofrecer detener, mantener en segundo plano o cancelar el cierre. |
| CORE-07 | Alta | REVIEW | Estado y métricas completas | RAM, CPU, historial de dos minutos, I/O, PID y jugadores conectados implementados. Pruebas de telemetría aprobadas; verificar con Java real y añadir TPS fiable. |
| CORE-08 | Alta | REVIEW | Descargas cancelables y reintentos | Cancelación, limpieza de parciales, reintentos transitorios y nuevo intento desde UI implementados; verificar interacción nativa. No hay reanudación por HTTP Range. |
| JVM-01 | Alta | REVIEW | Argumentos JVM | Perfiles, vista previa, validación, persistencia y arranque conectados; comprobar con servidor real y Linux. |
| JAVA-01 | Alta | REVIEW | Runtime personalizado | Selección nativa, vista previa y verificación real implementadas; comprobar interacción en escritorio y Linux. |
| LOAD-01 | Alta | REVIEW | Fabric | Catálogo compatible, instalación verificada y arranque hasta EULA implementados; probar conexión con mods y añadir actualización de loader. |
| LOAD-02 | Alta | TODO | NeoForge | Instalación, arranque y actualizaciones compatibles. |
| MOD-01 | Alta | TODO | Mods y plugins | Modrinth/Paper, versiones compatibles, dependencias y cambios revisables. |
| PACK-01 | Media | TODO | Importación de modpacks | `.mrpack` y validación segura de archivos. |
| IMPORT-01 | Alta | REVIEW | Importar servidor existente | Elegir carpeta, inspeccionar `.jar`, administrar o copiar sin tocar origen. |
| IMPORT-02 | Media | REVIEW | Importar mundo | El wizard copia un mundo elegido al crear el servidor; verificar con mundos reales de distintas versiones. |
| PLAYER-01 | Media | REVIEW | Jugadores, whitelist, bans y OP | Lectura de archivos reales y comandos en vivo implementados; verificar con servidores Vanilla y Paper en ejecución. |
| WORLD-01 | Media | REVIEW | Gestión de mundos | Listado, tamaño, imagen, importación, exportación, duplicado, cambio, copia, apertura y borrado implementados; verificar con mundos reales. |
| NET-01 | Alta | REVIEW | Red LAN y puerto local | Detección de dirección local, copia y comprobación TCP implementadas; verificar con varias interfaces y redes reales. |
| NET-02 | Alta | TODO | Red pública | UPnP/NAT-PMP, comprobación remota del puerto y retirada de mappings creados por la app. La consulta opcional de IP pública y la guía manual ya están conectadas. |
| TASK-01 | Media | TODO | Programador local | Copias periódicas, reinicios y arranque automático explícito. |
| UX-01 | Alta | TODO | QA de escritorio | Prueba manual 1280×720 y 1920×1080, teclado y lector de pantalla. |
| UX-02 | Media | REVIEW | Imagen de servidor | Elegir, validar, guardar, cambiar y quitar imagen; comprobarlo visualmente en la aplicación nativa. |
| UX-03 | Media | REVIEW | Barra lateral plegable y rediseño | Verificar visualmente el modo amplio y reducido, nombres emergentes, foco, escalas y movimiento en Windows. |
| UX-04 | Media | REVIEW | Asistente y configuración | Verificar los cinco pasos, la revisión final, los grupos de propiedades y el comportamiento con teclado. |
| UX-05 | Media | REVIEW | Eliminación de servidor | Confirmar en escritorio el diálogo centrado desde Configuración desplazada, restauración de foco, borrado administrado y conservación de carpetas externas importadas. |
| REL-01 | Alta | TODO | Distribución | Iconos, firmas y paquetes de Windows/Linux. |
| REL-02 | Media | REVIEW | Instalador Windows v0.6.4 | NSIS x64 generado; 35 pruebas Rust locales y perfiles JVM con Java 21 aprobados. Falta instalación, conexión desde Minecraft y revisión visual manual. |

Revisión 0.6.1: descargas cancelables y con reintento, Temurin JDK 16 como alternativa al JRE ausente, perfiles de skins, icono Minecraft compartido, fijados persistentes, menú contextual y gráficas simultáneas implementados. Prueba visual manual pendiente; no se usaron computer use ni subagentes.

Revisión 0.6.2: Fabric añadido al asistente y a importación. Bibliotecas verificadas por SHA-256, Minecraft por SHA-1, generación atómica y arranque sin descargas externas ocultas. La comprobación con Java real alcanza la EULA sin aceptarla; conexión desde el juego, mods y actualizaciones siguen pendientes.

Revisión 0.6.3: Java personalizado con vista previa, confirmación, verificación real y requisitos oficiales cacheados. Prueba con Java 21 aprobada; revisión nativa del selector y compatibilidad en Linux pendientes.

Revisión 0.6.4: biblioteca con búsqueda y vistas, rail oscuro y navegación lateral clara, controles antracita según la última preferencia, imágenes originales y sistema de motion implementados. Perfiles JVM recomendados/personalizados conectados al arranque. La revisión visual nativa y la medición de fluidez están pendientes; no se usaron subagentes ni computer use.
