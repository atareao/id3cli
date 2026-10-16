use clap::{CommandFactory, Parser, Subcommand};
use id3cli::output::print_block;
use id3cli::*;
use lofty::config::{ParseOptions, WriteOptions};
use lofty::file::AudioFile;
use lofty::id3::v2::Id3v2Tag;
use lofty::mpeg::MpegFile;
use lofty::ogg::OggPictureStorage;
use lofty::tag::TagExt;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

/// Lee el tag ID3v2 del archivo MP3, si existe.
///
/// Devuelve `None` tanto cuando el archivo no se puede interpretar como MPEG
/// como cuando es un MP3 sin tag ID3v2; el CLI trata ambos casos igual que
/// trataba el fallo de `id3::Tag::read_from_path`.
fn read_id3v2_tag(file: &Path) -> Option<Id3v2Tag> {
    let mut handle = File::open(file).ok()?;
    let mpeg = MpegFile::read_from(&mut handle, ParseOptions::default()).ok()?;
    mpeg.id3v2().cloned()
}

/// CLI para añadir metadatos y carátulas a archivos MP3, MP4, OGG, Opus y FLAC
#[derive(Parser, Debug)]
#[command(name = "id3cli")]
#[command(
    about = "Añade tags y carátulas a archivos MP3 (ID3v2.4), MP4 (átomos iTunes), OGG (comentarios Vorbis; Vorbis y Opus) y FLAC (comentarios Vorbis)",
    long_about = None
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
#[allow(clippy::large_enum_variant, clippy::too_many_arguments)]
enum Command {
    /// Mostrar todos los metadatos de los archivos MP3, MP4, OGG, Opus y FLAC
    Show {
        /// Uno o más archivos de audio (MP3, MP4, OGG, Opus y FLAC)
        #[arg(required = true, num_args = 1..)]
        files: Vec<PathBuf>,

        /// Emitir los metadatos como un único objeto JSON compacto en una línea
        #[arg(short, long)]
        json: bool,
    },
    /// Editar metadatos de los archivos MP3, MP4, OGG, Opus y FLAC
    Edit {
        /// Uno o más archivos de audio (MP3, MP4, OGG, Opus y FLAC)
        #[arg(required = true, num_args = 1..)]
        files: Vec<PathBuf>,

        /// Título de la canción
        #[arg(short, long)]
        title: Option<String>,

        /// Artista (se puede especificar múltiples veces)
        #[arg(short, long)]
        artist: Vec<String>,

        /// Álbum
        #[arg(short = 'A', long)]
        album: Option<String>,

        /// Año
        #[arg(short, long)]
        year: Option<i32>,

        /// Género
        #[arg(short, long)]
        genre: Option<String>,

        /// Número de pista (N o N/TOTAL)
        #[arg(short = 'T', long, value_name = "N|N/TOTAL")]
        track: Option<NumberWithTotal>,

        /// Temporada (TPOS - útil para podcasts; N o N/TOTAL)
        #[arg(short = 'S', long, value_name = "N|N/TOTAL")]
        season: Option<NumberWithTotal>,

        /// Fecha de grabación (YYYY-MM-DD o YYYY)
        #[arg(short = 'd', long)]
        date: Option<String>,

        /// Copyright
        #[arg(short = 'C', long)]
        copyright: Option<String>,

        /// Compositor (TCOM)
        #[arg(long)]
        composer: Option<String>,

        /// Subtítulo o descripción (TIT3)
        #[arg(long)]
        subtitle: Option<String>,

        /// Artista original (TOPE)
        #[arg(long)]
        original_artist: Option<String>,

        /// Artista del álbum / Publisher (TPE2)
        #[arg(long)]
        album_artist: Option<String>,

        /// Ruta del archivo de imagen para la carátula (JPG, PNG, WEBP)
        #[arg(short, long)]
        cover: Option<PathBuf>,

        /// Letra de la canción (lyrics)
        #[arg(short = 'L', long)]
        lyrics: Option<String>,

        /// URL asociada (sitio web del artista, página oficial, etc.)
        #[arg(short = 'u', long)]
        url: Option<String>,

        /// Marcar como compilación (Apple TCMP)
        #[arg(long)]
        compilation: bool,

        /// Orden de clasificación del álbum (Apple TSOA)
        #[arg(long)]
        album_sort: Option<String>,

        /// Orden de clasificación del artista (Apple TSOP)
        #[arg(long)]
        artist_sort: Option<String>,

        /// Orden de clasificación del título (Apple TSOT)
        #[arg(long)]
        title_sort: Option<String>,
    },
    /// Eliminar tags específicos de los archivos MP3, MP4, OGG, Opus y FLAC
    Remove {
        /// Uno o más archivos de audio (MP3, MP4, OGG, Opus y FLAC)
        #[arg(required = true, num_args = 1..)]
        files: Vec<PathBuf>,

        // Tags a eliminar. La lista se deriva de `id3cli::VALID_TAG_NAMES`, la
        // misma tabla de alias que comparten la clasificación de MP3, la de
        // MP4 y la de OGG, para que la ayuda no pueda quedar desincronizada de
        // los nombres realmente aceptados.
        #[arg(help = format!("Tags a eliminar, en inglés o en español ({VALID_TAG_NAMES})"), last = true)]
        tags: Vec<String>,

        /// Eliminar físicamente todos los tags del archivo (ID3v1/ID3v2 en MP3, metadatos ilst en MP4, el bloque de comentarios Vorbis en OGG y Opus preservando el vendor, y el mismo bloque en FLAC eliminado por completo, sin conservar el vendor — degradación documentada)
        #[arg(short, long)]
        all: bool,
    },
    /// Generar el script de completions para bash, zsh o fish
    Completions {
        /// Shell para el que generar el script (bash, zsh o fish)
        #[arg(value_parser = clap::value_parser!(GenerationShell))]
        shell: GenerationShell,
    },
}

