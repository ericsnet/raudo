# Método de medición

Todas las cifras del proyecto salen de mediciones reales con el mismo procedimiento.

## Entorno

- Windows 11 Pro build 26200, 64 bits.
- Rust 1.98.1 (x86_64-pc-windows-msvc).
- Binario compilado en modo release (`lto`, `codegen-units=1`).

## Arranque

`raudo-bench startup <raudo.exe> <carpeta>` lanza el binario con `RAUDO_TRACE=1`.
La aplicación escribe `first_frame_ms` en un archivo en cuanto la ventana está visible;
el valor se mide dentro del propio proceso, desde `main` hasta ese instante.

## Listado de carpetas

`raudo-bench list <carpeta> <pasadas>` hace una pasada de calentamiento y luego N
pasadas cronometradas con `std::time::Instant`. Se reporta mínimo, mediana y máximo.

Carpetas usadas:

- `C:\Windows\System32` — 5.033 entradas.
- `C:\Windows\WinSxS` — 24.896 entradas.

## Memoria

Tras el arranque, el banco espera dos segundos y lee el working set del proceso.
Para comparar con Files se usa el mismo criterio.

## Comparación con otros exploradores

- **Files 4.1.1**: se cierra, se relanza y se cronometra hasta que aparece su ventana
  principal; la memoria se lee tres segundos después.
- **Explorador de Windows**: no es una aplicación independiente. Vive dentro del proceso
  del shell (`explorer.exe`) junto con la barra de tareas, el menú Inicio y el escritorio.
  Su memoria incluye todo eso y su arranque y listado no se pueden cronometrar con el mismo
  método, así que se publica solo como contexto y se marca como no comparable.

## Limitaciones

- Una sola máquina y una sola sesión.
- Los tiempos de arranque varían con el estado del disco y el antivirus.
- `first_frame_ms` mide ventana visible, no listado completo: el listado llega después,
  por diseño.
