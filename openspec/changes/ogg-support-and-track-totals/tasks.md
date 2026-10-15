# Tasks

El cambio amplía el CLI a un tercer formato —OGG Vorbis, con `VorbisComments` en lugar de frames ID3v2.4 o de átomos `ilst`— y extiende `--track` y `--season` a la forma `N/T` (número sobre total) en los tres formatos. Los grupos 2, 4 y 5 son estrictamente test-first: cada tarea funcional se escribe primero como RED (un test que falla) y solo después como GREEN (la implementación mínima que lo pone en verde); ninguna tarea GREEN precede a su RED. Los grupos 1, 3, 6 y 7 fijan el baseline, el fixture, la documentación y la verificación de cierre.

## 1. Red de seguridad y baseline

Antes de tocar una línea de código de producción hay que dejar medido qué hace hoy el binario y cuántos tests hay de verdad con qué expectativas, porque el cambio mueve dos cosas que la suite actual afirma: que OGG es un formato no soportado y que `--track` y `--season` son enteros sin total. Sin esa foto previa no se puede atribuir ninguna regresión a una causa.

- [ ] 1.1 Registrar el baseline real de la suite completa, sin filtros y sin detenerse en el primer fallo.
      Verificación: `cargo test --no-fail-fast` MUST reportar **125 tests unitarios + 95 de integración = 220, 0 fallos**; anotar la salida literal, sin editar ni resumir, en `baseline.md` dentro del directorio de este change.
- [ ] 1.2 Caracterizar el comportamiento **actual** de `--track` y `--season` como enteros sin total, sobre MP3 y sobre MP4, con tests que hoy pasan y sin cambiar código de producción.
      Verificación: los tests nuevos MUST quedar en verde contra el código actual y MUST fijar la ausencia de total (MP3 escribe `TRCK`/`TPOS` con el número a secas y MP4 escribe `trkn`/`disk` con el campo de total a cero); ninguno de ellos puede asumir la forma `N/T`, que se introduce en el grupo 5.
- [ ] 1.3 Caracterizar el rechazo actual de OGG identificando los tests existentes que afirman que `OggS` es un formato no soportado, y anotar exactamente cuáles cambian de expectativa, uno a uno.
      Verificación: lista nominal en `baseline.md` (fichero, nombre del test y línea) de cada test que hoy exige el rechazo de `OggS`, con la expectativa nueva que tendrá tras el grupo 2 (deja de ser rechazo y pasa a ser aceptación), uno por uno y sin agrupar.
- [ ] 1.4 Documentar el desajuste ya existente entre la documentación y la suite.
      Verificación: `README.md` y `AGENTS.md` declaran 219 tests (124 unitarios + 95 de integración) y la suite real tiene 220 (125 + 95); registrar el `grep` de `219` y de `124` como hallazgo del baseline y como corrección obligatoria del grupo 6 (tareas 6.1 y 6.4).
- [ ] 1.5 Dejar constancia del baseline en el propio cambio.
      Verificación: el fichero `baseline.md`, dentro del directorio de este change, MUST contener la salida real de los comandos ejecutados (suite completa, `cargo test --lib`, `cargo test --test '*'`) y las listas deducidas en 1.2, 1.3 y 1.4.

> [!warning] Las tareas del grupo 1 van en un commit propio antes de tocar código de producción: si el baseline y la implementación se mezclan, ninguna regresión es atribuible a una causa.

## 2. Detección de Ogg Vorbis

El formato se decide por contenido y OGG entra en la lista de soportados, pero solo Vorbis: un contenedor Ogg es un sobre, no un códec, y Opus, Speex, FLAC dentro de Ogg y el vídeo Theora siguen fuera de alcance. La detección actual no basta con mirar los cuatro primeros bytes, así que el sondeo crece y el primer paquete se localiza leyendo el propio segmento de la página.

- [ ] 2.1 RED: escribir los tests unitarios de detección con **bytes reales** de un `.ogg` (el fixture del grupo 3 se usa aquí en cuanto exista) y con los casos de contenedor Ogg con otro códec.
      Verificación: `cargo test --lib` en rojo por los tests nuevos, que exigen `AudioFormat::OggVorbis` y todavía no existen; ningún test nuevo MUST usar bytes inventados de 5 bytes, porque con ellos no se puede comprobar el salto al primer paquete.
