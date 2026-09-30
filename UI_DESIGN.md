# Sistema visual de TCHLauncher

La referencia completa aportada por el usuario está en [design.md](design.md). Este documento registra su aplicación al producto.

- Superficie principal blanca y barra lateral clara. Los tokens actuales usan `--mist: #f7f7f6`, `--ink: #0d0d0d`, `--secondary: #5d5d5d` y `--muted: #737373`; la referencia original permanece en `design.md`.
- Separadores finos y superficies de énfasis gris claro.
- Tipografía del sistema y escala compacta también en la entrada, con titulares de hasta 24 px.
- Filas, tablas y divisores para datos operativos. Las tarjetas se reservan para grupos que de verdad necesitan un contenedor.
- Controles de aproximadamente 10 px de radio. Estado de foco visible y movimiento reducido cuando el sistema lo pide.
- Por petición posterior del usuario, la paleta incorpora latón en acciones y navegación, verde en ejecución, ámbar en operaciones y rojo en errores y eliminación. Cada estado conserva su etiqueta textual.
- Sin imágenes de relleno ni sombras ornamentales. El menú contextual usa una sombra discreta para distinguir su capa. La entrada admite dos tarjetas ornamentadas por la referencia explícita de aumentos de TFT, con geometría SVG propia.

La navegación se divide en servidores en la barra lateral y secciones del servidor en pestañas, con el estado y la acción principal sobre el contenido. La barra lateral admite un modo reducido que conserva las imágenes, el estado y las acciones. La preferencia se recuerda localmente. Al pasar el cursor o enfocar una imagen se muestra el nombre y el contexto del servidor; el botón tiene un nombre accesible para teclado y lectores de pantalla.

Las filas de servidor muestran una miniatura cuadrada a la izquierda, nombre, tipo y versión; el punto de estado se superpone a la esquina. Los formularios de creación e importación ofrecen una vista previa de la imagen elegida. Los selectores y los desplegables usan flechas de trazo fino coherentes; los campos numéricos evitan las flechas nativas. Los botones tienen una altura mínima de 40 px y el texto operativo se mantiene legible sin depender de etiquetas minúsculas.

Los controles de capacidad combinan un número editable y un deslizador con su escala visible. RAM inicial y máxima mantienen un orden válido; jugadores y objetivo de espacio se pueden ajustar también en Configuración. La barra de espacio del Resumen muestra el uso medido respecto al objetivo de aviso.

La creación recorre cinco pasos visibles, con avance controlado, retorno a pasos completados, resumen persistente y revisión final. Configuración agrupa las propiedades por identidad y conexión, juego y acceso; cada ajuste incluye una explicación breve. Eliminar un servidor requiere una confirmación que diferencia los archivos administrados de las carpetas externas importadas.

La entrada presenta dos opciones visuales: Crear en marfil y latón, e Importar en salvia. Marcos finos, emblemas geométricos y acciones al pie hacen visibles las dos rutas sin añadir texto promocional. Las tarjetas aparecen una vez y responden al cursor y la pulsación; se respeta el movimiento reducido.

Rendimiento tiene una pestaña propia. RAM y CPU se muestran simultáneamente, lado a lado cuando cabe y apiladas en ventanas estrechas. Ambas mantienen historial, valor actual, pico y huecos de datos reales. Las líneas usan latón y verde azulado. Resumen conserva las cifras inmediatas y la conexión.

Consola mantiene neutro el texto del mensaje y aplica etiquetas coloreadas a INFO (azul), SYSTEM (violeta), PLAYER (verde), CHAT (verde azulado), COMMAND (latón), WARN (ámbar) y ERROR (rojo). El nombre del nivel siempre es visible. El autocompletado general de WebView2 y del campo de comandos se desactiva.

El clic derecho en un servidor abre acciones de fijar/desfijar, configuración y eliminar. El menú se ajusta al viewport y admite flechas, Inicio/Fin y Escape. Los servidores fijados se guardan en SQLite y se ordenan primero. Eliminar reutiliza el diálogo nativo centrado con confirmación.

Jugadores incorpora la cabeza pixelada real de la skin cuando hay UUID y un perfil disponible; muestra una inicial si falta. La imagen del servidor se adapta a PNG 64 × 64 y se comparte con `server-icon.png`, incluido el icono de una carpeta importada. Configuración permite verificar y reinstalar Java, con versión y ruta reales. Las descargas muestran fase, bytes, velocidad, cancelación y reintento.

Mundos usa filas con la imagen real del mundo cuando existe `icon.png`, nombre, tamaño y estado principal. Las acciones de archivos están junto al mundo al que afectan. La importación se prepara en un formulario compacto; cambiar o borrar pide confirmación y explica la copia previa.

Las transiciones de contenido, pasos, diálogos y barra lateral duran entre 150 y 180 ms. El contenido aparece con opacidad; los pasos y diálogos usan un desplazamiento de 4 px. El indicador pulsa solo durante instalación, inicio o parada. Con movimiento reducido se desactivan las animaciones. No se usan animaciones permanentes para decorar la interfaz.

La ventana y sus dos columnas no permiten desplazamiento horizontal. La lista lateral y el contenido principal conservan su desplazamiento vertical independiente.

Los diálogos usan `dialog.showModal()` en la capa superior del navegador, mediante un portal al documento. Quedan centrados en la ventana visible independientemente del desplazamiento de Configuración; el fondo se bloquea y el diálogo admite desplazamiento propio si falta altura. Cancelar devuelve el foco al botón original sin mover la página. Tras eliminar un servidor, el contenido vuelve arriba y muestra el resultado.

Conexión muestra una dirección LAN seleccionable y copiable, junto al estado local del puerto. La guía de internet está plegada por defecto y explica la configuración manual sin presentar la IP pública como prueba de accesibilidad.

La v0.6.0 se ha verificado mediante compilación y revisión del código; su apariencia y funcionamiento con teclado en la aplicación nativa siguen pendientes de revisión manual.

## Verificación de esta revisión

Revisión de código y compilación, sin computer use ni subagentes por petición del usuario. La comprobación visual en Windows, navegación completa con teclado y conexión desde un cliente Minecraft quedan para prueba manual.

El paso Versión permite Vanilla, Paper y Fabric. Al elegir Fabric se consulta su catálogo compatible; el avance se desactiva mientras carga y ofrece reintento si falla. Las respuestas de la plataforma anterior se descartan. Importación detecta nombres de launchers Fabric y propone el archivo correspondiente; tipo y versión siguen editables.

## Java avanzado (0.6.3)

El selector personalizado permanece en un desplegable de Configuración. Muestra versión, ruta y compatibilidad antes de confirmar, con Cancelar como foco inicial. El panel distingue runtime gestionado, externo, incompatible y versión comprobada sin metadatos. Los errores se muestran junto al control; el runtime previo se conserva si la selección falla.
