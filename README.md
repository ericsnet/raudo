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
$env:CARGO_TARGET_DIR = 'E:\cargo-target\raudo'   # opcional, fuera del árbol
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

## Licencia

GPL-3.0-or-later. Ver [LICENSE](LICENSE).
