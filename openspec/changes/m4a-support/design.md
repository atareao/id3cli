# Design

## Context

El proyecto usa hoy `id3` 1.16.4, un crate que da acceso directo a los frames ID3 (`Tag`, `Frame`, `Content::Lyrics`, `Picture`) y que es la base de toda la API pública de `src/lib.rs` (`apply_metadata`, `add_lyrics`, `add_url`, `add_apple_metadata`, `add_cover_art`, `remove_tags`, `remove_all_id3`, `display_tags`). Esa API recibe `&mut id3::Tag`, así que cualquier cambio de backend es necesariamente un cambio de firma.

Un `.m4a` es otro contenedor y otro modelo de tags. Físicamente es un archivo ISO-BMFF con estructura de átomos y los metadatos viven en la ruta `moov.udta.meta.ilst`, como pares clave-valor de cuatro caracteres donde los átomos de copyright empiezan por `©`. `id3` no conoce nada de eso, y por tanto la extensión del archivo no es la única pieza que falta: falta el modelo de datos completo.

La situación de partida documental es que `openspec/specs/` está vacío: no hay ninguna spec previa del proyecto. Este cambio crea las tres primeras capacidades (`format-detection`, `mp3-tagging`, `m4a-tagging`) y, de paso, sirve como caracterización del comportamiento actual que exige la Phase 0 del `AGENTS.md` antes de modificar código heredado sin cobertura documental.

Hechos de referencia sobre los que se construye el diseño, en vez de inventarlos:

- **Baseline medido hoy:** 63 tests unitarios + 43 de integración = 106 tests en verde, 0 fallos. `AGENTS.md` y `README.md` siguen diciendo "99 tests (59 unit + 40 integration)": cifra desactualizada que este cambio corrige.
- **Mapeo verificado** contra la tabla `ILST_MAP` de `lofty` 0.25.4 y contra el código actual de `src/lib.rs`. La tabla del spec delta de `m4a-tagging` es la referencia autoritativa; el diseño no la repite. Punto delicado que queda constancia explícita: `--year` se escribe en `TYER` (frame de ID3v2.3, obsoleto en ID3v2.4) y `--date` en `TDRC`, y son frames distintos; el mapeo se ha verificado contra `src/lib.rs` y contra el fuente del crate `id3` 1.16.4.
- **Restricción de `covr`:** solo admite los tipos well-known JPEG (13), PNG (14), GIF (12) y BMP (27). WebP no existe en esa tabla y degrada a `Reserved(0)`.
- **Entorno:** `ffmpeg` está disponible en la máquina, lo que permite generar fixtures `.m4a` reales y deterministas. `lofty` 0.25.4 y `mp4ameta` 0.13.0 existen en crates.io.
- **Hoy no existe ninguna detección de formato**, ni por extensión ni por contenido: el CLI entrega la ruta directamente a `id3::Tag::read_from_path`, y la extensión del nombre se usa únicamente en `detect_mime_type` para el MIME de la carátula. Consecuencia observable: al no reconocer el contenedor, sobre un archivo no-MP3 el CLI no encuentra tags y, al escribir, genera un tag ID3v2 sobre un contenedor que no lo entiende, invalidando el archivo para los reproductores MP4. Esa consecuencia queda **a confirmar** en el grupo 1 de `tasks.md`.
- **La motivación del cambio** está en `proposal.md`; aquí no se repite.

## Goals / Non-Goals

**Goals:**

- Un único backend de tags, `lofty` 0.25.4, que cubra MP3 y MP4 sin duplicar lógica por formato.
- Paridad exacta del comportamiento MP3 actual: escritura siempre en ID3v2.4, los mismos frames por flag, `USLT` con idioma `spa`, carátulas `.webp` vía `APIC`, `remove --all` con borrado físico de ID3v1 e ID3v2, nombres de tag en inglés y español, y `edit` sin cambios que no reescribe el archivo.
- Soporte MP4 con el mapeo completo de la tabla verificada, incluidos fecha, numeración, ordenaciones, compilación y carátula.
- Detección de formato robusta por magic bytes, independiente de la extensión.
- Documentar explícitamente, y no esconder, los dos comportamientos que se degradan (WebP en `covr` y pérdida del idioma de la letra en M4A).

**Non-Goals:**

