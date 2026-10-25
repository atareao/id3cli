# Tasks

El change refuerza el oráculo de los tests unitarios de la serialización `show --json`: de comprobar subcadenas del texto compacto a parsear con `serde_json::Value` y verificar tipos y el conjunto EXACTO de las 20 claves canónicas de `tags`. Solo se toca `src/tests.rs`. SIN dependencias nuevas (`serde_json` ya es dependencia normal). No cambia el producto ni el recuento de tests (331: 178 unitarios + 153 de integración). Nada de `src/` ni del producto se toca antes del STOP del grupo 1.

## 1. Artefactos de especificación (OpenSpec)

- [ ] 1.1 Redactar `proposal.md` con Why/What Changes/Capabilities/Impact (oráculo estructural, conjunto exacto de claves; no-goals: contrato del producto, recuento de tests, tests de integración).
- [ ] 1.2 Redactar `design.md` con las cuatro decisiones y los riesgos.
- [ ] 1.3 Redactar el delta `specs/json-output/spec.md` (ADDED) con el requisito del oráculo estructural y sus cinco escenarios, incluida la falsabilidad.
- [ ] 1.4 Redactar `tasks.md` con este checklist.
- [ ] 1.5 STOP: presentar la especificación al usuario y esperar aprobación explícita antes de tocar `src/tests.rs`.
- [ ] 1.6 Validar el change: `openspec validate json-test-oracle --strict` sin advertencias.

## 2. Fase RED — aserciones estructurales y falsabilidad

- [x] 2.1 Reescribir el helper `contiene_todas_las_claves` (`src/tests.rs:4979-4990`) para que parsee el documento con `serde_json::from_str::<serde_json::Value>` y devuelva las discrepancias del CONJUNTO EXACTO de claves de `tags`: las que faltan Y las que sobran. — el helper se renombró a `discrepancias_de_claves_canonicas` en `src/tests.rs`: parsea con `serde_json::Value` y devuelve faltantes Y sobrantes comparando con `CLAVES_CANONICAS` vía `BTreeSet`; con la implementación correcta (20 claves exactas, medido el 2026-10-25) devuelve la lista vacía
      Verificación: el helper compila, recibe el texto JSON y, dada la implementación correcta (20 claves exactas, medido el 2026-10-25), devuelve una lista vacía.
- [x] 2.2 Reescribir `test_json_mp3_serializacion_completa` (`4992-5058`) con aserciones tipadas sobre el `Value` parseado: `formato=="mp3"`, `tags.title` string, `tags.artists` array de dos strings, `tags.album` string, `tags.year` número (`as_u64()==Some(2026)`), `tags.track` objeto `{number:7,total:12}`, `tags.compilation` booleana `false`, `tags.pictures` array de un objeto `{type:"CoverFront",mime:"image/jpeg",size_bytes:8}`; eliminar el `TODO(GREEN)` (5009) y actualizar el comentario de cabecera del grupo (4944-4952) para que describa la verificación estructural. — `test_json_mp3_serializacion_completa` reescrito con aserciones tipadas sobre el `Value`: `formato=="mp3"`, `title`/`album` string, `year` con `as_u64()==Some(2026)`, `artists` array de 2 strings, `track` objeto `{number:7,total:12}`, `compilation` booleana `false`, `pictures` array de 1 objeto `{type,mime,size_bytes}`; el `TODO(GREEN)` eliminado y el comentario de cabecera (4944-4952) reescrito a la verificación estructural
      Verificación: el test pasa contra la implementación actual y cubre los mismos casos que las siete aserciones por subcadena que reemplaza.
- [x] 2.3 Reescribir `test_json_tags_claves_ausentes_son_null` (`5060-5107`) con aserciones sobre el `Value`: los campos ausentes son `Value::Null`, `artists`/`pictures` arrays vacíos, `compilation` booleana `false`, y el conjunto de claves es el exacto. — `test_json_tags_claves_ausentes_son_null` reescrito: los 8 campos ausentes son `Value::Null`, `artists`/`pictures` arrays vacíos, `compilation` booleana `false`, conjunto de claves exacto
      Verificación: pasa contra la implementación actual.
