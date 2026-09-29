# TCHLauncher

Gestor local de servidores Minecraft para Windows y Linux, construido con Tauri, Rust, React y TypeScript. El código actual es una base funcional en desarrollo; [NEXT_TODOS.md](NEXT_TODOS.md) registra lo pendiente.

## Ejecutar

Requisitos de desarrollo: Rust, Node.js, pnpm y los prerrequisitos de Tauri 2 para el sistema operativo.

```sh
pnpm install
pnpm tauri dev
```

Para compilar el frontend: `pnpm build`. Para las pruebas Rust: `cargo test --manifest-path src-tauri/Cargo.toml`.

## Flujo disponible

1. Crear un servidor Vanilla o Paper con un asistente de cinco pasos: identidad, versión, mundo, capacidad y revisión. Se elige una versión release del catálogo de Mojang y, opcionalmente, un mundo existente.
2. Descargar un runtime Temurin local y el servidor desde Mojang o PaperMC. Los artefactos con hash publicado se verifican antes de usarse.
3. Leer y aceptar explícitamente la EULA de Minecraft.
4. Iniciar el servidor, ver la salida, enviar comandos y detenerlo de forma ordenada.
5. Editar `server.properties` y memoria con el servidor detenido.
6. Crear y restaurar copias de seguridad locales con el servidor detenido.
7. Importar una carpeta existente, copiándola a la biblioteca o administrándola en su ubicación original, y preparar Java sin sustituir su `.jar`.
8. Elegir una imagen PNG, JPG o WebP para un servidor nuevo, importado o ya existente. Se muestra en la barra lateral y junto al título del servidor.
9. Ajustar RAM, jugadores y objetivo de espacio escribiendo un valor o moviendo un deslizador. El objetivo de espacio compara el tamaño de la carpeta del servidor y avisa al superarlo; no limita el disco.
10. Consultar jugadores conocidos y conectados, lista blanca, operadores y bloqueos. Con el servidor en marcha, enviar comandos para gestionar permisos, expulsiones y bloqueos. Importar una lista blanca `.txt` o `whitelist.json` con vista previa.
11. Buscar y filtrar la consola por nivel, copiar las líneas visibles y exportar el log completo. El Resumen muestra eventos recientes detectados en la salida.
12. Reducir la barra lateral a una columna de imágenes de servidores, con nombre al pasar el cursor o enfocar con teclado. La elección se recuerda entre sesiones.
13. Gestionar mundos locales: importar una copia, duplicar, exportar, cambiar el mundo principal, abrir su carpeta y eliminarlo con copia previa. La pestaña Mundos requiere detener el servidor para operaciones de archivos.
14. Eliminar un servidor detenido desde Configuración con una confirmación que muestra el alcance. Las carpetas importadas administradas en su ubicación original permanecen en el disco.

Los datos se almacenan en el directorio de datos de la aplicación del sistema. No hay cuentas ni servicio cloud propio. Algunas descargas requieren acceso a internet.

## Generar un instalador de Windows

```sh
pnpm tauri build --bundles nsis
```

El instalador se genera en `src-tauri/target/release/bundle/nsis/`. El binario y el instalador actuales no están firmados digitalmente. Los cambios de v0.4.0 se describen en [RELEASE_NOTES_v0.4.0.md](RELEASE_NOTES_v0.4.0.md).

## Estado del proyecto

Consulta [ARCHITECTURE.md](ARCHITECTURE.md) para la estructura, [design.md](design.md) para la referencia visual aportada y [NEXT_TODOS.md](NEXT_TODOS.md) para funciones pendientes. Fabric, NeoForge, mods, plugins, importación de modpacks y la configuración de red pública aún no están implementados.