- FLAC, OGG, Opus, WAV, AIFF y APE. Fuera de alcance en esta iteración; se rechazan con un mensaje claro.
- Capítulos de `.m4b`. El contenedor `.m4b` se acepta como MP4 con `ilst`, pero sus capítulos quedan fuera de alcance.
- Escribir tags ID3 dentro de un contenedor MP4.
- Cambiar la interfaz de flags del CLI ni los nombres de tag de `remove`.
- Rediseñar el formato de salida de `show` más allá de lo necesario para cubrir los campos de MP4.
- Migrar a `mp4ameta` o mantener dos backends en paralelo.
- Unificar `--year` en `TDRC` para MP3. Se aborda en un cambio posterior e independiente llamado `id3v24-date-unification`, que aplicará a MP3 la misma regla de colisión ya aprobada para M4A (gana `--date` con warning). Queda deliberadamente fuera de este cambio.
- Traducir al español los errores propagados desde la capa de entrada/salida y desde la librería de metadatos (por ejemplo el error de permisos al escribir). Se caracterizan tal cual están y su traducción queda deliberadamente fuera de este cambio.

## Decisions

### 1. Backend único `lofty` 0.25.4, en lugar de un doble backend `id3` + `mp4ameta` o un híbrido

`lofty` cubre MP3 y MP4 con una sola API y ya trae resuelto el mapeo ID3↔MP4 en su tabla `ILST_MAP`, que es justo la pieza que de otro modo habría que escribir a mano y mantener. Es además un crate con licencia MIT/Apache y muy mantenido. El coste aceptado es reescribir `src/lib.rs` y adaptar los 106 tests, con el riesgo de regresión en MP3 que se mitiga en el grupo 1 de `tasks.md`.

Alternativas descartadas: `mp4ameta` 0.13.0 en paralelo a `id3` (dos dependencias, dos rutas de código y lógica duplicada de flags, carátulas, fechas y `remove`); híbrido `id3` para MP3 y `lofty` solo para MP4 (conserva el riesgo cero en MP3, pero mantiene dos modelos de tag y obliga a duplicar precisamente la capa de mapeo que se quiere unificar; además duplicaría la superficie de bugs en cada formato).

### 2. Detección por magic bytes, en lugar de por extensión

La extensión del nombre es una pista falsificable y no sobrevive a un renombrado. Detectar por contenido soporta que un `.m4a` renombrado a `.mp3` se siga editando correctamente, y hace que un archivo de formato no soportado se rechace en vez de tratarse como MP3 por defecto. Para MP3 se aceptan la cabecera `ID3` y la sincronía de frame MPEG (archivos sin tag inicial, `0xFF` con los bits altos del segundo byte a `1`), validando que los campos de versión, layer, bitrate y frecuencia de muestreo no sean reservados ni inválidos, para evitar falsos positivos. Para MP4, la detección MUST basarse solo en la existencia del átomo `ftyp` en el offset 4, **sin lista blanca de brands**, porque una lista blanca rechazaría archivos legítimos dentro del alcance acordado (un audiolibro `.m4b` lleva brand `M4B `). Los brands `M4A `, `M4B `, `mp42` e `isom` se conservan en este documento solo como contexto de lo observado en archivos reales, no como criterio de aceptación.

Alternativa descartada: seguir decidiendo por extensión y ampliar la lista (`mp3`, `m4a`, `mp4`, `m4b`). Es más barata pero deja el comportamiento actual roto ante renombrados, y no resuelve el caso de tener que rechazar explícitamente un formato no soportado, porque un `.flac` seguiría entrando por la rama MP3.

### 3. Warning y omisión para los campos sin equivalente, en lugar de átomo freeform o error

`--url` y `--original-artist` no tienen átomo equivalente en `ilst`. Ni un átomo freeform (`----:com.apple.iTunes:...`) ni un abort sirven: los átomos freeform no los lee iTunes, y abortar rompería scripts que hoy funcionan sobre MP3 con esos mismos flags. La decisión es emitir un warning por stderr, omitir ese campo, escribir el resto y terminar con código cero.

Alternativas descartadas: escribir átomos freeform propietarios (invisibles para iTunes y gratuitos de mantener, es decir, deuda disfrazada de cobertura); abortar la operación (rompe la compatibilidad hacia atrás de la CLI); ignorar el campo en silencio (el usuario no sabría que su dato se ha perdido).

### 4. `--date` tiene prioridad sobre `--year` en el átomo `©day`

`©day` es un único átomo que aloja fecha y año. Si se indican ambos, `--date` es el valor más específico y cabe en el átomo tal cual, mientras que el año es su prefijo. La decisión es que gane `--date` y que se avise por stderr de que el año ha quedado absorbido por la fecha. Si solo se indica uno, se escribe sin warning.

