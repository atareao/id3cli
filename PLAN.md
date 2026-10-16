# PLAN — Próxima iteración: fixes del e2e, release v0.6.0, soporte FLAC y salida JSON

## Estado y regla de oro

> **ESTADO: FASE 1 COMPLETADA (bugs del e2e corregidos y verificados). Pendientes de aprobación/ejecución: Fase 0 (release v0.6.0), Fase 2 (FLAC) y Fase 3 (JSON).**

Este plan consolida, en orden de ejecución:

1. **(0)** la release pendiente v0.6.0
2. **(1)** los dos bugs que el tooling e2e destapó
3. **(2)** la feature de soporte de FLAC (nuevo formato) — que **DEBE** ir antes que el JSON porque la salida JSON tiene que cubrir FLAC —
4. **(3)** la feature de volcado de tags en JSON

El usuario ha pedido explícitamente que este plan se guarde **SIN implementar nada**.

## Contexto — dónde estamos (todo verificado por el director)

- El e2e por extensión (`scripts/e2e/`, PR #25 fusionado) valida **312 aserciones en verde**: mp3 70, m4a 80, ogg 81, opus 81 (tras la ampliación de la Fase 1: +12 en m4a por la secuencia `edit → remove --all → edit` y +5 por formato por la segunda edición de `--cover`). Suite Rust: **283 tests** (163 unit + 120 integración). fmt/clippy limpios.
- PR #24 «release: v0.6.0 — OGG Vorbis, totales de pista y Ogg Opus» sigue **ABIERTO** en main (creado, CI en verde, pendiente de decisión/fusión del usuario).
- El change OpenSpec `e2e-local-helpers` está archivado (capabilities `e2e-tooling` y `e2e-verification` en `openspec/specs/`).
- **lofty 0.25.4 ya soporta FLAC**: `lofty::flac::FlacFile` con `vorbis_comments_tag: Option<VorbisComments>` y `pictures: Vec<(Picture, PictureInformation)>`; implementa `OggPictureStorage` (la MISMA interfaz que ya usa el backend OGG/Opus para carátulas); `flac/mod.rs` línea 58-105 muestra que `write_to` mezcla las pictures del tag y las del propio archivo al guardar; probe.rs mapea `fLaC` → `FileType::Flac` (línea 375). La lectura simétrica: `FlacFile::read_from(&mut file, ParseOptions::default())` + accessor `vorbis_comments()` generado por el derive.
- El proyecto **NO tiene serde ni serde_json** (`Cargo.toml` solo tiene lofty + clap; dev-dependency `id3` para tests). La feature JSON añadirá dependencias y, por la regla del repo (CI con `--locked`), **Cargo.lock DEBE commitearse en el mismo cambio**.
- Detección actual: `detect_format_from_bytes` trata `b"fLaC"` como NoSoportado (hay listas en src/lib.rs ~914-916, ~941-942, ~980, ~1003 que nombran FLAC entre los no soportados — habrá que invertirlas como se hizo con OpusHead).
- El e2e por extensión habrá que ampliarlo con `e2e_flac.sh` cuando exista FLAC (el runner común `e2e_lib.sh` ya es parametrizable por extensión).

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
- **Release:** FLAC + JSON + fixes en una **ÚNICA** release (precedente del repo: release única para OGG+Opus): bump minor → **v0.7.0** (v0.6.0 salió en la Fase 0). PR development → main.
- **Archive:** un change de OpenSpec por feature (`flac-support`, `json-output`) + uno por los fixes (o un change «e2e-findings-fixes» con dos capabilities según el criterio del director al aprobar); todos archivados al fusionar.
- **Falsabilidad e2e:** re-demostrar tras ampliar (aserción corrompida → exit 1).

## Riesgos / notas

- `--year`/`--date` en FLAC: mismo comportamiento Vorbis (claves separadas YEAR/DATE en el comentario; el e2e debe medir cómo los expone ffprobe — stream_tags, caja original, igual que OGG).
- FLAC puede llevar ID3v2 además de VorbisComments (assets de lofty: `flac_with_id3v2.flac`): fuera de alcance, documentar que el CLI lee/limpia solo el camino VorbisComments (y verificar qué hace lofty ante un FLAC con ID3v2: prioridad de tags — el spec lo fija; no bloquear).
- El JSON con carátula: si el spec decidiera incluir bytes (base64), el tamaño puede crecer mucho (una carátula JPG ~9 KB aquí, pero en general MB); por defecto **NO** incluir bytes, solo metadatos de presencia/formato (dar la ruta de cómo extraer con el cliente).
- El fix 1.1 (hdlr) puede requerir actualizar lofty o post-proceso manual: si lofty ya lo arregló en una versión > 0.25.4, evaluar el bump (implica Cargo.lock + regresión completa de los 281 tests + e2e) — el spec de fixes lo decide con evidencia.
- El fix 1.2 (cover sustituye) puede tener matices por formato: en MP4/OGG/Opus/FLAC lofty puede ya comportarse distinto (insert vs replace): el e2e ampliará la aserción en **TODOS** los formatos, no solo MP3 — decidir si el fix 1.2 se generaliza a los 5 (propuesta: sí, semántica «sustituir la carátula del tipo indicado» en todos los backends).

## Definition of Done (todo el plan)

- [ ] **Fase 0:** v0.6.0 publicada (PR #24 fusionado, tag y crates.io).
- [x] **Fase 1:** fixes 1.1 y 1.2 con tests RED→GREEN, e2e ampliado con las aserciones que hoy fallarían, sin regresiones (283 tests verdes; e2e 312 aserciones).
- [ ] **Fase 2:** FLAC soportado (detección, backend, CLI, fixture, ~7 tests integración, e2e_flac.sh, docs coherentes, hoja de ruta actualizada).
- [ ] **Fase 3:** `show --json` (o la ubicación que fije el spec) con estructura estable versionada, serde+serde_json con lock commiteado, tests unitarios + integración + cobertura e2e en los 5 formatos.
- [ ] **Fase 4:** suite completa verde, fmt/clippy/bash -n/just e2e (5 formatos) limpios, changes archivados, release v0.7.0 única, README/AGENTS/copilot al día.
- [ ] Ninguna de las fases 1-3 se toca sin aprobación explícita del usuario (este plan está **GUARDADO**, no en ejecución).