# Design

## Context

Este cambio tiene dos partes independientes: la carrera entre el push de la rama y el del tag en el pipeline de release, y el contenido del paquete publicado. La motivación de ambas está en `proposal.md`; aquí no se repite. Este documento fija cómo se resuelve cada una, qué alternativas se descartaron y por qué, y cómo se verificará.

Hechos de referencia sobre los que se construye el diseño, medidos y comprobados en vez de supuestos:

- **El pipeline hace dos push en el mismo paso.** El paso `Commit, tag and push` de `release-prepare.yml` ejecuta `git push … main` y, acto seguido, `git push … "v${V}"` contra la misma URL. El run **38527464056** aceptó el de la rama (`8fb0583..5a070b6 main -> main`) y rechazó el del tag con `! [remote rejected] v0.5.1 -> v0.5.1 (missing necessary objects)`.
- **La causa es la latencia, no el permiso.** Entre los dos push se midieron **0,49 s** en el run que falló, frente a **1,1 s** (run 38224385769) y **1,3 s** (run 38517244065, release 0.5.0) en los que funcionaron. El segundo push no transporta objetos —el cliente ya cree que el servidor los tiene— y la réplica que valida la referencia no había ingerido todavía el pack del primero. Es consistencia eventual del lado del servidor.
- **El fallo deja un estado a medias con dos trampas.** `main` queda con `0.5.1` y sin tag, `release.yml` no se dispara y el paso posterior `Sync development with main` se saltea. Reejecutar el job **no** es una recuperación válida, porque el bump se recalcula desde el estado actual (`vampus upgrade --patch` desde `0.5.1` daría `0.5.2`) y produciría una **release fantasma**. Y un tag empujado por una persona sí dispara `release.yml` (run 38527751799, `event: push`), mientras que los del propio pipeline con su `GH_PAT` no lo disparan (los cuatro últimos releases —`v0.4.1`, `v0.4.2`, `v0.4.3` y `v0.5.0`— figuran con `event: workflow_dispatch`), de modo que el dispatch explícito convive con el riesgo de doble publicación si alguien empuja el tag a mano y además lanza el dispatch.
- **`--atomic` está medido contra este remoto.** Con referencias desechables (rama `prueba/atomic-ff`, tags `prueba-atomic-*`; 0 refs de prueba en el remoto hoy), un push atómico de una rama no fast-forward junto con un tag nuevo produjo `! [rejected] 5ba4e85 -> prueba/atomic-ff (non-fast-forward)` y `! [rejected] prueba-atomic-ff -> prueba-atomic-ff (atomic push failed)`, con el tag sin crear y la rama intacta en `5a070b6`. El control sin `--atomic` creó el tag (`* [new tag] prueba-atomic-ctrl`) y solo rechazó la rama.
- **El paquete publicado lleva 54 ficheros, incluido el andamiaje.** Medido con `cargo package --list`: los tres workflows de `.github/workflows/*`, `.github/copilot-instructions.md`, todo `openspec/**` (configuración, specs y los changes archivados), `AGENTS.md`, `GIT_FLOW.md`, `RELEASE.md`, `cliff.toml`, `.justfile`, `.vampus.yml` y `.gitignore`.
- **Los fixtures son diminutos.** `tests/fixtures/` pesa **9.031 bytes** en total: `sample.mp3` 2.481, `sample.m4a` 3.295 y `generate_sample_m4a.sh` 3.255 bytes, así que el paquete publicado puede conservar su suite ejecutable a un coste irrelevante.
- **La verificación del manifiesto ya tiene precedente en el repositorio.** Los tests `test_manifiesto_keywords_validos` y `test_manifiesto_categories_validas` de `src/tests.rs` leen `Cargo.toml` con `include_str!("../Cargo.toml")` y fijan la **declaración** sin depender del directorio de trabajo; el **efecto** de los metadatos lo comprueba `cargo publish`, no un test.

**Nota de alcance: los requisitos de pipeline que ya viven en `crate-metadata` no se mueven.** `crate-metadata` ya contiene el refresco del lock antes del commit de release y el `--locked` de la integración continua, que son contratos del pipeline sin relación con la carrera del push ni con el contenido del paquete. Moverlos a `release-publishing` sería un refactor de spec sin necesidad: cambiaría la ubicación de requisitos ya aprobados y archivados sin aportar nada al problema que aquí se resuelve, y obligaría a una migración de delta a delta. `release-publishing` cubre **contratos nuevos** —la atomicidad del push y la lista blanca del paquete—, no una reubicación de lo existente. Por eso el change no modifica ninguna capability y se limita a añadir la nueva.