Alternativas descartadas: que gane `--year` (pierde información y es contraintuitivo); concatenar ambos en un solo valor (produce un `©day` inválido que ningún reproductor interpreta); escribir el año en un átomo freeform aparte (invisible para iTunes y contrario a la decisión 3).

### 5. Fixtures con audio real, generados una sola vez con `ffmpeg` y commiteados

El truco actual de `create_temp_mp3()` crea un MP3 falso con solo la cabecera ID3, y se creía suficiente para MP3 porque el tag vive en la cabecera del archivo. Un spike real contra `lofty` 0.25.4 demuestra que no lo es: sobre un fichero que solo tiene el header ID3v2.4 y ningún frame MPEG, lofty **no puede leer ni escribir** y falla con `failed to parse Mpeg file`. Tampoco sirven los fixtures de 4 bytes tipo `[0xFF,0xFB,0x90,0x44]` que ya usan los tests: con ellos la escritura funciona pero la lectura falla, así que rompen cualquier ida y vuelta. Con MP4 la limitación es la misma y por la misma razón: hace falta una estructura `ftyp`+`moov`+`udta`+`meta`+`ilst` real para poder escribir y releer metadatos.

La decisión es por tanto común a los dos formatos: generar con `ffmpeg` dos fixtures mínimos y deterministas con **audio real** —uno de MP3 **sin ningún tag ID3** y uno de `.m4a`—, commitearlos en `tests/fixtures/` y hacer que los helpers de test **copien** el fixture a un directorio temporal para operar sobre él, en lugar de fabricar bytes falsos. El MP3 se genera sin ningún tag ID3 para que la escritura de frames se pruebe desde cero y no desde un tag preexistente.

Alternativas descartadas: construir los fixtures byte a byte a mano en los tests (inviable para lofty, que exige contenido MPEG real para parsear, y frágil y ajeno al producto en el caso del contenedor MP4); generar el fixture en tiempo de test invocando `ffmpeg` (introduce una dependencia de entorno en la suite y hace los tests no herméticos); usar `mp4ameta` en los tests para construir el fixture (mete una dependencia de test que no existe en el producto y adelanta justo el diseño descartado en la decisión 1).

### 6. Se conserva `TYER` para `--year`, en lugar de unificar en `TDRC` dentro de este cambio

Se conserva `TYER` **aunque esté obsoleto en ID3v2.4**, para que la migración de backend sea puramente mecánica y la paridad de MP3 sea verificable con los tests actuales sin tocar sus expectativas. Se implementa con la API de bajo nivel de lofty (`Frame::Text` con `FrameId::new("TYER")`, que lofty acepta como ID válido de v2.3/4).

Alternativa descartada: unificar en `TDRC` **en este mismo cambio**. Mezcla un cambio de comportamiento intencionado con la migración de backend, y entonces el diff de los tests deja de distinguir una adaptación de firma de una expectativa relajada, que es justo la red de seguridad en la que se apoya todo el cambio. La unificación corresponde a un cambio posterior.

### 7. Los fixtures se cambian antes y bajo el backend actual, en un paso separado de la migración

El cambio de fixtures y la migración del backend MUST ser pasos separados, cada uno con su propio commit, porque si se hacen a la vez no se puede distinguir una regresión del backend de un problema del fixture: cualquier test en rojo sería ambiguo justo cuando la red de seguridad es lo único que hay para atribuir el fallo. El orden es estricto: primero se sustituyen los fixtures falsos por los reales **manteniendo `id3` como backend** y se exige que la suite completa de 106 tests siga verde con los fixtures nuevos, y solo después se migra el backend (grupo 2 de `tasks.md`).

Alternativa descartada: hacerlo todo junto en un único paso (ahorra un commit, pero destruye la capacidad de atribuir cualquier fallo, que es justo el valor de la red de seguridad).

### 8. La implementación usa la API de bajo nivel de ID3v2 (`lofty::id3::v2::Id3v2Tag`), en lugar del `lofty::tag::Tag` genérico

Se decidió así tras comprobar que el tag genérico no puede cumplir el contrato de paridad de MP3, por tres razones concretas y verificadas:

1. El frame `TYER`, que el contrato obliga a conservar para `--year`, **no tiene ningún `ItemKey`** en la tabla de `lofty`, y el tag genérico no expone API pública para insertar un frame sin mapeo.
2. El frame `USLT` de la letra **no permite fijar el código de idioma `spa`** desde el tag genérico: el idioma por defecto sería `XXX`, y solo el frame de bajo nivel permite fijarlo.
3. `remove_tags` necesita borrar frames **exactos**, incluido `TYER`, y no claves genéricas.

