# TCHLauncher

Gestor local de servidores Minecraft para Windows y Linux, construido con Tauri, Rust, React y TypeScript. El código actual es una base funcional en desarrollo; [NEXT_TODOS.md](NEXT_TODOS.md) registra lo pendiente.

## Ejecutar

Requisitos de desarrollo: Rust, Node.js, pnpm y los prerrequisitos de Tauri 2 para el sistema operativo.

```sh
pnpm install
pnpm tauri dev
```

Para compilar el frontend: `pnpm build`. Para las pruebas Rust: `cargo test --manifest-path src-tauri/Cargo.toml`. Las pruebas de telemetría se ejecutan con `pnpm test` (Node.js 24).

## Flujo disponible

1. Crear un servidor Vanilla, Paper o Fabric con un asistente de cinco pasos: identidad, versión, mundo, capacidad y revisión. Se elige una versión release del catálogo de Mojang y, opcionalmente, un mundo existente.
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

```sh
pnpm tauri build --bundles nsis
```

El instalador v0.6.0 de Windows x64 se ha generado en `src-tauri/target/release/bundle/nsis/TCHLauncher_0.6.0_x64-setup.exe`. El binario y el instalador actuales no están firmados digitalmente. Los cambios y las limitaciones se describen en inglés en [RELEASE_NOTES_v0.6.0.md](RELEASE_NOTES_v0.6.0.md).

## Estado del proyecto

Consulta [ARCHITECTURE.md](ARCHITECTURE.md) para la estructura, [design.md](design.md) para la referencia visual aportada y [NEXT_TODOS.md](NEXT_TODOS.md) para funciones pendientes. Fabric, NeoForge, mods, plugins, importación de modpacks y la configuración de red pública aún no están implementados.

### Cambios de 0.6.1

- Entrada con opciones visuales inspiradas en aumentos de TFT, acentos de color y movimiento breve.
- Menú de clic derecho para fijar/desfijar servidores, abrir configuración y solicitar eliminación confirmada.
- Cabezas de skins reales por UUID, con caché local y alternativa cuando no están disponibles.
- Etiquetas de consola coloreadas y autocompletado de WebView2 desactivado.
- Descargas cancelables con progreso, velocidad, reintentos y limpieza de parciales.
- Gestión visible de Java y fallback de JRE a JDK de la misma versión para Temurin 16.

Validación 0.6.1: 30 pruebas Rust locales, 6 de telemetría y las 2 comprobaciones con servicios oficiales aprobadas. Las comprobaciones con servicios oficiales son optativas: `cargo test --manifest-path src-tauri/Cargo.toml installs_real_minecraft_1171_and_temurin16_without_accepting_eula -- --ignored --nocapture`. Usan carpetas temporales y no aceptan la EULA. La prueba Fabric arranca Minecraft hasta el aviso de EULA y comprueba que no se haya creado un mundo. La revisión visual y la conexión desde un cliente Minecraft requieren prueba manual.

### Cambios de 0.6.2

Fabric aparece en creación e importación. Su catálogo muestra versiones de Minecraft compatibles, y la instalación elige un loader estable con bibliotecas verificadas y publicación atómica. La importación mantiene los archivos existentes, incluidas generaciones creadas por TCHLauncher. La comprobación real usa Minecraft 1.21.4 y Fabric Loader 0.19.5 y llega al aviso de EULA sin aceptarla. NeoForge, gestión de mods y actualización de loaders siguen pendientes.

### Java personalizado

En Configuración, abre **Java personalizado** y elige `bin/java.exe` (Windows) o `bin/java` (Linux). La aplicación verifica la versión requerida por Minecraft y muestra la ruta antes de confirmar. Puedes volver a Temurin gestionado desde el mismo panel. La comprobación de arranque usa los requisitos oficiales guardados, sin modificar el Java global.

Prueba dirigida con Java 21 instalado: define `TCH_TEST_JAVA` con la ruta del ejecutable y ejecuta `cargo test --manifest-path src-tauri/Cargo.toml verifies_real_custom_java_without_changing_an_incompatible_selection -- --ignored`.
