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

Sin infraestructura cloud propia ni suscripción obligatoria. Backend Rust como fuente de verdad del estado. Aceptación explícita de la EULA. Funciones incompletas registradas en `NEXT_TODOS.md`.

Los commits y el push continúan entre iteraciones. Por petición del usuario,
se acumulan 3 o 4 versiones antes de publicar otra release en GitHub.
La 0.48.0 está publicada como preview de Windows, solo con el EXE adjunto,
cumpliendo la última petición de publicar al terminar la tanda de cambios.
La última instrucción pide publicar la versión que cierre esta tanda de mejoras,
sin esperar al margen habitual de 3 o 4 versiones. Las versiones intermedias
siguen siendo locales. La publicación de la 0.48.0 cumple esa petición.
Los commits no cuentan como versiones.

El usuario asume la validación manual pendiente. Se indican las pruebas no
realizadas, pero su ausencia no bloquea ni detiene la implementación. No se usa
Computer Use ni se repiten solicitudes de capturas o aceptación visual.

## Brand Commitments

TCHLauncher. Estética Minecraft y End: negro/gris oscuro, acentos morados, controles rectangulares con relieve y tipografía Monocraft donde es legible. El usuario proporcionó el emblema voxel dorado. Los temas End, Obsidiana y Nether y una preferencia de tipografía se eligen en Ajustes. `design.md` registra la dirección vigente.

## Evidence on Hand

La especificación original, el design.md, las pautas de movimiento, referencias Minecraft/Minehut y capturas del launcher fueron proporcionados por el usuario, junto con un logo. El usuario ha aceptado el aspecto visual actual. Las comprobaciones funcionales y de accesibilidad conservan su alcance propio; no se usan computer use ni subagentes.

## Product Principles

- Una acción principal clara por pantalla.
- Estado del servidor confirmado por el proceso real.
- Errores legibles con información concreta.
- Archivos Minecraft normales y recuperables.
- Funciones reales antes que controles decorativos.