/// Backend con el que se procesa un archivo ya aceptado.
///
/// El enrutado es la única pieza que decide entre el camino MP3, el MP4, el OGG
/// y el FLAC; los subcomandos nunca vuelven a consultar el formato. Un único
/// backend OGG sirve a los dos códecs del contenedor: Vorbis y Opus. FLAC nativo
/// tiene su propio backend aunque reutilice las mismas funciones de comentarios
/// de Vorbis: el nombre del contenedor no debe colarse en los mensajes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Backend {
    /// MP3: se usa el backend ID3v2.
    Mp3,
    /// MP4/M4A: se usa el backend `ilst`.
    Mp4,
    /// OGG (Vorbis u Opus): se usa el backend de comentarios de Vorbis.
    Ogg,
    /// FLAC nativo: se usa el backend de comentarios de Vorbis, igual que OGG.
    Flac,
}

/// Determina el backend con el que hay que procesar un archivo, y ante un
/// fallo devuelve el motivo en español en lugar de terminar el proceso.
///
/// La decisión se toma sobre los *magic bytes* del archivo, nunca sobre la
/// extensión del nombre. Es la vía de detección del modo batch: el despliegue
/// por archivo recoge el `Err` con su ruta y continúa con el siguiente, y con
/// un solo archivo los motivos se informan con el mismo formato que el CLI
/// usaba (incluido el mensaje literal de archivo inexistente).
fn detect_supported_format_result(file: &Path) -> Result<Backend, String> {
    if !file.exists() {
        return Err(format!("El archivo '{}' no existe", file.display()));
    }

    match detect_format(file) {
        Ok(AudioFormat::Mp3) => Ok(Backend::Mp3),
        Ok(AudioFormat::Mp4) => Ok(Backend::Mp4),
        Ok(AudioFormat::OggVorbis) => Ok(Backend::Ogg),
        Ok(AudioFormat::OggOpus) => Ok(Backend::Ogg),
        Ok(AudioFormat::Flac) => Ok(Backend::Flac),
        Ok(AudioFormat::Unsupported) => Err(format!(
            "El archivo '{}' está en un formato de audio no soportado. \
             Formatos soportados: MP3, MP4, OGG, Opus y FLAC.",
            file.display()
        )),
        Ok(AudioFormat::Unknown) => Err(format!(
            "No se pudo determinar el formato del archivo '{}'. \
             Formatos soportados: MP3, MP4, OGG, Opus y FLAC.",
            file.display()
        )),
        Err(error) => Err(format!(
            "No se pudo leer el archivo '{}': {}",
            file.display(),
            error
        )),
    }
}

/// Convierte un error con su cadena de causas en el texto que el CLI mostraba
/// con `report_error`: el mensaje superficial y una línea «Causado por:» por
/// cada causa (lofty oculta el error de E/S real en su `Debug`, así que la
/// causa conserva el mensaje del sistema, p. ej. `Permission denied`).
fn motivo_con_causas(error: &dyn std::error::Error) -> String {
    let mut motivo = error.to_string();
    let mut source = error.source();
    while let Some(causa) = source {
        motivo.push_str(&format!("\n  Causado por: {causa}"));
        source = causa.source();
    }
    motivo
}

/// Informa de los fallos acumulados por el despliegue batch y termina con
/// código 1.
///
/// Con un único archivo se conserva el formato de error previo al modo batch
/// (`Error: <motivo>`, sin la ruta repetida): no-rotura N=1. Con varios, cada
/// fallo se informa como `Error: <ruta>: <motivo>`, uno por archivo fallido.
fn reportar_fallos(fallos: &[(PathBuf, String)], unico_archivo: bool) -> ! {
    if unico_archivo && let Some((_, motivo)) = fallos.first() {
        eprintln!("Error: {motivo}");
    } else {
        for (ruta, motivo) in fallos {
            eprintln!("Error: {}: {}", ruta.display(), motivo);
        }
    }
    std::process::exit(1);
}