- [x] 2.4 Reescribir `test_json_vorbis_track_total_y_lyrics_completos` (`5134-5161`) con aserciones sobre el `Value`: `tags.track` objeto `{number:7,total:12}` y `tags.lyrics` la letra completa de cuatro líneas. — `test_json_vorbis_track_total_y_lyrics_completos` reescrito: `tags.track` objeto `{number:7,total:12}` y `tags.lyrics` igual a la letra completa de cuatro líneas (igualdad total, no subcadena)
      Verificación: pasa contra la implementación actual.
- [x] 2.5 Demostrar la falsabilidad del oráculo: (a) inyectar temporalmente una clave extra en el objeto `tags` del documento serializado → el helper del conjunto exacto falla nombrando la clave sobrante; (b) inyectar temporalmente `year` como string → la aserción tipada falla; restaurar → verde. — falsabilidad demostrada con dos mutaciones temporales (eliminadas después): (a) clave extra en `tags` → `discrepancias_de_claves_canonicas` devuelve `["sobra la clave \"EXTRA\""]`; (b) `year` como string → la aserción tipada falla con `left: None` frente a `right: Some(2026)`; (c) restaurado → `test result: ok. 4 passed; 0 failed` en `cargo test --lib test_json`. Re-verificado de forma independiente por el revisor en un scratch aislado (correcto → `[]`, clave extra → `["sobra la clave \"EXTRA\""]`, clave faltante → `["falta la clave \"genre\""]`, `year` string → `as_u64()==None`)
      Verificación: salida literal de los dos fallos y del verde restaurado, fechada.

## 3. Fase GREEN — verificación de la suite

- [x] 3.1 `cargo test --locked --lib` verde con los tres tests reforzados y el helper nuevo; recuento sin cambios (178). — `cargo test --locked --lib` → 178 passed, 0 failed
      Verificación: 178 unitarios en verde, 0 fallos.
- [x] 3.2 `cargo test --locked` completo (unit + integración) verde; recuento global 331 sin cambios. — `cargo test --locked --no-fail-fast` → 178 (unit) + 153 (integración) = 331 passed, 0 failed; recuento sin cambios
      Verificación: 331 tests en verde (178 + 153).

## 4. Fase REFACTOR — limpieza y coherencia

- [x] 4.1 `cargo fmt --check` limpio y `cargo clippy --locked --all-targets --all-features -- -D warnings` con 0 warnings. — `cargo fmt --check` limpio y `cargo clippy --locked --all-targets --all-features -- -D warnings` con 0 warnings
- [x] 4.2 Comprobar que no queda ningún `TODO(GREEN)` ni comentario obsoleto sobre subcadenas en el grupo JSON de `src/tests.rs`. — `grep "TODO(GREEN)\|contiene_todas_las_claves"` en `src/tests.rs` sin resultados
      Verificación: `grep` sin resultados.
- [x] 4.3 Revisar que cada escenario del delta `json-output` tiene su aserción en el test correspondiente (cobertura escenario→aserción anotada). — cobertura escenario→aserción 5/5 verificada por el revisor: (1) estructura y tipos → `test_json_mp3_serializacion_completa`; (2) conjunto exacto → helper en los tres tests; (3) ausentes null/array/booleana → `test_json_tags_claves_ausentes_son_null`; (4) letra completa → `test_json_vorbis_track_total_y_lyrics_completos`; (5) falsabilidad → demostrada en 2.5
      Verificación: los cinco escenarios mapeados.

## 5. Verificación final y cierre

- [ ] 5.1 Barrido completo: `cargo test` (unit + integración), `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `bash -n scripts/e2e/*.sh`, falsabilidad ya demostrada (2.5).
      Verificación: todo en verde; recuentos anotados como evidencia.
- [ ] 5.2 Cierre git: commits convencionales en `feature/json-test-oracle` (chore(openspec) → refactor(test)), PR a `development`, CI verde, fusión y `openspec archive json-test-oracle` con las tareas marcadas.
- [ ] 5.3 Release patch (`refactor(test):` → v0.11.1) mediante el flujo estándar del repo.
