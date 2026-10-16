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

/// Imprime el error con toda su cadena de causas y termina con código 1.
///
/// Se imprime la causa subyacente porque `lofty` oculta el error de E/S real en
/// su `Debug` (`FileEncodingError { .. }`); la causa conserva el mensaje del
/// sistema (p. ej. `Permission denied`).
fn report_error(error: Box<dyn Error>) -> ! {
    eprintln!("Error: {}", error);
    let mut source = error.source();
    while let Some(cause) = source {
        eprintln!("  Causado por: {}", cause);
        source = cause.source();
    }
    std::process::exit(1);
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
    /// Mostrar todos los metadatos del archivo MP3, MP4, OGG, Opus y FLAC
    Show {
        /// Ruta del archivo de audio (MP3, MP4, OGG, Opus y FLAC)
        file: PathBuf,

        /// Emitir los metadatos como un único objeto JSON compacto en una línea
        #[arg(short, long)]
        json: bool,
    },
    /// Editar metadatos del archivo MP3, MP4, OGG, Opus y FLAC
    Edit {
        /// Ruta del archivo de audio (MP3, MP4, OGG, Opus y FLAC)
        file: PathBuf,

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
    /// Eliminar tags específicos del archivo MP3, MP4, OGG, Opus y FLAC
    Remove {
        /// Ruta del archivo de audio (MP3, MP4, OGG, Opus y FLAC)
        file: PathBuf,

        // Tags a eliminar. La lista se deriva de `id3cli::VALID_TAG_NAMES`, la
        // misma tabla de alias que comparten la clasificación de MP3, la de
        // MP4 y la de OGG, para que la ayuda no pueda quedar desincronizada de
        // los nombres realmente aceptados.
        #[arg(help = format!("Tags a eliminar, en inglés o en español ({VALID_TAG_NAMES})"))]
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

/// Comprueba que el archivo existe y que su formato (por contenido) está
/// soportado, y devuelve el backend con el que hay que procesarlo.
///
/// La decisión se toma sobre los *magic bytes* del archivo, nunca sobre la
/// extensión del nombre. Ante un formato no soportado (Speex, WAV, …) o
/// contenido no reconocible se escribe un mensaje en español en stderr, se sale
/// con código distinto de cero y **no se escribe ni un byte del archivo**.
///
/// Se mantiene el mensaje literal de archivo inexistente que ya usaba el CLI.
fn detect_supported_format(file: &Path) -> Backend {
    if !file.exists() {
        eprintln!("Error: El archivo '{}' no existe", file.display());
        std::process::exit(1);
    }

    match detect_format(file) {
        Ok(AudioFormat::Mp3) => Backend::Mp3,
        Ok(AudioFormat::Mp4) => Backend::Mp4,
        Ok(AudioFormat::OggVorbis) => Backend::Ogg,
        Ok(AudioFormat::OggOpus) => Backend::Ogg,
        Ok(AudioFormat::Flac) => Backend::Flac,
        Ok(AudioFormat::Unsupported) => {
            eprintln!(
                "Error: El archivo '{}' está en un formato de audio no soportado. \
                 Formatos soportados: MP3, MP4, OGG, Opus y FLAC.",
                file.display()
            );
            std::process::exit(1);
        }
        Ok(AudioFormat::Unknown) => {
            eprintln!(
                "Error: No se pudo determinar el formato del archivo '{}'. \
                 Formatos soportados: MP3, MP4, OGG, Opus y FLAC.",
                file.display()
            );
            std::process::exit(1);
        }
        Err(error) => {
            eprintln!(
                "Error: No se pudo leer el archivo '{}': {}",
                file.display(),
                error
            );
            std::process::exit(1);
        }
    }
}

/// Informa de un fallo al procesar un contenedor MP4 y termina con código 1.
///
/// El mensaje enumera los formatos soportados (MP3, MP4, OGG, Opus y FLAC)
/// además de la causa, de modo que un archivo con cabecera MP4 pero estructura
/// inválida se rechaza con un error legible y sin haberlo modificado.
fn report_mp4_error(file: &Path, error: impl std::fmt::Display) -> ! {
    eprintln!(
        "Error: No se pudo procesar el archivo MP4 '{}': {}",
        file.display(),
        error
    );
    eprintln!(
        "  El archivo tiene formato MP4/M4A pero no se pudo interpretar. \
         Formatos soportados: MP3, MP4, OGG, Opus y FLAC."
    );
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match &args.command {
        Command::Show { file, json } => match detect_supported_format(file) {
            Backend::Mp3 => {
                // Leer tag
                let tag = match read_id3v2_tag(file) {
                    Some(tag) => tag,
                    None => {
                        if *json {
                            // Decisión 3: sin tag, el modo JSON emite
                            // `{"formato": "mp3", "tags": null}` con exit 0 y
                            // stderr limpio; el aviso ⚠️ es del modo legible.
                            outln!("{}", json_empty("mp3"));
                        } else {
                            eprintln!("⚠️  No se encontraron tags ID3 en '{}'", file.display());
                        }
                        std::process::exit(0);
                    }
                };

                if *json {
                    outln!("{}", json_from_mp3(&tag));
                } else {
                    display_tags(&tag);
                }
            }
            Backend::Mp4 => {
                let tag = match read_mp4_tag(file) {
                    Ok(Some(tag)) => tag,
                    Ok(None) => {
                        if *json {
                            // Mismo contrato que el MP3 sin tags: objeto con
                            // `tags: null`, exit 0 y stderr limpio.
                            outln!("{}", json_empty("mp4"));
                        } else {
                            eprintln!("⚠️  No se encontraron metadatos en '{}'", file.display());
                        }
                        std::process::exit(0);
                    }
                    Err(error) => report_mp4_error(file, error),
                };

                if *json {
                    outln!("{}", json_from_mp4(&tag));
                } else {
                    display_tags_mp4(&tag);
                }
            }
            Backend::Ogg => {
                // El comentario de Vorbis siempre se materializa: en Ogg/Opus es
                // obligatorio en el contenedor. No hay un `Ok(None)` que tratar
                // como en MP4. `display_tags_ogg` ignora el vendor y `ENCODER` y,
                // cuando no hay ningún campo del mapeo, informa de que no se
                // encontraron metadatos y el proceso termina con código cero. En
                // Ogg/Opus las carátulas viven dentro del propio comentario, así
                // que el tag basta para mostrarlas.
                let tag = match read_vorbis_tag(file) {
                    Ok(tag) => tag,
                    Err(error) => report_vorbis_error(file, error),
                };

                if *json {
                    // Decisión 4: el JSON nombra el códec REAL del contenedor
                    // ("ogg" u "opus"), que el `Backend` fusiona; el nombre se
                    // resuelve por contenido, nunca del backend.
                    let formato = match detect_format(file) {
                        Ok(formato) => format_json_name(formato),
                        Err(error) => report_vorbis_error(file, error),
                    };
                    outln!("{}", json_from_vorbis(formato, &tag, tag.pictures()));
                } else {
                    display_tags_ogg(&tag);
                }
            }
            Backend::Flac => {
                // Como en Ogg, el comentario se materializa siempre (vacío si el
                // FLAC no tiene bloque `VORBIS_COMMENT`). Pero las carátulas de
                // un FLAC viven en bloques `PICTURE` del archivo, no en el
                // comentario: `read_flac_pictures` las lee aparte y
                // `display_tags_flac` las muestra con el mismo formato que
                // `display_tags_ogg` usa para las imágenes.
                let tag = match read_vorbis_tag(file) {
                    Ok(tag) => tag,
                    Err(error) => report_vorbis_error(file, error),
                };
                let pictures = match read_flac_pictures(file) {
                    Ok(pictures) => pictures,
                    Err(error) => report_vorbis_error(file, error),
                };

                if *json {
                    // En FLAC las carátulas son los bloques `PICTURE` del
                    // archivo (mismos que muestra el modo legible).
                    outln!("{}", json_from_vorbis("flac", &tag, &pictures));
                } else {
                    display_tags_flac(&pictures, &tag);
                }
            }
        },
        Command::Edit {
            file,
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
        } => match detect_supported_format(file) {
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
                    title.as_deref(),
                    artist,
                    album.as_deref(),
                    *year,
                    genre.as_deref(),
                    *track,
                    *season,
                    date.as_deref(),
                    copyright.as_deref(),
                    composer.as_deref(),
                    subtitle.as_deref(),
                    original_artist.as_deref(),
                    album_artist.as_deref(),
                );

                // Imprimir cambios aplicados
                if let Some(title) = title {
                    outln!("✓ Título: {}", title);
                }
                if !artist.is_empty() {
                    outln!("✓ Artista(s): {}", artist.join("; "));
                }
                if let Some(album) = album {
                    outln!("✓ Álbum: {}", album);
                }
                if let Some(year) = year {
                    outln!("✓ Año: {}", year);
                }
                if let Some(genre) = genre {
                    outln!("✓ Género: {}", genre);
                }
                if let Some(track) = track {
                    outln!("✓ Pista: {}", track);
                }
                if let Some(season) = season {
                    outln!("✓ Temporada: {}", season);
                }
                if let Some(date) = date {
                    outln!("✓ Fecha: {}", date);
                }
                if let Some(copyright) = copyright {
                    outln!("✓ Copyright: {}", copyright);
                }
                if let Some(composer) = composer {
                    outln!("✓ Compositor: {}", composer);
                }
                if let Some(subtitle) = subtitle {
                    outln!("✓ Subtítulo: {}", subtitle);
                }
                if let Some(original_artist) = original_artist {
                    outln!("✓ Artista original: {}", original_artist);
                }
                if let Some(album_artist) = album_artist {
                    outln!("✓ Artista del álbum: {}", album_artist);
                }

                // Añadir lyrics
                let mut lyrics_added = false;
                if let Some(lyrics_text) = lyrics {
                    add_lyrics(&mut tag, lyrics_text);
                    let line_count = lyrics_text.lines().count();
                    outln!("✓ Letra: {} línea(s)", line_count);
                    lyrics_added = true;
                }

                // Añadir URL
                let mut url_added = false;
                if let Some(url) = url {
                    add_url(&mut tag, url);
                    outln!("✓ URL: {}", url);
                    url_added = true;
                }

                // Añadir metadatos de Apple
                let apple_added = add_apple_metadata(
                    &mut tag,
                    *compilation,
                    album_sort.as_deref(),
                    artist_sort.as_deref(),
                    title_sort.as_deref(),
                );

                if *compilation {
                    outln!("✓ Compilación: Sí");
                }
                if let Some(sort) = album_sort {
                    outln!("✓ Orden álbum: {}", sort);
                }
                if let Some(sort) = artist_sort {
                    outln!("✓ Orden artista: {}", sort);
                }
                if let Some(sort) = title_sort {
                    outln!("✓ Orden título: {}", sort);
                }

                // Añadir carátula
                let mut cover_added = false;
                if let Some(cover_path) = cover {
                    if !cover_path.exists() {
                        eprintln!(
                            "Error: El archivo de carátula '{}' no existe",
                            cover_path.display()
                        );
                        std::process::exit(1);
                    }

                    let cover_data = fs::read(cover_path)?;
                    match add_cover_art(&mut tag, cover_path, cover_data) {
                        Ok(_) => {
                            outln!("✓ Carátula añadida desde: {}", cover_path.display());
                            cover_added = true;
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }

                // Guardar cambios
                if changed || cover_added || lyrics_added || url_added || apple_added {
                    if let Err(e) = tag.save_to_path(file, WriteOptions::default()) {
                        report_error(Box::new(e));
                    }
                    outln!("\n✅ Tags guardados correctamente en '{}'", file.display());
                } else {
                    outln!("\n⚠️  No se especificaron cambios. Usa --help para ver las opciones.");
                }
            }
            Backend::Mp4 => {
                // La carátula se valida ANTES de escribir nada: si la ruta no
                // existe o la extensión no está soportada, se sale con error sin
                // haber tocado el archivo (mismo contrato que en MP3).
                let cover_data = match cover {
                    Some(cover_path) => {
                        if !cover_path.exists() {
                            eprintln!(
                                "Error: El archivo de carátula '{}' no existe",
                                cover_path.display()
                            );
                            std::process::exit(1);
                        }
                        if let Err(e) = detect_cover_mime_type(cover_path) {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                        Some(fs::read(cover_path)?)
                    }
                    None => None,
                };

                let cover_input = match (cover, cover_data.as_deref()) {
                    (Some(path), Some(data)) => Some(Mp4Cover {
                        path: path.as_path(),
                        data,
                    }),
                    _ => None,
                };

                let mp4_options = Mp4EditOptions {
                    title: title.as_deref(),
                    artists: artist,
                    album: album.as_deref(),
                    album_artist: album_artist.as_deref(),
                    year: *year,
                    genre: genre.as_deref(),
                    track: *track,
                    season: *season,
                    date: date.as_deref(),
                    copyright: copyright.as_deref(),
                    composer: composer.as_deref(),
                    subtitle: subtitle.as_deref(),
                    lyrics: lyrics.as_deref(),
                    compilation: *compilation,
                    title_sort: title_sort.as_deref(),
                    artist_sort: artist_sort.as_deref(),
                    album_sort: album_sort.as_deref(),
                    cover: cover_input,
                    url: url.as_deref(),
                    original_artist: original_artist.as_deref(),
                };

                let report = match edit_mp4(file, &mp4_options) {
                    Ok(report) => report,
                    Err(error) => report_mp4_error(file, error),
                };

                // Confirmaciones. `--url` y `--original-artist` no tienen
                // átomo en MP4: `edit_mp4` ya avisa por stderr de su omisión,
                // así que aquí no se confirman.
                if let Some(title) = title {
                    outln!("✓ Título: {}", title);
                }
                if !artist.is_empty() {
                    outln!("✓ Artista(s): {}", artist.join("; "));
                }
                if let Some(album) = album {
                    outln!("✓ Álbum: {}", album);
                }
                if let Some(album_artist) = album_artist {
                    outln!("✓ Artista del álbum: {}", album_artist);
                }
                if let Some(year) = year
                    && date.is_none()
                {
                    outln!("✓ Año: {}", year);
                }
                if let Some(genre) = genre {
                    outln!("✓ Género: {}", genre);
                }
                if let Some(track) = track {
                    outln!("✓ Pista: {}", track);
                }
                if let Some(season) = season {
                    outln!("✓ Temporada: {}", season);
                }
                if let Some(date) = date {
                    outln!("✓ Fecha: {}", date);
                }
                if let Some(copyright) = copyright {
                    outln!("✓ Copyright: {}", copyright);
                }
                if let Some(composer) = composer {
                    outln!("✓ Compositor: {}", composer);
                }
                if let Some(subtitle) = subtitle {
                    outln!("✓ Subtítulo: {}", subtitle);
                }
                if let Some(lyrics_text) = lyrics {
                    outln!("✓ Letra: {} línea(s)", lyrics_text.lines().count());
                }
                if let Some(cover_path) = cover {
                    outln!("✓ Carátula añadida desde: {}", cover_path.display());
                }
                if *compilation {
                    outln!("✓ Compilación: Sí");
                }
                if let Some(sort) = album_sort {
                    outln!("✓ Orden álbum: {}", sort);
                }
                if let Some(sort) = artist_sort {
                    outln!("✓ Orden artista: {}", sort);
                }
                if let Some(sort) = title_sort {
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
                // existe o la extensión no está soportada, se sale con error sin
                // haber tocado el archivo (mismo contrato que en MP3 y MP4).
                let cover_data = match cover {
                    Some(cover_path) => {
                        if !cover_path.exists() {
                            eprintln!(
                                "Error: El archivo de carátula '{}' no existe",
                                cover_path.display()
                            );
                            std::process::exit(1);
                        }
                        if let Err(e) = detect_cover_mime_type(cover_path) {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                        Some(fs::read(cover_path)?)
                    }
                    None => None,
                };

                let cover_input = match (cover, cover_data.as_deref()) {
                    (Some(path), Some(data)) => Some(OggCover {
                        path: path.as_path(),
                        data,
                    }),
                    _ => None,
                };

                let ogg_options = OggEditOptions {
                    title: title.as_deref(),
                    artists: artist,
                    album: album.as_deref(),
                    album_artist: album_artist.as_deref(),
                    year: *year,
                    genre: genre.as_deref(),
                    track: *track,
                    season: *season,
                    date: date.as_deref(),
                    copyright: copyright.as_deref(),
                    composer: composer.as_deref(),
                    subtitle: subtitle.as_deref(),
                    lyrics: lyrics.as_deref(),
                    compilation: *compilation,
                    title_sort: title_sort.as_deref(),
                    artist_sort: artist_sort.as_deref(),
                    album_sort: album_sort.as_deref(),
                    cover: cover_input,
                    url: url.as_deref(),
                    original_artist: original_artist.as_deref(),
                };

                let report = match edit_ogg(file, &ogg_options) {
                    Ok(report) => report,
                    Err(error) => report_vorbis_error(file, error),
                };

                // Confirmaciones. `--url` y `--original-artist` no tienen clave
                // en el comentario Vorbis: `edit_ogg` ya avisa por stderr de su
                // omisión, así que aquí no se confirman.
                if let Some(title) = title {
                    outln!("✓ Título: {}", title);
                }
                if !artist.is_empty() {
                    outln!("✓ Artista(s): {}", artist.join("; "));
                }
                if let Some(album) = album {
                    outln!("✓ Álbum: {}", album);
                }
                if let Some(album_artist) = album_artist {
                    outln!("✓ Artista del álbum: {}", album_artist);
                }
                if let Some(year) = year {
                    outln!("✓ Año: {}", year);
                }
                if let Some(genre) = genre {
                    outln!("✓ Género: {}", genre);
                }
                if let Some(track) = track {
                    outln!("✓ Pista: {}", track);
                }
                if let Some(season) = season {
                    outln!("✓ Temporada: {}", season);
                }
                if let Some(date) = date {
                    outln!("✓ Fecha: {}", date);
                }
                if let Some(copyright) = copyright {
                    outln!("✓ Copyright: {}", copyright);
                }
                if let Some(composer) = composer {
                    outln!("✓ Compositor: {}", composer);
                }
                if let Some(subtitle) = subtitle {
                    outln!("✓ Subtítulo: {}", subtitle);
                }
                if let Some(lyrics_text) = lyrics {
                    outln!("✓ Letra: {} línea(s)", lyrics_text.lines().count());
                }
                if let Some(cover_path) = cover {
                    outln!("✓ Carátula añadida desde: {}", cover_path.display());
                }
                if *compilation {
                    outln!("✓ Compilación: Sí");
                }
                if let Some(sort) = album_sort {
                    outln!("✓ Orden álbum: {}", sort);
                }
                if let Some(sort) = artist_sort {
                    outln!("✓ Orden artista: {}", sort);
                }
                if let Some(sort) = title_sort {
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
        },
        Command::Remove { file, tags, all } => match detect_supported_format(file) {
            Backend::Mp3 => {
                // Eliminar todos los tags (ID3v1 e ID3v2) físicamente
                if *all {
                    let format = remove_all_id3(file)?;
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
                    eprintln!("Error: Debes especificar al menos un tag para eliminar");
                    eprintln!("Ejemplo: id3cli remove cancion.mp3 title artist");
                    std::process::exit(1);
                }

                // Leer tag
                let mut tag = match read_id3v2_tag(file) {
                    Some(tag) => tag,
                    None => {
                        eprintln!("Error: No se encontraron tags ID3 en '{}'", file.display());
                        std::process::exit(1);
                    }
                };

                // Eliminar tags
                let removed = remove_tags(&mut tag, tags);

                if removed {
                    if let Err(e) = tag.save_to_path(file, WriteOptions::default()) {
                        report_error(Box::new(e));
                    }
                    outln!("\n✅ Tags eliminados correctamente de '{}'", file.display());
                } else {
                    outln!("\n⚠️  No se encontraron tags válidos para eliminar.");
                }
            }
            Backend::Mp4 => {
                // Eliminar el bloque de metadatos `ilst` completo
                if *all {
                    match remove_all_mp4(file) {
                        Ok(true) => {
                            outln!("✅ Metadatos eliminados de '{}'", file.display());
                        }
                        Ok(false) => {
                            outln!("⚠️  El archivo '{}' no tenía metadatos", file.display());
                        }
                        Err(error) => report_mp4_error(file, error),
                    }
                    return Ok(());
                }

                // Verificar que se especificaron tags para eliminar
                if tags.is_empty() {
                    eprintln!("Error: Debes especificar al menos un tag para eliminar");
                    eprintln!("Ejemplo: id3cli remove cancion.m4a title artist");
                    std::process::exit(1);
                }

                // Leer tag
                let mut tag = match read_mp4_tag(file) {
                    Ok(Some(tag)) => tag,
                    Ok(None) => {
                        eprintln!("Error: No se encontraron metadatos en '{}'", file.display());
                        std::process::exit(1);
                    }
                    Err(error) => report_mp4_error(file, error),
                };

                // Eliminar tags (misma tabla de alias que en MP3)
                let removed = remove_tags_mp4(&mut tag, tags);

                if removed {
                    if let Err(e) = write_mp4_tag(file, &tag) {
                        report_mp4_error(file, e);
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
                if *all {
                    match remove_all_ogg(file) {
                        Ok(true) => {
                            outln!("✅ Metadatos eliminados de '{}'", file.display());
                        }
                        Ok(false) => {
                            outln!("⚠️  El archivo '{}' no tenía metadatos", file.display());
                        }
                        Err(error) => report_vorbis_error(file, error),
                    }
                    return Ok(());
                }

                remove_vorbis_tags(file, tags)?;
            }
            Backend::Flac => {
                // Eliminar por completo el bloque VORBIS_COMMENT y los bloques
                // PICTURE. A diferencia de Ogg/Opus (donde el comentario es
                // obligatorio y se vacía conservando el vendor), en FLAC lofty no
                // escribe un bloque de comentario vacío y el vendor se pierde:
                // degradación documentada del spec `flac-tagging`. `remove_all_flac`
                // cuenta también las carátulas de los bloques PICTURE del archivo
                // para decidir si había metadatos.
                if *all {
                    match remove_all_flac(file) {
                        Ok(true) => {
                            outln!("✅ Metadatos eliminados de '{}'", file.display());
                        }
                        Ok(false) => {
                            outln!("⚠️  El archivo '{}' no tenía metadatos", file.display());
                        }
                        Err(error) => report_vorbis_error(file, error),
                    }
                    return Ok(());
                }

                remove_vorbis_tags(file, tags)?;
            }
        },
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
