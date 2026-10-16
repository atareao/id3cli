# Spec Delta

## Purpose

Precisar el destino del total de pista y de temporada en MP4: hasta ahora `--track` y `--season` se escribían como un número entero sin total, y con este cambio el CLI SHALL escribir el átomo `trkn` y el átomo `disk` con el número y, cuando se indique, el total, que es la representación nativa del contenedor `ilst`. El resto del mapeo de flags a átomos MUST quedar exactamente igual. La sintaxis del flag, el mensaje de error de forma inválida, la conservación del total preexistente como regla general, el `show` del total y el borrado del par como unidad se especifican en la capability `track-numbering`; aquí solo se fija el destino del total en este formato.

## MODIFIED Requirements

### Requirement: Mapeo de los flags del CLI a los átomos `ilst`

El CLI SHALL escribir cada flag de `edit` en el átomo MP4 que le corresponde: `--title` en `©nam`, `--artist` en `©ART`, `--album` en `©alb`, `--album-artist` en `aART`, `--date` en `©day`, `--year` en `©day` (el mismo átomo que `--date`, con la colisión resuelta por el requisito «Colisión de `--year` y `--date` en el átomo `©day`»), `--genre` en `©gen`, `--track` en `trkn`, `--season` en `disk`, `--composer` en `©wrt`, `--copyright` en `cprt`, `--lyrics` en `©lyr`, `--cover` en `covr`, `--compilation` en `cpil`, `--title-sort` en `sonm`, `--artist-sort` en `soar`, `--album-sort` en `soal` y `--subtitle` en el átomo freeform `----:com.apple.iTunes:SUBTITLE`. Además, `--track` SHALL escribirse en el átomo `trkn`, que SHALL llevar el número y, cuando se indique, el total; `--season` SHALL escribirse en el átomo `disk` con número y total. El resto del mapeo MUST quedar exactamente como está. El CLI MUST NOT escribir ningún otro átomo para representar estos campos.

#### Scenario: Escritura de los átomos de texto básicos
- **WHEN** se ejecuta `edit` con `--title`, `--artist`, `--album`, `--album-artist` y `--genre` sobre un `.m4a`
- **THEN** el archivo MUST contener los átomos `©nam`, `©ART`, `©alb`, `aART` y `©gen` con los valores indicados

#### Scenario: Escritura de la numeración
- **WHEN** se ejecuta `edit --track 7 --season 2` sobre un `.m4a`
- **THEN** el archivo MUST contener el átomo `trkn` con la pista 7 y el átomo `disk` con el disco/temporada 2

#### Scenario: Escritura de la compilación
- **WHEN** se ejecuta `edit --compilation` sobre un `.m4a`
- **THEN** el archivo MUST contener el átomo `cpil` marcado como compilación

#### Scenario: Escritura de los créditos y las ordenaciones
- **WHEN** se ejecuta `edit` con `--composer`, `--copyright`, `--title-sort`, `--artist-sort` y `--album-sort` sobre un `.m4a`
- **THEN** el archivo MUST contener los átomos `©wrt`, `cprt`, `sonm`, `soar` y `soal` con los valores indicados

#### Scenario: El subtítulo se escribe como átomo freeform
- **WHEN** se ejecuta `edit --subtitle "Descripción corta"` sobre un `.m4a`
- **THEN** el archivo MUST contener el átomo `----:com.apple.iTunes:SUBTITLE` con ese valor

#### Scenario: Átomos no indicados no se tocan
- **WHEN** se ejecuta `edit --title "Nuevo"` sobre un `.m4a` que ya tenía artista, álbum y carátula
- **THEN** los átomos `©ART`, `©alb` y `covr` MUST permanecer intactos

#### Scenario: Total de pista en el átomo trkn
- **WHEN** se ejecuta `edit --track 7/12` sobre un `.m4a`
- **THEN** el átomo `trkn` MUST contener el número 7 y el total 12, y MUST releerse con esos dos valores

#### Scenario: Total de temporada en el átomo disk
- **WHEN** se ejecuta `edit --season 1/2` sobre un `.m4a`
- **THEN** el átomo `disk` MUST contener 1 y total 2

#### Scenario: Sin total explícito se conserva el total preexistente
- **WHEN** un `.m4a` ya tiene `trkn` con número 3 y total 12 y se ejecuta `edit --track 5`
- **THEN** el átomo `trkn` MUST quedar con número 5 y total 12
