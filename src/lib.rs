//! Librería para manipular tags ID3 en archivos MP3
//!
//! Esta librería proporciona funciones para añadir, modificar, eliminar y mostrar
//! tags ID3v2.4 en archivos MP3, incluyendo metadatos básicos, carátulas, lyrics,
//! URLs y metadatos específicos de Apple.

use id3::frame::{Content, Lyrics, Picture, PictureType};
use id3::{Frame, Tag, TagLike};
use std::path::Path;

/// Aplica los metadatos especificados al tag ID3
///
/// # Argumentos
///
/// * `tag` - Tag ID3 a modificar
/// * `title` - Título de la canción
/// * `artists` - Lista de artistas (se unirán con "; ")
/// * `album` - Nombre del álbum
/// * `year` - Año de publicación
/// * `genre` - Género musical
/// * `track` - Número de pista
/// * `season` - Temporada (TPOS - útil para podcasts)
/// * `date` - Fecha de grabación (formato YYYY-MM-DD o YYYY)
/// * `copyright` - Información de copyright
/// * `composer` - Compositor (TCOM)
/// * `subtitle` - Subtítulo o descripción (TIT3)
/// * `original_artist` - Artista original (TOPE)
/// * `album_artist` - Artista del álbum (TPE2)
///
/// # Retorna
///
/// `true` si se aplicó al menos un cambio, `false` en caso contrario
#[allow(clippy::too_many_arguments)]
pub fn apply_metadata(
    tag: &mut Tag,
    title: Option<&str>,
    artists: &[String],
    album: Option<&str>,
    year: Option<i32>,
    genre: Option<&str>,
    track: Option<u32>,
    season: Option<u32>,
    date: Option<&str>,
    copyright: Option<&str>,
    composer: Option<&str>,
    subtitle: Option<&str>,
    original_artist: Option<&str>,
    album_artist: Option<&str>,
) -> bool {
    let mut changed = false;

    if let Some(title) = title {
        tag.set_title(title);
        changed = true;
    }

    if !artists.is_empty() {
        let artist_string = artists.join("; ");
        tag.set_artist(&artist_string);
        changed = true;
    }

    if let Some(album) = album {
        tag.set_album(album);
        changed = true;
    }

    if let Some(year) = year {
        tag.set_year(year);
        changed = true;
    }

    if let Some(genre) = genre {
        tag.set_genre(genre);
        changed = true;
    }

    if let Some(track) = track {
        tag.set_track(track);
        changed = true;
    }

    if let Some(season_num) = season {
        tag.set_disc(season_num);
        changed = true;
    }

    if let Some(date) = date {
        // Intentar parsear la fecha - acepta YYYY, YYYY-MM, YYYY-MM-DD
        if let Ok(timestamp) = date.parse() {
            tag.set_date_recorded(timestamp);
            changed = true;
        }
    }

    if let Some(copyright) = copyright {
        tag.set_text("TCOP", copyright);
        changed = true;
    }

    if let Some(composer_text) = composer {
        tag.set_text("TCOM", composer_text);
        changed = true;
    }

    if let Some(subtitle_text) = subtitle {
        tag.set_text("TIT3", subtitle_text);
        changed = true;
    }

    if let Some(original_artist_text) = original_artist {
        tag.set_text("TOPE", original_artist_text);
        changed = true;
    }

    if let Some(album_artist_text) = album_artist {
        tag.set_album_artist(album_artist_text);
        changed = true;
    }

    changed
}

/// Añade letras (lyrics) al tag
///
/// Las letras se almacenan en un frame USLT (Unsynchronised lyrics)
/// con código de idioma "spa" (español).
///
/// # Retorna
///
/// `true` si se añadió el frame correctamente
pub fn add_lyrics(tag: &mut Tag, text: &str) -> bool {
    let lyrics_frame = Frame::with_content(
        "USLT",
        Content::Lyrics(Lyrics {
            lang: "spa".to_string(),
            description: String::new(),
            text: text.to_string(),
        }),
    );

    tag.add_frame(lyrics_frame);
    true
}

