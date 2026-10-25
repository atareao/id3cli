# Spec Delta

## Purpose

Los tests unitarios de la serialización de `show --json` verifican hoy el documento por SUBCADENAS del texto compacto de `serde_json::to_string`, un método que no puede expresar «exactamente estas claves y ninguna más», no comprueba tipos y ata las aserciones al espaciado exacto. Un `TODO(GREEN)` y el comentario de cabecera del grupo en `src/tests.rs` reconocen ese refuerzo pendiente. Este delta fija el oráculo: los tests MUST parsear el documento con `serde_json::Value` y verificar su ESTRUCTURA —tipos y conjunto exacto de las 20 claves canónicas—, con falsabilidad demostrada. NO cambia el contrato observable del producto (los requisitos de `json-output` sobre el JSON emitido siguen intactos): solo el modo de verificarlo en los tests.

## ADDED Requirements

### Requirement: Los tests de la serialización de `show --json` MUST verificar el documento parseado, no la estructura por subcadenas del texto

Los tests unitarios de la serialización del modo `--json` (`src/tests.rs`) MUST parsear el documento con `serde_json::from_str::<serde_json::Value>` y MUST verificar la estructura —tipos y valores— sobre el valor parseado, en lugar de comprobar la ESTRUCTURA mediante subcadenas en el texto compacto. La verificación del objeto `tags` MUST comprobar el CONJUNTO EXACTO de las 20 claves canónicas: todas presentes y ninguna de más. El oráculo MUST ser falsable: una mutación del documento —una clave de más, una clave de menos o un tipo cambiado— MUST hacer fallar la aserción correspondiente con un mensaje que nombre la discrepancia.

#### Scenario: Estructura y tipos verificados sobre el documento parseado

- **WHEN** se serializa un tag sintético con título, dos artistas (unidos por `"; "`), pista `7/12`, año 2026, compilación `false` y una carátula JPEG
- **THEN** las aserciones MUST leer los campos del `Value` parseado y comprobar sus tipos —`formato` string igual a `"mp3"`, `tags.title` string, `tags.artists` array de dos strings, `tags.year` número (`as_u64`), `tags.track` objeto `{number, total}`, `tags.compilation` booleana, `tags.pictures` array de un objeto `{type, mime, size_bytes}`— y MUST NOT comprobar la ESTRUCTURA mediante subcadenas del texto; la comprobación de que el documento es UNA sola línea es de compactitud —el modo batch emite NDJSON, un JSON por línea— y se conserva como aserción textual

#### Scenario: Conjunto exacto de claves canónicas

- **WHEN** se verifica el objeto `tags` de un documento parseado
- **THEN** el conjunto de claves MUST ser EXACTAMENTE las 20 canónicas (`title`, `artists`, `album`, `year`, `date`, `genre`, `track`, `season`, `copyright`, `composer`, `subtitle`, `original_artist`, `album_artist`, `url`, `lyrics`, `compilation`, `album_sort`, `artist_sort`, `title_sort`, `pictures`): ni falta ninguna ni sobra ninguna, y una clave de más MUST producir un fallo que la nombre

#### Scenario: Campos ausentes verificados como null, array vacío o booleana

- **WHEN** se serializa un tag que solo tiene álbum y el resto de campos no existen
- **THEN** sobre el `Value` parseado, `tags.title`, `tags.year`, `tags.date`, `tags.genre`, `tags.track`, `tags.season`, `tags.url` y `tags.lyrics` MUST ser `Value::Null`, `tags.artists` y `tags.pictures` MUST ser arrays vacíos y `tags.compilation` MUST ser la booleana `false`

#### Scenario: Letra completa verificada sobre el documento parseado

- **WHEN** se serializa un comentario Vorbis con una letra de cuatro líneas separadas por `\n`
- **THEN** `tags.lyrics` del `Value` parseado MUST ser la letra completa, con sus saltos de línea, sin truncar como el preview de 3 líneas del modo legible

#### Scenario: Falsabilidad de la verificación estructural

- **WHEN** se introduce una mutación en el documento —una clave extra en `tags`, o un tipo cambiado, como `year` en string— y se evalúa el oráculo
- **THEN** la aserción estructural correspondiente MUST fallar con un mensaje que nombre la discrepancia (conjunto exacto o tipo); restaurada la corrección, MUST volver a pasar
