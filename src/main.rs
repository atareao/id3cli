use clap::{Parser, Subcommand};
use id3::Tag;
use id3cli::*;
use std::fs;
use std::path::PathBuf;

/// CLI para añadir tags ID3 y carátulas a archivos MP3
#[derive(Parser, Debug)]
#[command(name = "id3cli")]
#[command(about = "Añade tags ID3 y carátulas a archivos MP3", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
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

        /// Eliminar tags específicos (title, artist, album, year, genre, track, season, date, copyright, cover, lyrics, url, compilation, album_sort, artist_sort, title_sort)
        #[arg(short, long)]
        remove: Vec<String>,
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
            let tag = match Tag::read_from_path(file) {
                Ok(tag) => tag,
                Err(_) => {
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
            remove,
        } => {
            // Verificar que el archivo MP3 existe
            if !file.exists() {
                eprintln!("Error: El archivo '{}' no existe", file.display());
                std::process::exit(1);
            }

            // Leer o crear tag
            let mut tag = match Tag::read_from_path(file) {
                Ok(tag) => {
                    println!("Tags existentes encontrados en '{}'", file.display());
                    tag
                }
                Err(_) => {
                    println!("Creando nuevos tags para '{}'", file.display());
                    Tag::new()
                }
            };

            // Procesar eliminación de tags si se especificó
            let mut removed = false;
            if !remove.is_empty() {
                removed = remove_tags(&mut tag, remove);
            }

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
            if changed || cover_added || removed || lyrics_added || url_added || apple_added {
                tag.write_to_path(file, id3::Version::Id3v24)?;
                println!("\n✅ Tags guardados correctamente en '{}'", file.display());
            } else {
                println!("\n⚠️  No se especificaron cambios. Usa --help para ver las opciones.");
            }
        }
    }

    Ok(())
}
