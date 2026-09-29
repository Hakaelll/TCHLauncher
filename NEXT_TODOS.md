# Trabajo pendiente

Estados: `DONE` solo para flujos conectados al backend y verificados por pruebas; `REVIEW` para implementación pendiente de validación integral.

| ID | Prioridad | Estado | Trabajo | Criterio de aceptación |
| --- | --- | --- | --- | --- |
| CORE-01 | Alta | REVIEW | Base Tauri, SQLite, servidores con UUID | Compila y registra múltiples instancias de forma persistente. |
| CORE-02 | Alta | REVIEW | Descarga Vanilla/Paper y Java | Crear e instalar en Windows y Linux con hashes comprobados. |
| CORE-03 | Alta | REVIEW | Inicio, consola y parada | Iniciar servidor real, detectar `Done`, enviar comando y detenerlo. |
| CORE-04 | Alta | REVIEW | Propiedades y memoria | Guardar y releer cambios con servidor detenido. |
| CORE-05 | Alta | REVIEW | Backups y restore | Copia consistente, snapshot previo y restauración verificable. |
| CORE-06 | Alta | REVIEW | Cierre con procesos activos | Ofrecer detener, mantener en segundo plano o cancelar el cierre. |
| CORE-07 | Alta | REVIEW | Estado y métricas completas | CPU y jugadores conectados implementados; verificar con procesos reales y añadir TPS fiable. |
| CORE-08 | Alta | TODO | Descargas cancelables y reintentos | Cancelar sin corrupción y reanudar desde UI. |
| LOAD-01 | Alta | TODO | Fabric y NeoForge | Instalación, arranque y actualizaciones compatibles. |
| MOD-01 | Alta | TODO | Mods y plugins | Modrinth/Paper, versiones compatibles, dependencias y cambios revisables. |
| PACK-01 | Media | TODO | Importación de modpacks | `.mrpack` y validación segura de archivos. |
| IMPORT-01 | Alta | REVIEW | Importar servidor existente | Elegir carpeta, inspeccionar `.jar`, administrar o copiar sin tocar origen. |
| IMPORT-02 | Media | REVIEW | Importar mundo | El wizard copia un mundo elegido al crear el servidor; verificar con mundos reales de distintas versiones. |
| PLAYER-01 | Media | REVIEW | Jugadores, whitelist, bans y OP | Lectura de archivos reales y comandos en vivo implementados; verificar con servidores Vanilla y Paper en ejecución. |
| WORLD-01 | Media | REVIEW | Gestión de mundos | Listado, tamaño, imagen, importación, exportación, duplicado, cambio, copia, apertura y borrado implementados; verificar con mundos reales. |
| NET-01 | Alta | TODO | Red LAN y pública | Detectar IP local, probar puerto, UPnP/NAT-PMP y guía manual. |
| TASK-01 | Media | TODO | Programador local | Copias periódicas, reinicios y arranque automático explícito. |
| UX-01 | Alta | TODO | QA de escritorio | Prueba manual 1280×720 y 1920×1080, teclado y lector de pantalla. |
| UX-02 | Media | REVIEW | Imagen de servidor | Elegir, validar, guardar, cambiar y quitar imagen; comprobarlo visualmente en la aplicación nativa. |
| UX-03 | Media | REVIEW | Barra lateral plegable y rediseño | Verificar visualmente el modo amplio y reducido, nombres emergentes, foco, escalas y movimiento en Windows. |
| UX-04 | Media | REVIEW | Asistente y configuración | Verificar los cinco pasos, la revisión final, los grupos de propiedades y el comportamiento con teclado. |
| UX-05 | Media | REVIEW | Eliminación de servidor | Confirmar en escritorio el borrado de archivos administrados y la conservación de carpetas externas importadas. |
| REL-01 | Alta | TODO | Distribución | Iconos, firmas y paquetes de Windows/Linux. |
| REL-02 | Media | REVIEW | Instalador Windows v0.4.0 | NSIS x64 generado; falta instalación y revisión visual manual en un sistema limpio. |
