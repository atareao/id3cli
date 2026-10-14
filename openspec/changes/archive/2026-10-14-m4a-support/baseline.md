# Baseline del grupo 1 — `m4a-support`

Registro del comportamiento real del CLI **antes** de tocar el backend. Este documento es la referencia autoritativa del recuento de tests y de la red de seguridad que protege la migración a `lofty`.

## Resultado de la suite

| Momento | Unitarios | Integración | Total | Fallos |
|---|---|---|---|---|
| Baseline inicial (antes del cambio) | 63 | 43 | **106** | 0 |
| Tras la caracterización del grupo 1 | 63 | 62 | **125** | 0 |

- Los 63 tests unitarios viven en `src/tests.rs`.
- Los tests de integración viven en `tests/integration_test.rs` y ejecutan el binario real con `Command`; el helper `create_temp_mp3()` no cuenta como test.
- El grupo 1 añadió **19 tests de integración** de caracterización, sin tocar código de producción.

> [!warning] Cifra desactualizada corregida
> `AGENTS.md` y `README.md` declaraban «99 tests (59 unit + 40 integration)». Esa cifra era incorrecta: el baseline real ya partía de 106 tests. Ambos documentos se corrigen al recuento medido.

## Evidencia: el CLI corrompe archivos no-MP3 (tarea 1.7)

Caracterización empírica del binario actual sobre un fichero `.m4a` real:

- **`show` sobre el `.m4a`:** imprime `⚠️ No se encontraron tags ID3` y **termina con código de salida 0**, en silencio. No distingue «archivo sin tags ID3» de «archivo que no es MP3».
- **`edit --title "Prueba"` sobre el mismo archivo:** **lo destruye**. El CLI antepone una cabecera ID3v2 (bytes `49 44 33 04 00`) al principio del archivo y empuja el átomo `ftyp` fuera del offset 4.
  - Tamaño antes: **29.677 bytes**. Tamaño después: **29.704 bytes** (**27 bytes más**, los de la cabecera ID3v2 insertada).
  - `ffprobe` deja de reconocer el archivo, con `moov atom not found` / `Invalid data found when processing input`.
- **A pesar de todo, el CLI informa** `✅ Tags guardados correctamente` y devuelve código de salida cero.

**Conclusión.** El CLI actual no solo no soporta MP4: **invalida silenciosamente cualquier contenedor que no sea MP3**, sin avisar y con un código de salida que miente. Esto justifica que la detección de formato rechace lo no soportado con un código de salida distinto de cero **antes** de abrir el archivo para escritura.

## Cobertura de los escenarios de `mp3-tagging`

Los 39 escenarios de `specs/mp3-tagging/spec.md` y el test que los cubre. Los marcados como **(parcial)** tienen cobertura conocida incompleta, descrita en la columna de notas.

