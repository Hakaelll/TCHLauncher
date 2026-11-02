# Product

<!-- impeccable:product-schema 1 -->

## Platform

adaptive

## Stack

Tauri 2, Rust, React, TypeScript estricto, Vite y SQLite, según la especificación proporcionada.

## Users

Personas que quieren alojar servidores Minecraft en su ordenador sin configurar manualmente Java, archivos y procesos, con acceso a ajustes avanzados cuando lo necesiten.

## Product Purpose

Crear y administrar varios servidores locales y permitir que otros jugadores se conecten mediante las opciones de red disponibles.

## Operating Context

Aplicación de escritorio para Windows 10/11. Linux está aplazado por petición del usuario; el trabajo futuro se registra en `LINUX_LATER.md`. Los servidores Minecraft y sus archivos permanecen en el equipo. La conexión a internet se utiliza para catálogos y descargas.

## Capabilities and Constraints

Sin infraestructura cloud propia ni suscripción obligatoria. Backend Rust como fuente de verdad del estado. Aceptación explícita de la EULA. Cierre del alcance y trabajo aplazado registrados en `NEXT_TODOS.md`.

Los commits y el push continúan entre iteraciones. Por petición del usuario,
se acumulan 3 o 4 versiones antes de publicar otra release en GitHub.
La 0.59.0 está publicada como preview de Windows, solo con el EXE adjunto.
Incluye los hitos 0.58 y 0.59 y cierra esta tanda de implementación por petición
del usuario. La etiqueta apunta al código empaquetado; el archivo remoto coincide
en nombre, tamaño y SHA-256 con el instalador local. No se sube un archivo de
checksum. Se conservan los instaladores locales 0.58 y 0.59. Se mantiene la
cadencia habitual de 3 o 4 versiones entre futuras publicaciones; los commits no
cuentan como versiones.

El 2 de noviembre de 2026 el usuario confirma las verificaciones manuales
pendientes con "tu sigue las verificaciones estan hechas". Se registra como
aceptación del usuario, sin atribuir al agente pruebas que no ejecutó ni inventar
resultados individuales de dispositivos. El alcance actual de Windows queda
cerrado. No se usa Computer Use ni se repiten solicitudes de capturas o aceptación.
El 31 de octubre de 2026 el usuario confirma que los fallos antiguos del otro
ordenador ya están resueltos. Se cierran esos informes por su confirmación y no
se vuelven a usar para bloquear el trabajo.

## Brand Commitments

TCHLauncher. Estética Minecraft y End: negro/gris oscuro, acentos morados, controles rectangulares con relieve y tipografía Monocraft donde es legible. El logo es un bloque geométrico negro con capas apiladas que representan servidores. El original tiene fondo transparente; los iconos usan su versión sobre blanco para mantener el contraste. Los temas End, Obsidiana y Nether y una preferencia de tipografía se eligen en Ajustes. `design.md` registra la dirección vigente.

La versión geométrica sustituye al logo ilustrado durante la revisión local de
la 0.60. GitHub sigue publicando la 0.59; se conservan las versiones locales 0.59
y 0.60. El apartado About de GitHub describe la aplicación en inglés.

## Evidence on Hand

La especificación original, el design.md, las pautas de movimiento, referencias Minecraft/Minehut y capturas del launcher fueron proporcionados por el usuario, junto con un logo. El usuario ha aceptado el aspecto visual actual. Las comprobaciones funcionales y de accesibilidad conservan su alcance propio; no se usan computer use ni subagentes.

## Product Principles

- Una acción principal clara por pantalla.
- Estado del servidor confirmado por el proceso real.
- Errores legibles con información concreta.
- Archivos Minecraft normales y recuperables.
- Funciones reales antes que controles decorativos.