La API de bajo nivel da acceso a los frames ID3v2 reales (`Frame::Text` con `FrameId::new("TYER")`, `Frame::Lyrics`, `Frame::Picture`) y por tanto conserva frame a frame la semántica del backend anterior.

Alternativa descartada: usar el `lofty::tag::Tag` genérico, que es más idiomático y requiere menos código, pero es incapaz de satisfacer los tres puntos anteriores sin perder la paridad del contrato.

### 9. El crate `id3` se conserva como dependencia de desarrollo, para que los tests relean con la librería anterior

Los tests de integración releen los tags con la librería anterior (`id3`), mientras la producción escribe con `lofty`. Ese reparto persigue dos cosas a la vez. Primero, que **ninguna de las 62 aserciones de integración cambia**: si el mismo lector que antes verifica lo que ahora escribe el backend nuevo, la red de seguridad de caracterización queda literalmente intacta y el diff de los tests solo puede contener adaptaciones de firma, nunca expectativas relajadas. Segundo, que la verificación sea **entre librerías distintas** —`lofty` escribe, `id3` lee—, lo que es evidencia más fuerte que releer con la propia librería: una lectura con el mismo crate que escribió comprobaría solo su coherencia interna, no que los bytes en disco siguen siendo los que el producto producía antes.

La producción no la usa: `cargo tree -e normal` no muestra `id3`; solo aparece en `-e dev`. La dependencia queda por tanto confinada al árbol de desarrollo y no viaja al binario distribuido.

Alternativa descartada: reescribir las aserciones de integración contra la API de `lofty`. Evita mantener una dependencia de desarrollo, pero obliga a tocar cada aserción —que es exactamente donde se cuelan los cambios de expectativa— y convierte la verificación en algo autorreferente, porque el lector ya no sería independiente del escritor.

### 10. Paridad de `remove` con la eliminación del último frame, en lugar de aceptar el borrado físico del bloque

La librería antigua, al guardar un tag vacío, dejaba una cabecera ID3v2 vacía de 10 bytes en el archivo; `lofty` en cambio **elimina físicamente el bloque ID3v2 entero**. Se decidió **preservar la paridad** con la librería antigua mediante un guardado específico en `main.rs` que, cuando el tag queda vacío, borra el bloque y antepone una cabecera vacía.

La razón es de diseño del CLI, no de nostalgia del byte: la herramienta ya tiene el subcomando `remove --all` para borrar físicamente los bloques, de modo que si `remove <tags>` también borrara el bloque al quitar el último frame, **la distinción entre ambos subcomandos desaparecería en ese caso límite**. Un usuario que hoy hace `remove title` sobre un MP3 con un solo tag obtiene un archivo con cabecera ID3 vacía, no un MP3 sin bloque ID3; aceptar el comportamiento de `lofty` cambiaría en silencio el efecto de un subcomando documentado.

Coste asumido y declarado: unas 18 líneas de manipulación de archivo en `main.rs` solo para conservar esa peculiaridad.

Alternativa descartada: aceptar el comportamiento de `lofty` por ser más limpio y actualizar el test correspondiente, documentándolo como cambio deliberado de comportamiento. Se descarta aquí para no mezclar cambios de comportamiento con la migración, según el criterio de la decisión 7.

## Risks / Trade-offs

