# TCHLauncher v0.2.0

Esta versión amplía la gestión de servidores locales Vanilla y Paper y actualiza la interfaz de capacidad.

## Cambios

- Campos numéricos y deslizadores sincronizados para RAM inicial, RAM máxima y jugadores máximos al crear o configurar un servidor.
- Objetivo de espacio configurable con campo numérico y deslizador al crear, importar o configurar un servidor. El Resumen compara este objetivo con el tamaño real de la carpeta del servidor y avisa si se supera. No impone una cuota al disco ni incluye las copias de seguridad.
- Sección Jugadores para consultar conexiones del proceso gestionado, jugadores conocidos, lista blanca, operadores y bloqueos registrados por Minecraft. Con el servidor en marcha, permite enviar comandos de acceso y moderación e importar una lista blanca desde `.txt` o `whitelist.json`.
- Migración de la base de datos existente: los servidores anteriores reciben un objetivo de espacio predeterminado de 20 GB.

## Requisitos y límites

- El instalador de Windows no está firmado digitalmente.
- Las descargas de Java y de los archivos del servidor requieren internet. Los servidores y sus datos se ejecutan y almacenan localmente.
- Fabric, NeoForge, mods, plugins, importación de mundos y modpacks, red pública y tareas programadas siguen pendientes. Consulta `NEXT_TODOS.md`.
- La experiencia visual nativa y el flujo de jugadores requieren una revisión manual con un servidor en ejecución.
