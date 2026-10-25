# Proposal

## Why

Los tests unitarios de la serialización de `show --json` (`src/tests.rs`) verifican el documento por SUBCADENAS del texto compacto (`json.contains("\"year\":2026")`, `json.starts_with(...)`). El propio comentario de cabecera del grupo (líneas 4944-4952: «La verificación es por subcadenas del JSON compacto de serde…; el GREEN puede reforzarlos parseando con `serde_json::Value`») y un `TODO(GREEN)` en la línea 5009 reconocen que ese refuerzo quedó pendiente. El método por subcadenas: (a) no puede expresar «exactamente estas claves y ninguna más»; (b) no verifica tipos —un `"year":"2026"` en texto o un `"year":2026` anidado en otro objeto darían falso positivo—; y (c) ata las aserciones al formato exacto de `serde_json::to_string` (un cambio de espacios o de orden rompería tests correctos). `serde_json` ya es dependencia normal del proyecto (`Cargo.toml`), de modo que reforzar el oráculo no añade dependencias.

## What Changes

Los tres tests de serialización JSON (`test_json_mp3_serializacion_completa`, `test_json_tags_claves_ausentes_son_null`, `test_json_vorbis_track_total_y_lyrics_completos`) y el helper `contiene_todas_las_claves` pasan a parsear el documento con `serde_json::from_str::<serde_json::Value>` y a verificar la ESTRUCTURA (tipos y valores) sobre el `Value`, no subcadenas del texto. La comprobación de claves pasa a exigir el conjunto EXACTO de las 20 claves canónicas de `tags` —todas presentes y ninguna de más—, algo que el método por subcadenas no puede expresar. Se elimina el `TODO(GREEN)` de la línea 5009 y se actualiza el comentario de cabecera del grupo. Sin cambios en el producto: `src/lib.rs` no se toca, solo `src/tests.rs`.

## Capabilities

### Modified Capabilities

- **json-output**: la verificación del contrato JSON en los tests unitarios MUST ser estructural —documento parseado con `serde_json::Value`— e incluir el conjunto exacto de las 20 claves canónicas y la comprobación de tipos, con falsabilidad demostrada.

## Impact

- `src/tests.rs`: helper `contiene_todas_las_claves` (líneas 4979-4990) y los tests `test_json_mp3_serializacion_completa` (4992-5058, con el `TODO(GREEN)` en 5009), `test_json_tags_claves_ausentes_son_null` (5060-5107) y `test_json_vorbis_track_total_y_lyrics_completos` (5134-5161); comentario de cabecera del grupo (4944-4952).
- NO se toca `src/lib.rs`, `src/main.rs`, `Cargo.toml`/`Cargo.lock` (`serde_json` ya es dependencia normal) ni `tests/integration_test.rs`.
- No-goals: cambiar el contrato observable del producto; cambiar el número de tests (siguen 331: 178 unitarios + 153 de integración); añadir tests de comportamiento nuevos; migrar los tests de integración de JSON (usan el binario y `jq` en e2e); tocar `test_json_empty_tags_null` ni `test_format_json_name_cinco_formatos` (ya usan `assert_eq!` exacto).
- Versionado: `refactor(test):` → patch automático (v0.11.1) en el próximo release.
