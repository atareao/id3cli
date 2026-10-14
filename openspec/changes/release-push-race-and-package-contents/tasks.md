# Tasks

## 1. RED: fijar el contrato nuevo en los tests

El cambio tiene dos contratos independientes. El primero sustituye los dos `git push` consecutivos del paso `Commit, tag and push` por un único `git push --atomic`, que convierte el fallo parcial medido hoy (la rama sube, el tag se rechaza con `missing necessary objects`) en fallo total. El segundo limita el paquete publicado con una lista blanca `include`, porque `cargo package --list` publica hoy 54 ficheros, incluido todo el andamiaje interno del repositorio. Este grupo mide el estado rojo de ambos contratos antes de tocar ningún fichero.

- [ ] 1.1 Añadir a `src/tests.rs` el test unitario `test_manifiesto_include_definido`, que lee `Cargo.toml` con `include_str!("../Cargo.toml")` y comprueba que `[package]` declara `include` con exactamente los siete patrones acordados: `src/**`, `tests/**`, `Cargo.toml`, `Cargo.lock`, `README.md`, `LICENSE` y `CHANGELOG.md`.
      Verificación: `cargo test test_manifiesto_include_definido` MUST fallar hoy, porque el manifiesto no declara `include`, y al pasar MUST fijar la lista exacta y su orden.
- [ ] 1.2 Medir el RED del paquete con la declaración actual del manifiesto.
      Verificación: `cargo package --list` MUST mostrar 54 ficheros y MUST incluir el andamiaje interno (`openspec/…`, `.github/workflows/…`, `AGENTS.md`).
- [ ] 1.3 Medir el RED de la atomicidad con refs desechables (una rama y un tag de prueba que se borran después del experimento).
      Verificación: el mismo push de una rama non-FF junto con un tag nuevo, sin `--atomic`, MUST crear el tag y rechazar solo la rama; el mismo push con `--atomic`, MUST NOT crear ninguno de los dos. Dejar escrito el resultado de los dos casos como evidencia del contrato.
- [ ] 1.4 Ejecutar la suite completa para fijar el contrato rojo.
      Verificación: `cargo test --no-fail-fast` MUST fallar exactamente en el test nuevo y MUST NOT fallar ninguno de los 219 tests existentes.

## 2. GREEN: implementar

Se implementa lo mínimo para satisfacer los contratos: la lista blanca en el manifiesto y el push atómico en el pipeline. No se añade ninguna dependencia ni se toca el contrato del lock, que pertenece a `crate-metadata`.

- [ ] 2.1 Declarar `include` en `[package]` de `Cargo.toml` con la lista blanca: `src/**`, `tests/**`, `Cargo.toml`, `Cargo.lock`, `README.md`, `LICENSE` y `CHANGELOG.md`.
      Verificación: `cargo test test_manifiesto_include_definido` MUST pasar.
- [ ] 2.2 Reescribir en `.github/workflows/release-prepare.yml` el paso `Commit, tag and push`: sustituir los dos `git push` seguidos por un único `git push --atomic "$REMOTE" main "v${V}"` con las dos referencias en el mismo comando, hasta tres intentos separados cinco segundos, y `::error::` más salida distinta de cero si no lo consigue, sin llegar a ejecutar `gh workflow run`.
      Verificación: el diff MUST dejar un solo `git push` en el paso y MUST NOT quedar ningún push del tag separado del de la rama.
- [ ] 2.3 Ejecutar la suite completa tras la implementación.
      Verificación: `cargo test --no-fail-fast` en verde con 220 tests y 0 fallos.
- [ ] 2.4 Medir el efecto de la lista blanca sobre el paquete.
      Verificación: `cargo package --list` MUST NOT mostrar el andamiaje, MUST seguir mostrando `src/**`, `tests/**` con sus fixtures, `README.md`, `LICENSE`, `CHANGELOG.md` y `Cargo.lock`, y el recuento MUST bajar respecto a los 54 ficheros del RED.

## 3. REFACTOR y documentación

- [ ] 3.1 Documentar en `RELEASE.md` el procedimiento de recuperación de un release a medias y las dos trampas: no reejecutar el job fallido, porque el bump se recalcula desde el estado actual (`vampus upgrade --patch` desde 0.5.1 daría 0.5.2); y no lanzar el dispatch si el tag ya disparó la publicación, porque un tag empujado por una persona sí dispara `release.yml` (run 38527751799, `event: push`) mientras que los que empuja el pipeline con `GH_PAT` no (los cuatro últimos releases son `event: workflow_dispatch`). Añadir además qué viaja en el paquete publicado.
      Verificación: `RELEASE.md` describe la recuperación manual (tag sobre el commit y `git push origin origin/main:development`), las dos trampas y el contenido del paquete.
- [ ] 3.2 Documentar en `AGENTS.md` la convención del push atómico del pipeline y que el paquete publicado no lleva el andamiaje del repositorio, una línea cada una con su razón.
      Verificación: `AGENTS.md` contiene ambas convenciones, cada una con la razón en la misma línea.
- [ ] 3.3 Actualizar `README.md`: si describe la instalación o el paquete, añadir qué contiene el paquete publicado.
      Verificación: `README.md` describe el contenido del paquete publicado (código, tests con fixtures y documentación pública; sin el andamiaje interno del repositorio).
- [ ] 3.4 Comprobar el formato.
      Verificación: `cargo fmt --check` sin diferencias.
- [ ] 3.5 Comprobar el linter en todos los targets y features.
      Verificación: `cargo clippy --all-targets --all-features -- -D warnings` sin avisos.

## 4. Cierre

- [ ] 4.1 Ejecutar la suite completa.
      Verificación: 220 tests, 0 fallos y sin warnings del compilador.
- [ ] 4.2 Validar el cambio contra OpenSpec.
      Verificación: `openspec validate release-push-race-and-package-contents --strict` sin errores.
- [ ] 4.3 Empaquetar y verificar de extremo a extremo.
      Verificación: `cargo package --list` final y `cargo publish --dry-run` MUST empaquetar, MUST verificar la compilación y MUST conservar `keywords` y `categories`.
- [ ] 4.4 Comprobar que el YAML es válido y que no quedan push sueltos.
      Verificación: el fichero MUST parsearse como YAML válido y dentro del paso MUST NOT quedar ningún `git push` del tag separado del de la rama.
- [ ] 4.5 Verificar el estado limpio ante un fallo de push atómico, repitiendo el experimento con el comando exacto escrito en el YAML sobre refs desechables.
      Verificación: ninguna referencia MUST cambiar cuando una de las dos es rechazada.
- [ ] 4.6 Archivar el cambio, con la aprobación previa del usuario.
      Verificación: `openspec archive release-push-race-and-package-contents` listo para ejecutar; requiere la aprobación previa del usuario.

## Seguimiento (fuera del cierre de este cambio)

Aquí NO se finge trabajo hecho: son comprobaciones posteriores a la próxima release real, no tareas de este cambio.

- Que el próximo release real use el push atómico y complete sus tres pasos (push, tag y publicación) sin intervención manual.
- Que, tras la próxima publicación, el `.crate` descargado de crates.io no contenga el andamiaje del repositorio.