## Goals / Non-Goals

**Goals:**

- Que un fallo del push deje el repositorio **sin commit de release, sin tag y sin publicación**, de modo que el siguiente push a `main` reintente la release desde el mismo estado, sin doble bump.
- Empujar `main` y el tag en una **única** operación atómica, con reintentos acotados y fallo explícito antes de disparar la publicación.
- Que el paquete publicado contenga solo los ficheros publicables del proyecto y **no** el andamiaje del repositorio.
- Conservar dentro del paquete la suite de tests y sus fixtures (9.031 bytes) para que el paquete publicado siga siendo comprobable.
- Que `Cargo.lock` siga viajando en el paquete, porque se publica un binario y la compilación debe ser reproducible con las versiones del lock.
- Verificar la declaración con un test unitario del manifiesto y el efecto con `cargo package --list` y `cargo publish --dry-run`.

**Non-Goals:**

- Publicar nada: este change deja el pipeline y el manifiesto listos para la **próxima** release, no adelanta una.
- Mover a `release-publishing` los requisitos del pipeline que ya viven en `crate-metadata` (ver la nota de alcance en `Context`).
- Proteger del fallo del paso posterior `gh workflow run` ni eliminar la posibilidad de doble disparo; ambas cosas son riesgo residual, no objetivo (ver `Risks / Trade-offs`).
- Cambiar el comportamiento del binario: ni flags, ni mensajes, ni códigos de salida, ni el manejo de tags de MP3 y MP4, ni la detección de formato.
- Ejecutar `cargo` desde la suite de tests (ver la decisión 5).

## Decisions

### 1. `--atomic` en un único push, en lugar de dos push separados con espera, del orden inverso o de reintentar solo el tag

La decisión es sustituir los dos `git push` del paso por **uno solo** que lleva las dos referencias:

```
git push --atomic "$REMOTE" main "v${V}"
```

Con `--atomic`, el servidor acepta el push **entero** o lo rechaza **entero**: si el tag no se puede crear, la actualización de `main` se deshace, y al revés. El experimento medido confirma el punto: con una rama no fast-forward y un tag nuevo en el mismo comando, ambos se rechazaron y el tag no llegó a crearse, mientras que el control sin `--atomic` creó el tag y dejó la rama sin actualizar. Esa conversión del fallo parcial en fallo total es lo que hace que el estado a medias sea **imposible por construcción** y no solo improbable.

**Alternativa descartada: mantener los dos push separados y añadir un `sleep` entre ellos.** No elimina la carrera, solo la hace menos probable: la ventana de consistencia del servidor no tiene una duración garantizada, así que un `sleep` corto puede no bastar y uno largo castiga todos los releases que sí funcionan. Y sobre todo deja intacto el problema de fondo: cuando el segundo push falla, el primero ya actualizó `main`, que es justo el estado a medias que se quiere evitar. Se descarta por no resolver el fallo parcial.

**Alternativa descartada: empujar primero el tag y luego la rama.** La carrera se reproduce al revés —el tag llegaría primero y la rama después— y el estado a medias cambiaría de forma pero no de gravedad: quedaría un tag sin la rama actualizada. Se descarta por simétrica e igualmente rota.

**Alternativa descartada: reintentar solo el tag.** Si el reintento del tag también falla, `main` ya quedó actualizado sin tag, que es exactamente el estado a medias que produce el bug actual. Un reintento parcial mitiga el síntoma más visible (el rechazo del tag) pero no el requisito de que un fallo deje el repositorio limpio. Se descarta porque la garantía buscada es de estado, no de probabilidad.

### 2. Reintentos con espera y fallo explícito, en lugar de confiar en que el push funcione a la primera

Aunque `--atomic` garantice que un fallo no deja estado a medias, el push puede fallar de verdad —por la misma latencia de réplica u otra transitoria—. La decisión es reintentarlo un número acotado de veces (hasta tres intentos) con una espera entre intentos (cinco segundos), y si se agotan, **fallar de forma explícita**: un `::error::` y salida distinta de cero **antes** de llegar al `gh workflow run`. Así el pipeline grita en vez de seguir adelante, y el disparo de la publicación no ocurre nunca sobre un estado incompleto.

