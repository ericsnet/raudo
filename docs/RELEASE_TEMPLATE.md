# Plantilla de release — Raudo

Toda release de Raudo se publica con **binario + explicaciones**. Nunca solo código.

## Adjuntos obligatorios

- `raudo.exe` — binario portable (Windows x64, compilado en release).
- `raudo.exe.sha256` — checksum SHA256 del binario.

## Cuerpo de la release

1. **Qué es**: una frase. Explorador de archivos para Windows, rápido, en Rust.
2. **Requisitos**: Windows 10/11 x64. No requiere instalación ni .NET.
3. **Cómo ejecutarlo**: descargar, comprobar el SHA256, doble clic. Se abre en la portada
   (`Inicio`); usa el panel o el buscador para moverte.
4. **Novedades de esta versión**: lista de cambios reales, agrupados por lo que nota el usuario.
5. **Limitaciones conocidas**: lo que todavía no hace (p. ej. abrir archivos con doble clic si
   no está, atajos de teclado pendientes...). Honesto, sin promesas.
6. **Licencia**: GPL-3.0-or-later, con enlace al `LICENSE` del repo.
7. **Verificación**: comando para comprobar el hash (`Get-FileHash .\raudo.exe -Algorithm SHA256`).

## Antes de publicar

- [ ] `cargo test --workspace` y `cargo clippy --workspace --all-targets -- -D warnings` limpios.
- [ ] Compilación release desde cero.
- [ ] Binario probado en una carpeta grande y maximizando la ventana.
- [ ] SHA256 generado y coincide con el binario subido.
- [ ] Versión del tag decidida por el operador.
