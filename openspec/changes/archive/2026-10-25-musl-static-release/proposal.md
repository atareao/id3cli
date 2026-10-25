# Proposal

## Why

El binario precompilado de Linux depende de la libc del sistema. El pipeline construye `x86_64-unknown-linux-gnu` sobre `ubuntu-latest`, y el artefacto resultante declara `NEEDED` sobre `libc.so.6`, `libm.so.6` y `libgcc_s.so.1`, con símbolos versión hasta `GLIBC_2.34` (medido el 2026-10-25 sobre `target/release/id3cli` con `readelf -d` y `objdump -T`; los símbolos que fijan el 2.34 son `__libc_start_main`, `pthread_attr_getguardsize`, `pthread_attr_getstack`, `pthread_key_create`, `pthread_key_delete` y `pthread_setspecific`, etiquetados así desde que glibc 2.34 fusionó libpthread dentro de libc). El binario arranca en Ubuntu 22.04+ y Debian 12+, y NO arranca en Ubuntu 20.04 (2.31), Debian 11 (2.31), RHEL 8 (2.28) ni CentOS 7 (2.17): falla con `version 'GLIBC_2.34' not found`. En paralelo, el README documenta la descarga `.../releases/latest/download/id3cli-linux-x86_64`, que NO existe: los assets medidos de v0.10.0 y v0.9.0 son solo `id3cli-x86_64-unknown-linux-gnu.tar.gz` y su `.sha256`, de modo que el comando de instalación publicado devuelve 404.

## What Changes

El binario precompilado pasa a `x86_64-unknown-linux-musl` con enlazado estático: sin entradas `NEEDED`, sin símbolos `GLIBC_*` y ejecutable en cualquier Linux x86_64. `release.yml` cambia el target, incorpora el gate que verifica el enlazado estático antes de publicar y publica además el binario desnudo (`id3cli-linux-x86_64`) junto al `.tar.gz` y su `.sha256`, de modo que la ruta documentada en el README existe. `ci.yml` añade un job que construye musl, verifica el estático y ejecuta la suite completa contra ese target, para no descubrir la rotura al publicar el tag. Sin cambios en `src/`, `tests/` ni dependencias: `Cargo.toml` y `Cargo.lock` no cambian.

## Capabilities

### Modified Capabilities

- **release-publishing**: el artefacto publicado pasa a ser un binario estático musl (sin dependencia de la libc del sistema), el release publica el binario desnudo además del archivo comprimido, y la integración continua construye y verifica el estático antes del tag.

## Impact

- `.github/workflows/release.yml`: target `x86_64-unknown-linux-musl`, `musl-tools`, `--locked`, gate del estático, asset desnudo además del `.tar.gz` + `.sha256`.
- `.github/workflows/ci.yml`: job nuevo `musl` (build + aserción de estático + `cargo test --locked --target x86_64-unknown-linux-musl`).
- `.justfile`: recipe `build-musl`.
- `README.md`: sección Instalación (la descarga documentada pasa a existir; nota del binario estático sin glibc; alternativa `.tar.gz` + verificación `sha256`).
- `AGENTS.md`: convención del binario estático (por qué musl, el suelo glibc 2.34 medido, `--locked` en el build).
- No-goals: binarios para Windows/macOS, `aarch64`, `panic = "abort"` (optimización independiente), `cargo install` (compila en la máquina del usuario y no le afecta), publicar gnu en paralelo (sustituido), y el cierre de los cabos sueltos de higiene del repositorio (worktree huérfano, issue #8 obsoleta).
- No-rotura: la salida del binario musl MUST ser idéntica a la del gnu sobre los mismos fixtures (medición en la fase RED).
- Versionado: `feat:` → minor automático por el pipeline (v0.11.0).