/// Añade URL al tag (WOAR - Official artist/performer webpage)
///
/// # Retorna
///
/// `true` si se añadió el frame correctamente
pub fn add_url(tag: &mut Tag, url: &str) -> bool {
    let url_frame = Frame::with_content("WOAR", Content::Link(url.to_string()));
    tag.add_frame(url_frame);
    true
}

/// Añade metadatos de Apple al tag
///
/// # Argumentos
///
/// * `compilation` - Si es true, marca el archivo como parte de una compilación (TCMP)
/// * `album_sort` - Orden de clasificación del álbum (TSOA)
/// * `artist_sort` - Orden de clasificación del artista (TSOP)
/// * `title_sort` - Orden de clasificación del título (TSOT)
///
/// # Retorna
///
/// `true` si se aplicó al menos un cambio
pub fn add_apple_metadata(
    tag: &mut Tag,
    compilation: bool,
    album_sort: Option<&str>,
    artist_sort: Option<&str>,
    title_sort: Option<&str>,
) -> bool {
    let mut changed = false;

    // TCMP - Compilation flag (1 = part of compilation)
    if compilation {
        tag.set_text("TCMP", "1");
        changed = true;
    }

    // TSOA - Album sort order
    if let Some(sort) = album_sort {
        tag.set_text("TSOA", sort);
        changed = true;
    }

    // TSOP - Performer/Artist sort order
    if let Some(sort) = artist_sort {
        tag.set_text("TSOP", sort);
        changed = true;
    }

    // TSOT - Title sort order
    if let Some(sort) = title_sort {
        tag.set_text("TSOT", sort);
        changed = true;
    }

    changed
}

/// Crea un Picture frame desde datos de imagen
///
/// La imagen se configura como carátula frontal (CoverFront) con
/// la descripción "Cover".
pub fn create_picture_frame(data: Vec<u8>, mime_type: &str) -> Picture {
    Picture {
        mime_type: mime_type.to_string(),
        picture_type: PictureType::CoverFront,
        description: "Cover".to_string(),
        data,
    }
}

/// Detecta el tipo MIME desde la extensión del archivo
///
/// # Formatos soportados
///
/// * JPG/JPEG → image/jpeg
/// * PNG → image/png
/// * WEBP → image/webp
///
/// # Errores
///
/// Retorna un error si la extensión no es soportada o no se puede determinar
pub fn detect_mime_type(path: &Path) -> Result<&'static str, String> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .ok_or_else(|| "No se pudo determinar la extensión del archivo".to_string())?;

    match extension.as_str() {
        "jpg" | "jpeg" => Ok("image/jpeg"),
        "png" => Ok("image/png"),
        "webp" => Ok("image/webp"),
        _ => Err(format!("Formato de imagen no soportado: .{}", extension)),
    }
}

/// Añade una carátula al tag desde un archivo
///
/// El tipo MIME se detecta automáticamente desde la extensión del archivo.
///
/// # Errores
///
/// Retorna un error si:
/// * El formato de imagen no es soportado
/// * No se puede determinar la extensión
pub fn add_cover_art(
    tag: &mut Tag,
    cover_path: &Path,
    cover_data: Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mime_type =
        detect_mime_type(cover_path).map_err(|e| format!("{} (soportados: jpg, png, webp)", e))?;
    let picture = create_picture_frame(cover_data, mime_type);
    tag.add_frame(picture);
    Ok(())
}