/// Informa de un fallo al procesar un archivo del backend de comentarios de
/// Vorbis (Ogg Vorbis, Opus o FLAC nativo) y termina con código 1.
///
/// El mensaje enumera los formatos soportados además de la causa, de modo que un
/// archivo reconocible por contenido pero con estructura inválida (página Ogg
/// malformada, FLAC ilegible…) se rechaza con un error legible y sin haberlo
/// modificado. Es el mismo helper para OGG y para FLAC porque ambos comparten el
/// backend de comentarios de Vorbis.
fn report_vorbis_error(file: &Path, error: impl std::fmt::Display) -> ! {
    eprintln!(
        "Error: No se pudo procesar el archivo '{}': {}",
        file.display(),
        error
    );
    eprintln!(
        "  El archivo tiene un comentario de Vorbis (Ogg Vorbis, Opus o FLAC nativo) \
         pero no se pudo interpretar. Formatos soportados: MP3, MP4, OGG, Opus y FLAC."
    );
    std::process::exit(1);
}

/// Elimina tags concretos (sin `--all`) del comentario de Vorbis de un archivo
/// Ogg Vorbis, Opus o FLAC nativo.
///
/// Compartida por las ramas `Backend::Ogg` y `Backend::Flac` de `remove`, que
/// solo se diferencian en el tratamiento de `--all` (el borrado de carátulas a
/// nivel de archivo en FLAC).
fn remove_vorbis_tags(file: &Path, tags: &[String]) -> Result<(), Box<dyn Error>> {
    // Verificar que se especificaron tags para eliminar
    if tags.is_empty() {
        eprintln!("Error: Debes especificar al menos un tag para eliminar");
        eprintln!("Ejemplo: id3cli remove cancion.ogg title artist");
        std::process::exit(1);
    }

    // Leer tag. El comentario siempre se materializa (obligatorio en Ogg/Opus;
    // vacío si el FLAC no tiene bloque `VORBIS_COMMENT`): no hay un `Ok(None)`
    // que tratar como en MP4.
    let mut tag = match read_vorbis_tag(file) {
        Ok(tag) => tag,
        Err(error) => report_vorbis_error(file, error),
    };

    // Eliminar tags (misma tabla de alias que en MP3 y MP4)
    let removed = remove_tags_ogg(&mut tag, tags);

    if removed {
        if let Err(e) = write_vorbis_tag(file, &tag) {
            report_vorbis_error(file, e);
        }
        outln!(
            "\n✅ Metadatos eliminados correctamente de '{}'",
            file.display()
        );
    } else {
        outln!("\n⚠️  No se encontraron tags válidos para eliminar.");
    }

    Ok(())
}

/// Opciones de edición del subcomando `edit`, tal y como las parsea clap.
///
/// Agrupa los flags de `Command::Edit` para pasarlos a [`edit_archivo`] sin
/// veinte parámetros: el modo batch procesa todos los archivos con las mismas
/// opciones, una vez que clap las ha resuelto.
struct EditArgs<'a> {
    title: Option<&'a str>,
    artists: &'a [String],
    album: Option<&'a str>,
    year: Option<i32>,
    genre: Option<&'a str>,
    track: Option<NumberWithTotal>,
    season: Option<NumberWithTotal>,
    date: Option<&'a str>,
    copyright: Option<&'a str>,
    composer: Option<&'a str>,
    subtitle: Option<&'a str>,
    original_artist: Option<&'a str>,
    album_artist: Option<&'a str>,
    cover: Option<&'a Path>,
    lyrics: Option<&'a str>,
    url: Option<&'a str>,
    compilation: bool,
    album_sort: Option<&'a str>,
    artist_sort: Option<&'a str>,
    title_sort: Option<&'a str>,
}

