# Spec Delta

## Purpose

El pipeline de release empuja `main` y el tag en dos `git push` consecutivos dentro del mismo paso, lo que abre una ventana de carrera entre el commit de release y su referencia: el 2026-10-14 el run `Prepare Release` 38527464056 aceptó `main` (`8fb0583..5a070b6`) y rechazó el tag con `! [remote rejected] v0.5.1 -> v0.5.1 (missing necessary objects)`, dejando la versión ya subida sin tag y `release.yml` sin disparar. En paralelo, `cargo package --list` publica 54 ficheros, incluido todo el andamiaje interno del repositorio (`.github/**`, `openspec/**`, `AGENTS.md`). Este delta fija dos contratos nuevos: el envío atómico con reintentos acotados y el contenido publicable del paquete mediante una lista blanca `include`.

Nota razonada: el `--locked` de la integración continua y el refresco de `Cargo.lock` antes del commit de release ya están contratados en la capability `crate-metadata` y NO se mueven aquí; esta capability cubre el push y el contenido del paquete, que son contratos nuevos e independientes del lock. El solapamiento es deliberado: `release-publishing` consume el estado que `crate-metadata` garantiza, sin reabrirlo.

## ADDED Requirements

### Requirement: El pipeline de release MUST empujar la rama y el tag en una única operación atómica

El paso `Commit, tag and push` de `.github/workflows/release-prepare.yml` MUST actualizar `main` y el tag `v${V}` con una sola invocación de `git push --atomic` que incluya las dos referencias, de modo que el objeto del commit de release y la referencia del tag viajen en el mismo pack. MUST NOT existir ventana entre el envío de la rama y el del tag, y el contrato MUST NOT admitir el estado parcial en que `main` queda actualizado y el tag no existe. Si el envío no lo consigue tras reintentos acotados, el pipeline MUST fallar con error explícito y MUST NOT disparar la publicación.

#### Scenario: Un único push que lleva las dos referencias

- **WHEN** el pipeline de release empuja la rama `main` y el tag `v${V}` recién creados
- **THEN** MUST hacerlo con una sola invocación de `git push --atomic` que incluya las dos referencias, de modo que el commit de release y la referencia del tag viajen en el mismo pack y MUST NOT existir ventana entre ambos; la medición del 2026-10-14 dejó constancia del fallo del montaje anterior: dos `git push` separados por 0,49 s en el run 38527464056, frente a 1,1 s y 1,3 s en los dos runs que funcionaron (release 0.5.0 y v0.4.3), con comando idéntico

#### Scenario: Fallo total en vez de parcial

- **WHEN** una de las dos referencias del push atómico es rechazada
- **THEN** ninguna MUST actualizarse y el servidor MUST rechazar las dos; el experimento del 2026-10-14 contra este remoto con refs desechables MUST reproducir ese contrato: el push atómico de una rama non-FF junto con un tag nuevo devolvió `! [rejected] 5ba4e85 -> prueba/atomic-ff (non-fast-forward)` y `! [rejected] prueba-atomic-ff -> prueba-atomic-ff (atomic push failed)`, el tag NO se creó y la rama quedó intacta, mientras que el mismo push sin `--atomic` sí creó el tag y solo rechazó la rama. El contrato MUST NOT admitir el estado en que `main` queda actualizado y el tag no existe

#### Scenario: Reintentos acotados

- **WHEN** el push atómico falla
- **THEN** el pipeline MUST reintentarlo hasta tres intentos separados cinco segundos antes de darse por vencido

#### Scenario: Estado limpio ante el fallo

- **WHEN** el pipeline agota los tres intentos sin empujar
- **THEN** MUST terminar con un `::error::` y salida distinta de cero, MUST NOT ejecutar `gh workflow run`, y la rama `main` remota MUST quedar sin el commit de release y sin el tag

#### Scenario: Sin doble bump

- **WHEN** un envío falla y hay que recuperar el release
- **THEN** la recuperación MUST ser volver a empujar a `main` desde el mismo estado, y el pipeline MUST NOT necesitar ni admitir como recuperación la reejecución del job fallido, que recalcularía la versión desde el estado actual (`vampus upgrade --patch` desde 0.5.1 daría 0.5.2)

### Requirement: El paquete publicado MUST limitarse a los ficheros publicables del proyecto

`Cargo.toml` MUST declarar en `[package]` una clave `include` con la lista blanca de los ficheros publicables: `src/**`, `tests/**`, `Cargo.toml`, `Cargo.lock`, `README.md`, `LICENSE` y `CHANGELOG.md`. El andamiaje interno del repositorio MUST NOT viajar en el paquete, y el paquete MUST seguir siendo verificable con `cargo publish --dry-run` conservando la metadata ya declarada.

#### Scenario: Lista blanca declarada

- **WHEN** se lee `Cargo.toml`
- **THEN** `[package]` MUST declarar una clave `include` con la lista blanca de patrones `src/**`, `tests/**`, `Cargo.toml`, `Cargo.lock`, `README.md`, `LICENSE` y `CHANGELOG.md`

#### Scenario: El andamiaje MUST NOT viajar

- **WHEN** se ejecuta `cargo package --list` con la lista blanca declarada
- **THEN** `openspec/`, `.github/`, `AGENTS.md`, `GIT_FLOW.md`, `RELEASE.md`, `cliff.toml`, `.justfile` y `.vampus.yml` MUST NOT aparecer en la salida; hoy, sin `include`, la salida muestra 54 ficheros e incluye todo ese andamiaje

#### Scenario: Lo necesario MUST seguir

- **WHEN** se ejecuta `cargo package --list` con la lista blanca declarada
- **THEN** MUST seguir presente `src/**`, `tests/**` con sus fixtures (`sample.mp3`, `sample.m4a` y `generate_sample_m4a.sh`, 9.031 bytes en total: 2.481, 3.295 y 3.255), `README.md`, `LICENSE`, `CHANGELOG.md` y `Cargo.lock`, de modo que el paquete publicado conserve su suite ejecutable; `Cargo.lock` MUST viajar porque el paquete publica un binario y la compilación debe ser reproducible

#### Scenario: El paquete MUST seguir verificándose

- **WHEN** se ejecuta `cargo publish --dry-run` con la lista blanca declarada
- **THEN** MUST empaquetar y MUST verificar la compilación del paquete, y MUST NOT perder las `keywords` ni las `categories` ya declaradas en el manifiesto
