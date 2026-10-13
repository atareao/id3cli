use clap::{Parser, Subcommand};
use id3cli::*;
use lofty::config::{ParseOptions, WriteOptions};
use lofty::file::AudioFile;
use lofty::id3::v2::Id3v2Tag;
use lofty::mpeg::MpegFile;
use lofty::tag::TagExt;
use lofty::tag::TagType;
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

/// Escribe el tag ID3v2 en el archivo conservando la paridad observable del
/// backend anterior.
///
/// `lofty` elimina físicamente el bloque ID3v2 cuando se guarda un tag sin
/// ningún frame; el contrato conserva un bloque ID3v2 vacío (como hacía el
/// crate `id3`), de modo que tras borrar el último frame el archivo sigue
/// teniendo un tag legible. Por eso, cuando el tag queda vacío se retira el
/// bloque existente y se antepone a mano una cabecera ID3v2.4 vacía de 10 bytes.
fn save_tag(file: &Path, tag: &Id3v2Tag) -> Result<(), Box<dyn std::error::Error>> {
    if !tag.is_empty() {
        tag.save_to_path(file, WriteOptions::default())?;
        return Ok(());
    }

    let mut handle = File::options().read(true).write(true).open(file)?;
    TagType::Id3v2.remove_from(&mut handle, WriteOptions::default())?;
    drop(handle);

    let audio = fs::read(file)?;
    let mut contents = Vec::with_capacity(10 + audio.len());
    // Cabecera ID3v2.4 vacía: "ID3", versión 4.0, flags 0, tamaño sincsafe 0.
    contents.extend_from_slice(b"ID3\x04\x00\x00\x00\x00\x00\x00");
    contents.extend_from_slice(&audio);
    fs::write(file, &contents)?;

    Ok(())
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

/// CLI para añadir tags ID3 y carátulas a archivos MP3
#[derive(Parser, Debug)]
#[command(name = "id3cli")]
#[command(about = "Añade tags ID3 y carátulas a archivos MP3", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
#[allow(clippy::large_enum_variant, clippy::too_many_arguments)]
enum Command {
    /// Mostrar todos los tags del archivo MP3
    Show {
        /// Ruta del archivo MP3
        file: PathBuf,
    },
    /// Editar tags ID3 del archivo MP3
    Edit {
        /// Ruta del archivo MP3
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

        /// Número de pista
        #[arg(short = 'T', long)]
        track: Option<u32>,

        /// Temporada (TPOS - útil para podcasts)
        #[arg(short = 'S', long)]
        season: Option<u32>,

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
    /// Eliminar tags específicos del archivo MP3
    Remove {
        /// Ruta del archivo MP3
        file: PathBuf,

        /// Tags a eliminar (title, artist, album, year, genre, track, season, date, copyright, cover, lyrics, url, compilation, album_sort, artist_sort, title_sort)
        tags: Vec<String>,

        /// Eliminar físicamente todos los tags ID3 (v1 y v2) del archivo
        #[arg(short, long)]
        all: bool,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match &args.command {
        Command::Show { file } => {
            // Verificar que el archivo MP3 existe
            if !file.exists() {
                eprintln!("Error: El archivo '{}' no existe", file.display());
                std::process::exit(1);
            }

            // Leer tag
            let tag = match read_id3v2_tag(file) {
                Some(tag) => tag,
                None => {
                    eprintln!("⚠️  No se encontraron tags ID3 en '{}'", file.display());
                    std::process::exit(0);
                }
            };

            display_tags(&tag);
        }
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
        } => {
            // Verificar que el archivo MP3 existe
            if !file.exists() {
                eprintln!("Error: El archivo '{}' no existe", file.display());
                std::process::exit(1);
            }

            // Leer o crear tag
            let mut tag = match read_id3v2_tag(file) {
                Some(tag) => {
                    println!("Tags existentes encontrados en '{}'", file.display());
                    tag
                }
                None => {
                    println!("Creando nuevos tags para '{}'", file.display());
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
                println!("✓ Título: {}", title);
            }
            if !artist.is_empty() {
                println!("✓ Artista(s): {}", artist.join("; "));
            }
            if let Some(album) = album {
                println!("✓ Álbum: {}", album);
            }
            if let Some(year) = year {
                println!("✓ Año: {}", year);
            }
            if let Some(genre) = genre {
                println!("✓ Género: {}", genre);
            }
            if let Some(track) = track {
                println!("✓ Pista: {}", track);
            }
            if let Some(season) = season {
                println!("✓ Temporada: {}", season);
            }
            if let Some(date) = date {
                println!("✓ Fecha: {}", date);
            }
            if let Some(copyright) = copyright {
                println!("✓ Copyright: {}", copyright);
            }
            if let Some(composer) = composer {
                println!("✓ Compositor: {}", composer);
            }
            if let Some(subtitle) = subtitle {
                println!("✓ Subtítulo: {}", subtitle);
            }
            if let Some(original_artist) = original_artist {
                println!("✓ Artista original: {}", original_artist);
            }
            if let Some(album_artist) = album_artist {
                println!("✓ Artista del álbum: {}", album_artist);
            }

            // Añadir lyrics
            let mut lyrics_added = false;
            if let Some(lyrics_text) = lyrics {
                add_lyrics(&mut tag, lyrics_text);
                let line_count = lyrics_text.lines().count();
                println!("✓ Letra: {} línea(s)", line_count);
                lyrics_added = true;
            }

            // Añadir URL
            let mut url_added = false;
            if let Some(url) = url {
                add_url(&mut tag, url);
                println!("✓ URL: {}", url);
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
                println!("✓ Compilación: Sí");
            }
            if let Some(sort) = album_sort {
                println!("✓ Orden álbum: {}", sort);
            }
            if let Some(sort) = artist_sort {
                println!("✓ Orden artista: {}", sort);
            }
            if let Some(sort) = title_sort {
                println!("✓ Orden título: {}", sort);
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
                        println!("✓ Carátula añadida desde: {}", cover_path.display());
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
                if let Err(e) = save_tag(file, &tag) {
                    report_error(e);
                }
                println!("\n✅ Tags guardados correctamente en '{}'", file.display());
            } else {
                println!("\n⚠️  No se especificaron cambios. Usa --help para ver las opciones.");
            }
        }
        Command::Remove { file, tags, all } => {
            // Verificar que el archivo MP3 existe
            if !file.exists() {
                eprintln!("Error: El archivo '{}' no existe", file.display());
                std::process::exit(1);
            }

            // Eliminar todos los tags (ID3v1 e ID3v2) físicamente
            if *all {
                let format = remove_all_id3(file)?;
                match format {
                    RemovedId3::None => {
                        println!("⚠️  El archivo '{}' no tenía tags ID3", file.display());
                    }
                    RemovedId3::Id3v1 => {
                        println!("✅ Tag ID3v1 eliminado de '{}'", file.display());
                    }
                    RemovedId3::Id3v2 => {
                        println!("✅ Tag ID3v2 eliminado de '{}'", file.display());
                    }
                    RemovedId3::Both => {
                        println!("✅ Tags ID3v1 e ID3v2 eliminados de '{}'", file.display());
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
                if let Err(e) = save_tag(file, &tag) {
                    report_error(e);
                }
                println!("\n✅ Tags eliminados correctamente de '{}'", file.display());
            } else {
                println!("\n⚠️  No se encontraron tags válidos para eliminar.");
            }
        }
    }

    Ok(())
}