/// Elimina tags específicos del archivo
///
/// # Tags soportados (con nombres alternativos en español)
///
/// * title/título - Título de la canción
/// * artist/artista - Artista
/// * album/álbum - Álbum
/// * year/año - Año
/// * genre/género - Género
/// * track/pista - Número de pista
/// * season/temporada - Temporada
/// * date/fecha - Fecha de grabación
/// * copyright - Copyright
/// * composer/compositor - Compositor
/// * subtitle/subtítulo/description/descripción - Subtítulo
/// * original_artist/artista_original - Artista original
/// * album_artist/artista_album - Artista del álbum
/// * cover/carátula - Carátula
/// * lyrics/letra - Letra
/// * url - URL
/// * compilation/compilación - Flag de compilación
/// * album_sort/orden_album - Orden de álbum
/// * artist_sort/orden_artista - Orden de artista
/// * title_sort/orden_titulo - Orden de título
///
/// # Retorna
///
/// `true` si se eliminó al menos un tag
pub fn remove_tags(tag: &mut Tag, tags_to_remove: &[String]) -> bool {
    let mut changed = false;

    for tag_name in tags_to_remove {
        let removed = match tag_name.to_lowercase().as_str() {
            "title" | "título" => {
                tag.remove_title();
                true
            }
            "artist" | "artista" => {
                tag.remove_artist();
                true
            }
            "album" | "álbum" => {
                tag.remove_album();
                true
            }
            "year" | "año" => {
                tag.remove_year();
                true
            }
            "genre" | "género" | "genero" => {
                tag.remove_genre();
                true
            }
            "track" | "pista" => {
                tag.remove_track();
                true
            }
            "season" | "temporada" => {
                tag.remove_disc();
                true
            }
            "date" | "fecha" => {
                tag.remove_date_recorded();
                true
            }
            "copyright" => {
                tag.remove("TCOP");
                true
            }
            "composer" | "compositor" => {
                tag.remove("TCOM");
                true
            }
            "subtitle" | "subtítulo" | "subtitulo" | "description" | "descripción"
            | "descripcion" => {
                tag.remove("TIT3");
                true
            }
            "original_artist" | "original-artist" | "artista_original" | "artista-original" => {
                tag.remove("TOPE");
                true
            }
            "album_artist" | "album-artist" | "artista_album" | "artista-album" => {
                tag.remove_album_artist();
                true
            }
            "cover" | "carátula" | "caratula" => {
                tag.remove_all_pictures();
                true
            }
            "lyrics" | "letra" => {
                tag.remove("USLT");
                true
            }
            "url" => {
                tag.remove("WOAR");
                true
            }
            "compilation" | "compilación" | "compilacion" => {
                tag.remove("TCMP");
                true
            }
            "album_sort" | "album-sort" | "orden_album" | "orden-album" => {
                tag.remove("TSOA");
                true
            }
            "artist_sort" | "artist-sort" | "orden_artista" | "orden-artista" => {
                tag.remove("TSOP");
                true
            }
            "title_sort" | "title-sort" | "orden_titulo" | "orden-titulo" => {
                tag.remove("TSOT");
                true
            }
            _ => {
                eprintln!(
                    "⚠️  Tag desconocido: '{}'. Tags válidos: title, artist, album, year, genre, track, season, date, copyright, composer, subtitle, original_artist, album_artist, cover, lyrics, url, compilation, album_sort, artist_sort, title_sort",
                    tag_name
                );
                false
            }
        };

        if removed {
            println!("✓ Eliminado: {}", tag_name);
            changed = true;
        }
    }

    changed
}

/// Elimina todos los tags del archivo MP3
///
/// Borra todos los frames ID3 conocidos: metadatos básicos, carátulas,
/// letras, URLs y metadatos de Apple.
///
/// # Retorna
///
/// `true` si se eliminó al menos un tag
pub fn remove_all_tags(tag: &mut Tag) -> bool {
    let has_frames = tag.frames().count() > 0;

    tag.remove_title();
    tag.remove_artist();
    tag.remove_album();
    tag.remove_year();
    tag.remove_genre();
    tag.remove_track();
    tag.remove_disc();
    tag.remove_date_recorded();
    tag.remove_album_artist();
    tag.remove_all_pictures();
    tag.remove_all_lyrics();

    for frame_id in &[
        "TCOP", "TCOM", "TIT3", "TOPE", "WOAR", "TCMP", "TSOA", "TSOP", "TSOT",
    ] {
        tag.remove(frame_id);
    }

    has_frames
}

