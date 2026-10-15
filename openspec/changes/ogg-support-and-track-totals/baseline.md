# Baseline

Constancia del grupo 1 («Red de seguridad y baseline») del change `ogg-support-and-track-totals`.

## Contexto

Fecha: 2026-10-15. Rama: `feature/ogg-support-and-track-totals`. Commit base de la rama: `954cfcf` (`docs(openspec): propone el soporte de OGG Vorbis y el total de pista`), creada sobre `development` `34f9355`.

Todos los comandos `cargo` se han ejecutado con `--locked` (sin él, `Cargo.lock` se reescribe y ensucia el árbol) y con `--no-fail-fast` cuando aplica (la suite se detiene en el primer binario que falla y ocultaría el resto).

## 1.1 Baseline de la suite

Se registró el estado de la suite completa **antes** de tocar código de producción, para tener un punto de comparación fiable contra el que medir cualquier regresión posterior. Salida literal de los tres comandos:

`cargo test --locked --no-fail-fast`:

```text
running 125 tests
test result: ok. 125 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 95 tests
test result: ok. 95 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.27s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`cargo test --locked --lib`:

```text
running 125 tests
test result: ok. 125 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`cargo test --locked --test '*'`:

```text
running 95 tests
test result: ok. 95 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.25s
```

Recuento de partida: **125 unitarios + 95 de integración = 220 tests, 0 fallos**. El `git status --short` queda limpio tras ejecutarlos (el lock no se toca gracias a `--locked`).

## 1.2 Caracterización de `--track` y `--season` sin total

Objetivo: fijar el comportamiento que existe hoy (número sin total) antes de introducir la forma `N/T` en el grupo 5, y hacerlo **sin cambiar código de producción**. No se han creado tests nuevos: se han ampliado cinco tests existentes con asserts de **ausencia de total** (92 líneas insertadas, 2 borradas, en 2 ficheros).

| Caso | Test | Fichero:línea | Qué queda fijado |
|---|---|---|---|
| MP3 pista | `test_cli_track_number` | `tests/integration_test.rs:353` | `TRCK` vale exactamente `"5"`, sin barra |
| MP3 temporada | `test_cli_add_season` | `tests/integration_test.rs:1578` | `TPOS` vale exactamente `"3"`, sin barra |
| MP3 pista y temporada juntas | `test_cli_podcast_with_season` | `tests/integration_test.rs:1623` | `TRCK`=`"5"` y `TPOS`=`"2"`, sin barra |
| MP4 pista | `test_m4a_writes_track_atom` | `src/tests.rs:1692` | el `data` de `trkn` tiene 8 bytes y su campo de total (bytes 4..6) es `0` |
| MP4 temporada | `test_m4a_writes_disk_atom_for_season` | `src/tests.rs:1732` | el `data` de `disk` tiene 8 bytes y su campo de total es `0` |

La verificación es entre caminos distintos, como es convención del repo: en MP3 escribe `lofty` y relee `id3` (dev-dependency usada justo para eso); en MP4 se parsean los bytes del átomo con el helper `data_box_after` que ya existía en `src/tests.rs:1436`, sin pasar por `lofty`. Cada bloque lleva comentario avisando de que la expectativa cambiará en el grupo 5, sin haber sido adaptada.

Salida tras la ampliación:

