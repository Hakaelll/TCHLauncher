# Sistema visual de TCHLauncher

La referencia completa aportada por el usuario está en [design.md](design.md). Este documento registra su aplicación al producto.

- Superficie principal blanca y barra lateral `#f9f9f9`.
- Texto primario `#0d0d0d`, secundario `#5d5d5d` y separadores `#0000001a`.
- Tipografía del sistema. Titulares amplios solo en la entrada; el trabajo diario usa una escala compacta.
- Filas, tablas y divisores para datos operativos. Las tarjetas se reservan para grupos que de verdad necesitan un contenedor.
- Controles de aproximadamente 10 px de radio. Estado de foco visible y movimiento reducido cuando el sistema lo pide.
- El color aparece solo para distinguir estados de operación y errores, junto a texto explícito.
- Sin sombras ornamentales, gradientes, imágenes de relleno ni cuadrículas de tarjetas.

La navegación se divide en servidores en la barra lateral y secciones del servidor en pestañas, con el estado y la acción principal sobre el contenido. La barra lateral admite un modo reducido que conserva las imágenes, el estado y las acciones. La preferencia se recuerda localmente. Al pasar el cursor o enfocar una imagen se muestra el nombre y el contexto del servidor; el botón tiene un nombre accesible para teclado y lectores de pantalla.

Las filas de servidor muestran una miniatura cuadrada a la izquierda, nombre, tipo y versión; el punto de estado se superpone a la esquina. Los formularios de creación e importación ofrecen una vista previa de la imagen elegida. Los selectores y los desplegables usan flechas de trazo fino coherentes; los campos numéricos evitan las flechas nativas. Los botones tienen una altura mínima de 40 px y el texto operativo se mantiene legible sin depender de etiquetas minúsculas.

Los controles de capacidad combinan un número editable y un deslizador con su escala visible. RAM inicial y máxima mantienen un orden válido; jugadores y objetivo de espacio se pueden ajustar también en Configuración. La barra de espacio del Resumen muestra el uso medido respecto al objetivo de aviso.

La creación recorre cinco pasos visibles, con avance controlado, retorno a pasos completados, resumen persistente y revisión final. Configuración agrupa las propiedades por identidad y conexión, juego y acceso; cada ajuste incluye una explicación breve. Eliminar un servidor requiere una confirmación que diferencia los archivos administrados de las carpetas externas importadas.

El Resumen utiliza una única superficie oscura para agrupar estado y métricas reales. El resto se organiza en filas y separadores: recursos, conexión, actividad y copias. Jugadores usa una banda de cifras destacadas; Configuración da espacio a la imagen elegida; la Consola distingue hora, nivel y mensaje con búsqueda y filtro. Estas superficies sirven a tareas concretas y mantienen la paleta sobria de la referencia.

Mundos usa filas con la imagen real del mundo cuando existe `icon.png`, nombre, tamaño y estado principal. Las acciones de archivos están junto al mundo al que afectan. La importación se prepara en un formulario compacto; cambiar o borrar pide confirmación y explica la copia previa.

Las transiciones de entrada y los cambios de estado duran entre 130 y 180 ms. El indicador pulsa solo durante instalación, inicio o parada. Con movimiento reducido se desactivan las animaciones. No se usan animaciones permanentes para decorar la interfaz.

La ventana y sus dos columnas no permiten desplazamiento horizontal. La lista lateral y el contenido principal conservan su desplazamiento vertical independiente.
