# Linux — aplazado

Por petición del usuario, el trabajo activo se centra exclusivamente en Windows.
Linux se conserva como trabajo futuro; no condiciona los siguientes cambios o
instaladores de Windows. No se afirma soporte Linux verificado.

## Preparación existente

- `scripts/linux/Dockerfile`: entorno Debian Bookworm con Node, Rust y las
  dependencias de Tauri. Instala con los archivos de bloqueo, ejecuta pruebas
  frontend/nativas y compila un paquete `.deb` antes de exportarlo.
- `scripts/verify_linux.ps1`: invoca ese entorno y exporta paquetes, registros,
  versiones de herramientas y checksums a una carpeta vacía del proyecto.
- `.dockerignore`: excluye compilaciones del equipo, Git y configuración local.
- `.gitignore`: excluye `artifacts/`.

El script PowerShell pasa el análisis de sintaxis. El contenedor **no se ha
compilado ni ejecutado**. Docker Desktop recibió una solicitud de inicio, pero
su motor Linux no estuvo disponible; la consulta de estado que quedó sin
respuesta se canceló. No se generó un paquete Linux.

## Al retomar

1. Confirmar un motor Linux disponible y espacio suficiente, sin modificar
   configuraciones o eliminar recursos ajenos.
2. Revisar las versiones de imágenes/herramientas y los prerrequisitos oficiales:
   [Tauri Linux](https://v2.tauri.app/start/prerequisites/) y
   [paquetes Debian](https://v2.tauri.app/distribute/debian/).
3. Ejecutar desde la raíz `./scripts/verify_linux.ps1`, conservar todos los
   registros y resolver fallos reales de compilación/pruebas. La base inicial
   prevista es Debian Bookworm; no extrapolar compatibilidad a todas las distros.
4. Instalar el paquete en un escritorio Linux y comprobar ventana, bandeja,
   diálogos de archivos, fuentes/iconos, teclado, permisos y cierre.
5. Verificar Java gestionado/personalizado, instalación/importación de los cuatro
   motores, arranque/parada, consola, copias/restauración y juego tras aceptación
   humana de EULA. Una compilación no prueba estas operaciones.
6. Revisar inicio de sesión, entradas de autostart, procesos huérfanos, enlaces y
   archivos especiales, almacenamiento, scheduler y red real.
7. Determinar distribución final (`.deb`/AppImage), instalación, actualización,
   metadatos, integridad y alcance de plataformas soportadas.

Las menciones históricas a Linux en NEXT_TODOS/README describen la arquitectura
y pruebas pendientes; no son un plan activo mientras esté aplazado.
