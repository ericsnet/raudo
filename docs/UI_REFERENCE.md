# Referencia visual — Raudo

La UI de Raudo se construye imitando el Explorador **Files** que el operador usa como
referencia. Este documento fija lo que hay que reproducir. Ante la duda, se imita la
referencia; no se reinterpreta.

## Estructura de la ventana

1. **Barra de pestañas** arriba, con pestañas redondeadas, botón de cerrar y `+`.
2. **Barra de navegación**: atrás, adelante, subir, refrescar, botón de inicio, caja de
   migas con iconos por segmento, buscador a la derecha y botón de vista de lista.
3. **Barra de acciones** debajo: `+ Nuevo` y acciones de edición (cortar, copiar, pegar,
   renombrar, compartir, eliminar) a la izquierda; orden y filtros a la derecha.
4. **Panel lateral**:
   - Inicio.
   - **Anclados**: Escritorio, Descargas, Documentos, Imágenes, Música, Vídeos,
     Papelera de reciclaje y accesos propios (Proyectos, Iconos...).
   - **Unidades de disco**: cada unidad con letra y etiqueta.
   - **Unidades en la nube**: OneDrive si existe.
   - **Red** y **WSL** al final, y **Configuración** anclada abajo.
5. **Contenido**: portada con tarjetas de unidades o lista de archivos con columnas
   `Nombre`, `Fecha de modificación`, `Tipo`, `Tamaño` (y `Etiqueta`/`Estado` cuando
   existan datos).
6. **Barra de estado** abajo con el número de elementos.

## Aspecto

- Fondo ilustrado a toda la ventana, visible a través de paneles translúcidos.
- Paneles y filas oscuros, con transparencia ligera (`rgba`), esquinas redondeadas.
- Acento azul para selección y foco; texto principal claro y secundario gris.
- Filas altas (≈ 38 px) con icono del sistema, nombre y metadatos alineados.
- Sin sombras grandes ni degradados.

## Comportamiento

- Clic: selecciona. Doble clic: abre carpeta o archivo.
- Clic derecho: menú contextual (Abrir, Abrir con…, Copiar ruta, Renombrar, Eliminar,
  Nueva carpeta, Terminal, Propiedades, Refrescar).
- Maximizar y restaurar funcionan como en cualquier ventana de Windows.
