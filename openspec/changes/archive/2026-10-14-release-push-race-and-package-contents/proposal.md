# Proposal

## Why

Este cambio agrupa **dos problemas independientes** que comparten un mismo cierre, la publicación. No comparten causa ni código —uno es una carrera de red en el pipeline de release y el otro es la lista de ficheros que viaja al paquete publicado—, pero sí una misma rama, una misma capability nueva y un mismo momento de verificación.

**1. La carrera entre el push de la rama y el del tag deja releases a medias.** El run `Prepare Release` **38527464056**, disparado por el merge del PR #15 a `main`, falló en el paso `Commit, tag and push`. El push de la rama fue aceptado por el servidor (`8fb0583..5a070b6 main -> main`) y el del tag fue rechazado acto seguido:

```
! [remote rejected] v0.5.1 -> v0.5.1 (missing necessary objects)
```

El paso hace **dos `git push` consecutivos contra la misma URL dentro del mismo paso**. El primero transporta los objetos del commit recién creado; el segundo —el del tag— no transporta ningún objeto nuevo, porque el cliente ya cree que el servidor los tiene, y la réplica del servidor que valida la referencia todavía no había ingerido el pack del push anterior. Es una ventana de consistencia eventual, no un permiso mal puesto.

La prueba de que es una carrera y no una configuración rota es la separación temporal medida entre los dos push: **0,49 s** en el run que falló, frente a **1,1 s** (run 38224385769) y **1,3 s** (run 38517244065, release 0.5.0) en los dos que sí funcionaron. El comando era idéntico en los tres casos; lo único que cambió fue la latencia. El fallo es, por tanto, dependiente del tiempo y no reproducible a voluntad.

El estado que deja ese fallo es exactamente el que hay que evitar: `main` ya tiene la versión **0.5.1** subida y **sin tag**, de modo que `release.yml` nunca se dispara, y el paso `Sync development with main` —que va después del que falla— queda **salteado**. Hubo que recuperar a mano: crear el tag sobre `5a070b6` y empujar `origin/main:development`.

De esa recuperación manual salen dos trampas medidas que el arreglo debe respetar:

- **Reejecutar el job fallido no es una recuperación válida.** El bump se recalcula desde el estado actual, así que un `vampus upgrade --patch` sobre un árbol ya en `0.5.1` daría `0.5.2`: un rerun habría producido una **release fantasma**. Por eso el arreglo no puede apoyarse en reintentar el job entero, sino en dejar el estado limpio para el siguiente push.
- **Un tag empujado por una persona SÍ dispara `release.yml`, pero uno empujado por el propio pipeline con su `GH_PAT` NO.** Los cuatro últimos releases —`v0.4.1`, `v0.4.2`, `v0.4.3` y `v0.5.0`— figuran todos con `event: workflow_dispatch`, mientras que el tag que empujó una persona para recuperar la 0.5.1 disparó el run 38527751799 con `event: push`. Esa es la razón de que el paso lance `gh workflow run release.yml` explícitamente, y también la razón de que, si alguien empuja el tag a mano **y además** lanza el dispatch, se publique dos veces.

La solución se midió hoy contra este mismo remoto con referencias desechables (rama `prueba/atomic-ff` y tags `prueba-atomic-*`, ya borrados; quedan **0** refs de prueba en el remoto): `git push --atomic` con las dos referencias en el **mismo** comando. El experimento real fue un push atómico de una rama no fast-forward junto con un tag nuevo, y produjo:

```
! [rejected]      5ba4e85 -> prueba/atomic-ff (non-fast-forward)
! [rejected]      prueba-atomic-ff -> prueba-atomic-ff (atomic push failed)
```

El tag **no** se creó y la rama quedó intacta en `5a070b6`. El control con el **mismo** push pero **sin** `--atomic` creó el tag (`* [new tag] prueba-atomic-ctrl`) y solo rechazó la rama. Conclusión medida: `--atomic` convierte el fallo parcial en fallo total, y GitHub soporta la operación contra este remoto.

**2. El paquete publicado lleva todo el andamiaje del repositorio.** Medido hoy con `cargo package --list`, el paquete que se publicaría lleva **54 ficheros** (y **59** con los 5 ficheros de este propio change ya en el árbol), entre ellos la infraestructura interna que no tiene ningún valor para quien consume la crate: los tres workflows de `.github/workflows/*`, `.github/copilot-instructions.md`, todo `openspec/**` (configuración, specs y los changes archivados), `AGENTS.md`, `GIT_FLOW.md`, `RELEASE.md`, `cliff.toml`, `.justfile`, `.vampus.yml` y `.gitignore`. Cargo no lee una lista negra por defecto que los excluya, y el resultado es un paquete inflado con material ajeno al artefacto.

La primera lista blanca que se probó —sin anclar los patrones a la raíz— tampoco cerraba el contrato: `include` usa la semántica de gitignore, de modo que un patrón sin barra inicial casa a **cualquier profundidad**. Medido con `cargo package --list`, el paquete bajó a **16** entradas, pero entre ellas seguía colándose un fichero ajeno del andamiaje: `openspec/changes/archive/2026-10-14-remove-empty-id3-header/README.md`, arrastrado por el patrón `README.md`. La lista anclada —con barra inicial en cada patrón— deja el paquete en **15** entradas y **cero** ficheros del andamiaje. El anclaje no es un detalle de estilo: es parte del contrato «el andamiaje MUST NOT viajar», y solo se descubrió midiendo el paquete, no razonando sobre él.

## What Changes

El contrato de este cambio tiene dos puntos, uno por cada problema:

