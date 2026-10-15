# Tasks

## 1. RED: fijar el contrato nuevo en los tests

El cambio tiene dos contratos independientes. El primero sustituye los dos `git push` consecutivos del paso `Commit, tag and push` por un único `git push --atomic`, que convierte el fallo parcial medido hoy (la rama sube, el tag se rechaza con `missing necessary objects`) en fallo total. El segundo limita el paquete publicado con una lista blanca `include` cuyos patrones de primer nivel van anclados a la raíz, porque `include` usa la semántica de gitignore y, medido con `cargo package --list`, una lista sin anclar dejaba viajar un fichero ajeno del andamiaje. Este grupo mide el estado rojo de ambos contratos antes de tocar ningún fichero.

- [x] 1.1 Añadir a `src/tests.rs` el test unitario `test_manifiesto_include_definido`, que lee `Cargo.toml` con `include_str!("../Cargo.toml")` y comprueba que `[package]` declara `include` con exactamente los siete patrones anclados acordados: `/src/**`, `/tests/**`, `/Cargo.toml`, `/Cargo.lock`, `/README.md`, `/LICENSE` y `/CHANGELOG.md`.
      Verificación: el test tuvo **dos rojos** deliberados. El primero exigía la lista blanca y falló porque el manifiesto no declaraba `include`; el segundo exigió el anclaje y volvió a fallar con el manifiesto aún sin anclar (`tiene "src/**" en la posición 0, pero se esperaba "/src/**"`). Tras corregir el manifiesto pasó a verde y fija la lista exacta y su orden.
- [x] 1.2 Medir el RED del paquete con la declaración actual del manifiesto.
      Verificación: `cargo package --list` mostró **54** ficheros e incluía el andamiaje interno (`openspec/…`, `.github/workflows/…`, `AGENTS.md`); con los 5 ficheros de este propio change ya en el árbol el recuento era **59**. Cadena medida: 54 → 59.
- [x] 1.3 Medir el RED de la atomicidad con refs desechables (una rama y un tag de prueba que se borran después del experimento).
      Verificación: se midieron los dos casos. Caso normal —push atómico de una rama nueva junto con un tag nuevo—: los **dos** aceptados, `exit 0` (`* [new branch]` y `* [new tag]`). Caso de fallo —rama non-fast-forward junto con un tag nuevo—: `error: atomic push failed for ref refs/heads/prueba/atomic-ok. status: 2`, con `! [rejected] … (non-fast-forward)` y `! [rejected] prueba-atomic-ko -> prueba-atomic-ko (atomic push failed)`, `exit 1`; la rama quedó intacta y el tag **no** se creó. El control sin `--atomic` solo rechazó la rama y creó el tag. Las refs de prueba se borraron después: **0** refs de prueba en el remoto.
- [x] 1.4 Ejecutar la suite completa para fijar el contrato rojo.
      Verificación: `cargo test --no-fail-fast` falló exactamente en el test nuevo y no falló ninguno de los 219 tests existentes.

## 2. GREEN: implementar

Se implementa lo mínimo para satisfacer los contratos: la lista blanca anclada en el manifiesto y el push atómico en el pipeline. No se añade ninguna dependencia ni se toca el contrato del lock, que pertenece a `crate-metadata`.

- [x] 2.1 Declarar `include` en `[package]` de `Cargo.toml` con la lista blanca anclada: `/src/**`, `/tests/**`, `/Cargo.toml`, `/Cargo.lock`, `/README.md`, `/LICENSE` y `/CHANGELOG.md`.
      Verificación: `cargo test test_manifiesto_include_definido` pasa en verde.
- [x] 2.2 Reescribir en `.github/workflows/release-prepare.yml` el paso `Commit, tag and push`: sustituir los dos `git push` seguidos por un único `git push --atomic "$REMOTE" main "v${V}"` con las dos referencias en el mismo comando, hasta tres intentos separados cinco segundos, y `::error::` más salida distinta de cero si no lo consigue, sin llegar a ejecutar `gh workflow run`.
      Verificación: el diff deja un solo `git push` en el paso (`git push --atomic`), sin ningún push del tag separado del de la rama.
- [x] 2.3 Ejecutar la suite completa tras la implementación.
      Verificación: `cargo test --no-fail-fast` en verde con 220 tests (125 unitarios + 95 de integración) y 0 fallos.
- [x] 2.4 Medir el efecto de la lista blanca sobre el paquete.
      Verificación: con la lista sin anclar, `cargo package --list` daba **16** entradas y colaba `openspec/changes/archive/2026-10-14-remove-empty-id3-header/README.md` (arrastrado por el patrón `README.md`); con la lista anclada baja a **15** entradas, con **cero** ficheros del andamiaje y conservando `/src/**`, `/tests/**` con sus fixtures, `README.md`, `LICENSE`, `CHANGELOG.md` y `Cargo.lock`. Cadena medida: 54 → 59 → 16 → 15.

## 3. REFACTOR y documentación

