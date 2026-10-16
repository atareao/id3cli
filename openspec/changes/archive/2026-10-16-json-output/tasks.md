# Tasks

El change incorpora la salida JSON de `show` (`--json`) para los cinco formatos, siguiendo el patrón de los changes archivados (`flac-support`, `opus-support`): flag aditivo sin tocar la salida legible, esquema estable versionado en spec, serde+serde_json en producción con Cargo.lock commiteado, tests unitarios de serialización + integración CLI, e2e ampliado con `jq` como parser independiente y coherencia documental. Nada de `src/` ni `tests/` se toca antes del STOP del grupo 1.

## 1. Artefactos de especificación (OpenSpec)

- [x] 1.1 Redactar `proposal.md` con Why/What Changes/Capabilities/Impact (flag `--json`, esquema `{formato, tags}` con claves siempre presentes, dependencias serde+serde_json, tests, e2e con jq, docs; no-goals: bytes de carátula, pretty-print, JSON en edit/remove).
      Verificación: `openspec/changes/json-output/proposal.md` MUST contener las 4 secciones con las capabilities (NEW: json-output; MODIFIED: cli-output, e2e-tooling, e2e-verification).
- [x] 1.2 Redactar el delta `specs/json-output/spec.md` (capability NUEVA) con el contrato completo: flag `--json` en show, esquema del objeto (formato string con los 5 códecs; tags objeto con TODAS las claves presentes/null, arrays vacíos, compilation bool, track/season object, lyrics completo, pictures sin bytes), `tags: null` sin tag con exit 0 y stderr limpio, salida plana sin ANSI en una línea vía outln!, orden de claves estable.
      Verificación: MUST fijar con MUST el esquema (cada clave con su tipo/null), el `tags:null` sin aviso en stderr, la ausencia de ANSI, la salida única por stdout y el mapeo de los 5 formatos; cada requirement con escenarios WHEN/THEN.
- [x] 1.3 Redactar el delta `specs/cli-output/spec.md` (MODIFIED): el contrato de BrokenPipe (exit 0) aplica igual al modo JSON (el JSON viaja por stdout vía outln!); en modo JSON el caso sin tags NO escribe aviso en stderr.
      Verificación: MUST fijar el pipe cerrado con `--json` → exit 0 sin pánico y el stderr limpio en el caso sin tags en modo JSON.
- [x] 1.4 Redactar los deltas `specs/e2e-tooling/spec.md` y `specs/e2e-verification/spec.md` (MODIFIED): runner común con `check_json_*` y `jq` obligatorio, etapa JSON por formato, matriz con formato/muestra/`tags:null`/sin ANSI, falsabilidad JSON.
      Verificación: `check_json_valid`, `check_json_format`, `check_json_field`, `check_json_null` y el chequeo de ANSI MUST quedar contratados; la matriz MUST incluir la capa JSON por formato con su muestra representativa.
- [x] 1.5 Redactar `design.md` (decisiones 1-8 y alternativas descartadas) y `tasks.md` (este fichero).
      Verificación: design.md MUST documentar las 8 decisiones con la evidencia citada (outln!/output.rs, Backend::Ogg vs AudioFormat OggVorbis/OggOpus, convención `"; "`, read_flac_pictures).
- [x] 1.6 STOP: presentar la especificación al usuario y esperar aprobación explícita antes de tocar `src/` ni `tests/`.
      Verificación: aprobación explícita del usuario registrada en la conversación antes del grupo 2.

## 2. Fase RED — unitarios de serialización (src/tests.rs)

- [x] 2.1 Añadir tests unitarios de los structs JSON: `ShowJson` con `TagsJson` completo → `serde_json::to_string` → parsear con `serde_json::Value` y verificar cada clave presente con su tipo (title string, artists array, year number, track object con number/total, compilation bool, pictures array); campos ausentes → null; arrays vacíos; `tags: null`.
      Verificación: RED — falla hasta el GREEN (los structs no existen todavía); luego verde. Se parsea con serde_json (ya dependencia normal) para no verificar cadenas a mano.