**Alternativa descartada: confiar en el push único a la primera.** Es lo que hace hoy y es lo que falló una vez en cuatro releases. Con un fallo de red transitorio, el pipeline se cae sin reintentar aunque el siguiente intento habría bastado, y deja al operador la recuperación manual con sus dos trampas. Se descarta porque el coste de tres intentos es de segundos y el de una release a medias es de intervención humana.

### 3. `include` (lista blanca) en vez de `exclude` (lista negra)

Para acotar el paquete se declara en `[package]` de `Cargo.toml` la lista blanca `include`: `src/**`, `tests/**`, `Cargo.toml`, `Cargo.lock`, `README.md`, `LICENSE` y `CHANGELOG.md`. Todo lo que no está en la lista deja de viajar al paquete, así que hoy desaparece el andamiaje completo (los tres workflows, `.github/copilot-instructions.md`, todo `openspec/**`, `AGENTS.md`, `GIT_FLOW.md`, `RELEASE.md`, `cliff.toml`, `.justfile`, `.vampus.yml` y `.gitignore`).

**Alternativa descartada: `exclude` con la lista negra del andamiaje.** Obligaría a enumerar lo que **no** queremos, y cualquier fichero interno que se añada en el futuro —un workflow nuevo, una carpeta de tooling, un documento de proceso— volvería a colarse en el paquete sin que nadie lo note, porque la lista negra no lo menciona. Un `exclude` es correcto solo mientras su enumeración se mantenga completa, y nada en el repositorio obliga a mantenerla. La lista blanca invierte la carga de la prueba: lo que no se declara explícitamente como publicable no se publica, así que el andamiaje futuro queda fuera por defecto.

### 4. Mantener `tests/**` con sus fixtures dentro del paquete publicado, en lugar de excluir los tests

El paquete conserva la suite de tests y sus fixtures porque los tres ficheros de `tests/fixtures/` suman **9.031 bytes** —`sample.mp3` 2.481, `sample.m4a` 3.295 y `generate_sample_m4a.sh` 3.255—, un coste de empaquetado irrelevante.

**Alternativa descartada: publicar sin tests.** Dejaría el paquete publicado sin forma de comprobarse: quien lo descargue para auditarlo o para reproducir la verificación no tendría ni los tests ni los ficheros de entrada con los que ejercitarlos. El ahorro de 9 KB no compensa perder la trazabilidad del artefacto publicado. Se descarta por coste/beneficio adverso.

### 5. La verificación del contenido del paquete se apoya en `cargo package --list` y `cargo publish --dry-run`, no en un test de la suite que ejecute `cargo`

Son **dos comprobaciones** las que cierran este requisito, ninguna de ellas un test de la suite:

- `cargo package --list` enumera el contenido real del paquete: el andamiaje MUST NOT aparecer y los ficheros necesarios —incluidos `tests/**`, `Cargo.lock` y `CHANGELOG.md`— MUST seguir presentes.
- `cargo publish --dry-run` empaqueta, compila y verifica el paquete sin publicarlo, que es la última red antes de una publicación real.

El **test unitario** que se añade a la suite fija la **declaración** del manifiesto —que `include` contiene la lista blanca acordada— leyendo `Cargo.toml` con `include_str!("../Cargo.toml")`, sin depender del directorio de trabajo. Es el mismo reparto que ya usa el repositorio con `keywords` y `categories`: el test fija la declaración, la orden externa comprueba el efecto.

**Alternativa descartada: un test de la suite que ejecute `cargo package --list`.** Dependería de que `cargo` esté instalado, del directorio de trabajo y de la resolución del índice de dependencias, de modo que sería frágil y ajeno a lo que un test unitario debe garantizar. La separación declaración/efecto que ya usa el proyecto es más robusta y no añade acoplamiento de la suite a la cadena de herramientas de empaquetado.

### 6. La documentación de la recuperación de un release a medias va a `RELEASE.md` como tarea, no como requisito del spec

El **requisito** de la capability es que un fallo deje el estado limpio, y con la atomicidad ya lo hace: `main` sin el commit de release, sin tag y sin publicación. Eso es un contrato verificable y pertenece al spec. El **conocimiento operativo** —que no se debe reejecutar el job fallido porque el bump se recalcula y produciría una release fantasma, y que no se debe lanzar el dispatch si el tag ya disparó la publicación porque se publicaría dos veces— no es un contrato que una prueba pueda fijar: son trampas de operación humana. Por eso va a `RELEASE.md` como **tarea del change**, con la explicación de cómo recuperar un release a medias (crear el tag sobre el commit de release y empujar `main:development`) y de cuándo no intervenir.