- [x] 3.1 Documentar en `RELEASE.md` el procedimiento de recuperación de un release a medias y las dos trampas: no reejecutar el job fallido, porque el bump se recalcula desde el estado actual (`vampus upgrade --patch` desde 0.5.1 daría 0.5.2); y no lanzar el dispatch si el tag ya disparó la publicación, porque un tag empujado por una persona sí dispara `release.yml` (run 38527751799, `event: push`) mientras que los que empuja el pipeline con `GH_PAT` no (los cuatro últimos releases son `event: workflow_dispatch`). Añadir además qué viaja en el paquete publicado.
      Verificación: `RELEASE.md` contiene la sección «Qué se publica» y la sección «Recuperación de un release a medias», con la recuperación manual (tag sobre el commit y `git push origin origin/main:development`), los dos avisos —nunca reejecutar el job fallido, y que el arreglo solo actúa en el release siguiente— y el contenido del paquete.
- [x] 3.2 Documentar en `AGENTS.md` la convención del push atómico del pipeline y que el paquete publicado no lleva el andamiaje del repositorio, una línea cada una con su razón.
      Verificación: `AGENTS.md` contiene las dos convenciones nuevas en `## Key conventions`, cada una con su razón, y la referencia a `RELEASE.md` en `## Git workflow`.
- [x] 3.3 Actualizar `README.md`: si describe la instalación o el paquete, añadir qué contiene el paquete publicado.
      Verificación: `README.md` incorpora la subsección «Desde crates.io» dentro de «Instalación», que describe el contenido del paquete publicado (código, tests con fixtures y documentación pública; sin el andamiaje interno del repositorio).
- [x] 3.4 Comprobar el formato.
      Verificación: `cargo fmt --check` sin diferencias.
- [x] 3.5 Comprobar el linter en todos los targets y features.
      Verificación: `cargo clippy --all-targets --all-features -- -D warnings` sin avisos.

## 4. Cierre

- [x] 4.1 Ejecutar la suite completa.
      Verificación: 220 tests (125 unitarios + 95 de integración), 0 fallos y sin warnings del compilador.
- [x] 4.2 Validar el cambio contra OpenSpec.
      Verificación: `openspec validate release-push-race-and-package-contents --strict` sin errores.
- [x] 4.3 Empaquetar y verificar de extremo a extremo.
      Verificación: `cargo package --list` final con **15** entradas y **cero** del andamiaje; la inspección del `.crate` real confirmó los 15 ficheros y que su manifiesto normalizado conserva `include` anclado, `keywords` y `categories`; y `cargo publish --dry-run` empaquetó (**15** ficheros, 331.0KiB, 74.0KiB comprimidos), actualizó el índice, verificó la compilación y terminó con `warning: aborting upload due to dry run`, conservando `keywords` y `categories`.
- [x] 4.4 Comprobar que el YAML es válido y que no quedan push sueltos.
      Verificación: el fichero se parsea como YAML válido y dentro del paso `Commit, tag and push` queda **un solo** `git push` (el único otro `git push` del fichero es el de `Sync development with main`, intacto).
- [x] 4.5 Verificar el estado limpio ante un fallo de push atómico, repitiendo el experimento con el comando exacto escrito en el YAML sobre refs desechables.
      Verificación: con dos referencias válidas el push atómico acepta las dos y termina con `exit 0`; con una rama non-fast-forward y un tag nuevo falla como un todo (`atomic push failed`, `exit 1`), la rama queda intacta y el tag **no** se crea. Las refs de prueba se borraron después: **0** refs de prueba en el remoto.
- [x] 4.6 Archivar el cambio, con la aprobación previa del usuario.
      Verificación: `openspec archive release-push-race-and-package-contents` se ejecuta en esta misma rama, acto seguido a este cierre; requiere la aprobación previa del usuario.

## Seguimiento (fuera del cierre de este cambio)

Aquí NO se finge trabajo hecho: estas comprobaciones no eran tareas de este cambio, sino seguimiento posterior a la release real. Ya están hechas y verificadas tras la 0.5.2, así que quedan cerradas y marcadas, sin dejar de estar fuera del cierre del cambio.

- [x] Que el próximo release real use el push atómico y complete sus tres pasos (push, tag y publicación) sin intervención manual. Verificado el 2026-10-15: la release 0.5.2 lo confirmó. El paso `Commit, tag and push` del run `Prepare Release` 38684185477 subió rama y tag en un único push (`a86c6cd..1b10f4c main -> main` y `* [new tag] v0.5.2 -> v0.5.2`), sin ningún `::warning::el push atomico fallo`, es decir a la primera y sin reintentos, y el paso `Sync development with main` sincronizó `development` con `main`. Los tres pasos (push, tag y publicación) se completaron sin intervención manual: `release.yml` (run 38684207819) terminó en verde con el binario, la GitHub Release y la publicación en crates.io.
- [x] Que, tras la **próxima** publicación real, el `.crate` descargado de crates.io no contenga el andamiaje del repositorio. Verificado el 2026-10-15: descargado desde `https://crates.io/api/v1/crates/id3cli/0.5.2/download` y listado su contenido, el paquete tiene **15 ficheros** (los 13 del proyecto más `.cargo_vcs_info.json` y `Cargo.toml.orig`, que genera cargo) y **cero** ficheros del andamiaje: ni `openspec/`, ni `.github/`, ni `AGENTS.md`, ni `GIT_FLOW.md`, ni `RELEASE.md`, ni `cliff.toml`, ni `.justfile`, ni `.vampus.yml`.