- [ ] 2.2 GREEN: implementar la detección de Ogg Vorbis.
      Verificación: existe `AudioFormat::OggVorbis`; `OggS` sale de la lista de magic bytes no soportados; el sondeo del archivo pasa de 16 a 512 bytes; el primer paquete se localiza en `27 + header[26]` y su firma se compara con `\x01vorbis`; Opus (`OpusHead`), Speex, FLAC dentro de Ogg y el vídeo (Theora) MUST seguir devolviendo formato no soportado; y una página incompleta (cabecera sin el primer paquete completo) MUST devolver formato no determinado, no Ogg Vorbis.
- [ ] 2.3 GREEN: enrutar el CLI al backend OGG y corregir los mensajes que enumeran los formatos soportados.
      Verificación: `Backend::Ogg` existe en `src/main.rs` y los **tres** mensajes en español que enumeran formatos soportados pasan a decir MP3, MP4 y OGG; `grep` sobre esos literales MUST NOT dejar ninguno que enumere solo dos formatos.
- [ ] 2.4 GREEN: convertir los tests de rechazo de OGG detectados en 1.3 en tests de aceptación, y fijar el rechazo de lo que sigue fuera.
      Verificación: los tests nominalmente listados en 1.3 cambian de expectativa uno a uno (aceptación en lugar de rechazo) y MUST quedar tests explícitos de que Opus, Speex y FLAC en Ogg siguen rechazados, con código de salida distinto de cero, mensaje en español y el archivo intacto byte a byte.

Verificación de cada tarea del grupo: `cargo test` en verde (unitarios y de integración) con el número de tests anotado en cada caso y comparado contra los 220 del baseline.

## 3. Fixture real de OGG

Los tests de OGG no pueden usar bytes fabricados: hace falta un archivo Vorbis real que `lofty` parsee y que `ffprobe` reconozca. El fixture se genera con `ffmpeg` y se versiona, junto con el script que lo regenera.

- [ ] 3.1 Crear `tests/fixtures/sample.ogg` con `ffmpeg`.
      Verificación: `libvorbis`, seno de 0,6 s a 44,1 kHz, mono, con `-map_metadata -1 -fflags +bitexact -flags:a +bitexact`, tamaño pequeño y commiteado en `tests/fixtures/`; `ffprobe` MUST reconocerlo como audio Vorbis reproducible.
- [ ] 3.2 Crear `tests/fixtures/generate_sample_ogg.sh` que regenere el fixture y **verifique estructura, no igualdad de bytes**.
      Verificación: el script comprueba patrón `OggS`, versión 0, primer paquete en `27 + header[26]` empezando por `\x01vorbis`, presencia del vendor y de la clave `ENCODER`, y que `ffprobe` reconoce el archivo; MUST NOT comparar el fichero generado contra el versionado byte a byte.
- [ ] 3.3 Commit propio para el fixture y el script.
      Verificación: el commit MUST contener `tests/fixtures/sample.ogg` y `tests/fixtures/generate_sample_ogg.sh` y MUST NOT contener cambios de `src/` ni de `tests/` que no sean los helpers que copian el fixture a un temporal.

> [!note] Medido: el comentario de un `.ogg` de `ffmpeg` lleva siempre el vendor y `encoder=Lavc…`, y `-fflags +bitexact` no los elimina, así que el fixture NO es byte a byte reproducible entre versiones de `ffmpeg`. La verificación del script MUST ser estructural.

## 4. Backend OGG (VorbisComments)

La escritura sobre OGG pasa por un único mapeo flag → clave de comentario, con las mismas 18 asignaciones que los otros dos formatos, y con las mismas degradaciones explícitas donde el contenedor no da para más. Como en el grupo 2, cada comportamiento se fija primero con un test que falla.

- [ ] 4.1 RED: escribir los tests de escritura y relectura de las claves del mapeo, cubriendo los 18 flags, con un lector independiente.
      Verificación: `cargo test` en rojo por los tests nuevos de OGG; la relectura MUST hacerse con un lector que no sea la propia función de lectura del CLI (parseo directo del comentario Vorbis), para que la verificación sea entre dos caminos distintos y no una tautología.
- [ ] 4.2 GREEN: implementar la escritura de metadatos sobre `VorbisComments`.
      Verificación: existe una tabla única `VORBIS_COMMENT_MAP` (flag → clave) y una `apply_metadata_ogg` que la recorre, en lugar de ramas dispersas por función; la tabla MUST cubrir las 18 asignaciones del spec y MUST estar en un solo sitio.