**Alternativa descartada: convertir las trampas en requisitos del spec.** Un requisito como «un operador MUST NOT reejecutar el job fallido» no es verificable de forma automática ni describe un comportamiento del sistema, sino una advertencia sobre su uso; meterlo en el spec lo degradaría a prosa no comprobable y diluiría el contrato real. Se descarta por higiene del spec.

## Risks / Trade-offs

- **`--atomic` es una capacidad del servidor y se ha verificado contra este remoto, no de forma general.** El experimento se hizo contra `github.com/atareao/id3cli` y confirmó que el servidor soporta la operación y la rechaza como un todo. Si el remoto cambiara de servidor o de política, `--atomic` podría no estar disponible; el push fallaría de forma explícita y visible, y la corrección sería de una vuelta atrás a los dos push, con el problema original de nuevo sobre la mesa.
- **`--atomic` no protege del fallo del paso posterior `gh workflow run`.** El riesgo residual es real y ya se observó el 2026-10-12, cuando ese disparo falló con `gh: Bad credentials (HTTP 401)`. Si el tag existe —atomicidad resuelta— y el disparo no se hace, la publicación queda pendiente de un dispatch manual. Este change no lo arregla: el estado es limpio en cuanto a rama y tag, pero no autopublicable si el token falla.
- **El doble disparo sigue siendo posible y se mitiga solo con la documentación.** Un tag empujado a mano dispara `release.yml` por `event: push`, y el dispatch explícito del pipeline lo dispara también; si coinciden, se publica dos veces. La mitigación es de proceso —`RELEASE.md` lo advierte—, no técnica.
- **El `include` hay que mantenerlo cuando se añada un fichero necesario al proyecto.** Si en el futuro se añade un fichero que deba viajar al paquete y se olvida declararlo en la lista blanca, el paquete publicado podría no compilar. Se mitiga con `cargo publish --dry-run` en el cierre del change y en adelante, que empaqueta y compila el contenido real antes de publicar.
- **Este change no publica nada.** Deja el pipeline y el manifiesto listos para la **próxima** release, así que su efecto no se observa en un release real hasta que este change esté en `main` y se dispare uno nuevo. Es una consecuencia aceptada: cualquier arreglo del pipeline solo surte efecto en el release siguiente.

## Migration Plan

El orden es estricto y cada paso se cierra con la suite verde.

1. **RED.** Añadir el test unitario del manifiesto que fija la declaración de `include` (leído con `include_str!("../Cargo.toml")`) y medir el estado de partida: `cargo package --list` mostrando los 54 ficheros con andamiaje, y la ausencia de atomicidad con el experimento de referencias desechables. El test debe estar en rojo antes de tocar `Cargo.toml`.
2. **GREEN.** Declarar la lista blanca `include` en `[package]` de `Cargo.toml` y reescribir el paso `Commit, tag and push` de `release-prepare.yml` como un único `git push --atomic` con reintentos y fallo explícito. Al final de este paso, el test unitario pasa y `cargo package --list` ya no muestra el andamiaje.
3. **REFACTOR y documentación.** Limpiar el paso del pipeline y documentar en `RELEASE.md` el procedimiento de recuperación de un release a medias y el contenido del paquete, en `AGENTS.md` la convención del push atómico y del contenido publicado, y en `README.md` qué lleva el paquete publicado. Reejecutar la suite tras la limpieza.
4. **Cierre.** Suite completa en verde, `openspec validate --strict`, `cargo package --list` confirmando que el andamiaje no aparece y que los ficheros necesarios siguen, `cargo publish --dry-run` empaquetando y compilando sin publicar, y la comprobación del estado limpio ante un fallo de push —con las referencias desechables— de modo que un rechazo no deje rama ni tag.

**Aviso operativo:** el arreglo del pipeline solo surte efecto en el **próximo** release, porque el workflow que se ejecuta es el de la rama que dispara el evento, no el de la rama donde se desarrolla el cambio.

**Rollback:** el proyecto sigue Git Flow, así que este change se desarrolla en la rama `fix/release-push-race-and-package-contents` sobre `development` y no se mezcla hasta que la suite completa está verde. Las dos partes son independientes entre sí: revertir la del paquete no afecta a la del push ni al contrario, así que se pueden revertir por separado.
