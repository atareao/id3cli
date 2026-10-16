# Design

## Problema

Aplicar la misma edición a varios archivos hoy requiere un bucle de shell (`for f in *.mp3; do id3cli edit...; done`), que duplica invocaciones del proceso, ignora las degradaciones por formato del CLI real y fragmenta el contrato de avisos. La solución debe ser aditiva (sin tocar el comportamiento con un único archivo), testable en TDD y coherente con la arquitectura del repo (lógica por archivo en lib.rs, salida solo por la capa `output.rs`).

## Decisión 1 — `<FILE>...` en los subcomandos existentes, no un subcomando `batch`

`show`, `edit` y `remove` aceptan `Vec<PathBuf>` (`num_args = 1..`, `required = true`). Es el patrón de rm/cp/ffmpeg y mantiene la superficie plana del CLI; un subcomando `batch` aparte duplicaría el contrato completo de opciones. Sin archivos → clap responde error de argumento requerido (exit 2), igual que hoy.

## Decisión 2 — No-rotura estricta con N=1

Con un único archivo, la salida (legible, JSON, avisos, exit) es byte a byte la actual. Internamente el batch itera; el caso N=1 no antecede cabecera ni cambia el formato. Se verifica contra el binario de referencia v0.9.x (`0bed51f`) guardado antes del cambio.

## Decisión 3 — `show` con N>1: cabecera legible y NDJSON

- Legible: cada bloque se antecede de `=== <ruta> ===` (una línea, sin depender del modo de color). Con N=1 no hay cabecera.
- `--json`: NDJSON — una línea JSON por archivo en el orden dado (p. ej. `jq -s .` devuelve el array). Con N=1, la línea única actual intacta. Cada objeto es exactamente el `ShowJson` actual del archivo (mismo esquema `{formato, tags}`).

## Decisión 4 — Exit agregado: 0 solo si todos OK; los fallos no abortan el resto

El batch procesa TODOS los archivos: si uno falla (inexistente, formato no soportado, error de escritura), los demás se procesan igual; al final, si hubo algún fallo, se informa por stderr (mensaje en español con la ruta y el motivo, uno por archivo fallido) y el proceso termina con exit ≠ 0. Solo con todos OK termina con exit 0. Los avisos (sin tags, degradaciones MP4/OGG) conservan su contrato por archivo y NO cuentan como fallo.

## Decisión 5 — Formato mixto

`edit --title X a.mp3 b.m4a c.ogg` despacha cada archivo a su backend vía `detect_supported_format`: las degradaciones por formato (p. ej. `--url` en M4A/OGG) se emiten por stderr SOLO para el archivo que las sufre; el resto se edita sin aviso.

## Decisión 6 — Completions sin cambios

El script de completions se regenera sobre el Command real: clap_complete emitirá `show <FILE>...` automáticamente. Ningún cambio en esa capa; solo la verificación de que el script generado refleja los tres subcomandos con argumentos múltiples.

## Decisión 7 — e2e dedicado

`scripts/e2e/e2e_batch.sh` genera 2-3 audios (mp3 + m4a; reutiliza `e2e_gen_audio`), aplica `edit` batch, verifica con `show` y ffprobe que TODOS se editaron, valida el NDJSON con `jq -s` (n líneas = n archivos), comprueba que un archivo inexistente produce rc ≠ 0 mientras el resto se edita, y re-demuestra la falsabilidad (corromper una aserción → exit 1). La recipe `just e2e` pasa a ejecutar 5 formatos + completions + batch en serie con una sola construcción.

## Decisión 8 — Test documental intacto

Tras GREEN se miden la suite (unit + integración) y el e2e (aserciones por script) y en REFACTOR docs se actualizan README (escudo, desglose, sección modo batch, roadmap), AGENTS.md y copilot-instructions con los recuentos medidos. Las keywords de Cargo.toml NO se tocan; Cargo.lock NO cambia (ninguna dependencia nueva).

## Alternativas descartadas

- Subcomando `batch`: duplica el contrato de opciones; menos estándar.
- Wildcards propios/globbing en el CLI: la expansión es del shell; el CLI recibe la lista ya expandida.
- Paralelismo: fuera de alcance; el batch es secuencial y determinista.
- Salida JSON como array en una sola línea con N>1: rompe el contrato «una línea por `show`» y complica el streaming; NDJSON es lo consumible por `jq -s`.

## Riesgos y mitigaciones

- **No-rotura N=1**: verificada byte a byte contra el binario de referencia antes/después del GREEN (guarda en `/tmp/opencode/v091_ref/`).
- **Formato exacto de la cabecera**: fijado por medición en GREEN (`=== <ruta> ===`), estable y sin dependencia de color.
- **Mensajes de clap al variar el value_name**: `FILE` → `FILE` (plural implícito por `...`); los tests de integración existentes que usan un solo archivo no se ven afectados; si el mensaje de «falta el archivo» cambia de forma, se ajusta SOLO en tests nuevos.
- **Recuentos de tests suben**: se miden y se reflejan en README/AGENTS/copilot en REFACTOR (test documental como red de seguridad).