- **Rama y tag en una única operación atómica.** El paso que cierra la release MUST empujar `main` y el tag en un **único** `git push` con `--atomic`, de modo que un rechazo de cualquiera de las dos referencias MUST abortar la operación completa. El push MUST reintentarse un número acotado de veces con una espera entre intentos, y si no lo consigue MUST fallar de forma explícita —mensaje de error y salida distinta de cero— **antes** de disparar la publicación. Como consecuencia de la atomicidad, un fallo MUST dejar el repositorio **sin commit de release, sin tag y sin publicación**, de modo que el siguiente push a `main` reintente la release desde el mismo estado, sin doble bump.
- **El paquete publicado se limita a los ficheros publicables.** `Cargo.toml` MUST declarar en `[package]` una lista blanca `include` que contenga `/src/**`, `/tests/**`, `/Cargo.toml`, `/Cargo.lock`, `/README.md`, `/LICENSE` y `/CHANGELOG.md`, con los patrones de primer nivel **anclados a la raíz**, y MUST NOT incluya el andamiaje del repositorio (workflows, `openspec/**`, `AGENTS.md`, `GIT_FLOW.md`, `RELEASE.md`, `cliff.toml`, `.justfile`, `.vampus.yml`, `.gitignore`). El anclaje es parte del contrato: sin la barra inicial, `include` usa la semántica de gitignore y un patrón como `README.md` casa a cualquier profundidad, de modo que el paquete arrastraba `openspec/changes/archive/2026-10-14-remove-empty-id3-header/README.md` y el contrato «el andamiaje MUST NOT viajar» no se cumplía. `Cargo.lock` MUST seguir viajando porque el paquete publica un binario y la compilación debe ser reproducible con las versiones del lock. La suite de tests y sus fixtures MUST conservarse dentro del paquete: pesan 9.031 bytes en total (`sample.mp3` 2.481, `sample.m4a` 3.295, `generate_sample_m4a.sh` 3.255) y a cambio el paquete publicado conserva una forma de comprobarse.

**Consecuencia observable:** el paquete publicado pasa de **54** ficheros (59 con los de este change) a **15** entradas con la lista blanca anclada —13 del repositorio (los 4 de `src/`, `tests/integration_test.rs` y los 3 de `tests/fixtures/`, `README.md`, `LICENSE`, `CHANGELOG.md`, `Cargo.toml` y `Cargo.lock`) más 2 que genera cargo (`.cargo_vcs_info.json` y `Cargo.toml.orig`)— con **cero** ficheros del andamiaje, y el pipeline deja de poder terminar con `main` actualizado y el tag ausente. **El comportamiento del binario no cambia en absoluto**: ni los flags, ni los mensajes, ni los códigos de salida; ni los tags de MP3 o MP4; ni la detección de formato. Este change no publica nada: deja el pipeline y el manifiesto listos para la **próxima** release.

## Capabilities

### New Capabilities

- `release-publishing`: contrato del cierre de la release. Fija que el pipeline empuja rama y tag en una única operación atómica, con reintentos acotados y fallo explícito, de modo que un fallo deja el repositorio sin commit de release, sin tag y sin publicación; y que el paquete publicado se limita a la lista blanca de ficheros publicables declarada en `Cargo.toml`, excluyendo el andamiaje del repositorio. El delta se materializará en `specs/release-publishing/spec.md`.

### Modified Capabilities

Ninguna. Las cinco capacidades existentes —`cli-output`, `crate-metadata`, `format-detection`, `mp3-tagging` y `m4a-tagging`— quedan intactas: este cambio no toca la escritura de la salida estándar, ni los metadatos del manifiesto destinados a crates.io, ni la detección de formato, ni la edición de MP3 ni la de MP4. Los dos problemas que se atacan aquí —la carrera del push y el contenido del paquete— no están contratados por ninguna de ellas, sino que se caracterizan por primera vez en una capacidad nueva. En particular, los requisitos del pipeline que ya viven dentro de `crate-metadata` —el refresco del lock antes del commit de release y el `--locked` de la integración continua— **no se mueven** a `release-publishing`; el razonamiento está en `design.md`.

## Impact

- `.github/workflows/release-prepare.yml`: el paso `Commit, tag and push` deja de hacer dos `git push` separados y pasa a un único `git push --atomic` con la rama y el tag, con hasta tres intentos separados por una espera y un fallo explícito (`::error::` y salida distinta de cero) antes del `gh workflow run`.
- `Cargo.toml`: se declara la lista blanca `include` en el bloque `[package]`, con los patrones de primer nivel anclados a la raíz (`/src/**`, `/tests/**`, `/Cargo.toml`, `/Cargo.lock`, `/README.md`, `/LICENSE`, `/CHANGELOG.md`).
- `src/tests.rs`: se añade **un** test unitario del manifiesto que lee `Cargo.toml` con `include_str!` y fija la declaración de `include`, con el mismo patrón que `test_manifiesto_keywords_validos` y `test_manifiesto_categories_validas`.
- `RELEASE.md`: se documenta el procedimiento de recuperación ante un release a medias y el contenido del paquete publicado.
- `AGENTS.md`: se documentan la convención del push atómico en el pipeline y la del contenido publicado del paquete.
- `README.md`: se documenta qué lleva el paquete publicado.
- `openspec/specs/release-publishing/spec.md`: **archivo nuevo**, al archivar el change.
- **Sin dependencias nuevas.** Ni de producción ni de desarrollo: el arreglo del pipeline es de shell y el del paquete es una declaración del manifiesto, así que no se toca `Cargo.lock` más allá de lo que ya hará el propio pipeline.