| # | Escenario | Test(s) | Notas |
|---|---|---|---|
| 1 | Edición de un MP3 sin tags previos | `test_cli_adds_title`, `test_cli_adds_multiple_fields` | |
| 2 | Edición de un MP3 con tag ID3v1 existente | `test_cli_edit_preserves_existing_id3v1` | |
| 3 | Reescritura preserva la versión v2.4 | `test_cli_rewrite_preserves_id3v24`, `test_cli_writes_id3v24_header` | |
| 4 | Archivo no escribible | `test_cli_edit_non_writable_file_fails` | (parcial) mensaje propagado en inglés |
| 5 | Mapeo de los campos básicos | `test_cli_adds_multiple_fields`, `test_cli_all_metadata_with_track`, `test_cli_podcast_metadata` | |
| 6 | Mapeo de los campos de fecha | `test_cli_year_writes_tyer` | |
| 7 | Mapeo de `--date` | `test_cli_date_writes_tdrc` | |
| 8 | `--year` y `--date` coexisten sin pisarse | `test_cli_year_and_date_coexist` | |
| 9 | `--year` escribe `TYER` | `test_cli_year_writes_tyer` | |
| 10 | Mapeo de los créditos y las ordenaciones | `test_cli_add_composer`, `test_cli_add_subtitle`, `test_cli_add_compilation`, `test_cli_add_sort_orders`, `test_add_all_new_tags` | |
| 11 | Mapeo de `--url` y `--original-artist` en MP3 | `test_cli_add_url`, `test_cli_add_original_artist` | |
| 12 | Campo no indicado no se toca | `test_cli_preserves_existing_tags`, `test_apply_metadata_partial`, `test_cli_cover_preserves_existing_metadata` | |
| 13 | Tres artistas con separador | `test_cli_multiple_artists`, `test_multiple_artists` | |
| 14 | Un único artista sin separador | `test_cli_adds_title`, `test_multiple_artists` | |
| 15 | El separador no se aplica al artista del álbum | `test_cli_album_artist_separator_not_applied` | |
| 16 | Letra con idioma español | `test_cli_add_lyrics`, `test_add_lyrics` | |
| 17 | Letra vacía | `test_cli_lyrics_empty` | |
| 18 | Carátula JPEG y PNG | `test_cli_cover_png`, `test_add_cover_art`, `test_add_cover_art_png`, `test_detect_mime_type_jpeg`, `test_detect_mime_type_png` | |
| 19 | Carátula WebP en MP3 | `test_cli_cover_webp`, `test_add_cover_art_webp`, `test_detect_mime_type_webp` | |
| 20 | Extensión de imagen no soportada | `test_cli_cover_unsupported_format`, `test_detect_mime_type_unsupported` | |
| 21 | Archivo de carátula inexistente | `test_cli_cover_missing_file_fails` | |
| 22 | Archivo con ID3v1 e ID3v2 | `test_cli_remove_all`, `test_remove_all_id3_removes_both_v1_and_v2` | (parcial) literal exacto del mensaje por stdout |
| 23 | Archivo sin ningún tag | `test_cli_remove_all_no_tags_warns`, `test_remove_all_id3_no_tags_returns_none` | (parcial) literal del aviso por stdout |
| 24 | Archivo con un solo tipo de tag | `test_cli_remove_all_removes_v1`, `test_remove_all_id3_removes_only_v1`, `test_remove_all_id3_removes_v2` | (parcial) los tres estados intermedios no se distinguen por mensaje |
| 25 | El audio sobrevive al borrado | `test_cli_remove_all_preserves_audio` | |
| 26 | Eliminación con nombres en español | `test_cli_remove_spanish_names_all_four`, `test_remove_tags_spanish` | |
| 27 | Eliminación con nombres en inglés | `test_cli_remove_title`, `test_cli_remove_multiple_tags`, `test_remove_title` | |
| 28 | Combinación de nombres en ambos idiomas | `test_cli_remove_mixed_language_names` | |
| 29 | Nombre de tag desconocido | `test_cli_remove_unknown_tag_warns_and_does_not_modify`, `test_remove_unknown_tag` | |
| 30 | `remove` sin tags y sin `--all` | `test_cli_remove_without_tags_fails` | |
| 31 | Límite del plegado de acentos | `test_cli_remove_accent_folding_limit` | |
| 32 | Invocación sin flags | `test_cli_edit_without_flags_does_not_touch_file` | |
| 33 | Marca de tiempo intacta | `test_cli_edit_without_flags_does_not_touch_file` | (parcial) se compara el contenido, no el `mtime` |
| 34 | Un flag sin valor efectivo sí escribe | `test_cli_edit_empty_title_writes` | |
| 35 | Añadir carátula conserva los metadatos | `test_cli_cover_preserves_existing_metadata` | |
| 36 | Cambio de un solo campo | `test_cli_preserves_existing_tags`, `test_apply_metadata_partial` | |
| 37 | Archivo inexistente | `test_cli_fails_with_nonexistent_file` | |
| 38 | Error de escritura del tag | `test_cli_edit_non_writable_file_fails` | (parcial) mensaje propagado en inglés |
| 39 | Éxito confirmado en la salida estándar | `test_cli_adds_title`, `test_cli_add_compilation`, `test_cli_remove_title` | (parcial) se comprueba el éxito, no el literal exacto del mensaje |

