# Design

## Context

El comportamiento que este cambio retira es un apaño introducido durante la migración del backend de tags (`id3` → `lofty` 0.25.4). Al guardar un tag que queda vacío, la librería antigua dejaba en el archivo una **cabecera ID3v2 vacía de 10 bytes**, mientras que `lofty` elimina el bloque entero. Para conservar la paridad byte a byte se añadió un caso especial en `src/main.rs` que, cuando el tag queda vacío, borra el bloque y **vuelve a escribir el archivo anteponiendo la cabecera vacía a mano**: unas 11 líneas que leen y reescriben el archivo completo. Ese caso especial **solo se activa al eliminar el último campo**; con cualquier otro número de campos el guardado sigue el camino normal.

La extensión del apaño se midió con un experimento sobre una copia desechable del binario: sin él fallan **exactamente 7 tests de integración**, todos con el mismo error (`Failed to read tag: NoTag`), porque releen el archivo esperando encontrar una etiqueta.

La spec de `mp3-tagging` **no fija** este comportamiento: el único requisito que menciona borrado físico es el de `--all`, y solo dice que `--all` MUST NOT limitarse a vaciar los campos. Nada obliga a que `remove <campo>` deje una cabecera vacía. La migración ya está archivada y verificada, así que ya no hay paridad que preservar y el apaño solo conserva coste.

## Decisions

### 1. Retirar el apaño y dejar que el guardado delegue siempre en la librería

Al eliminar el último campo, el guardado pasa por el camino normal de la librería, de modo que el bloque ID3v2 desaparece del archivo. Es el comportamiento que la librería ya produce sin intervención y elimina la única manipulación binaria a mano del guardado.

Alternativa descartada: **conservarlo**, que es lo que se hizo durante la migración y por tanto la opción de mínimo diff. Se descarta porque mantiene 11 líneas de manipulación binaria a mano y 10 bytes de relleno en el archivo a cambio de nada funcional: el único efecto observable es que `show` informa de que hay etiquetas cuando en realidad no hay ninguna.

### 2. Aceptar que en ese caso límite `remove <campo>` y `remove --all` coincidan

Cuando el tag tiene un solo campo, eliminar ese campo y eliminar todos los campos producen el mismo archivo: un MP3 sin bloque ID3v2. Esto difumina la distinción entre ambos subcomandos en esa situación concreta y **se acepta de forma explícita**: en los dos casos el usuario ha pedido quedarse sin ese metadato, y el archivo sin bloque de metadatos es el resultado correcto. Queda constancia de que es una consecuencia conocida y no un descuido, y el escenario correspondiente del spec delta lo recoge como comportamiento aceptado deliberadamente.

### 3. Actualizar los 7 tests al contrato nuevo sin debilitarlos

Hoy esos tests releen el archivo con `.expect("Failed to read tag")` y se caen si no hay etiqueta, que es precisamente un efecto colateral del apaño. Pasarán a afirmar explícitamente que **no queda ninguna etiqueta** y que el audio sobrevive a la operación. Esto **refuerza** la verificación en vez de relajarla: la aserción pasa a ser precisa sobre el estado final del archivo, en lugar de depender de que exista una etiqueta para poder comprobar algo. Se mantiene además la verificación entre librerías distintas: la relectura sigue haciéndose con el crate antiguo `id3`, que se conserva como dependencia de desarrollo.

### 4. Eliminar en el mismo cambio la importación que quede sin usar

Hallazgo medido: hay tests de integración que comprueban que el CLI **no escribe nada por stderr**, y como invocan el binario a través de `cargo run`, un aviso del compilador por una importación huérfana acabaría en stderr y **haría fallar esos tests**. Por eso la importación que quede sin usar al retirar el caso especial MUST eliminarse en este mismo cambio, y no basta con que el código compile.

## Risks / Trade-offs

- **[Los tests que releen con la librería antigua usaban `.expect()` y se caen cuando no hay etiqueta]** → se reescriben para afirmar la ausencia de etiqueta y la supervivencia del audio, sin relajar ninguna otra expectativa; la verificación sigue haciéndose con el crate `id3` como lector independiente del escritor.
- **[Una importación huérfana rompería los tests que exigen `stderr` vacío]** → eliminarla en el mismo cambio (ver decisión 4); el criterio de cierre incluye el binario compilando sin avisos, no solo sin errores.
- **[El caso límite en que `remove <campo>` y `remove --all` coinciden]** → aceptado y documentado (ver decisión 2), con el compromiso de que quede reflejado también en el spec delta, para que la coincidencia sea contrato escrito y no comportamiento emergente.
- **[Compatibilidad con archivos ya tocados por versiones anteriores, que pueden llevar una cabecera ID3v2 vacía de 10 bytes]** → ese archivo sigue leyéndose y editándose con normalidad: la cabecera vacía no impide ninguna operación, y en la siguiente escritura el bloque vacío desaparece de forma natural al delegar el guardado en la librería.
