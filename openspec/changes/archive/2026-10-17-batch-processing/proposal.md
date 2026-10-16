# Proposal

## Why

Un usuario que quiere aplicar la misma edición a una colección (un álbum, un directorio de episodios de podcast) hoy debe invocar `id3cli edit` una vez por archivo —o escribir un bucle de shell—, y `show`/`remove` sufren lo mismo. El modo batch (`<FILE>...` en `show`, `edit` y `remove`) es la pieza de ergonomía estándar de un gestor de metadatos multiarchivo: la misma operación, N archivos, un solo proceso, una sola lectura del CLI. Sin esto, el batch vive en scripts ajenos que se desincronizan de las reglas reales (degradaciones MP4/OGG, avisos por stderr, exit codes).

## What Changes

Los subcomandos `show`, `edit` y `remove` pasan de `FILE` único a `<FILE>...` (1..n) — `Vec<PathBuf>` con `num_args = 1..` y `required = true` en clap. La operación se aplica a todos los archivos en el orden dado, reutilizando la lógica por archivo existente (despacho por `detect_supported_format` a cada backend). Con `show` y N>1: modo legible antecede cada bloque con la cabecera `=== <ruta> ===`; `show --json` emite NDJSON (una línea JSON por archivo, consumible con `jq -s`). Con N=1 la salida es byte a byte la actual (no-rotura estricta, verificada contra el binario de referencia). Exit agregado: 0 solo si TODOS los archivos se procesan; si alguno falla, el resto se procesa igual y los fallos se informan por stderr con exit ≠ 0. Sin archivos → error de clap (exit 2). El script de completions refleja automáticamente `<FILE>...` (clap_complete regenera sobre el Command real). Se añade el e2e `e2e_batch.sh` y la recipe `just e2e` pasa a cubrir 5 formatos + completions + batch.

## Capabilities

### New Capabilities

- **batch-processing**: contrato del modo batch — `<FILE>...` (1..n) en show/edit/remove, despacho por archivo al backend correcto, cabecera `=== <ruta> ===` en show legible con N>1, NDJSON en `show --json` con N>1, no-rotura N=1, exit agregado (0 solo si todos OK; fallos por stderr y proceso continúa), sin archivos → error de clap.

### Modified Capabilities

- **cli-output**: la superficie del CLI pasa de `FILE` a `<FILE>...` en los tres subcomandos; el contrato de pipe cerrado cubre la salida batch; los avisos por archivo siguen el contrato de stderr.
- **e2e-tooling**: el runner cubre un script nuevo no ligado a una sola extensión (`e2e_batch.sh`); la recipe `just e2e` pasa a ejecutar 5 formatos + completions + batch.
- **e2e-verification**: la matriz gana la capa batch (edit batch verificado con show/ffprobe por archivo, NDJSON con `jq -s`, archivo inexistente → rc ≠ 0 con el resto editado, falsabilidad).

## Impact

- `src/main.rs`: `file: PathBuf` → `files: Vec<PathBuf>` en Show/Edit/Remove (`#[arg(required = true, num_args = 1..)]`); bucles de despacho; cabeceras y NDJSON; exit agregado.
- `src/lib.rs`: funciones de batch testables (cabecera de bloque, juntado NDJSON, ejecución por archivo con acumulación de fallos) reutilizando la lógica por archivo existente.
- `src/tests.rs` (+unitarios), `tests/integration_test.rs` (+tests CLI batch), sin cambios de dependencias (no se añade ningún crate; Cargo.lock intacto — regla `--locked` intacta).
- `scripts/e2e/e2e_batch.sh` (nuevo), `.justfile` (recipe `e2e` ampliada y recipe `e2e-batch`).
- README/AGENTS.md/copilot-instructions (sección modo batch + recuentos de tests medidos + roadmap).
- No-rotura: con N=1 las salidas legible y JSON son byte a byte la actuales (verificado contra el binario de referencia v0.9.x); el test documental exige recuentos medidos y keywords intactas (`["id3", "mp3", "mp4", "metadata", "cli"]`).
- No-goals: subcomando `batch` aparte, paralelismo, aplicar operaciones DIFERENTES por archivo en una sola invocación, wildcards/glob propios (la expansión la hace el shell).