**Resumen.** Tras los tests del grupo 1, **todos los escenarios tienen al menos un test**. Quedan con cobertura parcial conocida:

- Los escenarios cuyo contrato atañe a **mensajes por stdout**: 22, 23, 24 y 39 verifican el efecto sobre el archivo y el código de salida, pero no fijan el texto exacto de la confirmación o del aviso.
- Los escenarios de **estados intermedios de `remove --all`**: 22, 23 y 24 cubren la eliminación física y el aviso, pero no diferencian por mensaje los cuatro estados previos (sin tags / solo ID3v1 / solo ID3v2 / ambos).
- Los escenarios de **mensajes propagados desde la librería**: 4 y 38 fijan el comportamiento y el código de salida, pero el texto real está en inglés.
- El escenario de **marca de tiempo**: 33 comprueba que el archivo no se toca, no que el `mtime` quede intacto.

La cobertura parcial de 22, 23 y 24 cierra parte del hueco en el grupo 3 (detección por contenido) y el grupo 8; ninguno de estos huecos relaja una expectativa existente.

## Divergencias conocidas del baseline (caracterizadas, no corregidas)

- **El mensaje de error de permisos al escribir no está en español.** El CLI propaga el error de la librería, que aparece como:
  `Error: Io(Os { code: 13, kind: PermissionDenied, message: "Permission denied" })`
  El spec de `mp3-tagging` (requisito «Mensajes de error del CLI en español») se ha alineado con esta realidad: los mensajes que el CLI redacta por sí mismo están en español; los propagados desde la capa de E/S o desde la librería de metadatos pueden aparecer en inglés. **La traducción queda fuera de alcance** de este cambio.
- **El frame `TYER` (año) está obsoleto en ID3v2.4** pero se conserva a propósito para garantizar la paridad durante la migración: `--year` sigue escribiendo `TYER` y `--date` sigue escribiendo `TDRC`, sin pisarse. Su unificación en `TDRC` corresponde al cambio posterior e independiente **`id3v24-date-unification`**.

## Red de seguridad

Los tests del grupo 1 son la red que protege la migración de backend: **ningún test existente puede ver relajada su expectativa en los grupos siguientes.** Las adaptaciones permitidas son solo de firma, nunca de comportamiento esperado.

Comandos de verificación obligatorios en cada cierre de grupo:

