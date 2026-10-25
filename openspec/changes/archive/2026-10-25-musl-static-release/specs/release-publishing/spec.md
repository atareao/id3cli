# Spec Delta

## Purpose

El artefacto precompilado de Linux depende hoy de la libc del sistema: el pipeline construye `x86_64-unknown-linux-gnu` sobre `ubuntu-latest` y el binario declara entradas `NEEDED` sobre `libc.so.6`, `libm.so.6` y `libgcc_s.so.1`, con símbolos versión hasta `GLIBC_2.34` (medido el 2026-10-25 sobre `target/release/id3cli` con `readelf -d` y `objdump -T`; el 2.34 lo fijan `__libc_start_main` y cinco símbolos `pthread_*`, etiquetados así desde que glibc 2.34 fusionó libpthread dentro de libc). El binario arranca en Ubuntu 22.04+ y Debian 12+, y NO arranca en Ubuntu 20.04 (2.31), Debian 11 (2.31), RHEL 8 (2.28) ni CentOS 7 (2.17). Además, el README documenta la descarga `releases/latest/download/id3cli-linux-x86_64`, que no existe: los assets medidos de v0.10.0 y v0.9.0 son solo `id3cli-x86_64-unknown-linux-gnu.tar.gz` y su `.sha256`. Este delta fija el artefacto estático musl, el binario desnudo descargable y la verificación en integración continua. NO toca el push atómico ni la lista blanca `include`, que siguen contratados en los requisitos ya presentes.

## ADDED Requirements

### Requirement: El binario precompilado MUST ser un ejecutable estático y MUST NOT requerir la libc del sistema

El job `build` de `.github/workflows/release.yml` MUST construir el target `x86_64-unknown-linux-musl` con `cargo build --release --locked` y MUST publicar el binario resultante, de modo que el artefacto sea un ejecutable estático: MUST NOT declarar entradas `NEEDED` en su cabecera ELF y MUST NOT referenciar símbolos versión `GLIBC_*`. El pipeline MUST verificar esa condición antes de publicar y MUST fallar, sin publicar, si el binario no es estático.

#### Scenario: Sin dependencias dinámicas

- **WHEN** se inspecciona el binario publicado con `readelf -d` y `ldd`
- **THEN** `readelf -d` MUST NOT listar ninguna entrada `NEEDED`, y `ldd` MUST responder `not a dynamic executable` o `statically linked`; la medición del 2026-10-25 sobre el artefacto gnu de `target/release/id3cli` dejó constancia del estado que se corrige: cuatro entradas `NEEDED` —`libgcc_s.so.1`, `libm.so.6`, `libc.so.6` y `ld-linux-x86-64.so.2`

#### Scenario: Sin símbolos glibc

- **WHEN** se listan los símbolos versionados del binario con `objdump -T`
- **THEN** MUST NOT aparecer ningún símbolo con versión `GLIBC_*`; la medición del 2026-10-25 sobre el artefacto gnu dio como versión más alta `GLIBC_2.34`, exigida por `__libc_start_main`, `pthread_attr_getguardsize`, `pthread_attr_getstack`, `pthread_key_create`, `pthread_key_delete` y `pthread_setspecific`

#### Scenario: Ejecutable sin la glibc del build

- **WHEN** el binario se ejecuta en un sistema sin la glibc de la máquina que lo construyó (un contenedor musl/alpine)
- **THEN** MUST arrancar y MUST producir la misma salida que en un sistema con glibc para `--help` y `show` sobre un fixture; medido el 2026-10-25 en un contenedor `alpine` (musl, sin glibc): el binario estático ejecutó `show` con código 0, mientras el artefacto gnu no llegó a arrancar (`sh: ...: not found`, le falta el cargador de glibc)

### Requirement: El release MUST publicar el binario desnudo además del archivo comprimido

Los assets de la release MUST incluir el binario desnudo con el nombre `id3cli-linux-x86_64` —ejecutable, con su bit de ejecución— además de `id3cli-x86_64-unknown-linux-musl.tar.gz` y `id3cli-x86_64-unknown-linux-musl.tar.gz.sha256`, de modo que la ruta documentada en el README MUST existir y MUST devolver el binario.

#### Scenario: Asset desnudo presente

- **WHEN** se listan los assets de la release publicada
- **THEN** MUST aparecer `id3cli-linux-x86_64` junto al `.tar.gz` y a su `.sha256`; la medición del 2026-10-25 sobre v0.10.0 y v0.9.0 dejó constancia del estado que se corrige: solo `id3cli-x86_64-unknown-linux-gnu.tar.gz` y `id3cli-x86_64-unknown-linux-gnu.tar.gz.sha256`

#### Scenario: La ruta documentada responde

- **WHEN** se descarga `https://github.com/atareao/id3cli/releases/latest/download/id3cli-linux-x86_64`
- **THEN** MUST devolver el binario con HTTP 200, de modo que el comando de instalación del README —`wget`, `chmod +x`, `mv`— MUST funcionar tal cual

#### Scenario: El archivo y su suma siguen

- **WHEN** se listan los assets
- **THEN** MUST seguir presente el `.tar.gz` con su `.sha256` correspondiente, de modo que la descarga auditable no se pierda

### Requirement: La integración continua MUST construir y verificar el binario estático antes del tag

`.github/workflows/ci.yml` MUST incorporar un job que construya `x86_64-unknown-linux-musl` con `--locked`, MUST verificar el enlazado estático del binario y MUST ejecutar la suite completa contra ese target, de modo que una pérdida del estático o una regresión del binario musl MUST fallar en la pull request y MUST NOT descubrirse al publicar el tag. El job preexistente sobre gnu MUST seguir en verde.

#### Scenario: Job musl en la puerta de entrada

- **WHEN** se abre o se actualiza una pull request
- **THEN** MUST ejecutarse un job que construya musl, verifique el estático y corra `cargo test --locked --target x86_64-unknown-linux-musl`, además del job gnu existente

#### Scenario: Falsabilidad del gate

- **WHEN** el binario deja de ser estático —medido el 2026-10-25: el gate falla sobre el artefacto gnu, con 4 entradas `NEEDED` y `GLIBC_2.34` como versión más alta, y pasa sobre el musl, con 0 y 0; forzar la pérdida del estático con `-C target-feature=-crt-static` ni siquiera enlaza, porque el target de rustup solo trae `libc.a` estática
- **THEN** la verificación MUST fallar con salida distinta de cero y MUST impedir la publicación

#### Scenario: Sin regresión en el job gnu

- **WHEN** se ejecuta la integración continua con el cambio aplicado
- **THEN** el job sobre `x86_64-unknown-linux-gnu` (fmt, clippy, build, test) MUST seguir en verde

### Requirement: La salida del binario estático MUST ser idéntica a la del binario dinámico de referencia

El binario musl y el binario gnu de la misma versión MUST producir salidas byte a byte idénticas para `show` (legible) y `show --json` sobre los mismos fixtures, de modo que el cambio de enlace MUST NOT alterar ningún contrato observable del CLI.

#### Scenario: No-rotura medida sobre los cinco formatos

- **WHEN** se ejecutan `show` y `show --json` con ambos binarios sobre los fixtures mp3, m4a, ogg, opus y flac
- **THEN** la salida de los dos MUST ser byte a byte idéntica en los cinco formatos

#### Scenario: La matriz e2e MUST pasar con el binario estático

- **WHEN** se ejecuta la matriz e2e —`scripts/e2e/*.sh --bin <binario musl>`— contra el binario estático
- **THEN** MUST pasar con el mismo recuento de aserciones que contra el binario gnu