- [ ] 4.3 GREEN: implementar la lectura, la escritura y el `show` de un `.ogg`.
      Verificación: el vendor y la clave `ENCODER` MUST NOT mostrarse como metadatos del usuario; y un `.ogg` sin campos de usuario MUST informar de que no hay metadatos con código de salida **cero**, igual que MP3 y MP4.
- [ ] 4.4 GREEN: implementar la carátula en `METADATA_BLOCK_PICTURE`.
      Verificación: el MIME se deduce de la extensión; la información de imagen se construye de forma explícita con `from_picture(...).unwrap_or_default()` para que una imagen corta o un `.webp` no aborten la edición; y en OGG NO hay degradación de WebP (a diferencia de MP4), con test que lo fije.
- [ ] 4.5 GREEN: implementar la letra.
      Verificación: la letra se escribe en la clave `LYRICS` y se muestra además en `UNSYNCEDLYRICS` como visualización adicional; un test de ida y vuelta MUST recuperar el texto completo.
- [ ] 4.6 GREEN: implementar los campos sin equivalente en OGG.
      Verificación: `--url` y `--original-artist` se omiten con aviso por stderr, sin escribir ninguna clave cruda (ni clave de reemplazo) y sin abortar el resto de la edición; test de integración que comprueba el aviso, el código de salida cero y que los demás campos indicados sí se escriben.
- [ ] 4.7 GREEN: implementar `remove` y el caso sin cambios.
      Verificación: `remove <campo>` usa la misma tabla de alias que MP3 y MP4 (inglés y español); `remove --all` vacía el comentario preservando el vendor y el audio; y `edit` sin flags MUST NOT reescribir el archivo.
- [ ] 4.8 GREEN: preservar lo ajeno al editarlo.
      Verificación: las claves de terceros (`ENCODER`, `MUSICBRAINZ_*`) y el vendor sobreviven a un `edit`, con test que compara el conjunto de claves antes y después.

Verificación del grupo: `cargo test` en verde y ejecuciones manuales de `cargo run -- show`, `cargo run -- edit` y `cargo run -- remove` sobre el fixture `tests/fixtures/sample.ogg`, con la salida anotada en el change.

## 5. Totales de pista y temporada en los tres formatos

`--track` y `--season` dejan de ser enteros y aceptan `N/T`, con validación en el parseo del propio CLI para que una forma inválida no llegue nunca a la librería. El total se escribe donde cada formato lo guarda y, sobre todo, se conserva el que ya estuviera ahí.

- [ ] 5.1 RED: escribir los tests de la forma `N/T` en los tres formatos y de las formas inválidas.
      Verificación: `cargo test` en rojo; cubrir `7/`, `/12`, `7/12/3`, `7/x` y la cadena vacía, comprobando en cada caso archivo intacto (mismo hash) y código de salida distinto de cero con mensaje en español.
- [ ] 5.2 GREEN: validar la forma en el parseo de `clap`.
      Verificación: existe un tipo validado que implementa `FromStr` con mensaje de error en español, de modo que un valor inválido falla antes de abrir el archivo; las firmas de las funciones de la librería quedan adaptadas a ese tipo y `cargo check` MUST NOT dejar firmas intermedias con enteros sueltos.
- [ ] 5.3 GREEN: implementar la escritura del total en los tres formatos.
      Verificación: MP3 escribe `TRCK`/`TPOS` como `n/total`; MP4 usa los átomos `trkn`/`disk` con su campo de total; OGG escribe además `TRACKTOTAL`/`DISCTOTAL`; un test por formato MUST releer el total escrito.
- [ ] 5.4 GREEN: conservar el total preexistente.
      Verificación: en los tres formatos, indicar el número **sin** total sobre un archivo que ya tenía total MUST conservar el total anterior; y sobre un archivo que no lo tenía, MUST NOT inventarse un total; test por formato y por caso.
- [ ] 5.5 GREEN: implementar la lectura y el borrado del total.
      Verificación: `show` muestra el total cuando existe y MUST NOT mostrar el campo cuando no existe; `remove track` y `remove season` eliminan número y total **como unidad** en los tres formatos, con test que comprueba que ni el número ni el total sobreviven.
- [ ] 5.6 Verificar que los tests de caracterización del grupo 1 (sin total) siguen pasando sin haber sido relajados.
      Verificación: los tests de 1.2 MUST seguir en verde y su diff MUST mostrar solo adaptaciones de tipo, ningún cambio de expectativa.

## 6. Documentación de los tres formatos

