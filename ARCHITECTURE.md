# Arquitectura

`src/` contiene la interfaz React y un cliente tipado de comandos Tauri. `src-tauri/src/` contiene módulos de almacenamiento, instalación, procesos, propiedades, jugadores y copias. Los comandos del backend validan y leen de disco; el frontend no decide por sí mismo si un servidor ha arrancado.

El registro de servidores y copias está en SQLite (`database/tchlauncher.sqlite`). Cada servidor tiene UUID y directorio propio bajo `servers/<uuid>`. Java gestionado está en `runtimes/` y las copias en `backups/<uuid>/`. El objetivo de espacio se guarda por servidor y se migra con un valor predeterminado de 20 GB para registros anteriores. Mide la carpeta del servidor, sin incluir las copias. `server.properties`, `eula.txt`, mundos y logs siguen siendo archivos ordinarios.

Las imágenes elegidas por el usuario se validan por tamaño y firma PNG, JPG o WebP, y se copian a `server-images/<uuid>` bajo los datos de la aplicación. La interfaz recibe una URL de datos para mostrarlas; el archivo de origen no se modifica. Las imágenes se pueden sustituir o quitar desde Configuración.

La instalación consulta el catálogo oficial de Mojang y la API de PaperMC. La versión de Java procede de los metadatos de Minecraft. El runtime se descarga desde Eclipse Temurin, se verifica mediante SHA-256 y se extrae en un directorio temporal antes de moverlo a su destino. El jar Vanilla se verifica con SHA-1; Paper utiliza SHA-256 cuando la respuesta del servicio lo proporciona. No se acepta EULA automáticamente.

`ProcessManager` mantiene los procesos activos y emite eventos Tauri de estado y consola. Conserva el último estado terminal durante la sesión para que un fallo siga visible después de retirar el proceso. Envía `stop` por stdin durante inicio o ejecución y fuerza la terminación solo tras 30 segundos; las peticiones repetidas de parada se ignoran. La salida de inicio solo puede promover un proceso que aún esté en `starting`. Si se cierra la ventana con servidores activos, el usuario puede cancelarlo, detenerlos o mantener la aplicación en la bandeja del sistema. Al iniciar la aplicación, el estado se reconstruye a partir de los procesos bajo control de esta instancia; no se recuperan procesos externos u huérfanos.

`get_metrics` toma muestras del proceso Java con `sysinfo` en una tarea bloqueante separada: RAM física del proceso, CPU normalizada sobre los núcleos del equipo y contadores acumulados de lectura/escritura. Cada respuesta incluye servidor, PID y hora de muestreo; la primera lectura de CPU puede no estar disponible. El tamaño de la carpeta se cachea durante 60 segundos. La interfaz consulta aproximadamente cada tres segundos sin solapar consultas y conserva un historial de dos minutos por servidor en `App`. `telemetry.ts` descarta respuestas tardías, reinicia el historial cuando cambia el proceso, calcula tasas con el tiempo real entre muestras y separa huecos mayores de 15 segundos. El historial no se guarda en disco y no mide TPS.

Las propiedades se releen al cambiar el estado del servidor, porque Minecraft puede generar valores durante el primer inicio. Las respuestas antiguas se descartan y las modificaciones pendientes del usuario tienen prioridad hasta guardar. `Modal.tsx` presenta los diálogos mediante un portal y la capa superior nativa, con foco inicial, cierre por Escape cuando procede y restauración del foco sin desplazar el contenido.

`log.rs` añade una marca UTC a cada línea capturada, clasifica el nivel y algunos eventos operativos, y lee un tramo reciente para la Consola y el Resumen. Acepta líneas antiguas sin marca. La exportación copia el log completo a una ruta elegida por el usuario sin modificar el original. Limpiar la vista de Consola es un filtro local y conserva el historial y los eventos del Resumen.

La sección Jugadores lee los archivos Vanilla que Minecraft mantiene para jugadores conocidos, lista blanca, operadores y bloqueos. Las modificaciones con el servidor en marcha se envían por la consola, y los conectados se derivan de los eventos de entrada y salida del proceso gestionado. Tras reiniciar la aplicación, la lista de conectados se reconstruye a partir de nuevos eventos.

Las copias se hacen con el servidor detenido y se activan por intercambio de directorios. La restauración crea antes una copia del estado actual.

La importación inspecciona una carpeta existente mediante un selector nativo. Puede registrarla en su ubicación original o copiarla a la biblioteca. El `.jar` importado permanece intacto; se prepara únicamente el runtime Java compatible.

`world.rs` identifica carpetas de mundo inmediatas mediante `level.dat` comprimido y datos NBT básicos. Puede importar, duplicar y exportar a través de copias preparadas en directorios temporales y renombradas al finalizar. Rechaza nombres de ruta inseguros y enlaces simbólicos. Cambiar el mundo principal actualiza `level-name` después de guardar una copia; borrar también crea antes una copia y nunca permite eliminar el mundo activo. La creación de un servidor puede importar un mundo elegido a `world` antes de instalarlo. Las operaciones de archivos reservan el servidor y bloquean su inicio hasta terminar.

`network.rs` lee el puerto y la IP de escucha desde `server.properties`, obtiene la dirección IPv4 de la ruta local sin enviar paquetes y comprueba el puerto TCP local. El Resumen presenta la dirección LAN cuando puede determinarse. La consulta de IP pública se realiza solo si el usuario la solicita; utiliza el servicio gratuito [ipify](https://www.ipify.org/) y no comprueba la accesibilidad desde internet. UPnP, NAT-PMP y la comprobación remota del puerto siguen pendientes.

## Limitaciones actuales

La base todavía no gestiona tareas cancelables, mapeo automático de puertos ni el resto de plataformas y gestores especificados. Se detallan en `NEXT_TODOS.md`.
