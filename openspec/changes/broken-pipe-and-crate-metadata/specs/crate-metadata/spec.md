# Spec Delta

## Purpose

El manifiesto del paquete no declaraba `keywords` ni `categories`, así que la ficha de crates.io salía vacía y la crate no se podía encontrar ni clasificar por esas vías. Este delta fija los valores y las reglas que el manifiesto MUST cumplir para que `cargo publish` no falle y la ficha muestre la metadata.

## ADDED Requirements

### Requirement: El manifiesto MUST declarar keywords válidos para crates.io

`Cargo.toml` MUST declarar en `[package]` una lista de `keywords` no vacía y MUST respetar las reglas de crates.io: como máximo cinco keywords, cada una de 20 caracteres o menos, ASCII en minúsculas y compuesta solo por letras, dígitos, `-` y `_`. Los valores fijados por este cambio son `id3`, `mp3`, `mp4`, `metadata` y `cli`.

#### Scenario: Cantidad permitida
- **WHEN** se lee la lista `keywords` declarada en `Cargo.toml`
- **THEN** el número de elementos MUST estar entre 1 y 5, ambos inclusive

#### Scenario: Forma de cada keyword
- **WHEN** se inspecciona cada keyword declarada
- **THEN** cada una MUST tener 20 caracteres o menos, MUST ser ASCII en minúsculas y MUST contener únicamente caracteres del conjunto `[a-z0-9_-]`

#### Scenario: Los valores fijados en este cambio
- **WHEN** se lee la lista `keywords` declarada en `Cargo.toml`
- **THEN** los valores MUST ser exactamente `id3`, `mp3`, `mp4`, `metadata` y `cli`

#### Scenario: La metadata MUST ser legible por las herramientas
- **WHEN** se ejecuta `cargo metadata --no-deps`
- **THEN** la salida MUST exponer las keywords declaradas en el manifiesto

### Requirement: Las categories MUST pertenecer a la lista oficial de crates.io

`Cargo.toml` MUST declarar en `[package]` una lista de `categories` cuyos slugs pertenezcan a la taxonomía oficial de crates.io. Los valores fijados por este cambio son `command-line-utilities`, `multimedia::audio` y `filesystem`, verificados el 2026-10-14 contra la lista oficial de categorías y contra la metadata publicada de la crate `lofty` 0.25.4.

#### Scenario: Los valores fijados en este cambio
- **WHEN** se lee la lista `categories` declarada en `Cargo.toml`
- **THEN** los valores MUST ser exactamente `command-line-utilities`, `multimedia::audio` y `filesystem`, verificados el 2026-10-14 contra `https://crates.io/api/v1/categories` y contra la metadata publicada de la crate `lofty` 0.25.4

#### Scenario: Un slug inválido MUST NOT llegar al servidor
- **WHEN** se declara una categoría cuyo slug no existe en la lista oficial de crates.io
- **THEN** la publicación MUST ser rechazada por el servidor con un error, de modo que el contrato exige fijar únicamente slugs ya verificados y MUST NOT depender de que el servidor los corrija o los ignore

#### Scenario: La ficha de crates.io MUST mostrar la metadata
- **WHEN** la crate se publica y se consulta la API de crates.io
- **THEN** la respuesta MUST devolver las keywords y las categories declaradas en el manifiesto, en lugar de las listas vacías actuales

### Requirement: El Cargo.lock MUST quedar con la versión publicada tras una release

El commit de release MUST dejar la entrada del propio paquete en `Cargo.lock` con la misma versión que `Cargo.toml`, de modo que un `cargo` en limpio MUST NOT modificar el árbol de trabajo. El incumplimiento actual queda como evidencia: el commit `9681456` (release de la 0.5.0) solo tocó `.vampus.yml`, `CHANGELOG.md` y `Cargo.toml`, y hoy `main` y `development` declaran `0.4.3` en el lock frente a `0.5.0` en el manifiesto.

La comprobación de este requisito MUST NOT hacerse con un test de Rust que lea `Cargo.lock` del árbol de trabajo: cualquier orden de cargo normaliza el lock a la versión del manifiesto **antes** de invocar a `rustc`, así que el test embebería ya el valor corregido y daría un falso verde. Es un hecho medido: con el lock commiteado en `0.4.3` y el manifiesto en `0.5.0`, ese test pasaba, y por eso se ha retirado del código.

#### Scenario: La versión del lock coincide con la del manifiesto
- **WHEN** se comparan la versión declarada para el paquete en `Cargo.lock` y el campo `version` de `Cargo.toml`
- **THEN** ambas MUST coincidir

#### Scenario: Un cargo en limpio MUST NOT ensuciar el árbol
- **WHEN** se ejecuta cualquier orden de cargo sobre un árbol recién clonado
- **THEN** `Cargo.lock` MUST NOT reescribirse y `git status` MUST quedar sin cambios

#### Scenario: La integración continua MUST detectar el desajuste
- **WHEN** el lock commiteado no coincide con el manifiesto
- **THEN** la integración continua MUST fallar en lugar de actualizar el lock en silencio: las órdenes de cargo de la CI MUST validar el lock (`--locked`) en vez de regenerarlo, y MUST incluir la primera orden que resuelve dependencias, para que el desajuste se detecte antes de que otra orden normalice el lock

#### Scenario: El pipeline MUST refrescar el lock antes de commitear
- **WHEN** el pipeline de release sube la versión del manifiesto
- **THEN** MUST refrescar `Cargo.lock` antes de crear el commit de release, de forma que el lock entre en ese mismo commit y la comprobación de la CI siga pasando
