# Arquitectura

`src/` contiene la interfaz React y un cliente tipado de comandos Tauri. `src-tauri/src/` contiene módulos de almacenamiento, instalación, procesos, propiedades, jugadores y copias. Los comandos del backend validan y leen de disco; el frontend no decide por sí mismo si un servidor ha arrancado.

El registro de servidores y copias está en SQLite (`database/tchlauncher.sqlite`). Cada servidor tiene UUID y directorio propio bajo `servers/<uuid>`. Java gestionado está en `runtimes/` y las copias en `backups/<uuid>/`. El objetivo de espacio se guarda por servidor y se migra con un valor predeterminado de 20 GB para registros anteriores. Mide la carpeta del servidor, sin incluir las copias. `server.properties`, `eula.txt`, mundos y logs siguen siendo archivos ordinarios.

Las imágenes elegidas por el usuario se validan por tamaño y firma PNG, JPG o WebP, y se copian a `server-images/<uuid>` bajo los datos de la aplicación. La interfaz recibe una URL de datos para mostrarlas; el archivo de origen no se modifica. Las imágenes se pueden sustituir o quitar desde Configuración.

La instalación consulta el catálogo oficial de Mojang y la API de PaperMC. La versión de Java procede de los metadatos de Minecraft. El runtime se descarga desde Eclipse Temurin, se verifica mediante SHA-256 y se extrae en un directorio temporal antes de moverlo a su destino. El jar Vanilla se verifica con SHA-1; Paper utiliza SHA-256 cuando la respuesta del servicio lo proporciona. No se acepta EULA automáticamente.

`ProcessManager` mantiene los procesos activos y emite eventos Tauri de estado y consola. Envía `stop` por stdin y fuerza la terminación solo tras 30 segundos. Si se cierra la ventana con servidores activos, el usuario puede cancelarlo, detenerlos o mantener la aplicación en la bandeja del sistema. Al iniciar la aplicación, el estado se reconstruye a partir de los procesos bajo control de esta instancia; no se recuperan procesos externos u huérfanos.

La sección Jugadores lee los archivos Vanilla que Minecraft mantiene para jugadores conocidos, lista blanca, operadores y bloqueos. Las modificaciones con el servidor en marcha se envían por la consola, y los conectados se derivan de los eventos de entrada y salida del proceso gestionado. Tras reiniciar la aplicación, la lista de conectados se reconstruye a partir de nuevos eventos.

Las copias se hacen con el servidor detenido y se activan por intercambio de directorios. La restauración crea antes una copia del estado actual.

La importación inspecciona una carpeta existente mediante un selector nativo. Puede registrarla en su ubicación original o copiarla a la biblioteca. El `.jar` importado permanece intacto; se prepara únicamente el runtime Java compatible.

## Limitaciones actuales

La base todavía no gestiona tareas cancelables, importación de mundos ni el resto de plataformas y gestores especificados. Se detallan en `NEXT_TODOS.md`.