Con tres formatos, la documentación pasa a ser parte del contrato y por eso se verifica con tests: la coherencia entre `README.md`, `Cargo.toml` y la ayuda del binario deja de depender de la disciplina de quien escribe. Los ficheros que no viajan en el paquete se comprueban con `grep` registrado, no con un test, para no romper la verificación del paquete publicado.

- [ ] 6.1 Actualizar `README.md` a los tres formatos.
      Verificación: los formatos admitidos son los tres; OGG Vorbis sale de la lista de no soportados, que deja Opus, Speex, FLAC, WAV, AIFF y APE; hay una sección o tabla de mapeo por formato; las degradaciones quedan por formato (WebP se degrada en MP4 y no en OGG; la letra pierde el idioma en MP4 y OGG frente al `spa` de MP3); hay ejemplos de uso con `.ogg`; la hoja de ruta queda actualizada; y el **recuento real de tests** sustituye al 219 desincronizado.
- [ ] 6.2 Añadir el test de coherencia documental (unitario o de integración).
      Verificación: el test lee `README.md` y `Cargo.toml` y comprueba que los tres formatos figuran como soportados, que existe una sección de mapeo por formato, que OGG está fuera de la lista de no soportados, que la `description` nombra los tres formatos y que las cinco `keywords` siguen intactas.
- [ ] 6.3 Añadir el test de integración de la ayuda del CLI.
      Verificación: el test lanza el binario con `--help`, `show --help`, `edit --help` y `remove --help` y comprueba que los cuatro nombran los tres formatos y que la ayuda documenta la forma `N` y la forma `N/T`.
- [ ] 6.4 Actualizar `AGENTS.md` y `.github/copilot-instructions.md`.
      Verificación: ambos describen los tres backends, las convenciones por formato, las degradaciones, la lista de formatos no soportados y el recuento de tests correcto (125 + 95 = 220), sin rastro del 219 ni del 124.
- [ ] 6.5 Registrar la evidencia `grep` de los ficheros que **no** viajan en el paquete.
      Verificación: la evidencia (comando y salida) queda guardada dentro de este change para `AGENTS.md` y `.github/copilot-instructions.md`, que no pueden verificarse con un test de la suite porque un `include_str!` sobre un fichero fuera de la lista blanca del paquete rompería la verificación del paquete.
- [ ] 6.6 Actualizar `Cargo.toml`.
      Verificación: la `description` nombra los tres formatos; las `keywords` quedan **sin tocar** (las cinco intactas y en el mismo orden).

> [!warning] La documentación NO puede contradecir a los specs: si al escribirla aparece una discrepancia con lo implementado, se corrige el código o el spec, no se documenta lo que no es cierto.

## 7. Verificación final y cierre

Cierre con la suite completa, prueba manual sobre los tres formatos y repaso escenario a escenario de los specs del change, con la evidencia fechada dentro del propio change.

- [ ] 7.1 Ejecutar las tres comprobaciones mecánicas completas.
      Verificación: `cargo test --no-fail-fast` completo, `cargo fmt --check` y `cargo clippy --all-targets --all-features --locked -- -D warnings` limpios, con la salida anotada y el recuento de tests comparado contra los 220 del baseline.
- [ ] 7.2 Probar el flujo de extremo a extremo con el binario sobre los tres formatos.
      Verificación: `edit` con metadatos completos, `show`, `remove` de un campo y `remove --all` sobre MP3, MP4 y OGG, comprobando que el audio sigue intacto y reproducible en los tres.
- [ ] 7.3 Probar la carátula WebP en OGG frente a MP4 con la misma imagen.
      Verificación: en OGG MUST quedar escrita sin degradar; en MP4 MUST quedar degradada con aviso por stderr; ambas ejecuciones con código de salida cero.
- [ ] 7.4 Comprobar el paquete publicado.
      Verificación: `cargo package --list` MUST seguir publicando el paquete sin el andamiaje del repositorio (y los fixtures de `tests/**` sí viajan, incluido `tests/fixtures/sample.ogg`), con el recuento de ficheros anotado.
- [ ] 7.5 Repasar cada escenario de los specs del change y anotar el test que lo cubre.
      Verificación: tabla escenario → test en el change; los escenarios sin test MUST quedar marcados como verificados manualmente con su evidencia.
- [ ] 7.6 Cerrar con la evidencia fechada de todo lo anterior dentro del change.
      Verificación: todas las casillas de este fichero marcadas y la evidencia fechada registrada, lista para `openspec archive ogg-support-and-track-totals` tras la aprobación del usuario.
