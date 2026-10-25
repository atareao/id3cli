# Design

## Problema

El artefacto precompilado es el único camino de instalación «sin compilar», y hoy arrastra una dependencia de la libc del sistema que lo inutiliza en distribuciones con glibc anterior a 2.34. Al mismo tiempo, el README promete una descarga que no existe. Ambas cosas son del mismo dominio —el artefacto publicado y su distribución— y se resuelven con un solo cambio, sin tocar el código del CLI.

## Decisión 1 — musl estático, sin coexistencia con gnu

Se construye `x86_64-unknown-linux-musl` y el artefacto gnu se retira. Un CLI Rust sin `dlopen`, sin NSS y sin código C propio no obtiene nada de enlazar dinámicamente la libc: el único efecto observable era el suelo de glibc. Publicar los dos targets obliga al usuario a elegir y duplica la matriz sin ventaja. Descartado también `+crt-static` sobre gnu (enlazado estático de glibc, desaconsejado por glibc y frágil ante NSS/dlopen) y «construir sobre base vieja» (mantiene el dinámico y añade complejidad de contenedor).

## Decisión 2 — Gate del estático en el pipeline, no solo comprobación manual

La verificación se codifica en el pipeline (`readelf -d` sin `NEEDED`, `objdump -T` sin `GLIBC_*`) y falla el job. Una propiedad que se comprueba a mano se pierde en el siguiente cambio de workflow; el repo ya aplica ese patrón (la falsabilidad del e2e). El gate convierte «estático» en un contrato ejecutable.

## Decisión 3 — Publicar el binario desnudo además del archivo

El README documenta `releases/latest/download/id3cli-linux-x86_64` y los assets reales son solo el `.tar.gz`: el comando publicado devuelve 404. Se publica además el binario desnudo (con el mismo nombre que el README documenta) manteniendo el `.tar.gz` y su `.sha256` para quien prefiera descarga auditable. Es la unión de lo documentado y lo verificable; la alternativa (reescribir el README para descomprimir) empeora la instalación para esconder el fallo.

## Decisión 4 — CI construye y prueba musl; el e2e sigue siendo local

`ci.yml` no ejecuta e2e hoy (solo fmt, clippy, build y test). El cambio añade un job musl que construye, verifica el estático y corre la suite completa contra el target, sin arrastrar la matriz e2e a la nube: mantiene el límite actual del pipeline y evita depender de `ffprobe`/`jq` del runner. La matriz e2e se ejecuta contra el binario musl en local —es el oráculo de siempre— y queda registrada en las tasks.

## Decisión 5 — No-rotura medida, no asumida

El cambio de libc podría, en teoría, alterar salidas (formato de números, orden, tamaños impresos). No se asume: se capturan `show` y `show --json` con ambos binarios sobre los cinco fixtures y se comparan byte a byte. Si apareciera una diferencia, es un hallazgo de la fase RED y se trata antes de tocar el pipeline.

## Decisión 6 — Verificación local: sondeo del toolchain y respaldo con contenedor

La máquina de desarrollo no tiene `musl-gcc`, ni `apt`, ni `sudo`: el build musl local depende de que `rustup target add x86_64-unknown-linux-musl` + `rust-lld` (self-contained) enlace sin linker externo. Si el sondeo falla, el respaldo es construir en un contenedor —hay `podman` en `/usr/bin/podman`— con una imagen de toolchain musl. La fuente de verdad del artefacto publicado es CI; lo local es evidencia.

## Alternativas descartadas

- Publicar gnu y musl en paralelo: duplica la matriz y obliga a elegir sin ventaja real.
- `+crt-static` sobre `x86_64-unknown-linux-gnu`: enlazado estático de glibc, desaconsejado y frágil.
- Construir sobre una base antigua (estilo manylinux): mantiene el dinámico y añade complejidad de contenedor.
- `panic = "abort"`: optimize independiente que cambia la estrategia de pánico; fuera de alcance.
- `aarch64-unknown-linux-musl` y binarios Windows/macOS: fuera de alcance de este change.
- Reimplementar la matriz e2e en CI: amplía el alcance del pipeline y depende de herramientas del runner.

## Riesgos y mitigaciones

- **`musl-gcc` ausente en local**: sondeo del toolchain + respaldo con `podman`; CI como fuente de verdad del artefacto.
- **El enlace musl cambia algún byte de la salida**: medido en RED contra el binario gnu de la misma versión antes de tocar el pipeline.
- **El nombre del archivo cambia** (`...-linux-gnu` → `...-linux-musl`): el cuerpo del release deriva el nombre del fichero, y el README documenta el binario desnudo, cuyo nombre NO cambia.
- **Fila del binario desnudo en el cuerpo del release**: el `sed` actual podría producir una etiqueta fea; se ajusta la generación o se excluye el desnudo de la tabla (task 4.3).
- **`--locked` añadido al build de release**: alinea con la regla del repo (CI con `--locked`); si el lock y el manifiesto divergieran, el release falla en vez de actualizar el lock en silencio.
- **La suite completa contra musl alarga CI**: aceptado a cambio de no descubrir la rotura al publicar el tag.