/// Procesa un archivo del subcomando `show`.
///
/// En modo legible (con `cabecera`, para lotes de más de un archivo) imprime la
/// cabecera `=== <ruta> ===` y el bloque del backend, con la misma lógica que
/// el despliegue con un solo archivo. En modo JSON acumula la línea del archivo
/// en `lineas` (el NDJSON se emite una sola vez al final del lote). El aviso de
/// «sin tags/metadatos» —que en el flujo con un solo archivo termina con exit
/// 0— no aborta el lote: el proceso termina con código 0 por el final natural
/// de `main`.
fn show_archivo(
    file: &Path,
    json: bool,
    lineas: &mut Vec<String>,
    cabecera: bool,
) -> Result<(), String> {
    // La detección va PRIMERO: con una ruta inexistente o un formato no
    // soportado se devuelve el motivo sin emitir cabecera ni bloque, y el modo
    // batch acumula el fallo y continúa con el resto de archivos (sin dejar
    // cabeceras huérfanas).
    let backend = detect_supported_format_result(file)?;

    if cabecera {
        outln!("{}", batch_show_heading(file));
    }

    match backend {
        Backend::Mp3 => {
            // Leer tag
            let tag = match read_id3v2_tag(file) {
                Some(tag) => tag,
                None => {
                    if json {
                        lineas.push(json_empty("mp3"));
                    } else {
                        eprintln!("⚠️  No se encontraron tags ID3 en '{}'", file.display());
                    }
                    return Ok(());
                }
            };

            if json {
                lineas.push(json_from_mp3(&tag));
            } else {
                display_tags(&tag);
            }
        }
        Backend::Mp4 => {
            let tag = match read_mp4_tag(file) {
                Ok(Some(tag)) => tag,
                Ok(None) => {
                    if json {
                        lineas.push(json_empty("mp4"));
                    } else {
                        eprintln!("⚠️  No se encontraron metadatos en '{}'", file.display());
                    }
                    return Ok(());
                }
                Err(error) => {
                    return Err(motivo_mp4(file, error));
                }
            };

            if json {
                lineas.push(json_from_mp4(&tag));
            } else {
                display_tags_mp4(&tag);
            }
        }
        Backend::Ogg => {
            // El comentario de Vorbis siempre se materializa: en Ogg/Opus es
            // obligatorio en el contenedor. No hay un `Ok(None)` que tratar
            // como en MP4. En Ogg/Opus las carátulas viven dentro del propio
            // comentario, así que el tag basta para mostrarlas.
            let tag = match read_vorbis_tag(file) {
                Ok(tag) => tag,
                Err(error) => {
                    return Err(motivo_vorbis(file, error));
                }
            };

            if json {
                // El JSON nombra el códec REAL del contenedor ("ogg" u
                // "opus"), que el `Backend` fusiona; el nombre se resuelve por
                // contenido, nunca del backend.
                let formato = match detect_format(file) {
                    Ok(formato) => format_json_name(formato),
                    Err(error) => {
                        return Err(motivo_vorbis(file, error));
                    }
                };
                lineas.push(json_from_vorbis(formato, &tag, tag.pictures()));
            } else {
                display_tags_ogg(&tag);
            }
        }
        Backend::Flac => {
            // Como en Ogg, el comentario se materializa siempre (vacío si el
            // FLAC no tiene bloque `VORBIS_COMMENT`). Pero las carátulas de un
            // FLAC viven en bloques `PICTURE` del archivo, no en el comentario.
            let tag = match read_vorbis_tag(file) {
                Ok(tag) => tag,
                Err(error) => {
                    return Err(motivo_vorbis(file, error));
                }
            };
            let pictures = match read_flac_pictures(file) {
                Ok(pictures) => pictures,
                Err(error) => {
                    return Err(motivo_vorbis(file, error));
                }
            };

            if json {
                lineas.push(json_from_vorbis("flac", &tag, &pictures));
            } else {
                display_tags_flac(&pictures, &tag);
            }
        }
    }

    Ok(())
}

/// Construye el motivo de un fallo del backend MP4 con el mismo texto de dos
/// líneas que el CLI usaba para MP4: causa y enumera los formatos soportados.
fn motivo_mp4(file: &Path, error: impl std::fmt::Display) -> String {
    format!(
        "no se pudo procesar el archivo MP4 '{}': {}\n  El archivo tiene formato \
         MP4/M4A pero no se pudo interpretar. Formatos soportados: MP3, MP4, OGG, \
         Opus y FLAC.",
        file.display(),
        error
    )
}

/// Construye el motivo de un fallo del backend de comentarios de Vorbis (Ogg
/// Vorbis, Opus o FLAC nativo) con el mismo texto de dos líneas que
/// [`report_vorbis_error`] usaba: causa y enumera los formatos soportados.
fn motivo_vorbis(file: &Path, error: impl std::fmt::Display) -> String {
    format!(
        "no se pudo procesar el archivo '{}': {}\n  El archivo tiene un comentario de Vorbis \
         (Ogg Vorbis, Opus o FLAC nativo) pero no se pudo interpretar. Formatos soportados: \
         MP3, MP4, OGG, Opus y FLAC.",
        file.display(),
        error
    )
}