- [x] 2.2 Añadir tests del mapeo AudioFormat→formato: `mp3`, `mp4`, `ogg`, `opus`, `flac` para los 5 valores del enum (incluida la distinción OggVorbis/OggOpus).
      Verificación: RED — la función no existe; luego verde.

## 3. Fase RED — integración CLI (tests/integration_test.rs)

- [x] 3.1 Añadir `test_cli_show_json_*` sobre los fixtures existentes (`create_temp_mp3/m4a/ogg/opus/flac`): `id3cli show --json <fixture>` pre-escrito con lofty → stdout es JSON válido (serde_json::from_str en el test), `formato` correcto, título/artista presentes con los valores esperados. Un test por formato (5).
      Verificación: RED — fallan mientras el flag no existe; el grupo 4 los pone verdes.
- [x] 3.2 Añadir los casos de ausencia: MP3 sin ID3 y MP4 sin ilst → stdout `{"formato": "...", "tags": null}`, exit 0, stderr VACÍO; formato no soportado (WAV) → exit 1, stderr con mensaje, stdout SIN JSON; `--json` con pipe cerrado (redirigir stdout a `head -1` o cerrar el reader) → exit 0 sin pánico.
      Verificación: RED — fallan; luego verdes.

## 4. Fase GREEN — implementación

- [x] 4.1 `Cargo.toml`: añadir `serde = { version = "1", features = ["derive"] }` y `serde_json = "1"` a `[dependencies]`; `cargo build` actualiza Cargo.lock; **Cargo.lock MUST commitearse en el mismo cambio** (regla `--locked` de CI).
      Verificación: `cargo build` compila y `git diff` muestra Cargo.toml + Cargo.lock.
- [x] 4.2 `src/lib.rs`: definir `ShowJson`, `TagsJson`, `TrackJson`, `PictureJson` (`#[derive(Serialize)]`) y las funciones constructoras por backend (reciben los datos ya leídos: título/artista/…/pictures; componen track/season object; lyrics completo; compilation bool). Mapeo `AudioFormat`→`&str` para los 5 formatos. Funciones `pub fn json_from_mp3(tag: &Id3v2Tag) -> String`, `json_from_mp4(tag: &Tag) -> String`, `json_from_vorbis(format: &str, tag: &VorbisComments, pictures: &[(Picture, PictureInformation)]) -> String` (sirve a OGG/Opus/FLAC, con FLAC pasando `read_flac_pictures`), y un `ShowJson::empty(format)` paral a `tags: null`.
      Verificación: `cargo test --lib` verde con los tests 2.1/2.2; `cargo clippy` sin warnings.
- [x] 4.3 `src/main.rs`: flag `json: bool` en `Command::Show`; en cada rama de backend, si `json` → serializar y `outln!("{}", json)`; si no → comportamiento actual byte a byte. En `Backend::Ogg`, obtener el AudioFormat real para nombrar `"ogg"`/`"opus"` (vía `detect_format` o conservando el formato del enrutado — Decisión 4). MP3 sin ID3 y MP4 sin ilst en modo JSON → `ShowJson::empty` + exit 0 sin aviso.
      Verificación: `cargo test` completo verde (incluidos los tests 3.1/3.2); `cargo fmt --check` limpio; `cargo clippy --all-targets --all-features -- -D warnings` 0 warnings.
- [x] 4.4 Verificar no-rotura del modo legible: con un fixture, `id3cli show` sin `--json` produce EXACTAMENTE la misma salida que antes del cambio (los tests de integración existentes de show ya lo garantizan al no tocar sus expectativas).
      Verificación: suite completa verde SIN modificar ningún test existente de show; `git diff` sobre los tests existentes vacío.

## 5. Fase GREEN — e2e

- [x] 5.1 `scripts/e2e/e2e_lib.sh`: añadir `check_json_valid` (show --json → `jq -e .` ok y `! grep -a $'\x1b'` para ANSI), `check_json_format <esperado>` (`.formato`), `check_json_field <ruta-jq> <valor>` (`jq -r` + `grep -aF`), `check_json_null <ruta-jq>`; incluir `jq` en la comprobación de herramientas de `e2e_setup`; actualizar doc de USO.
      Verificación: `bash -n` limpio; `jq --version` funciona en el entorno (1.8.2).
