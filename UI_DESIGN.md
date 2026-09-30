# Sistema visual de TCHLauncher

La referencia completa aportada por el usuario está en [design.md](design.md). Este documento registra su aplicación al producto.

- Superficie principal blanca y barra lateral clara. Los tokens actuales usan `--mist: #f7f7f6`, `--ink: #0d0d0d`, `--secondary: #5d5d5d` y `--muted: #737373`; la referencia original permanece en `design.md`.
- Separadores finos y superficies de énfasis gris claro.
- Tipografía del sistema y escala compacta también en la entrada, con titulares de hasta 24 px.
- Filas, tablas y divisores para datos operativos. Las tarjetas se reservan para grupos que de verdad necesitan un contenedor.
- Controles de aproximadamente 10 px de radio. Estado de foco visible y movimiento reducido cuando el sistema lo pide.
- Los estados de operación y errores se distinguen mediante texto, iconos y contraste dentro de la paleta gris.
- Sin sombras ornamentales, gradientes, imágenes de relleno ni cuadrículas de tarjetas.

La navegación se divide en servidores en la barra lateral y secciones del servidor en pestañas, con el estado y la acción principal sobre el contenido. La barra lateral admite un modo reducido que conserva las imágenes, el estado y las acciones. La preferencia se recuerda localmente. Al pasar el cursor o enfocar una imagen se muestra el nombre y el contexto del servidor; el botón tiene un nombre accesible para teclado y lectores de pantalla.

Las filas de servidor muestran una miniatura cuadrada a la izquierda, nombre, tipo y versión; el punto de estado se superpone a la esquina. Los formularios de creación e importación ofrecen una vista previa de la imagen elegida. Los selectores y los desplegables usan flechas de trazo fino coherentes; los campos numéricos evitan las flechas nativas. Los botones tienen una altura mínima de 40 px y el texto operativo se mantiene legible sin depender de etiquetas minúsculas.

Los controles de capacidad combinan un número editable y un deslizador con su escala visible. RAM inicial y máxima mantienen un orden válido; jugadores y objetivo de espacio se pueden ajustar también en Configuración. La barra de espacio del Resumen muestra el uso medido respecto al objetivo de aviso.

La creación recorre cinco pasos visibles, con avance controlado, retorno a pasos completados, resumen persistente y revisión final. Configuración agrupa las propiedades por identidad y conexión, juego y acceso; cada ajuste incluye una explicación breve. Eliminar un servidor requiere una confirmación que diferencia los archivos administrados de las carpetas externas importadas.

La entrada muestra «Tu primer servidor», una frase breve y las acciones Crear e Importar. No usa un icono grande ni tarjetas de promoción. El Resumen agrupa estado, jugadores, tiempo activo, RAM y CPU en una franja clara. Rendimiento y Conexión comparten la siguiente fila; espacio, actividad y copias usan divisores. Jugadores usa una banda de cifras destacadas; Configuración da espacio a la imagen elegida; la Consola distingue hora, nivel y mensaje con búsqueda y filtro.

Rendimiento permite alternar entre RAM y CPU, con una gráfica de los últimos dos minutos, valor actual y pico. El historial se conserva al cambiar de sección durante la sesión. Se muestran tasas medidas de lectura y escritura, PID y heap máximo de Java; el heap no se presenta como RAM física consumida. Las muestras ausentes aparecen como un guion y los huecos largos interrumpen la línea. Los contadores de advertencias y errores describen únicamente el tramo reciente del registro.

Mundos usa filas con la imagen real del mundo cuando existe `icon.png`, nombre, tamaño y estado principal. Las acciones de archivos están junto al mundo al que afectan. La importación se prepara en un formulario compacto; cambiar o borrar pide confirmación y explica la copia previa.

Las transiciones de contenido, pasos, diálogos y barra lateral duran entre 150 y 180 ms. El contenido aparece con opacidad; los pasos y diálogos usan un desplazamiento de 4 px. El indicador pulsa solo durante instalación, inicio o parada. Con movimiento reducido se desactivan las animaciones. No se usan animaciones permanentes para decorar la interfaz.

La ventana y sus dos columnas no permiten desplazamiento horizontal. La lista lateral y el contenido principal conservan su desplazamiento vertical independiente.

Los diálogos usan `dialog.showModal()` en la capa superior del navegador, mediante un portal al documento. Quedan centrados en la ventana visible independientemente del desplazamiento de Configuración; el fondo se bloquea y el diálogo admite desplazamiento propio si falta altura. Cancelar devuelve el foco al botón original sin mover la página. Tras eliminar un servidor, el contenido vuelve arriba y muestra el resultado.

Conexión muestra una dirección LAN seleccionable y copiable, junto al estado local del puerto. La guía de internet está plegada por defecto y explica la configuración manual sin presentar la IP pública como prueba de accesibilidad.

La v0.6.0 se ha verificado mediante compilación y revisión del código; su apariencia y funcionamiento con teclado en la aplicación nativa siguen pendientes de revisión manual.
