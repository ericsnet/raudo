# Raudo

Un explorador de archivos para Windows, escrito en Rust, que abre al instante
y no se queda pensando.

Raudo lee el sistema de archivos directamente por Win32 (`FindFirstFileExW` con
large fetch) en lugar de pedirle cada carpeta al shell de Windows, y dibuja la
lista de forma virtualizada con Slint. El objetivo es el acabado visual de los
exploradores modernos sin su coste: fondo propio, iconos cuidados y menú
contextual, con arranque y navegación inmediatos.

## Estado

Fase 0: esqueleto, listado rápido y banco de pruebas. Todavía no es un
explorador usable a diario.

| Fase | Estado |
|---|---|
| 0 · Esqueleto, listado y medición | en curso |
| 1 · Pestañas, breadcrumb, columnas, iconos | pendiente |
| 2 · Abrir, renombrar, papelera, copiar/pegar | pendiente |
| 3 · Búsqueda instantánea (índice NTFS) | pendiente |
| 4 · Fondo, temas, menú contextual | pendiente |
| 5 · Instalador, winget, release | pendiente |

## Compilar

Requiere Windows 10/11, Rust stable y MSVC.

```powershell
cargo build --release
cargo run --release -p raudo-app -- C:\Windows
```

## Medir

```powershell
cargo build --release
cargo run --release -p raudo-bench -- all .\target\release\raudo.exe C:\Windows\System32
cargo run --release -p raudo-bench -- list C:\Windows\System32
```

Cada cifra que aparezca aquí sale del banco de pruebas; no hay estimaciones.

## Rendimiento (medido, Windows 11 26200, Rust 1.98.1)

| Métrica | Files 4.1.1 | Raudo 0.1.0 | Explorador de Windows |
|---|---|---|---|
| Primera ventana | 849 ms | **17 ms** | no comparable¹ |
| RAM en reposo | 384 MB | **78,9 MB** | 1.080 MB¹ |
| 5.033 entradas (System32) | - | **5,99 ms** | no medible¹ |
| 24.896 entradas (WinSxS) | - | **47,27 ms** | no medible¹ |
| Binario release | - | **10,2 MB** | - |

¹ El Explorador de Windows vive dentro del proceso del shell (`explorer.exe`), con la barra de
tareas, el menú Inicio y el escritorio. Su memoria incluye todo eso y su arranque/listado no se
puede cronometrar con el mismo método; la fila es contexto, no comparación directa.

Detalle del método y limitaciones: [docs/BENCHMARKS.md](docs/BENCHMARKS.md).
El backend de render por defecto es el software de Slint; `SLINT_BACKEND=winit-femtovg`
activa OpenGL.

## Licencia

GPL-3.0-or-later. Ver [LICENSE](LICENSE).