```sh
cargo test
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

El grupo 8 (verificación de integración final) compara el resultado contra este baseline: recuento de tests, presencia del frame `TYER` en `--year` (tarea 1.6) y ausencia de cambios de expectativa en el diff de `src/tests.rs` y `tests/integration_test.rs`.

## Regresiones que la red de seguridad detectó al migrar

Al compilar la suite contra el nuevo backend `lofty`, fallaron **8 tests**, y no por los tests ni por cambios de expectativa, sino por **dos cambios de comportamiento reales de la librería nueva**:

1. **Siete tests** de `remove`: `lofty`, al guardar un tag vacío, elimina físicamente el bloque ID3v2 completo, mientras el crate `id3` dejaba una cabecera vacía de 10 bytes. Se preservó la paridad con un guardado específico en `main.rs` (ver decisión 10 del diseño).
2. **Un test**: el `Debug` del error de guardado de `lofty` usa `finish_non_exhaustive` y oculta la causa subyacente, de modo que el mensaje `Permission denied` desaparecía de la salida de error. Se restauró recorriendo la cadena de errores en `main.rs`.

**Conclusión.** Estos dos casos habrían pasado desapercibidos sin los tests de caracterización del grupo 1, y son la evidencia de que esa red cumple su función: los fallos no venían de una expectativa mal escrita, sino de comportamiento nuevo de la librería que solo se manifiesta al releer con un lector independiente.

**Recuento tras cerrar el grupo 2.** La suite sigue en **125 tests (63 unitarios + 62 de integración) y 0 fallos**, y los 62 tests de integración **no han requerido ningún cambio** gracias a que releen con `id3` (decisión 9 del diseño).

## Recuento tras cerrar el grupo 3 (detección de formato)

- La suite pasa a **142 tests (73 unitarios + 69 de integración), 0 fallos**.
- El grupo 3 añadió **10 tests unitarios** de detección por *magic bytes* en `src/tests.rs` y **7 tests de integración** en `tests/integration_test.rs`: MP3 renombrado a `.m4a` (se edita como MP3), rechazo de un `.m4a` en `show`/`edit`/`remove`, rechazo de un MP4 con nombre `.mp3` (se decide por contenido), rechazo de `.flac`/`.ogg`/`.wav`/`.aif`, archivo vacío, contenido no reconocible y archivo demasiado corto.
- **Ningún test del grupo 1 o 2 ha visto relajada su expectativa.** En particular, la detección reconoce el fixture MP3 real **sin cabecera ID3** por la vía de la sincronía de frame MPEG válida; si eso fallara sería un bug de detección, no un test a arreglar.
- Implementación: `detect_format` y `detect_format_from_bytes` en `src/lib.rs`; la guarda `require_mp3_format` en `src/main.rs` se invoca al inicio de `show`, `edit` y `remove` y **antes** de cualquier apertura en escritura, de modo que ningún formato no soportado llega a tocarse.
- Verificación manual del caso crítico (destrucción silenciosa del baseline): copia de un `.m4a` real de `ffmpeg` (9.435 bytes) y `edit --title "X"` → salida con código 1, mensaje en español y hash SHA-256 idéntico antes y después (`b54527ed…`); `ffprobe` sigue reconociéndolo como contenedor MP4 válido.

## Recuento tras cerrar el grupo 4 (backend de metadatos MP4)

- La suite pasa a **189 tests (113 unitarios + 76 de integración), 0 fallos**.
- El grupo 4 (metadatos `ilst`, tareas 4.1–4.11) es el que construye la escritura y la lectura de los átomos de metadatos MP4 sobre el backend `lofty`, y es el grupo que hace pasar la suite de 142 a 189 tests.
- **Ningún test de los grupos 1–3 ha visto relajada su expectativa.** Los tests de integración de MP3 preexistentes siguen releyendo con `id3` (dependencia de desarrollo), de modo que la comparación sigue siendo entre librerías distintas y no una relectura con la librería nueva.

## Recuento tras cerrar el grupo 5 (carátulas y letras en MP4)

- La suite pasa a **203 tests (119 unitarios + 84 de integración), 0 fallos**.
- El grupo 5 añadió **6 tests unitarios** en `src/tests.rs` (carátula JPEG tipo 13, carátula PNG tipo 14, WebP tipo reservado 0 con aviso y resto de campos, extensión no soportada sin escribir, borrado de `covr` conservando el resto y ausencia de idioma en `©lyr`) y **8 tests de integración** en `tests/integration_test.rs` (JPEG, PNG, WebP, ruta inexistente, `.tiff`, `remove cover`, letra visible con `show` y conservación del idioma `spa` en el `USLT` de MP3).
- **Ningún test de los grupos 1–4 ha visto relajada su expectativa.**
- La carátula de MP4 (`covr`) se escribe en `edit_mp4`, que **valida el MIME antes de tocar el disco**: una extensión no soportada devuelve error sin modificar un solo byte. El tipo de dato real se comprueba leyendo el `data` box del contenedor ISO-BMFF, no se asume.
- WebP: `lofty` 0.25.4 devuelve error si se le entrega un MIME de imagen que no reconoce, así que la carátula se escribe **dejando el MIME deliberadamente sin establecer**, que la librería mapea al tipo de dato reservado (0).

## Recuento tras cerrar el grupo 6 (campos sin equivalente en MP4)

- La suite pasa a **211 tests (122 unitarios + 89 de integración), 0 fallos**.
- El grupo 6 añadió **3 tests unitarios** en `src/tests.rs` (texto del aviso, dos avisos sin cambio real ni reescritura del archivo, y omisión de `--url` conservando la escritura de `©nam`) y **5 tests de integración** en `tests/integration_test.rs` (`--url` + `--title`, `--original-artist` + `--album`, ambos flags con archivo byte a byte idéntico, avisos solo en stderr y paridad de MP3 con `WOAR`/`TOPE` sin aviso).
- **Ningún test de los grupos 1–5 ha visto relajada su expectativa.**
- `--url` y `--original-artist` no tienen átomo en `ilst`: se omiten con un warning por stderr (`⚠️  El campo '<flag>' no tiene átomo equivalente en MP4; se omite.`), sin crear ningún átomo *freeform* de reemplazo. Los avisos se acumulan en `Mp4EditReport::warnings` (probado en unitarios) y `edit_mp4` los emite por stderr, de modo que no contaminan stdout.
- **«Ningún cambio real que escribir»:** como los dos campos omitidos no marcan `report.changed`, `edit_mp4` **no llama** a `write_mp4_tag`; si son los únicos flags el archivo queda byte a byte idéntico (verificado por hash SHA-256 en la prueba manual y por igualdad de bytes en los tests).

## Auditoría de paridad de MP3 (tarea 8.5)

**Veredicto: la paridad de comportamiento en MP3 se preservó.** La auditoría la realizó un revisor independiente en modo solo lectura, comparando el estado auditado del repositorio contra los commits anteriores a la migración del backend de metadatos.

### Tests de integración preexistentes

- **Ninguna aserción modificada.** El diff de `tests/integration_test.rs` entre el commit anterior a la migración y el estado auditado es de **1121 inserciones y 1 sola eliminación**.
- Esa única línea eliminada es un `use` (una importación). Una aserción modificada aparecería en el diff como un borrado **más** una inserción; no hay ningún borrado más, así que ninguna aserción fue reescrita.
- El recuento de tests del fichero es **idéntico** antes y después de la migración.
- Esto es posible porque esos tests releen los tags con la librería anterior (`id3`, dependencia de desarrollo) y **no** con la nueva: la verificación es entre librerías distintas, no una relectura con el mismo código que escribió.

### Tests unitarios adaptados

- De los **63 tests unitarios** se adaptaron **59**.
- Los **4 no tocados** son los de `detect_mime_type`, que no usan la API de tags y por tanto no necesitaban adaptación.
- La auditoría comprobó **uno por uno** que la adaptación preservó **la expectativa**, no solo la firma: mismo frame y mismo valor.

### `--year` sigue escribiendo `TYER`, no `TDRC`

Confirmado a **tres niveles**:

- En producción: `set_year_frame` inserta el frame `TYER`.
- En un test de integración preexistente, que afirma que `TYER` vale el año y que además `TDRC` está **ausente**.
- En los tests unitarios del año.

Ningún test se ha conformado con `TDRC` para el año.

### Debilitamiento de la red de seguridad: ninguno encontrado

- Sin tests marcados para omitirse.
- Sin aserciones trivializadas.
- Sin comprobaciones que hayan pasado de verificar un valor a verificar solo su existencia.
- Los tipos MIME de las carátulas se comparan **por valor exacto**.
- Las comprobaciones que usan «existe / no existe» **ya eran así antes** de la migración; no son una relajación introducida por este cambio.

### Huecos de cobertura encontrados y cerrados

La auditoría encontró **dos escenarios de la spec de paridad sin cobertura**, y ya están cerrados con **tres tests nuevos**:

- El cambio de un solo campo con `edit` preservando el resto.
- La carátula JPEG y PNG comprobando que **los bytes de la imagen quedan intactos** en el archivo (antes, la comprobación de integridad de bytes solo existía a nivel unitario y solo para JPEG).

### Falsa alarma documentada

> [!warning] No volver a señalar «Tags guardados» → «Metadatos guardados» como hallazgo
> El diff de `src/main.rs` muestra que las confirmaciones de éxito pasan de «Tags guardados» a «Metadatos guardados». Se verificó que ese cambio afecta **únicamente al camino MP4** (código nuevo de este mismo cambio, donde «Tags» era terminología incorrecta) y que **el texto del camino MP3 permanece byte a byte idéntico**.

No hay divergencia con la caracterización del baseline ni violación de paridad. Queda constancia explícita para que una auditoría futura no lo vuelva a marcar como hallazgo.
