# Tasks

## 1. RED: fijar el contrato nuevo en los tests

El contrato nuevo es el contrario del apaño actual: al eliminar el último campo, el bloque ID3v2 desaparece y el archivo queda solo con audio. Este grupo fija ese contrato en los tests antes de tocar el código de producción.

- [ ] 1.1 Actualizar los 7 tests de integración afectados para que comprueben el contrato nuevo: tras eliminar el último campo MUST NOT quedar ninguna etiqueta en el archivo y el audio MUST sobrevivir. Los tests son exactamente: `test_cli_remove_lyrics`, `test_cli_remove_url`, `test_cli_remove_compilation`, `test_cli_remove_apple_sort_orders`, `test_cli_remove_new_tags`, `test_cli_remove_season` y `test_cli_remove_mixed_language_names`.
      Verificación: `cargo test --no-fail-fast` MUST fallar exactamente en esos 7 tests (el código todavía conserva el bloque vacío) y MUST NOT fallar ningún otro test.
- [ ] 1.2 Comprobar que ningún test ha visto relajada su expectativa: solo cambia la comprobación del estado final del archivo, y la relectura sigue haciéndose con la librería antigua, de modo que la verificación sigue siendo entre librerías distintas.
      Verificación: el recuento total de tests MUST seguir siendo 214 y el diff de `tests/integration_test.rs` MUST NOT eliminar ninguna aserción que no sea la relectura afectada.

## 2. GREEN: retirar el apaño

Se retira el caso especial de compatibilidad que quedó de la migración del backend de MP3.

- [ ] 2.1 Simplificar `save_tag` en `src/main.rs` para que delegue siempre en la librería: eliminar el caso especial del tag vacío y la reescritura manual del archivo que anteponía una cabecera ID3v2 vacía de 10 bytes.
      Verificación: los 7 tests de la tarea 1.1 MUST pasar y `cargo test --no-fail-fast` MUST quedar en verde con 214 tests y 0 fallos.
- [ ] 2.2 Eliminar la importación que queda huérfana al retirar el caso especial.
      Verificación: `cargo build` MUST NOT emitir ningún aviso, y los tests que exigen que el CLI no escriba nada por `stderr` MUST pasar (son los que verifican que no se emite ningún aviso del compilador, ya que invocan el binario con `cargo run`).

## 3. REFACTOR y limpieza

- [ ] 3.1 Comprobar el formato.
      Verificación: `cargo fmt --check` sin diferencias.
- [ ] 3.2 Comprobar el linter en todos los targets y features.
      Verificación: `cargo clippy --all-targets --all-features -- -D warnings` sin warnings.
- [ ] 3.3 Comprobar que la interfaz del CLI no ha cambiado y que el comportamiento de MP4 no se ve afectado. El cambio es puramente de comportamiento sobre el contenido del archivo: no cambia flags, ni mensajes, ni códigos de salida.
      Verificación: comparar la superficie de flags de `cargo run -- --help`, `cargo run -- show --help`, `cargo run -- edit --help` y `cargo run -- remove --help` con la de antes del cambio (MUST ser idéntica) y ejecutar la suite completa en verde con 214 tests.

## 4. Cierre

- [ ] 4.1 Ejecutar la suite completa.
      Verificación: `cargo test` en verde, 0 fallos, con 214 tests anotados: no se añaden ni se eliminan tests, solo se actualizan 7.
- [ ] 4.2 Validar el cambio contra OpenSpec.
      Verificación: `openspec validate remove-empty-id3-header --strict` sin errores.
- [ ] 4.3 Verificación manual de extremo a extremo sobre un MP3 real: anotar el tamaño del archivo, escribir un campo con `edit`, borrar ese campo con `remove`, y comprobar que el archivo vuelve exactamente a su tamaño y contenido originales.
      Verificación: el tamaño antes y después MUST coincidir, el contenido MUST ser idéntico al original y `show` MUST informar de que no hay etiquetas.
- [ ] 4.4 Archivar el cambio.
      Verificación: `openspec archive remove-empty-id3-header` listo para ejecutar; requiere la aprobación previa del usuario.
