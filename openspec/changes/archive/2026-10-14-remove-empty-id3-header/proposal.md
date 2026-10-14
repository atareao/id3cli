# Proposal

## Why

Durante la migración del backend de tags de MP3 del crate `id3` a `lofty` se añadió un apaño de compatibilidad en el guardado: cuando el tag queda vacío, `save_tag` (`src/main.rs`) reescribe el archivo a mano para anteponer una cabecera ID3v2 vacía de 10 bytes, porque la librería antigua lo hacía así y `lofty` borra el bloque entero. Ese código no aporta ninguna capacidad: su único propósito era preservar la paridad durante la migración.

Ahora que la migración está archivada y verificada, ya no hay nada que preservar y el apaño puede retirarse.

## What Changes

- Al eliminar el **último campo** de un MP3, el archivo MUST NOT quedarse con un bloque ID3v2 vacío: el bloque de metadatos desaparece y el archivo vuelve a contener solo audio, que es el comportamiento nativo de la librería.
- Desaparece el caso especial de guardado: guardar un tag deja de reescribir el archivo a mano y pasa a delegar en la librería en **todos** los casos.
- Los 7 tests de integración que hoy asumen que, tras borrar el último campo, sigue existiendo una etiqueta (y que fallan si no existe) se actualizan al contrato nuevo **sin debilitar ninguna expectativa**: pasarán a comprobar que no queda ninguna etiqueta y que el audio sobrevive.
- **Aviso explícito:** en ese caso límite (borrar el último campo) el efecto sobre el archivo **coincide** con el de `remove --all`. Se acepta de forma deliberada, porque es justo lo que el usuario ha pedido al borrar el último campo; la opción de conservar el apaño fue descartada.
- **No es un cambio BREAKING de la interfaz del CLI**: flags, mensajes y códigos de salida no cambian. Sí es un cambio **observable en el contenido del archivo** (el tamaño y la presencia del bloque ID3v2 vacío).

## Capabilities

### New Capabilities

Ninguna. Este cambio no introduce comportamiento nuevo, sino que retira un comportamiento heredado; no procede crear una capacidad nueva.

### Modified Capabilities

- `mp3-tagging`: se le añade un requisito sobre qué debe quedar en el archivo cuando se elimina el último campo del tag (el bloque de metadatos desaparece y solo queda audio). El delta se materializará en `specs/mp3-tagging/spec.md` como `## ADDED Requirements`, porque hoy el spec guarda silencio sobre este caso.

## Impact

- `src/main.rs`: desaparece el caso especial de guardado; los dos puntos que guardan un tag de MP3 (el de `edit` y el de `remove`) pasan a guardar directamente a través de la librería; y se elimina una importación que queda sin usar. **Este último detalle es crítico:** hay tests que comprueban que el CLI no emite nada por `stderr`, y un aviso del compilador por una importación sin usar acabaría en `stderr` y los haría fallar, por lo que la importación que quede huérfana MUST eliminarse en el mismo cambio.
- `tests/integration_test.rs`: 7 tests actualizan su relectura del archivo al contrato nuevo.
- `openspec/specs/mp3-tagging/spec.md`: se le añade el requisito (vía el delta de este cambio).
- Interfaz del CLI: **sin cambios** (mismos flags, mismos mensajes, mismos códigos de salida).
- Sin dependencias nuevas ni cambios en `Cargo.toml`.
