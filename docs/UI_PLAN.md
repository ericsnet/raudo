# Plan de UI — Raudo

Documento de trabajo. La UI se construye contra esto, no a base de parches.

## Diagnóstico (2026-11-22)

Bugs y decisiones flojas detectadas en la primera versión con estructura de explorador:

1. **Ventana clavada a 1280×800.** `width`/`height` fijos en el `.slint`: al maximizar,
   la superficie de dibujo no crecía y quedaba un marco vacío. Corregido: tamaño inicial
   desde Rust (`Window::set_size`) y la ventana sigue al usuario.
2. **Sin pantalla completa.** Slint expone `Window::set_fullscreen`; faltaba la acción.
   Añadido botón `⛶` y se expondrá F11.
3. **Breadcrumb estirado.** El layout repartía el ancho sobrante entre migas y separadores.
   Corregido con `horizontal-stretch: 0` en cada pieza.
4. **Cabecera "Nombre" descentrada.** El texto se centraba en la columna en vez de
   alinearse a la izquierda.
5. **Portada con huecos enormes.** Los `Text` se estiraban verticalmente. Cada bloque
   tiene altura explícita y un espaciador absorbe el resto.
6. **Panel lateral sin iconos.** Solo texto: no se reconoce de un vistazo.
7. **Sin iconos de archivo/carpeta.** Diferencia visual más grande contra el Explorador.
8. **Sin fondo.** Soportado por archivo (`%APPDATA%\Raudo\background.png` o
   `RAUDO_BACKGROUND`), pero no hay imagen por defecto ni ajustes.

## Principios

- Rejilla de 4 px para todo el espaciado. Nada de valores sueltos.
- Un solo acento (`#5b9dff`). El resto es escala de grises azulada.
- Cada estado tiene color propio: normal, hover, seleccionado, activo, deshabilitado.
- Nada decorativo que no comunique: sin sombras grandes, sin degradados, sin emojis.
- La lista manda: el resto de la ventana no compite con ella.

## Tokens

| Token | Valor |
|---|---|
| Fondo base | `#131519` |
| Panel | `#191b1f` |
| Barra / cabecera | `#1b1e22` |
| Tarjeta | `#23272d` |
| Borde sutil | `#2b313a` |
| Texto principal | `#e9edf2` |
| Texto secundario | `#9aa1a9` |
| Texto terciario | `#767d85` |
| Acento | `#5b9dff` |
| Peligro (disco lleno) | `#e06c6c` |

Tipografía: 24 (título de portada), 13 (cuerpo y listas), 11–12 (metadatos y cabeceras).
Radios: 6 (botones), 8 (barras y migas), 10 (tarjetas).

## Estructura

```
┌ pestañas (40) ─────────────────────────────────────────────┐
├ navegación (46): ← → ↑ ⌂ | migas | buscar | ⟳ ⛶ ───────────┤
│ panel (240)  │ contenido                                   │
│ Anclados     │  portada (unidades) o lista + columnas      │
│ Unidades     │  barra de estado (26)                       │
└──────────────┴─────────────────────────────────────────────┘
```

Tamaño mínimo: 900×560. Por debajo de 1000 px de ancho el panel se estrecha a 200;
por debajo de 800 se oculta y queda el acceso por migas.

## Iconos

- Iconos reales del shell de Windows por extensión y por carpeta, 32 px, extraídos una
  vez por tipo (`SHGetFileInfoW` con `SHGFI_USEFILEATTRIBUTES`) y cacheados en memoria.
- En disco: `%APPDATA%\Raudo\icons\` con clave `tipo-hash.png`, para que el segundo
  arranque no toque el shell.
- Fallback: carpeta y archivo dibujados como vector si la extracción falla.
- Nunca se extraen iconos por archivo en la ruta caliente: solo al pintar la fila visible.

## Estados

| Elemento | Normal | Hover | Activo / seleccionado |
|---|---|---|---|
| Fila | fondo base | `#22262c` | `#2d3a4e` |
| Elemento de panel | texto secundario | `#2a2f36` | barra de acento + texto principal |
| Pestaña | texto secundario | `#23272d` | `#2a2f36` + texto principal |
| Botón de barra | texto secundario | `#2f343b` | — |

## Teclado (fase 1b)

Flechas para moverse, Enter para abrir, Backspace para subir, F5 refrescar,
F11 pantalla completa, Ctrl+T pestaña nueva, Ctrl+W cerrar pestaña,
Ctrl+L ruta editable.

## Orden de trabajo

1. Cierre de bugs estructurales: tamaño de ventana, fullscreen, breadcrumb, cabeceras,
   portada. **(hecho)**
2. Iconos del shell con caché + iconos del panel y de las tarjetas de unidad.
3. Teclado y selección con foco visible.
4. Fondo por defecto y pantalla de ajustes mínima (tema, fondo, tamaño de fila).
5. Menú contextual propio y apertura de archivos (fase 2).
