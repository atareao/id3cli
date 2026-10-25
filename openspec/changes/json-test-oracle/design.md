# Design

## Problema

El oráculo de los tests unitarios de la serialización JSON quedó por subcadenas del texto compacto. Ese método es frágil y débil: no distingue una clave del objeto `tags` de la misma cadena anidada en un valor, no comprueba tipos y depende del espaciado exacto de `serde_json::to_string`. El contrato del producto (`json-output`: 20 claves siempre presentes, arrays que nunca son `null`, `track`/`season` como objetos, `compilation` booleana, `pictures` sin bytes) merece un oráculo que verifique la ESTRUCTURA, no el texto.

## Decisión 1 — Parsear con `serde_json::Value` en vez de comparar subcadenas

`serde_json` ya es dependencia normal (`Cargo.toml`), así que parsear el documento con `serde_json::from_str::<serde_json::Value>` no añade nada al árbol de dependencias. La comprobación pasa a ser sobre el valor tipado: `value["tags"]["year"].as_u64() == Some(2026)`, `value["tags"]["artists"].as_array()` de dos strings, etc. Esto elimina la dependencia del espaciado y hace imposible el falso positivo por cadena anidada. Descartado: mantener subcadenas con expresiones regulares (sigue atado al texto y no verifica tipos).

## Decisión 2 — El conjunto de claves MUST ser exacto, no solo contener las canónicas

El helper hoy (`contiene_todas_las_claves`) comprueba que cada una de las 20 claves aparece como `"clave":`, pero NO detecta una clave de más. El nuevo oráculo compara el CONJUNTO exacto de claves del objeto `tags` con las 20 canónicas: informa tanto de las que faltan como de las que sobran. Es la mejora que el método por subcadenas no puede expresar, y es la que da valor real al cambio.

## Decisión 3 — Alcance acotado a `src/tests.rs`, sin tocar el producto

El contrato del producto no cambia: `json_from_mp3`, `json_from_vorbis`, `json_empty` y `format_json_name` quedan intactos. Solo se refuerza el oráculo de los tests unitarios. Los tests `test_json_empty_tags_null` y `test_format_json_name_cinco_formatos` ya usan `assert_eq!` sobre el texto exacto (que es legítimo porque certifican el formato exacto) y NO se tocan. Los tests de integración de JSON y los de e2e (`jq`) usan el binario real y quedan fuera de alcance.

## Decisión 4 — Falsabilidad demostrada con una mutación

Un oráculo más fuerte debe poder fallar. Se demuestra con dos mutaciones: (a) añadir una clave extra a `tags` → falla la comprobación de conjunto exacto; (b) cambiar un tipo (p. ej. `year` a string) → falla la aserción tipada. El método anterior por subcadenas no detectaría (a) en ningún caso. La falsabilidad se documenta con la salida literal y se restaura.

## Alternativas descartadas

- Comparar con `serde_json::to_string` de un `Value` esperado y `assert_eq!` sobre el texto completo: seguiría siendo una comparación textual y rompería ante cualquier reordenación de claves; además acopla el test a la implementación.
- Validar contra un esquema JSON (jsonschema): añade una dependencia nueva y sobre-ingeniería para 20 claves fijas.
- Regex sobre el texto: sigue atado al texto, no verifica tipos ni el conjunto exacto.

## Riesgos y mitigaciones

- **Que el refactor baje la cobertura real**: cada aserción por subcadena existente MUST tener su equivalente estructural; el checklist las enumera una a una. Se comprueba que los tres tests siguen cubriendo los mismos escenarios del spec `json-output`.
- **`serde_json::Value` con números**: `year` se compara con `as_u64()` (entero sin signo), no con `as_f64()`, para no aceptar `2026.0`; documentado en la aserción.
- **El recuento de tests no cambia**: no se añaden ni se quitan tests, solo se refuerzan las aserciones; la suite sigue en 331.
