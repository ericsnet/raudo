# Decisiones

Registro corto de las decisiones que condicionan el proyecto y por qué se
tomaron. Se añade una entrada cuando cambia algo estructural.

## 2026-11-22 · Nacimiento

- **Nombre: Raudo.** Corto, en español y sin conflicto en GitHub (49 repos con
  el nombre, ninguno de un explorador de archivos).
- **Problema:** Files 4.1.1, el explorador alternativo de referencia en
  Windows, tarda mucho en abrir y cargar carpetas. Medido en reposo: 585 MB de
  RAM. El acabado visual sí se quiere conservar.
- **Alcance inicial:** navegación intensiva, apertura de archivos y búsqueda
  frecuente. No es un gestor de sincronización ni un cliente de nube.

## UI: Slint 1.19

- Se elige Slint por render GPU, listas virtualizadas nativas y control fino
  del dibujo (fondo, filas, iconos) con menos esfuerzo que dibujar Win32 a mano
  y más acabado que egui.
- Consecuencia: al enlazar Slint, la aplicación se distribuye como
  **GPL-3.0-or-later**. Para un explorador de escritorio abierto es aceptable.
- Alternativa descartada: egui (MIT, más rápido de escribir, peor acabado) y
  Tauri (mejor acabado por hora, pero webview y arranque más pesado justo en lo
  que se quiere ganar).

## Rendimiento: no pasar por el shell

- Listado con `FindFirstFileExW` + `FindExInfoBasic` +
  `FIND_FIRST_EX_LARGE_FETCH`. Nada de `IShellFolder` en la ruta caliente.
- La lectura de carpeta ocurre en un hilo y se entrega a la UI por el event
  loop; la ventana se muestra antes de terminar el listado.
- Lista virtualizada: la UI solo instancia las filas visibles.
- Iconos y miniaturas: se extraen en segundo plano, con concurrencia limitada
  y caché en disco (fase 1 en adelante).

## Búsqueda: índice NTFS con elevación

- Índice construido desde el MFT (`FSCTL_ENUM_USN_DATA`) en un proceso
  elevado, persistido en disco y cargado por mmap. Mantenimiento incremental
  con el USN journal.
- Se pide UAC una vez; sin permisos, plan B es un recorrido paralelo, más lento
  pero con la misma UI.
- Descartado por ahora: servicio de Windows permanente. Añade instalación y
  superficie de ataque; se estudia si el producto lo necesita.

## Renderer: software por defecto

Medido con el mismo binario: `winit-femtovg` (OpenGL) 205–294 ms hasta la primera
ventana y 133–202 MB de RAM; `winit-software` 17 ms y 80 MB. Para una UI de texto
y filas virtualizadas el software renderer es suficiente y la diferencia es
demasiado grande como para ignorarla. Se fija por defecto en `raudo-app` (solo si
`SLINT_BACKEND` no está definido). Si la Fase 4 necesita GPU para blur, sombras o
animaciones, se cambia con mediciones delante.

## Entorno de compilación

- `CARGO_TARGET_DIR=E:\cargo-target\raudo`. C: tiene poco espacio libre y el
  árbol de artefactos no debe crecer dentro de OneDrive.
- Artefactos de medición e informes en `artifacts/`, fuera de git.