- **Regresión en MP3 al cambiar de backend** → red de seguridad previa: caracterizar el comportamiento actual y ejecutar la suite completa de 106 tests en verde antes de tocar `lib.rs` (grupo 1 de `tasks.md`); a partir de ahí, cada grupo debe dejar la suite verde y la migración se hace sin tocar el resto de comportamiento en el mismo paso.
- **Cambia la API pública de `lib.rs`** porque hoy las funciones reciben `&mut id3::Tag` → el cambio de firma es inevitable y se asume como parte del alcance; los tests son el consumidor principal de esa API, así que la adaptación queda contenida en `src/tests.rs` y `tests/integration_test.rs`, y el binario `src/main.rs` es el único otro consumidor.
- **`remove --all` dependía de `id3::v1v2::remove_from_path`, y su equivalente directo en `lofty` está roto** → `TagType::remove_from_path` de `lofty` 0.25.4 falla con `EBADF` (*Bad file descriptor*), porque `VerifiedFile::new_from_path` envuelve un descriptor abierto en solo lectura sobre el que después intenta escribir. Mitigación verificada: el estado previo se determina sobre los **bytes del propio archivo** —cabecera `ID3` al inicio para v2 y bloque `TAG` en los últimos 128 bytes para v1—, que es exactamente la definición física que usaba el crate `id3`, de modo que no depende de que `lofty` logre parsear el contenedor; y el borrado se delega en `lofty` mediante `TagType::remove_from(...)` sobre un descriptor abierto explícitamente en **lectura/escritura**, tratando ID3v2 e ID3v1 por separado. Los cuatro estados previos (ninguno / solo ID3v1 / solo ID3v2 / ambos) se detectan y se borran correctamente, y el audio sobrevive. El observable a conservar (mensajes por estado previo y audio intacto) está fijado por los escenarios de `mp3-tagging` y verificado con tests de integración.
- **Los fixtures M4A no se pueden falsificar con 10 bytes** → usar `ffmpeg` para generar un fixture real y commitearlo en `tests/fixtures/`, con un helper que lo copie a un temporal; el fixture se versiona para que la suite no dependa de `ffmpeg` en tiempo de ejecución.
- **WebP deja de funcionar en `covr`** → es un cambio de comportamiento observable (en MP3 sigue funcionando vía `APIC`), así que se documenta en el README y se emite un warning explícito al usar `--cover` con `.webp` sobre un MP4, en lugar de degradar en silencio a `Reserved(0)`.
- **Pérdida del idioma de la letra en M4A** → `©lyr` no tiene campo de idioma; el texto se conserva pero `spa` no se puede preservar. Se documenta en el README como limitación conocida del contenedor, no como bug, y el spec de `m4a-tagging` lo fija como comportamiento esperado.
- **Dependencia nueva sin caché previa** → `lofty` 0.25.4 entra en `Cargo.toml` y sale `id3`; el primer `cargo build` descarga el árbol de dependencias, coste puntual de build que no afecta al diseño.
- **`TYER` está obsoleto en ID3v2.4 y varios reproductores leen `TDRC`** → el año puede no verse fuera del CLI; se conserva por paridad en este cambio y se corrige en el cambio posterior `id3v24-date-unification`, de modo que el aviso y el arreglo no se mezclan con la migración.
- **La detección por lista de brands del `ftyp` rechazaría archivos legítimos, como los audiolibros `.m4b` con brand `M4B `** → la detección se basa solo en la existencia de `ftyp` en el offset 4, sin lista blanca.
- **La detección de MP3 por sincronía de frame MPEG puede dar falsos positivos con archivos cuyo primer byte sea `0xFF` y el segundo tenga los bits altos a `1`** → se validan los campos de versión, layer, bitrate y frecuencia de muestreo de la cabecera.

## Migration Plan

El orden es estricto y cada paso se cierra con la suite verde; no se abre el siguiente grupo con tests en rojo.

1. **Caracterizar.** Fijar el comportamiento actual de MP3 en la spec `mp3-tagging` y respaldarlo con los 106 tests existentes en verde. Sin esta red de seguridad no se toca el backend.
2. **Migrar el backend conservando paridad.** Sustituir `id3` por `lofty` en `Cargo.toml` y reescribir `src/lib.rs` y `src/main.rs` contra la API nueva, adaptando los tests sin cambiar ninguna expectativa de comportamiento. Al final de este paso, el comportamiento observable de MP3 es el de antes.
3. **Añadir la detección de formato.** Introducir la detección por magic bytes y el enrutado al backend correspondiente, con el rechazo explícito de los formatos fuera de alcance.
4. **Añadir el soporte MP4.** Incorporar la escritura y lectura de átomos `ilst`, el mapeo de la tabla, las reglas de colisión y omisión, la carátula y la letra, con los fixtures `.m4a` reales.
5. **Actualizar la documentación.** `Cargo.toml`, `README.md`, `AGENTS.md`, los textos de `about`/`help` de `clap` y los mensajes en español que hoy dicen "MP3".
6. **Trabajo diferido.** Con la migración cerrada y la suite en verde, el cambio posterior e independiente `id3v24-date-unification` unificará `--year` en `TDRC`, aplicando a MP3 la misma regla de colisión ya aprobada para M4A (gana `--date` con warning). Ese cambio tendrá su propio spec y sus propios tests; aquí solo se deja constancia de la deuda, que no se aborda en este alcance.

**Rollback:** el proyecto sigue Git Flow, así que este cambio se desarrolla en una rama `feature/*` sobre `development` y no se mezcla hasta que la suite completa está verde. Mientras el grupo 2 no esté cerrado y verde, revertir es descartar la rama: el código que funciona hoy en `development` no se toca hasta que la migración esté completa y verificada.
