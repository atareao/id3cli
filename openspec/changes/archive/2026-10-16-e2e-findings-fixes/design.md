# Design

## Contexto

El e2e (285 aserciones, PR #25) destapó dos bugs del CLI, ambos reproducidos con el binario de `development` el 2026-10-16 y documentados como hallazgos en tasks.md 3.5 del change `e2e-local-helpers`. Este change los convierte en fixes con SDD/TDD.

## Decisión 1 — Fix 1.1 (MP4 `hdlr`): garantizar la visibilidad externa de las etiquetas

- **Evidencia.** Reproducción: `edit` → `remove --all` → `edit` sobre `.m4a`; `show` lee los campos finales, `ffprobe format_tags` solo lista brands; `strings` del archivo con el bug muestra 1 `hdlr` frente a 2 en el archivo sano (falta el del árbol `meta`).
- **Causa raíz.** `lofty` 0.25.4 reescribe el archivo reconstruyendo el `meta` sin su hijo `hdlr` cuando el `ilst` fue eliminado por `remove --all` (`TagType::Mp4Ilst.remove_from`). El demuxer de ffmpeg exige `hdlr` para parsear `ilst`. `lofty` 0.25.4 es la última versión publicada (verificado con `cargo search lofty` 2026-10-16): **no hay fix upstream**; el fix es nuestro.
- **Opciones evaluadas.**
  - (a) **Post-proceso del archivo tras `save_to_path`:** reabrir el archivo y reinsertar (o reparar) el átomo `hdlr` en el `meta`. Coste: hay que parsear átomos MP4 o delegar en `lofty`; riesgo de corromper offsets si se hace a mano.
  - (b) **Manipulación en el camino de escritura:** en `write_mp4_tag`, en lugar de escritura plana, garantizar que el `Tag`/estructura que `lofty` serializa lleva el `hdlr` (p. ej. forzando un camino de escritura distinto, o un archivo temporal que pase por `TaggedFile` y reconstrucción). Coste: depende de la API de `lofty` 0.25.4 para MP4 (hay que comprobar si `Mp4Ilst` o `Ilst::new` ofrecen algún hook del árbol); si no, cae en (a).
  - (c) **Bump de `lofty`:** descartada — 0.25.4 es la última publicada.
- **Decisión.** El spec fija el **qué** (toda escritura MP4 produce `meta` con `hdlr` y las etiquetas son visibles por ffprobe tras cualquier secuencia); el **cómo** se elige al implementar entre (a) y (b) con la evidencia de qué permite la API de `lofty` 0.25.4, priorizando (b) si existe el hook y (a) si hay que reparar a mano (verificación por tests del árbol resultante). El test de integración de la suite usa `lofty` (relectura) más la comprobación de visibilidad; el oráculo externo de la visibilidad es el e2e con `ffprobe`.
- **Trade-off aceptado.** El fix puede implicar una escritura en dos pasos (mayor coste de I/O) a cambio de la compatibilidad real con reproductores externos, que es el objetivo del fix.

## Decisión 2 — Fix 1.2 (MP3 `--cover`): semántica de sustitución

- **Evidencia.** Reproducción: `edit --cover cover1.jpg` → `edit --cover cover2.jpg` sobre MP3 da `Carátulas: 2 imagen(es)` (CoverFront 2161 + CoverFront 2146 bytes).
- **Causa raíz.** `lofty` `Id3v2Tag::insert_picture` no deduplica: inserta un segundo `APIC`.
- **Decisión.** En el camino de edición de carátula MP3 (`add_cover_art` y/o `apply_metadata`), eliminar los frames `APIC` de tipo `CoverFront` existentes antes de insertar la nueva (`insert_picture`). Semántica fijada por el spec: se sustituye la del **mismo tipo** (CoverFront) y se **conservan** las de otros tipos. Alternativa descartada: eliminar todas las carátulas (rompería la conservación de `BackCover` y es más destructiva de lo necesario).
- **Generalización.** `push_mp4_cover` (MP4) y `push_ogg_cover` (OGG/Opus) también insertan pictures vía `lofty` y pueden acumular. El change prevé generalizar la semántica «sustituir la carátula del tipo indicado» a los backends donde el test RED demuestre la acumulación; donde ya se sustituya, el e2e fija el comportamiento y no se toca el código. Decisión final con evidencia al implementar.

## Decisión 3 — Contexto del change

- Un único change `e2e-findings-fixes` con tres deltas (m4a-tagging, mp3-tagging, e2e-verification): los dos bugs comparten origen (hallazgos del e2e), flujo (RED→GREEN) y ciclo de vida (commits `fix:`, PR a development, release en la Fase 4 del plan director). Alternativa descartada: dos changes separados (más overhead de planificación sin beneficio: no hay dependencia ni riesgo que aislar).
- Commit `fix:` (verificado: `release-prepare.yml:38` hace `grep -q "^feat"` → `--minor`; `fix:` NO libera minor). PR a `development`, nunca a `main` (la release se decide en el plan director, Fase 4).
- E2e: las aserciones nuevas se añaden a `e2e_m4a.sh` y `e2e_mp3.sh` (y los formatos que la evidencia de la Decisión 2 indique); no se crean scripts nuevos.
- Falsabilidad: tras ampliar, re-demostrar que una aserción corrompida del nuevo flujo termina en exit 1.

## Alternativas descartadas (resumen)

- Bump de `lofty` para el `hdlr` (no existe versión superior; 0.25.4 es la última).
- «Documentar el bug» en lugar de arreglar: rechazado por el usuario al priorizar la Fase 1 — la metadata invisible a reproductores externos es un defecto de compatibilidad real, no una limitación aceptable.
- Eliminar todas las carátulas al re-editar `--cover`: rechazado por destructivo (pierde `BackCover` y otros tipos legítimos).
- Fix solo en MP3 para la carátula: la generalización se decide por evidencia al implementar, no por supuesto.