# PLAN — Próxima iteración: fixes del e2e, release v0.6.0, soporte FLAC y salida JSON

## Estado y regla de oro

> **✅ TODAS LAS FASES COMPLETADAS (2026-10-16): FASES 0-4 y FASE 5 CERRADAS. Releases v0.7.0 (FLAC), v0.8.0 (JSON) y v0.9.0 (completions bash/zsh/fish) publicadas en crates.io y GitHub Releases. Changes OpenSpec `json-output` y `shell-completions` archivados (2026-10-16). Plan director ejecutado en su totalidad.**

Este plan consolida, en orden de ejecución:

1. **(0)** la release v0.6.0 — **✅ COMPLETADA** (v0.6.0 publicada: PR #24 fusionado, tag y crates.io)
2. **(1)** los dos bugs que el tooling e2e destapó — **✅ COMPLETADOS** (change OpenSpec `e2e-findings-fixes`, fixes 1.1 y 1.2)
3. **(2)** la feature de soporte de FLAC (nuevo formato) — **✅ COMPLETADA (2026-10-16)** vía el change OpenSpec `flac-support` (archivado como `2026-10-16-flac-support`; PR #27 fusionado en `development`) — precedió al JSON porque la salida JSON tiene que cubrir FLAC
4. **(3)** la feature de volcado de tags en JSON — **✅ COMPLETADA (2026-10-16)** vía el change OpenSpec `json-output` (archivado como `2026-10-16-json-output`): flag `show --json` aditivo, serde+serde_json con Cargo.lock commiteado, 306 tests (169 unit + 137 int), e2e con jq 479 aserciones (91/101/96/96/95), PR #29 → `development` (`d45621a`) + archive (`fb6b928`). La release v0.7.0 (solo FLAC) ya salió; el JSON será **v0.8.0** (minor por `feat:`).

El usuario ha pedido explícitamente que este plan se guarde **SIN implementar nada**.

## Contexto — dónde estamos (todo verificado por el director)

- El e2e por extensión (`scripts/e2e/`, PR #25 fusionado) valida **392 aserciones en verde** sobre los CINCO formatos: mp3 70, m4a 80, ogg 81, opus 81 y flac 80 (la ampliación de la Fase 2 añadió `e2e_flac.sh`; la Fase 1 aportó +12 en m4a por la secuencia `edit → remove --all → edit` y +5 por formato por la segunda edición de `--cover`). Suite Rust: **292 tests (164 unitarios + 128 integración)**. fmt/clippy limpios.
- PR #24 «release: v0.6.0 — OGG Vorbis, totales de pista y Ogg Opus» **fusionado**: v0.6.0 publicada (changelog + tag + crates.io) — Fase 0 completada.
- El change OpenSpec `e2e-local-helpers` está archivado (capabilities `e2e-tooling` y `e2e-verification` en `openspec/specs/`).
- **lofty 0.25.4 ya soporta FLAC**: `lofty::flac::FlacFile` con `vorbis_comments_tag: Option<VorbisComments>` y `pictures: Vec<(Picture, PictureInformation)>`; implementa `OggPictureStorage` (la MISMA interfaz que ya usa el backend OGG/Opus para carátulas); `flac/mod.rs` línea 58-105 muestra que `write_to` mezcla las pictures del tag y las del propio archivo al guardar; probe.rs mapea `fLaC` → `FileType::Flac` (línea 375). La lectura simétrica: `FlacFile::read_from(&mut file, ParseOptions::default())` + accessor `vorbis_comments()` generado por el derive.
- El proyecto **NO tiene serde ni serde_json** (`Cargo.toml` solo tiene lofty + clap; dev-dependency `id3` para tests). La feature JSON añadirá dependencias y, por la regla del repo (CI con `--locked`), **Cargo.lock DEBE commitearse en el mismo cambio**.
- Detección actual: `detect_format_from_bytes` reconoce `b"fLaC"` (offset 0, `'f'` minúscula) → `AudioFormat::Flac` (las viejas listas de «no soportado» de src/lib.rs se invirtieron como se hizo con OpusHead). `\x7fFLAC` DENTRO del contenedor Ogg sigue siendo NoSoportado (códec FLAC en Ogg ≠ FLAC nativo).
- El e2e por extensión ya cubre FLAC: `scripts/e2e/e2e_flac.sh` + recipe `e2e-flac` en el justfile; `just e2e` recorre los cinco formatos (el runner común `e2e_lib.sh` es parametrizable por extensión).

## Fase 0 — Cerrar la release pendiente (decisión del usuario, sin código)

- Fusionar el PR #24 (development → main): libera OGG Vorbis + totales de pista/temporada + Ogg Opus como **v0.6.0** (bump minor automático por los `feat:`).
- **Verificación:** CI del PR #24 en verde (ya medido: 35s/32s), `git fetch origin main && git log --oneline origin/main -3` muestra el commit de release y el tag `v0.6.0`.
- **Salida:** v0.6.0 publicada (changelog + tag + crates.io). Esto desbloquea el resto del plan sobre una base publicada.

## Fase 1 — Los dos bugs detectados por el e2e (fixes en src/, con SDD/TDD)

> **✅ COMPLETADA (2026-10-16)** vía el change OpenSpec `e2e-findings-fixes` (4 artefactos, aprobado por el usuario; TDD RED→GREEN). Fix 1.1: post-proceso `ensure_mp4_meta_hdlr` en `write_mp4_tag` (inserta el `hdlr` de 33 B si el `meta` carece de él; idempotente; lofty 0.25.4 sin fix upstream). Fix 1.2: sustitución de carátula en los 4 backends — `remove_picture_type(CoverFront)` en MP3/OGG/Opus y vaciado de `covr` en MP4 (lofty relee toda carátula MP4 como `PictureType::Other`, verificado en mp4/ilst/read.rs:413). Test RED → verde; e2e ampliado 285→312 aserciones con falsabilidad demostrada (aserción corrupta → exit 1); suite 281→283 (163 unit + 120 integración); README documenta la semántica de sustitución. Ramas/PR pendientes de esta fase: ver grupo 5 de tasks.md (rama `fix/e2e-findings` → PR a `development`).

Ambos hallazgos fueron documentados en el PR #25 y en tasks.md 3.5 del change `e2e-local-helpers`. Cada fix debe seguir el flujo OpenSpec + TDD del repo (change propio o ampliación: el director decidirá al aprobar; mínimo: tests RED que reproduzcan el bug, fix mínimo, tests verdes, ampliar el e2e con la aserción que hoy fallaría).

### 1.1 — MP4: metadata invisible a ffmpeg/reproductores tras `remove --all` + `edit`

- **Síntoma:** tras `id3cli remove --all` sobre un `.m4a`, un `edit` posterior escribe la metadata (lisible por `id3cli show` y por lavish/lofty) pero `ffprobe format_tags` **NO** la ve (solo brands). Los reproductores externos (VLC, QuickTime) percibirían las etiquetas como perdidas.
- **Causa raíz (identificada):** lofty reconstruye el átomo `meta` SIN el hijo `hdlr` que el demuxer de ffmpeg exige para parsear `ilst`.
- **Fix propuesto:** garantizar que la escritura MP4 produce un átomo `meta` con `hdlr` válido (bien añadiéndolo explícitamente en el camino de escritura, bien post-proceso del archivo, bien una actualización de lofty si existe — verificar versión/issue). El fix debe validarse con `ffprobe -show_entries format_tags` (debe listar los tags) y con `id3cli show`.
- **Verificación:** test de integración RED (reproduce: remove --all + edit + ffprobe ve los tags) → fix → verde; ampliar `e2e_m4a.sh` con esta aserción (hoy NO la ejercita porque el e2e escribe sobre audio recién generado).

### 1.2 — MP3: `edit --cover` añade una segunda carátula en vez de sustituir

- **Síntoma:** `edit --cover` sobre un MP3 que ya tiene carátula da «Carátulas: 2 imagen(es)» (CoverFront JPG + PNG); lofty `insert_picture` no deduplica.
- **Fix propuesto:** en el camino `apply_metadata` de MP3, eliminar la carátula existente del mismo tipo (o todas) antes de insertar la nueva — decidir la semántica (¿sustituir solo CoverFront, o cualquier tipo?). El `remove --cover` ya existe como ruta de borrado; reutilizarla.
- **Verificación:** test RED (editar cover dos veces → 1 carátula al final, la última) → fix → verde; ampliar `e2e_mp3.sh` con la aserción «segunda edición de carátula sustituye, no acumula».

## Fase 2 — Feature: soporte de FLAC (DEBE preceder al JSON)

> **✅ ESTADO: COMPLETADA (2026-10-16)** vía el change OpenSpec `flac-support`, archivado como `2026-10-16-flac-support`: commit `feat:` **ece7bab** → PR **#27** → fusionado en `development` (**9f28232**) → CI en verde → archive + commit `docs(openspec)` (**2fca65f**). Recuentos verificados: suite Rust **292 tests (164 unitarios + 128 integración)**, 0 fallos (`cargo test --no-fail-fast`); `cargo fmt --check` y `cargo clippy --all-targets --all-features -- -D warnings` limpios; e2e **392 aserciones (mp3 70, m4a 80, ogg 81, opus 81, flac 80)**, 0 fallidas, con recipe `e2e-flac` y `just e2e` sobre los 5 formatos. Degradaciones FLAC documentadas (verificadas empíricamente, no supuestas): (1) la carátula viaja como bloque PICTURE separado del archivo (lofty la materializa así; `show` la lee de los bloques vía `read_flac_pictures`/`display_tags_flac`); (2) `remove --all` **ELIMINA** el bloque VORBIS_COMMENT completo — el vendor **NO se conserva** en FLAC, a diferencia de OGG/Opus (lofty no escribe un comentario vacío al vaciarlo). Medición ffprobe (ffprobe n9.0.2): la metadata de FLAC se expone en **`format_tags`** (NO `stream_tags`, a diferencia del contenedor Ogg), con normalizaciones a minúsculas idénticas a OGG (`TRACKNUMBER→track`, `DISCNUMBER→disc`, `ALBUMARTIST→album_artist`) y `TRACKTOTAL`/`DISCTOTAL` en mayúsculas; tras `remove --all` el `format_tags` queda vacío. Todo ello quedó reflejado en el spec archivado, el README, AGENTS.md y copilot-instructions. El resto de la sección queda como registro histórico del diseño e hitos.

Cambio grande alineado con el patrón ya establecido con Opus: FLAC usa el tag `VorbisComments` (campo `vorbis_comments_tag` de `FlacFile`) → el pipeline de metadatos se reutiliza; cambian detección, lectura/escritura despachada y enrutado del CLI.

### 2.1 — Detección por contenido

- Nueva variante `AudioFormat::Flac` en el enum (src/lib.rs, junto a OggVorbis/OggOpus).
- `detect_format_from_bytes`: `b"fLaC"` (offset 0, 4 bytes con `'f'` minúscula) → `AudioFormat::Flac`. **Fuera de alcance del contenedor Ogg**: `\x7fFLAC` DENTRO de un Ogg sigue siendo NoSoportado (es el códec FLAC en Ogg, distinto del FLAC nativo — documentar la distinción, igual que se distingue Theora).
- Actualizar las listas/comentarios que hoy ponen FLAC entre los no soportados (src/lib.rs ~914-916, ~941-942, ~980, ~1003 y las que aparezcan) y los mensajes «Formatos soportados: …» en src/main.rs (~199, ~207 y demás) → **«MP3, MP4, OGG, Opus y FLAC.»**
- **Tests RED/GREEN:** invertir los tests que fijan el rechazo de `fLaC` (unitarios de detección + test de integración CLI que rechaza .flac, si existe) + test nuevo `test_detect_format_flac` (bytes fLaC → Flac, con cualquier extensión: contenido mandante, no extensión).

### 2.2 — Lectura/escritura despachada (patrón `read_vorbis_tag`)

- Generalizar el despacho existente (hoy: `OggVorbis` → `VorbisFile`, `OggOpus` → `OpusFile`) para añadir `Flac` → `FlacFile::read_from` + `vorbis_comments().clone()`.
- La escritura: `save_to_path(path, WriteOptions::default())` inspecciona el contenedor real (el `write_to` de `FlacFile` mezcla tag.pictures + file.pictures al guardar — verificado en flac/mod.rs:58-105), así que el mismo camino de escritura sirve.
- **Carátula:** `FlacFile` implementa `OggPictureStorage` → la función/utility de carátula del backend OGG (la que inserta/lee pictures) debe funcionar igual; verificar el roundtrip JPG/PNG con `-c copy` como en OGG.
- **Degradaciones a documentar** (hereda del backend VorbisComments): `--url` y `--original-artist` se omiten con aviso por stderr; no hay clave inventada. Igual que OGG/Opus.
- **Decisión de diseño pendiente para el spec:** ¿nuevo `Backend::Flac` en main.rs (un backend propio que enruta a las mismas funciones VorbisComments), o generalizar el nombre? El director tiende a `Backend::Flac` explícito (misma filosofía que `Backend::Ogg` sirviendo a dos códecs: aquí el backend sirve a un formato con tag VorbisComments). Lo fija el spec.

### 2.3 — CLI, fixture y e2e

- **main.rs:** nuevo enrutado `Ok(AudioFormat::Flac) => Backend::Flac` + ramas en show/edit/remove (o verificación de que el pipeline compartido cubre); ayuda/doc comments actualizados.
- **Fixture:** `tests/fixtures/sample.flac` REAL (ffmpeg: `-f lavfi -i anullsrc=r=48000:cl=mono -t 0.6 -c:a flac -map_metadata -1 -fflags +bitexact -flags:a +bitexact`) + `tests/fixtures/generate_sample_flac.sh` verificador estructural (patrón `fLaC`, bloques METADATA_BLOCK, comentario Vorbis sin campos de usuario, ffprobe codec=flac) + helper `create_temp_flac()` en tests/integration_test.rs.
- **Tests:** detección unitaria + integración CLI show/edit/remove sobre sample.flac (replicando el patrón de los tests Opus: ~7 tests) + ampliar los tests de coherencia documental (hoja de ruta: FLAC `- [x]`).
- **e2e:** nuevo `scripts/e2e/e2e_flac.sh` (18 tags + url/original_artist omitidos con aviso + carátula JPG/PNG + remove/--all con vendor) y recipe `e2e-flac` en el justfile + incluir flac en `just e2e`.
- **Documentación:** README (tabla de formatos, mapeo de claves FLAC, hoja de ruta, escudo de tests), AGENTS.md, .github/copilot-instructions.md — mismo barrido que en Opus.
- **Recuento esperado:** suite Rust 281 + N tests nuevos; e2e 285 + ~76 aserciones flac.

## Fase 3 — Feature: volcado de tags en JSON (DESPUÉS de FLAC)

> **Nota (2026-10-16):** con FLAC ya soportado (Fase 2 completada), la cobertura JSON debe incluir los **CINCO formatos: MP3, MP4/M4A, OGG, Opus y FLAC nativo**. El resto del diseño de esta fase queda como está.

> **✅ COMPLETADA (2026-10-16)** vía el change OpenSpec `json-output` (archivado como `2026-10-16-json-output`). TDD completo: RED (9 tests JSON de integración fallando + E0425 en unitarios por símbolos inexistentes) → GREEN (serde/serde_json en `[dependencies]`, structs `ShowJson`/`TagsJson`/`TrackJson`/`PictureJson` + `json_from_mp3`/`json_from_mp4`/`json_from_vorbis`/`json_empty`/`format_json_name` en src/lib.rs, flag `-j, --json` y enrutado por backend en src/main.rs con el nombre real del códec ogg/opus vía `AudioFormat`). Decisiones 1-8 del design cumplidas: flag aditivo (modo legible byte a byte idéntico, verificado contra el binario v0.7.0), esquema `{formato, tags}` con claves siempre presentes, `tags:null` con exit 0 y stderr vacío, formato del códec real, lyrics completo, pictures solo metadatos, e2e con jq v1.8.2 como parser independiente, test documental intacto (recuentos medidos: 306 = 169+137). Suite 292 → **306 tests**; e2e 392 → **479 aserciones** (mp3 91, m4a 101, ogg 96, opus 96, flac 95) con falsabilidad JSON demostrada (corrupción 'flac'→'mp3' → exit 1). Docs: README (escudo, desglose, contrato del JSON, capa 3 del e2e, roadmap), AGENTS.md y copilot-instructions con 306 tests. Git: rama `feature/json-output` → PR **#29** → fusionado en `development` (**d45621a**, CI verde 31 s) → archive + commit `chore(openspec)` (**fb6b928**). Release resultante: **v0.8.0** (minor por `feat:`) en la Fase 4.

### 3.1 — Diseño de la salida (decidir en el spec, propuesta aquí)

- **Ubicación:** proponer un nuevo flag en `show` (p. ej. `id3cli show --json FILE`) o subcomando dedicado; el spec fija la elección (el director tiende a `show --json`: la salida JSON es la misma información que ya muestra show, en formato máquina).
- **Estructura JSON estable y versionada en el spec:** objeto con `formato` (MP3/MP4/OGG/Opus/FLAC), `tags` (objeto con las claves canónicas del proyecto — title, artist (array si multi), album, year, genre, track y season (objeto con number/total o cadena nativa), date, copyright, composer, subtitle, original_artist, album_artist, lyrics, url, compilation, album_sort/artist_sort/title_sort), y `caratula` (presencia + formato + quizá tamaño; **NO** bytes en base64 por defecto — decisión del spec; los bytes deben seguir siendo extraíbles sin JSON). **Multi-artista:** array o cadena `"; "` — decidir (propuesta: array en JSON).
- **Reglas:** JSON válido a stdout con `\n` final; sin colores ANSI (show hoy puede usar estilos — el modo json debe forzar salida plana); errores siguen a stderr con exit ≠ 0; modo json sobre archivo sin tags → objeto con tags vacíos (o null) — el spec fija la semántica; roundtrip estable (misma salida para misma entrada, orden de claves estable documentado).
- **Degradaciones:** en JSON se refleja la verdad del archivo (url/original_artist solo presentes si el formato los escribió; en MP4/OGG/Opus/FLAC ausentes u omisión documentada).

### 3.2 — Dependencias

- Añadir `serde` (derive) y `serde_json` a `[dependencies]`; **Cargo.lock se actualiza y DEBE commitearse en el mismo commit** (regla `--locked` del CI/AGENTS.md). Alternativa evaluada: serialización manual con `format!`/`write!` — descartada por mantenibilidad y coherencia (el spec justifica).
- **Estructuras de datos:** tipos de salida serializables (p. ej. un struct `ShowJson` con `#[derive(Serialize)]`) que mapeen los tags leídos por cada backend a las claves canónicas del 3.1.

### 3.3 — Tests y e2e

- **Tests unitarios:** serialización (estructuras correctas, multi-artista array, track total object, ausencias), y por formato si hace falta.
- **Tests de integración:** `id3cli show --json sample.ogg` (o .flac) → parsear el stdout con un parser JSON en el test (¿serde_json disponible como dev-dependency? o `python3 -m json.tool`; el repo ya usa python3 en los generadores) y verificar claves/valores; salida plana (sin ANSI); error → stderr + exit ≠ 0.
- **e2e:** ampliar runner común con una aserción `check_json_tag` (ejecuta `show --json` y valida con python3 -m json.tool + consulta de claves con python) o un script `e2e_json.sh` dedicado que recorra los 5 formatos; el spec decide (propuesta: integrarlo en los scripts por extensión con una aserción JSON, para que cada formato valide su propia verdad).
- **README:** sección/documentación del modo JSON con ejemplo de salida.

## Fase 4 — Verificación final y cierre

- **Barrido completo:** `cargo test --no-fail-fast` (recuento final), `cargo fmt --check`, `cargo clippy --all-targets --all-features --locked -- -D warnings`, `bash -n scripts/e2e/*.sh`, `just e2e` (cinco formatos ahora) en verde, `openspec status` 4/4 por change.
- **Flujo git:** ramas `feature/*` desde development por change (`fixes-mp4-mp3`, `flac-support`, `json-output`); commits convencionales: `fix:` para los bugs (1.1 y 1.2 — fix NO libera minor, pero sube patch: decidir si los dos fixes viajan en la misma release que FLAC/JSON o en una patch v0.6.1 antes — el director propone: un PR de fixes a development y que viajen en la release grande; el spec de release decide), `feat:` para FLAC y JSON (minor).
- **Release:** la release v0.7.0 (soporte FLAC) ya se publicó (PR #28 + pipeline `release-prepare` + `release.yml`, 2026-10-16). La Fase 4 cierra la Fase 3 con una **liberación única v0.8.0** del JSON: PR development → main, bump minor automático por el `feat:` en CI (no tocar Cargo.toml local). Verbificar el checklist de la Fase 4 (barrido completo + archive del change `json-output`).
- **Archive:** un change de OpenSpec por feature (`flac-support`, `json-output`) — ambos archivados (`2026-10-16-flac-support`, `2026-10-16-json-output`).
- **Falsabilidad e2e:** re-demostrar tras ampliar (aserción corrompida → exit 1).
- **Estado 2026-10-16:** ✅ FASE 4 CERRADA. Barrido completo verde (306 tests, fmt, clippy -D warnings 0, bash -n, openspec validate --specs 14 items, e2e 479 aserciones con `cargo build --locked`), no-rotura del modo legible verificada contra el binario v0.7.0 (salida idéntica en los 5 fixtures; solo `show --json` difiere, rc 2 → 0 por el flag nuevo). Release **v0.8.0** publicada: PR **#30** development → main (`0598c1a`), `release-prepare.yml` OK (commit `chore: release v0.8.0` `1af3013` + tag `v0.8.0`), `release.yml` OK (GitHub Release «id3cli v0.8.0» publicada 2026-10-16T18:21:18Z), crates.io **max_version 0.8.0** (2026-10-16). `development` sincronizado con `main` (ambos en `1af3013`). Último eslabón de la cadena de release probada en v0.7.0, sin desviaciones.

## Fase 5 — Feature: completions para bash, zsh y fish (DESPUÉS de v0.8.0)

> **✅ COMPLETADA (2026-10-16)** vía el change OpenSpec `shell-completions` (8 artefactos, aprobado por el usuario con «adelante»; TDD RED→GREEN). Subcomando `id3cli completions <SHELL>` (bash/zsh/fish exactos) que emite el script de completions por stdout, generado en runtime con `clap_complete ~4.5` (misma minor que clap 4.5.x) sobre el Command real del CLI vía `CommandFactory`; `GenerationShell` (FromStr con error en español) + `generate_completions()` en lib.rs; `print_block()` en output.rs (bloques multilínea con el mismo contrato de pipe cerrado → exit 0). El oráculo se alineó a la subcadena estable `complete -F _id3cli` (la salida es la NATIVA del generador: `complete -F _id3cli -o nosort -o bashdefault -o default id3cli`, validada con `bash -n`; desviación de post-procesado rechazada por el director). e2e dedicado `e2e_completions.sh` (20 aserciones) + `just e2e` ampliado a 6 scripts; falsabilidad re-demostrada (corrupción → exit 1). Suite 306→318 (174 unit + 144 int); e2e 479→499 aserciones; docs al día (README, AGENTS, copilot); test documental verde. PR #31 → squash `b2fb0d6` en `development`; change archivado `2026-10-16-shell-completions`; specs base 14→15 items. Release prevista: v0.9.0.

### 5.1 — Diseño (resumen para el checkpoint)

- Subcomando **`id3cli completions <SHELL>`** con `SHELL ∈ {bash, zsh, fish}` exactos; generación en runtime con **`clap_complete = "4.5"`** (misma minor que clap 4.5.54) sobre el Command real del CLI (vía `CommandFactory` → `Args::command()`).
- Lógica en lib.rs: enum `GenerationShell` (FromStr con error en español nombrando los tres shells, Display) + `generate_completions(shell, &mut clap::Command) -> String` testeable con un Command mínimo; main.rs añade el subcomando (value_parser) y enruta.
- Salida por la capa `output.rs`: nueva `print_block(&str)` para bloques multilínea con el MISMO contrato que `print_line` (pipe cerrado → exit 0; otros errores de escritura → exit 1 con mensaje en español; nunca `println!`).
- En el e2e: `bash -n` como oráculo de sintaxis del script bash generado (parser real), marcadores `#compdef id3cli` (zsh) y `complete -c id3cli` (fish), subcomandos presentes, pipe cerrado → exit 0, shell no soportado → exit ≠ 0, y falsabilidad re-demostrada (corromper el marcador esperado → exit 1).

### 5.2 — Grupos de tareas (tasks.md del change, TDD estricto)

1. Artefactos de especificación + **STOP de aprobación del usuario** (nada de `src/` ni `tests/` antes).
2. RED unitarios (`src/tests.rs`): `GenerationShell` (FromStr/Display), marcadores por shell sobre un `clap::Command` mínimo (`complete -F _id3cli id3cli` en bash, `#compdef id3cli` en zsh, `complete -c id3cli` en fish) y nombres de subcomandos en el script.
3. RED integración (`tests/integration_test.rs`): `completions bash|zsh|fish` → rc 0 + marcador + stderr vacío; shell inválido → exit ≠ 0 + error en español con los tres shells; pipe cerrado → exit 0.
4. GREEN: `clap_complete` en `[dependencies]` + Cargo.lock commiteado; `GenerationShell`/`generate_completions`/`print_block`; subcomando en `main.rs`.
5. GREEN e2e: `scripts/e2e/e2e_completions.sh` (bash -n, marcadores, subcomandos, pipe, shell inválido, falsabilidad) + recipe `just e2e` ampliada (5 formatos + completions, una sola construcción).
6. REFACTOR docs: README (sección completions con instalación en `.bashrc`/`.zshrc`/`config.fish`, recuentos medidos del escudo/desglose, roadmap), AGENTS.md, copilot-instructions; test documental verde.
7. Verificación y cierre: barrido completo (cargo test/fmt/clippy, `bash -n scripts/e2e/*.sh`, `just e2e`, `openspec validate --specs`), no-rotura de show/edit/remove, archive del change, PR a `development`, release única **v0.9.0**.

## Riesgos / notas

- `--year`/`--date` en FLAC: mismo comportamiento Vorbis (claves separadas YEAR/DATE en el comentario; el e2e debe medir cómo los expone ffprobe — stream_tags, caja original, igual que OGG).
- FLAC puede llevar ID3v2 además de VorbisComments (assets de lofty: `flac_with_id3v2.flac`): fuera de alcance, documentar que el CLI lee/limpia solo el camino VorbisComments (y verificar qué hace lofty ante un FLAC con ID3v2: prioridad de tags — el spec lo fija; no bloquear).
- El JSON con carátula: si el spec decidiera incluir bytes (base64), el tamaño puede crecer mucho (una carátula JPG ~9 KB aquí, pero en general MB); por defecto **NO** incluir bytes, solo metadatos de presencia/formato (dar la ruta de cómo extraer con el cliente).
- El fix 1.1 (hdlr) puede requerir actualizar lofty o post-proceso manual: si lofty ya lo arregló en una versión > 0.25.4, evaluar el bump (implica Cargo.lock + regresión completa de los 292 tests + e2e) — el spec de fixes lo decide con evidencia.
- El fix 1.2 (cover sustituye) puede tener matices por formato: en MP4/OGG/Opus/FLAC lofty puede ya comportarse distinto (insert vs replace): el e2e ampliará la aserción en **TODOS** los formatos, no solo MP3 — decidir si el fix 1.2 se generaliza a los 5 (propuesta: sí, semántica «sustituir la carátula del tipo indicado» en todos los backends).
- El JSON NO incluye los bytes de las carátulas (solo type/mime/size_bytes). `lyrics` sí va completo (no el preview de 3 líneas). El orden de claves del JSON lo fija el orden de campos del struct Serialize (estable y documentado).

## Definition of Done (todo el plan)

- [x] **Fase 0:** v0.6.0 publicada (PR #24 fusionado, tag y crates.io).
- [x] **Fase 1:** fixes 1.1 y 1.2 con tests RED→GREEN, e2e ampliado con las aserciones que hoy fallarían, sin regresiones (292 tests verdes = 164 unitarios + 128 integración; e2e 392 aserciones en los cinco formatos).
- [x] **Fase 2:** FLAC soportado (detección, backend `Backend::Flac` en el CLI, fixture `sample.flac`, tests integración, e2e_flac.sh, docs coherentes, hoja de ruta actualizada; change `flac-support` archivado; suite 292 tests; e2e 392 aserciones: 70+80+81+81+80).
- [x] **Fase 3:** `show --json` implementado (change `json-output` archivado), serde+serde_json con lock commiteado, 306 tests (169 unit + 137 int), e2e JSON por formato con jq (479 aserciones), PR #29 fusionado.
- [x] **Fase 4:** suite completa verde (306 tests, fmt, clippy 0, bash -n, openspec 14 items, e2e 479 aserciones), change `json-output` archivado, README/AGENTS/copilot al día, release **v0.8.0 única** publicada (PR #30, tag `v0.8.0`, GitHub Release + crates.io `max_version: 0.8.0`), `development` sincronizado con `main`.
- [x] **Fase 5:** completions para bash/zsh/fish (change `shell-completions` archivado), suite 318 (174+144), e2e 499 aserciones con `bash -n` como oráculo y falsabilidad, PR #31 fusionado, release única v0.9.0 publicada.
- [ ] Ninguna de las fases 1-3 se toca sin aprobación explícita del usuario (este plan está **GUARDADO**, no en ejecución).