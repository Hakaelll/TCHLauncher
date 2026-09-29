# Seguridad

- Los comandos Rust validan nombre, versión, memoria, puerto y propiedades.
- Las descargas solo aceptan HTTPS y usan archivos `.part`; los hashes publicados se verifican.
- La extracción de Java rechaza rutas fuera del destino y enlaces simbólicos ZIP. Las copias no siguen enlaces simbólicos.
- La EULA solo se escribe tras una acción explícita.
- Los servidores se ejecutan como usuario normal, sin privilegios elevados.
- Las imágenes de servidor se limitan a 2 MB, se comprueba su firma PNG, JPG o WebP y se guardan bajo los datos locales de la aplicación.

Pendiente antes de distribuir: auditoría de redirecciones y hosts de descarga, cancelación de descargas, política de cierre de procesos, y prueba de restauración ante fallos de disco.
