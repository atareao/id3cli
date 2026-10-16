# Spec Delta

## Purpose

Precisar el destino del total de pista y de temporada en MP3: hasta ahora `--track` y `--season` se escribían como un número entero sin total, y con este cambio el CLI SHALL escribir el frame `TRCK` y el frame `TPOS` con la forma `n` o `n/total`, que es la representación nativa del contenedor ID3v2.4. El resto del mapeo de flags a frames MUST quedar exactamente igual. La sintaxis del flag, el mensaje de error de forma inválida, la conservación del total preexistente como regla general, el `show` del total y el borrado del par como unidad se especifican en la capability `track-numbering`; aquí solo se fija el destino del total en este formato.

## MODIFIED Requirements

### Requirement: Mapeo de los flags del CLI a los frames ID3v2.4

El CLI SHALL escribir cada flag de `edit` en el frame ID3v2.4 que le corresponde hoy, sin cambiar ningún mapeo: `--title` en `TIT2`, `--artist` en `TPE1`, `--album` en `TALB`, `--album-artist` en `TPE2`, `--genre` en `TCON`, `--track` en `TRCK`, `--season` en `TPOS`, `--composer` en `TCOM`, `--copyright` en `TCOP`, `--subtitle` en `TIT3`, `--original-artist` en `TOPE`, `--url` en `WOAR`, `--compilation` en `TCMP` con el valor `"1"`, `--title-sort` en `TSOT`, `--artist-sort` en `TSOP`, `--album-sort` en `TSOA`, `--lyrics` en `USLT` y `--cover` en `APIC`. En cuanto a las fechas, `--year` SHALL escribirse en el frame `TYER` y `--date` en el frame `TDRC`; son dos frames distintos y NO se sobrescriben entre sí. Además, `--track` SHALL escribirse en el frame `TRCK` como el número (`n`) cuando no se indique total, o como `n/total` cuando sí se indique; `--season` SHALL escribirse en el frame `TPOS` con la misma forma. El resto del mapeo MUST quedar exactamente como está. El CLI MUST NOT omitir ni redirigir a otro frame ninguno de estos campos en MP3.

#### Scenario: Mapeo de los campos básicos
- **WHEN** se ejecuta `edit` con `--title`, `--artist`, `--album`, `--album-artist`, `--genre`, `--track` y `--season` sobre un MP3
- **THEN** el archivo MUST contener sendos frames `TIT2`, `TPE1`, `TALB`, `TPE2`, `TCON`, `TRCK` y `TPOS` con los valores indicados

#### Scenario: Mapeo de los campos de fecha
- **WHEN** se ejecuta `edit --year 2026` sobre un MP3
- **THEN** el archivo MUST contener el frame `TYER` con el valor `2026`

#### Scenario: Mapeo de `--date`
- **WHEN** se ejecuta `edit --date 2026-10-13` sobre un MP3
- **THEN** el archivo MUST contener el frame `TDRC` con el valor `2026-10-13`

#### Scenario: `--year` y `--date` coexisten sin pisarse
- **WHEN** se ejecutan `edit --year 2026` y `edit --date 2026-10-13` sobre el mismo MP3
- **THEN** el archivo MUST contener simultáneamente el frame `TYER` con `2026` y el frame `TDRC` con `2026-10-13`, sin que ninguno sobrescriba al otro

#### Scenario: `--year` escribe `TYER`
- **WHEN** se ejecuta `edit --year 2026` sobre un MP3 sin tags previos
- **THEN** el archivo MUST contener el frame `TYER` con `2026` y MUST NOT contener un frame `TDRC`

#### Scenario: Mapeo de los créditos y las ordenaciones
- **WHEN** se ejecuta `edit` con `--composer`, `--copyright`, `--subtitle`, `--compilation`, `--title-sort`, `--artist-sort` y `--album-sort` sobre un MP3
- **THEN** el archivo MUST contener `TCOM`, `TCOP`, `TIT3`, `TCMP` con el valor `"1"`, `TSOT`, `TSOP` y `TSOA` con los valores indicados

#### Scenario: Mapeo de `--url` y `--original-artist` en MP3
- **WHEN** se ejecuta `edit` con `--url` y `--original-artist` sobre un MP3
- **THEN** el archivo MUST contener el frame `WOAR` con la URL y el frame `TOPE` con el artista original, sin ningún warning ni omisión

#### Scenario: Campo no indicado no se toca
- **WHEN** se ejecuta `edit` indicando solo `--title` sobre un MP3 que ya tenía artista y álbum
- **THEN** los frames de artista y álbum MUST permanecer intactos

#### Scenario: Total de pista en TRCK
- **WHEN** se ejecuta `edit --track 7/12` sobre un MP3
- **THEN** el frame `TRCK` MUST contener `7/12` y MUST releerse como número 7 con total 12

#### Scenario: Total de temporada en TPOS
- **WHEN** se ejecuta `edit --season 1/2` sobre un MP3
- **THEN** el frame `TPOS` MUST contener `1/2`

#### Scenario: Sin total explícito se conserva el total preexistente
- **WHEN** un MP3 ya tiene `TRCK` con `3/12` y se ejecuta `edit --track 5`
- **THEN** el frame `TRCK` MUST quedar en `5/12`, conservando el total que ya estaba

#### Scenario: Número sin total sobre un archivo sin total
- **WHEN** se ejecuta `edit --track 5` sobre un MP3 cuyo `TRCK` no tiene total
- **THEN** el frame `TRCK` MUST quedar en `5`
