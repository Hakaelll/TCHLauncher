# Seguridad

- Los comandos Rust validan nombre, versión, memoria, puerto y propiedades.
- Las descargas solo aceptan HTTPS y usan archivos `.part`; los hashes publicados se verifican.
- La extracción de Java rechaza rutas fuera del destino y enlaces simbólicos ZIP. Las copias no siguen enlaces simbólicos.
- La EULA solo se escribe tras una acción explícita.
- Los servidores se ejecutan como usuario normal, sin privilegios elevados.
- Las imágenes de servidor se limitan a 2 MB, se decodifican con límites de dimensiones y memoria, y se publican como PNG 64 × 64 en `server-icon.png`. Cambiarlas requiere el servidor detenido.
- Los perfiles de skins se consultan por UUID en Mojang; solo se aceptan texturas del host oficial sin redirecciones, con límites de respuesta y decodificación.
- La cancelación interrumpe conexiones y cuerpos HTTP, limpia parciales y mantiene los artefactos anteriores hasta verificar la descarga nueva. Los reintentos automáticos se limitan a fallos transitorios.

Pendiente antes de distribuir: auditoría completa de hosts de descarga y política de cierre durante tareas activas, y prueba de restauración ante fallos de disco.

## Custom Java executables

Custom Java requires an explicit executable selection and a separate confirmation. Only `java`/`java.exe` filenames are accepted; paths are canonicalized and must reference a file. Filename validation does not establish executable trust: selecting a custom runtime executes that user-chosen program with `-version`. Inspection has a five-second timeout, hidden Windows console and bounded output reading. Compatibility is checked before persistence and at launch when the official requirement is cached. No system Java settings are changed.

## JVM profiles and cover artwork

Custom options are individual process arguments, never shell text. Memory, entrypoint and external argument-file overrides are rejected. Java agents remain available to advanced users and execute user-selected code during validation; the editor states that they must be trusted. Artwork inputs remain bounded to 2 MB, 4096×4096 and 64 MB decode allocation. The full cover is private local application data; the Minecraft icon is derived from the same image. Publication rolls back previous files on failure.