```text
running 125 tests
test result: ok. 125 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 95 tests
test result: ok. 95 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.27s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

El recuento no cambia: 220 (125 + 95). `cargo fmt --check` sin salida (exit 0) y `cargo clippy --all-targets --all-features --locked -- -D warnings` limpio (exit 0).

## 1.3 Tests que hoy exigen el rechazo de `OggS`

Lista nominal, uno a uno y sin agrupar:

### 1. `src/tests.rs:1338` — `test_detect_format_unsupported_magic_bytes`

En la línea 1339 declara `let magics: [&[u8]; 6] = [b"fLaC", b"OggS", b"RIFF", b"FORM", b"MAC ", b"wvpk"];` y exige `AudioFormat::Unsupported` para cada uno. **Sí cambia de expectativa**: `OggS` sale del array (que pasa a 5 elementos) y la aceptación de Ogg Vorbis se prueba con bytes reales del fixture, no con esta lista. Lo que hoy es «no soportado» por ser Ogg pasa a ser una decisión por códec.

### 2. `tests/integration_test.rs:2842` — `test_cli_rejects_unsupported_formats_and_leaves_file_untouched`

En la línea 2843 declara `let cases: [(&str, &str, &[u8]); 4]` y su segundo caso (línea 2845) es `("ogg", "ogg", b"OggS\x00\x02")`, al que el test añade 32 bytes a cero. **No cambia de expectativa: sigue siendo rechazo, pero por otro motivo, y esto es un hallazgo del baseline.** Los bytes de ese caso no forman un Ogg Vorbis válido: tras `OggS` la versión es `0x00` y el byte 26 —el número de segmentos de la página— queda en `0x00` por el relleno de ceros, de modo que el primer paquete caería en el offset 27 y ahí hay `0x00`, no `\x01vorbis`. Con el contrato nuevo eso es «contenedor Ogg con un códec no soportado» y el resultado es el mismo que hoy: código de salida distinto de cero, mensaje en español y archivo intacto. Por tanto este caso **se mantiene** en la lista de rechazos; lo que hay que hacer en el grupo 2 es añadir los casos de aceptación con el fixture real (tarea 2.1) y comprobar que la etiqueta del caso no induzca a error.

**Corrección medida al cerrar el grupo 2.** La predicción de este baseline era que esos bytes darían «contenedor Ogg con un códec no soportado» (`Unsupported`). **Es incorrecta y queda corregida aquí.** La medición con el binario ya enrutado da `Unknown`, «no se pudo determinar el formato», y el mensaje literal de stderr es:

```text
Error: No se pudo determinar el formato del archivo '<ruta>'. Formatos soportados: MP3, MP4 y OGG.
```

El motivo: el byte 26 de la página es `0`, así que la página **no declara ningún segmento** y no hay primer paquete del que leer un identificador de códec; sin identificador no se puede ni afirmar que el contenedor sea Ogg soportado ni que sea de otro códec, y el contrato lo deja en «formato no determinado». El test sigue pasando **sin cambiar su expectativa**, porque tanto el mensaje de formato no soportado como el de formato no determinado enumeran los formatos soportados; lo que se corrigió fue la **etiqueta y el comentario del caso**, que en el grupo 2 pasó a llamarse `ogg_sin_paquete` con un comentario que explica que es una página Ogg sin primer paquete. Los casos de «códec no soportado» de verdad (Opus y Speex, con páginas Ogg válidas de un solo paquete) se cubren ahora en un test nuevo del grupo 2: `test_cli_rejects_ogg_other_codecs_and_leaves_file_untouched` en `tests/integration_test.rs`, que comprueba código de salida distinto de cero, mensaje en español y archivo intacto byte a byte. Esto es exactamente lo que justifica registrar el baseline con evidencia en lugar de con suposiciones; la hipótesis era razonable y aun así la medición la desmintió.

### 3. Expectativas que el grupo 2 también mueve (el literal del mensaje)

No son tests de rechazo de `OggS`, pero fijan el texto del mensaje y quedarán desactualizados al decir este MP3, MP4 y OGG:

- `tests/integration_test.rs:3273` — `test_cli_unsupported_format_message_lists_mp3_and_mp4`: su nombre y su assert exigen «MP3 y MP4».
- `tests/integration_test.rs:2767` — descripción del assert «debe enumerar los formatos soportados (MP3 y MP4)», dentro del test de MP4 malformado.
- Los tres literales del mensaje están en `src/main.rs` líneas 194-195, 203 y 232, con comentarios de documentación en las líneas 39, 178 y 221 que también enumeran solo dos formatos.

### 4. Líneas desplazadas al cerrar el grupo 2

- `tests/integration_test.rs:2842` — el test de rechazos de formato (`test_cli_rejects_unsupported_formats_and_leaves_file_untouched`) pasó a la **2843**, y su caso `ogg` a la **2845**; ese mismo caso se reetiquetó como `ogg_sin_paquete`.
- El assert «debe enumerar los formatos soportados (MP3 y MP4)» sigue en `tests/integration_test.rs:2767` y ahora menciona los tres formatos.
- El test que se llamaba `test_cli_unsupported_format_message_lists_mp3_and_mp4` (línea 3273) se renombró a `test_cli_unsupported_format_message_lists_mp3_mp4_and_ogg` y vive en la **3564**.
- El array de magics de `test_detect_format_unsupported_magic_bytes` pasó de 6 a **5** elementos al salir `OggS` (sigue en `src/tests.rs:1338`).

> [!note]
> Los números de línea de los apartados 1 a 3 de esta sección son los del momento del baseline (commit `4d516dd`) y se conservan tal cual a propósito; esta es la foto actualizada.

## 1.4 Desajuste entre la documentación y la suite

Hallazgo registrado antes de tocar nada: la documentación declara un recuento de tests que no es el real. Búsquedas literales y sus líneas:

- `README.md:6` — escudo `tests-219%20passing`.
- `README.md:656` — «219 tests (124 unitarios + 95 de integración)».
- `README.md:659` — `cargo test              # Ejecutar todos los tests (219)`.
- `AGENTS.md:9` — `cargo test                       # 219 tests (124 unit + 95 integración)`.

Real, medido en 1.1: **220 (125 unitarios + 95 de integración)**, es decir 125 unitarios y no 124. Corrección obligatoria en las tareas 6.1 y 6.4 del change.

## 1.5 Constancia

`baseline.md` es la constancia de este grupo. El grupo 1 se commitea **antes** de tocar `src/lib.rs` y `src/main.rs`: si el baseline y la implementación se mezclan, ninguna regresión es atribuible a una causa. Las listas nominales de 1.2 y 1.3 son las que gobiernan las tareas 2.1, 2.4, 5.6 y 6.4.