/// Aplica los metadatos indicados a un archivo y devuelve el motivo del fallo
/// en español, o `Ok(())` si se completó.
///
/// Es el contenido de la rama `edit` del CLI (detección del formato por
/// contenido y edición por backend), convertido de «imprimir el error y
/// terminar» a «devolver el motivo»: el modo batch lo despliega por archivo
/// acumulando los fallos sin abortar el resto.
fn edit_archivo(file: &Path, args: &EditArgs<'_>) -> Result<(), String> {
    match detect_supported_format_result(file)? {
        Backend::Mp3 => {
            // Leer o crear tag
            let mut tag = match read_id3v2_tag(file) {
                Some(tag) => {
                    outln!("Tags existentes encontrados en '{}'", file.display());
                    tag
                }
                None => {
                    outln!("Creando nuevos tags para '{}'", file.display());
                    Id3v2Tag::new()
                }
            };

            // Aplicar metadatos
            let changed = apply_metadata(
                &mut tag,
                args.title,
                args.artists,
                args.album,
                args.year,
                args.genre,
                args.track,
                args.season,
                args.date,
                args.copyright,
                args.composer,
                args.subtitle,
                args.original_artist,
                args.album_artist,
            );

            // Imprimir cambios aplicados
            if let Some(title) = args.title {
                outln!("✓ Título: {}", title);
            }
            if !args.artists.is_empty() {
                outln!("✓ Artista(s): {}", args.artists.join("; "));
            }
            if let Some(album) = args.album {
                outln!("✓ Álbum: {}", album);
            }
            if let Some(year) = args.year {
                outln!("✓ Año: {}", year);
            }
            if let Some(genre) = args.genre {
                outln!("✓ Género: {}", genre);
            }
            if let Some(track) = args.track {
                outln!("✓ Pista: {}", track);
            }
            if let Some(season) = args.season {
                outln!("✓ Temporada: {}", season);
            }
            if let Some(date) = args.date {
                outln!("✓ Fecha: {}", date);
            }
            if let Some(copyright) = args.copyright {
                outln!("✓ Copyright: {}", copyright);
            }
            if let Some(composer) = args.composer {
                outln!("✓ Compositor: {}", composer);
            }
            if let Some(subtitle) = args.subtitle {
                outln!("✓ Subtítulo: {}", subtitle);
            }
            if let Some(original_artist) = args.original_artist {
                outln!("✓ Artista original: {}", original_artist);
            }
            if let Some(album_artist) = args.album_artist {
                outln!("✓ Artista del álbum: {}", album_artist);
            }

            // Añadir lyrics
            let mut lyrics_added = false;
            if let Some(lyrics_text) = args.lyrics {
                add_lyrics(&mut tag, lyrics_text);
                let line_count = lyrics_text.lines().count();
                outln!("✓ Letra: {} línea(s)", line_count);
                lyrics_added = true;
            }

            // Añadir URL
            let mut url_added = false;
            if let Some(url) = args.url {
                add_url(&mut tag, url);
                outln!("✓ URL: {}", url);
                url_added = true;
            }

            // Añadir metadatos de Apple
            let apple_added = add_apple_metadata(
                &mut tag,
                args.compilation,
                args.album_sort,
                args.artist_sort,
                args.title_sort,
            );

            if args.compilation {
                outln!("✓ Compilación: Sí");
            }
            if let Some(sort) = args.album_sort {
                outln!("✓ Orden álbum: {}", sort);
            }
            if let Some(sort) = args.artist_sort {
                outln!("✓ Orden artista: {}", sort);
            }
            if let Some(sort) = args.title_sort {
                outln!("✓ Orden título: {}", sort);
            }

            // Añadir carátula
            let mut cover_added = false;
            if let Some(cover_path) = args.cover {
                if !cover_path.exists() {
                    return Err(format!(
                        "El archivo de carátula '{}' no existe",
                        cover_path.display()
                    ));
                }

                let cover_data = fs::read(cover_path).map_err(|error| error.to_string())?;
                match add_cover_art(&mut tag, cover_path, cover_data) {
                    Ok(_) => {
                        outln!("✓ Carátula añadida desde: {}", cover_path.display());
                        cover_added = true;
                    }
                    Err(error) => {
                        return Err(error.to_string());
                    }
                }
            }

            // Guardar cambios
            if changed || cover_added || lyrics_added || url_added || apple_added {
                if let Err(e) = tag.save_to_path(file, WriteOptions::default()) {
                    return Err(motivo_con_causas(&e));
                }
                outln!("\n✅ Tags guardados correctamente en '{}'", file.display());
            } else {
                outln!("\n⚠️  No se especificaron cambios. Usa --help para ver las opciones.");
            }
        }
        Backend::Mp4 => {
            // La carátula se valida ANTES de escribir nada: si la ruta no
            // existe o la extensión no está soportada, se devuelve el error sin
            // haber tocado el archivo (mismo contrato que en MP3).
            let cover_data = match args.cover {
                Some(cover_path) => {
                    if !cover_path.exists() {
                        return Err(format!(
                            "El archivo de carátula '{}' no existe",
                            cover_path.display()
                        ));
                    }
                    detect_cover_mime_type(cover_path)?;
                    Some(fs::read(cover_path).map_err(|error| error.to_string())?)
                }
                None => None,
            };

            let cover_input = match (args.cover, cover_data.as_deref()) {
                (Some(path), Some(data)) => Some(Mp4Cover { path, data }),
                _ => None,
            };

            let mp4_options = Mp4EditOptions {
                title: args.title,
                artists: args.artists,
                album: args.album,
                album_artist: args.album_artist,
                year: args.year,
                genre: args.genre,
                track: args.track,
                season: args.season,
                date: args.date,
                copyright: args.copyright,
                composer: args.composer,
                subtitle: args.subtitle,
                lyrics: args.lyrics,
                compilation: args.compilation,
                title_sort: args.title_sort,
                artist_sort: args.artist_sort,
                album_sort: args.album_sort,
                cover: cover_input,
                url: args.url,
                original_artist: args.original_artist,
            };

            let report = match edit_mp4(file, &mp4_options) {
                Ok(report) => report,
                Err(error) => {
                    return Err(motivo_mp4(file, error));
                }
            };

            // Confirmaciones. `--url` y `--original-artist` no tienen
            // átomo en MP4: `edit_mp4` ya avisa por stderr de su omisión,
            // así que aquí no se confirman.
            if let Some(title) = args.title {
                outln!("✓ Título: {}", title);
            }
            if !args.artists.is_empty() {
                outln!("✓ Artista(s): {}", args.artists.join("; "));
            }
            if let Some(album) = args.album {
                outln!("✓ Álbum: {}", album);
            }
            if let Some(album_artist) = args.album_artist {
                outln!("✓ Artista del álbum: {}", album_artist);
            }
            if let Some(year) = args.year
                && args.date.is_none()
            {
                outln!("✓ Año: {}", year);
            }
            if let Some(genre) = args.genre {
                outln!("✓ Género: {}", genre);
            }
            if let Some(track) = args.track {
                outln!("✓ Pista: {}", track);
            }
            if let Some(season) = args.season {
                outln!("✓ Temporada: {}", season);
            }
            if let Some(date) = args.date {
                outln!("✓ Fecha: {}", date);
            }
            if let Some(copyright) = args.copyright {
                outln!("✓ Copyright: {}", copyright);
            }
            if let Some(composer) = args.composer {
                outln!("✓ Compositor: {}", composer);
            }
            if let Some(subtitle) = args.subtitle {
                outln!("✓ Subtítulo: {}", subtitle);
            }
            if let Some(lyrics_text) = args.lyrics {
                outln!("✓ Letra: {} línea(s)", lyrics_text.lines().count());
            }
            if let Some(cover_path) = args.cover {
                outln!("✓ Carátula añadida desde: {}", cover_path.display());
            }
            if args.compilation {
                outln!("✓ Compilación: Sí");
            }
            if let Some(sort) = args.album_sort {
                outln!("✓ Orden álbum: {}", sort);
            }
            if let Some(sort) = args.artist_sort {
                outln!("✓ Orden artista: {}", sort);
            }
            if let Some(sort) = args.title_sort {
                outln!("✓ Orden título: {}", sort);
            }

            if report.changed {
                outln!(
                    "\n✅ Metadatos guardados correctamente en '{}'",
                    file.display()
                );
            } else {
                outln!("\n⚠️  No se especificaron cambios. Usa --help para ver las opciones.");
            }
        }
        Backend::Ogg | Backend::Flac => {
            // La carátula se valida ANTES de escribir nada: si la ruta no
            // existe o la extensión no está soportada, se devuelve el error sin
            // haber tocado el archivo (mismo contrato que en MP3 y MP4).
            let cover_data = match args.cover {
                Some(cover_path) => {
                    if !cover_path.exists() {
                        return Err(format!(
                            "El archivo de carátula '{}' no existe",
                            cover_path.display()
                        ));
                    }
                    detect_cover_mime_type(cover_path)?;
                    Some(fs::read(cover_path).map_err(|error| error.to_string())?)
                }
                None => None,
            };

            let cover_input = match (args.cover, cover_data.as_deref()) {
                (Some(path), Some(data)) => Some(OggCover { path, data }),
                _ => None,
            };

            let ogg_options = OggEditOptions {
                title: args.title,
                artists: args.artists,
                album: args.album,
                album_artist: args.album_artist,
                year: args.year,
                genre: args.genre,
                track: args.track,
                season: args.season,
                date: args.date,
                copyright: args.copyright,
                composer: args.composer,
                subtitle: args.subtitle,
                lyrics: args.lyrics,
                compilation: args.compilation,
                title_sort: args.title_sort,
                artist_sort: args.artist_sort,
                album_sort: args.album_sort,
                cover: cover_input,
                url: args.url,
                original_artist: args.original_artist,
            };

            let report = match edit_ogg(file, &ogg_options) {
                Ok(report) => report,
                Err(error) => {
                    return Err(motivo_vorbis(file, error));
                }
            };

            // Confirmaciones. `--url` y `--original-artist` no tienen clave
            // en el comentario Vorbis: `edit_ogg` ya avisa por stderr de su
            // omisión, así que aquí no se confirman.
            if let Some(title) = args.title {
                outln!("✓ Título: {}", title);
            }
            if !args.artists.is_empty() {
                outln!("✓ Artista(s): {}", args.artists.join("; "));
            }
            if let Some(album) = args.album {
                outln!("✓ Álbum: {}", album);
            }
            if let Some(album_artist) = args.album_artist {
                outln!("✓ Artista del álbum: {}", album_artist);
            }
            if let Some(year) = args.year {
                outln!("✓ Año: {}", year);
            }
            if let Some(genre) = args.genre {
                outln!("✓ Género: {}", genre);
            }
            if let Some(track) = args.track {
                outln!("✓ Pista: {}", track);
            }
            if let Some(season) = args.season {
                outln!("✓ Temporada: {}", season);
            }
            if let Some(date) = args.date {
                outln!("✓ Fecha: {}", date);
            }
            if let Some(copyright) = args.copyright {
                outln!("✓ Copyright: {}", copyright);
            }
            if let Some(composer) = args.composer {
                outln!("✓ Compositor: {}", composer);
            }
            if let Some(subtitle) = args.subtitle {
                outln!("✓ Subtítulo: {}", subtitle);
            }
            if let Some(lyrics_text) = args.lyrics {
                outln!("✓ Letra: {} línea(s)", lyrics_text.lines().count());
            }
            if let Some(cover_path) = args.cover {
                outln!("✓ Carátula añadida desde: {}", cover_path.display());
            }
            if args.compilation {
                outln!("✓ Compilación: Sí");
            }
            if let Some(sort) = args.album_sort {
                outln!("✓ Orden álbum: {}", sort);
            }
            if let Some(sort) = args.artist_sort {
                outln!("✓ Orden artista: {}", sort);
            }
            if let Some(sort) = args.title_sort {
                outln!("✓ Orden título: {}", sort);
            }

            if report.changed {
                outln!(
                    "\n✅ Metadatos guardados correctamente en '{}'",
                    file.display()
                );
            } else {
                outln!("\n⚠️  No se especificaron cambios. Usa --help para ver las opciones.");
            }
        }
    }

    Ok(())
}