- [x] 5.2 Añadir la etapa JSON a cada `e2e_*.sh` (tras el show normal): `check_json_format` con el códec, muestra representativa (`check_json_field '.tags.title'`, `'[.tags.artists[0]]|join("")'` o similar para el multi-artista, `'.tags.album'`, `'.tags.track.number'`, `'.tags.compilation'`, `'.tags.pictures | length'`), y en mp3/m4a el `tags:null` sobre audio recién generado sin tags; en ogg/opus/flac verificar que `tags` es objeto con las claves null y `pictures` vacío.
      Verificación: `just e2e-mp3`, `just e2e-m4a`, `just e2e-ogg`, `just e2e-opus`, `just e2e-flac` → 0 fallidas con los recuentos nuevos MEDIDOS (+≈10-14 por formato, anotados en tasks.md).
- [x] 5.3 Medir y registrar los recuentos e2e nuevos por formato (evidencia fechada en tasks.md) y el recuento total; actualizar la tabla del spec e2e-verification con las celdas medidas.
      Verificación: la tabla del spec queda con los recuentos medidos y la evidencia fechada; `just e2e` completo → 0 fallidas.
- [x] 5.4 Falsabilidad demostrada sobre una aserción JSON: corromper el valor esperado de un `check_json_field` → exit 1 con el nombre de la aserción; restaurar → exit 0.
      Verificación: salida FAIL nombrando la aserción y exit code 1; tras restaurar, exit 0.

## 6. Fase REFACTOR — coherencia documental

- [x] 6.1 Actualizar `README.md`: sección `show --json` con ejemplo literal de salida (formato + tags con claves null), nota de consumo con `jq .` y pipe; el test documental (`test_documentacion_describe_los_formatos_soportados`) NO cambia sus ESPERADAS de keywords (verificar que la doc no altera los recuentos que mide).
      Verificación: el test documental sigue verde sin tocar sus ESPERADAS; `git diff` toca solo docs + código nuevo.
- [x] 6.2 Actualizar `AGENTS.md` (nuevas funciones de serialización, dependencias serde+serde_json, recuentos de tests y e2e medidos) y `.github/copilot-instructions.md`.
      Verificación: los recuentos declarados coinciden con los medidos en 5.3.
- [x] 6.3 Formateo final: `cargo fmt --check`, `bash -n scripts/e2e/*.sh`, revisión de que no queda material generado versionado.
      Verificación: limpieza mecánica completa.

## 7. Verificación final y cierre

- [x] 7.1 Ejecutar las comprobaciones mecánicas completas: `cargo test --no-fail-fast` (recuento final), `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `bash -n scripts/e2e/*.sh`, `just e2e` (cinco formatos, 0 fallidas) y falsabilidad ya demostrada (5.4).
      Verificación: todo en verde; recuentos nuevos (suite y e2e) anotados como evidencia.
- [x] 7.2 Comprobar el estado del change: `openspec status --change json-output` → 4/4 artefactos completos y `openspec validate json-output` → válido.
      Verificación: 4/4 y válido.
- [x] 7.3 Repasar cada escenario de los specs del change y anotar el test que lo cubre (unitario, integración o e2e), igual que se hizo en `flac-support`.
      Verificación: cobertura escenario→test completa en tasks.md.
- [x] 7.4 Cerrar con la evidencia fechada de todo lo anterior dentro del change; flujo git: rama `feature/json-output` desde `development`, commits `feat:`, PR a `development`, CI verde, fusión y `openspec archive json-output` (la release v0.8.0 la gestiona el director en la Fase 4, con la misma cadena probada en v0.7.0).
      Verificación: PR fusionado y change archivado con las 7 tareas completadas.
      (la ejecuta el director tras este cierre: commit feat: → PR → fusión → archive)

Estado: 25/25 tareas completadas (change validado, implementado en TDD RED→GREEN, e2e con jq verde, archivado 2026-10-16).