# Sistema visual de TCHLauncher

La referencia completa aportada por el usuario está en [design.md](design.md). Este documento registra su aplicación al producto.

- Superficie principal blanca y barra lateral `#f9f9f9`.
- Texto primario `#0d0d0d`, secundario `#5d5d5d` y separadores `#0000001a`.
- Tipografía del sistema. Escala compacta, con títulos principales de hasta 24 px.
- Filas, tablas y divisores para datos operativos. Las tarjetas se reservan para grupos que de verdad necesitan un contenedor.
- Controles de aproximadamente 10 px de radio. Estado de foco visible y movimiento reducido cuando el sistema lo pide.
- El color aparece solo para distinguir estados de operación y errores, junto a texto explícito.
- Sin sombras ornamentales, gradientes, imágenes de relleno ni cuadrículas de tarjetas.

La navegación se divide en servidores en la barra lateral y secciones del servidor en pestañas, con el estado y la acción principal sobre el contenido.

Las filas de servidor muestran una miniatura cuadrada a la izquierda, nombre, tipo y versión; el punto de estado se superpone a la esquina. Los formularios de creación e importación ofrecen una vista previa de la imagen elegida. Los selectores y los desplegables usan flechas de trazo fino coherentes; los campos numéricos evitan las flechas nativas. Los botones tienen una altura mínima de 40 px y el texto operativo se mantiene legible sin depender de etiquetas minúsculas.

Los controles de capacidad combinan un número editable y un deslizador con su escala visible. RAM inicial y máxima mantienen un orden válido; jugadores y objetivo de espacio se pueden ajustar también en Configuración. La barra de espacio del Resumen muestra el uso medido respecto al objetivo de aviso.