/// Muestra todos los tags del archivo MP3 en formato legible
///
/// Imprime a stdout todos los metadatos encontrados en el tag,
/// incluyendo título, artista, álbum, año, género, pista, fecha,
/// copyright, compositor, subtítulo, artista original, artista del álbum,
/// carátulas, letras, URL y metadatos de Apple.
pub fn display_tags(tag: &Tag) {
    println!("\n📋 Tags ID3 encontrados:\n");
    println!("═══════════════════════════════════════");

    if let Some(title) = tag.title() {
        println!("🎵 Título:    {}", title);
    }

    if let Some(artist) = tag.artist() {
        println!("🎤 Artista:   {}", artist);
    }

    if let Some(album) = tag.album() {
        println!("💿 Álbum:     {}", album);
    }

    if let Some(year) = tag.year() {
        println!("📅 Año:       {}", year);
    }

    if let Some(date) = tag.date_recorded() {
        println!("📆 Fecha:     {}", date);
    }

    if let Some(genre) = tag.genre() {
        println!("🎸 Género:    {}", genre);
    }

    if let Some(track) = tag.track() {
        println!("#️⃣  Pista:     {}", track);
    }

    if let Some(season) = tag.disc() {
        println!("📺 Temporada: {}", season);
    }

    if let Some(copyright) = tag.get("TCOP").and_then(|f| f.content().text()) {
        println!("©️  Copyright: {}", copyright);
    }

    if let Some(composer) = tag.get("TCOM").and_then(|f| f.content().text()) {
        println!("🎼 Compositor: {}", composer);
    }

    if let Some(subtitle) = tag.get("TIT3").and_then(|f| f.content().text()) {
        println!("📄 Subtítulo: {}", subtitle);
    }

    if let Some(original_artist) = tag.get("TOPE").and_then(|f| f.content().text()) {
        println!("🎙️  Artista original: {}", original_artist);
    }

    if let Some(album_artist) = tag.album_artist() {
        println!("👥 Artista del álbum: {}", album_artist);
    }

    // Mostrar URL si existe
    for frame in tag.frames() {
        if frame.id() == "WOAR"
            && let Content::Link(url) = frame.content()
        {
            println!("🌐 URL: {}", url);
            break;
        }
    }

    let pictures: Vec<_> = tag.pictures().collect();
    if !pictures.is_empty() {
        println!("🖼️  Carátulas: {} imagen(es)", pictures.len());
        for (i, pic) in pictures.iter().enumerate() {
            println!(
                "   [{}] Tipo: {:?}, MIME: {}, Tamaño: {} bytes",
                i + 1,
                pic.picture_type,
                pic.mime_type,
                pic.data.len()
            );
        }
    }

    // Mostrar lyrics si existen
    for frame in tag.frames() {
        if let Content::Lyrics(lyrics) = frame.content() {
            println!("📝 Letra ({}):", lyrics.lang);
            // Mostrar solo las primeras 3 líneas como preview
            let lines: Vec<&str> = lyrics.text.lines().collect();
            for line in lines.iter().take(3) {
                println!("   {}", line);
            }
            if lines.len() > 3 {
                println!("   ... ({} líneas más)", lines.len() - 3);
            }
            break; // Solo mostrar el primer frame de lyrics
        }
    }

    // Mostrar metadatos de Apple si existen
    if let Some(compilation) = tag.get("TCMP").and_then(|f| f.content().text())
        && compilation == "1"
    {
        println!(" Compilación: Sí");
    }

    if let Some(album_sort) = tag.get("TSOA").and_then(|f| f.content().text()) {
        println!("🔤 Orden álbum: {}", album_sort);
    }

    if let Some(artist_sort) = tag.get("TSOP").and_then(|f| f.content().text()) {
        println!("🔤 Orden artista: {}", artist_sort);
    }

    if let Some(title_sort) = tag.get("TSOT").and_then(|f| f.content().text()) {
        println!("🔤 Orden título: {}", title_sort);
    }

    // Mostrar otros frames si existen
    let frame_count = tag.frames().count();
    if frame_count > 0 {
        println!("\n📦 Total de frames: {}", frame_count);
    }

    println!("═══════════════════════════════════════\n");
}

#[cfg(test)]
mod tests;