/// Procesa un archivo del subcomando `remove` y devuelve el motivo del fallo en
/// español, o `Ok(())` si se completó.
///
/// Es el contenido de la rama `remove` del CLI, convertido igual que
/// [`edit_archivo`]: los fallos se devuelven para que el modo batch los acumule
/// sin abortar el resto de archivos.
fn remove_archivo(file: &Path, tags: &[String], all: bool) -> Result<(), String> {
    match detect_supported_format_result(file)? {
        Backend::Mp3 => {
            // Eliminar todos los tags (ID3v1 e ID3v2) físicamente
            if all {
                let format = remove_all_id3(file).map_err(|error| error.to_string())?;
                match format {
                    RemovedId3::None => {
                        outln!("⚠️  El archivo '{}' no tenía tags ID3", file.display());
                    }
                    RemovedId3::Id3v1 => {
                        outln!("✅ Tag ID3v1 eliminado de '{}'", file.display());
                    }
                    RemovedId3::Id3v2 => {
                        outln!("✅ Tag ID3v2 eliminado de '{}'", file.display());
                    }
                    RemovedId3::Both => {
                        outln!("✅ Tags ID3v1 e ID3v2 eliminados de '{}'", file.display());
                    }
                }
                return Ok(());
            }

            // Verificar que se especificaron tags para eliminar
            if tags.is_empty() {
                return Err("Debes especificar al menos un tag para eliminar".to_string());
            }

            // Leer tag
            let mut tag = match read_id3v2_tag(file) {
                Some(tag) => tag,
                None => {
                    return Err(format!(
                        "No se encontraron tags ID3 en '{}'",
                        file.display()
                    ));
                }
            };

            // Eliminar tags
            let removed = remove_tags(&mut tag, tags);

            if removed {
                if let Err(e) = tag.save_to_path(file, WriteOptions::default()) {
                    return Err(format!(
                        "no se pudo guardar el archivo '{}': {}",
                        file.display(),
                        e
                    ));
                }
                outln!("\n✅ Tags eliminados correctamente de '{}'", file.display());
            } else {
                outln!("\n⚠️  No se encontraron tags válidos para eliminar.");
            }
        }
        Backend::Mp4 => {
            // Eliminar el bloque de metadatos `ilst` completo
            if all {
                match remove_all_mp4(file) {
                    Ok(true) => {
                        outln!("✅ Metadatos eliminados de '{}'", file.display());
                    }
                    Ok(false) => {
                        outln!("⚠️  El archivo '{}' no tenía metadatos", file.display());
                    }
                    Err(error) => {
                        return Err(motivo_mp4(file, error));
                    }
                }
                return Ok(());
            }

            // Verificar que se especificaron tags para eliminar
            if tags.is_empty() {
                return Err("Debes especificar al menos un tag para eliminar".to_string());
            }

            // Leer tag
            let mut tag = match read_mp4_tag(file) {
                Ok(Some(tag)) => tag,
                Ok(None) => {
                    return Err(format!(
                        "No se encontraron metadatos en '{}'",
                        file.display()
                    ));
                }
                Err(error) => {
                    return Err(motivo_mp4(file, error));
                }
            };

            // Eliminar tags (misma tabla de alias que en MP3)
            let removed = remove_tags_mp4(&mut tag, tags);

            if removed {
                if let Err(e) = write_mp4_tag(file, &tag) {
                    return Err(motivo_mp4(file, e));
                }
                outln!(
                    "\n✅ Metadatos eliminados correctamente de '{}'",
                    file.display()
                );
            } else {
                outln!("\n⚠️  No se encontraron tags válidos para eliminar.");
            }
        }
        Backend::Ogg => {
            // Vaciar el comentario de Vorbis preservando el vendor
            if all {
                match remove_all_ogg(file) {
                    Ok(true) => {
                        outln!("✅ Metadatos eliminados de '{}'", file.display());
                    }
                    Ok(false) => {
                        outln!("⚠️  El archivo '{}' no tenía metadatos", file.display());
                    }
                    Err(error) => {
                        return Err(motivo_vorbis(file, error));
                    }
                }
                return Ok(());
            }

            remove_vorbis_tags(file, tags).map_err(|error| error.to_string())?;
        }
        Backend::Flac => {
            // Eliminar por completo el bloque VORBIS_COMMENT y los bloques
            // PICTURE. A diferencia de Ogg/Opus (donde el comentario es
            // obligatorio y se vacía conservando el vendor), en FLAC lofty no
            // escribe un bloque de comentario vacío y el vendor se pierde:
            // degradación documentada del spec `flac-tagging`. `remove_all_flac`
            // cuenta también las carátulas de los bloques PICTURE del archivo
            // para decidir si había metadatos.
            if all {
                match remove_all_flac(file) {
                    Ok(true) => {
                        outln!("✅ Metadatos eliminados de '{}'", file.display());
                    }
                    Ok(false) => {
                        outln!("⚠️  El archivo '{}' no tenía metadatos", file.display());
                    }
                    Err(error) => {
                        return Err(motivo_vorbis(file, error));
                    }
                }
                return Ok(());
            }

            remove_vorbis_tags(file, tags).map_err(|error| error.to_string())?;
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match &args.command {
        Command::Show { files, json } => {
            // Modo batch: un archivo por invocación mantiene la salida byte a
            // byte igual (sin cabeceras y con la línea JSON única); con varios,
            // el modo legible separa cada bloque con `=== <ruta> ===` y el modo
            // JSON emite NDJSON (una línea por archivo) al final del lote. Un
            // fallo por archivo (ruta inexistente, formato no soportado) NO
            // aborta el resto: el fallido no emite cabecera ni línea y el
            // motivo se informa al final con `reportar_fallos` (exit ≠ 0).
            let mut lineas: Vec<String> = Vec::new();
            let mut fallos: Vec<(PathBuf, String)> = Vec::new();
            let cabecera = !*json && files.len() > 1;
            for file in files {
                if let Err(motivo) = show_archivo(file, *json, &mut lineas, cabecera) {
                    fallos.push((file.clone(), motivo));
                }
            }
            if *json && !lineas.is_empty() {
                print_block(&batch_ndjson(&lineas));
            }
            if !fallos.is_empty() {
                reportar_fallos(&fallos, files.len() == 1);
            }
        }
        Command::Edit {
            files,
            title,
            artist,
            album,
            year,
            genre,
            track,
            season,
            date,
            copyright,
            composer,
            subtitle,
            original_artist,
            album_artist,
            cover,
            lyrics,
            url,
            compilation,
            album_sort,
            artist_sort,
            title_sort,
        } => {
            // Modo batch: cada archivo se edita con las mismas opciones; un
            // fallo no aborta el resto y se informa al final como
            // «Error: <ruta>: <motivo>» con exit distinto de cero.
            let edit_args = EditArgs {
                title: title.as_deref(),
                artists: artist,
                album: album.as_deref(),
                year: *year,
                genre: genre.as_deref(),
                track: *track,
                season: *season,
                date: date.as_deref(),
                copyright: copyright.as_deref(),
                composer: composer.as_deref(),
                subtitle: subtitle.as_deref(),
                original_artist: original_artist.as_deref(),
                album_artist: album_artist.as_deref(),
                cover: cover.as_deref(),
                lyrics: lyrics.as_deref(),
                url: url.as_deref(),
                compilation: *compilation,
                album_sort: album_sort.as_deref(),
                artist_sort: artist_sort.as_deref(),
                title_sort: title_sort.as_deref(),
            };
            let outcome = batch_dispatch(files, |file| edit_archivo(file, &edit_args));
            if outcome.has_errors() {
                reportar_fallos(&outcome.errors, files.len() == 1);
            }
        }
        Command::Remove { files, all, .. } => {
            // Con `--all`, todos los tokens posicionales son archivos. Sin
            // `--all`, el positional variádico de clap acumula también los tags
            // (el `tags` con `last = true` solo se llena tras `--`), así que se
            // conserva la sintaxis histórica `remove <archivo> <tags…>`: el
            // primer token es el archivo y el resto son los tags a eliminar.
            // (El batch de `remove` sin `--all` requiere ese orden: un único
            // archivo seguido de sus tags.)
            let (archivos, tags_a_aplicar): (Vec<PathBuf>, Vec<String>) = if *all {
                (files.clone(), Vec::new())
            } else {
                let mut it = files.iter();
                let primero = it.next().cloned();
                (
                    primero.into_iter().collect(),
                    it.map(|token| token.to_string_lossy().into_owned())
                        .collect(),
                )
            };
            // Modo batch: un fallo por archivo no aborta el resto y se informa
            // al final como «Error: <ruta>: <motivo>» con exit distinto de cero.
            let outcome = batch_dispatch(&archivos, |file| {
                remove_archivo(file, &tags_a_aplicar, *all)
            });
            if outcome.has_errors() {
                reportar_fallos(&outcome.errors, archivos.len() == 1);
            }
        }
        Command::Completions { shell } => {
            // El script se genera sobre el árbol de comandos real del CLI (el
            // mismo que `Args::command()` construye con todos los subcomandos y
            // flags) y se emite como un bloque, sin salto de línea extra.
            let mut cmd = Args::command();
            let script = generate_completions(*shell, &mut cmd);
            print_block(&script);
        }
    }

    Ok(())
}
