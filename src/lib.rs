//! Librería para manipular tags ID3 en archivos MP3
//!
//! Esta librería proporciona funciones para añadir, modificar, eliminar y mostrar
//! tags ID3v2.4 en archivos MP3, incluyendo metadatos básicos, carátulas, lyrics,
//! URLs y metadatos específicos de Apple.
//!
//! El backend de tags es el crate [`lofty`]. Las funciones trabajan sobre el tipo
//! de tag concreto [`Id3v2Tag`], porque varios frames del contrato de paridad no
//! tienen equivalente en el tag genérico `lofty::tag::Tag` y requieren la API de
//! bajo nivel:
//!
//! * `TYER` (año), que es el frame de ID3v2.3 que se conserva a propósito para
//!   `--year`; la tabla de `lofty` solo mapea `TDRC`.
//! * `USLT` (letra), cuyo código de idioma debe fijarse a `spa`.

use lofty::TextEncoding;
use lofty::config::WriteOptions;
use lofty::id3::v2::{
    Frame, FrameId, Id3v2Tag, TextInformationFrame, UnsynchronizedTextFrame, UrlLinkFrame,
};
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::tag::Accessor;
use lofty::tag::TagType;
use lofty::tag::items::Timestamp;
use std::fs::OpenOptions;
use std::path::Path;

/// Construye un [`FrameId`] a partir de un identificador conocido en tiempo de
/// compilación.
///
/// Todos los identificadores que se pasan a esta función son constantes literales
/// de 3 o 4 caracteres alfanuméricos en mayúscula, por lo que la conversión nunca
/// falla.
fn frame_id(id: &'static str) -> FrameId<'static> {
    FrameId::new(id).expect("los identificadores de frame ID3v2 son constantes válidas")
}

/// Inserta (o reemplaza) un frame de texto (`T...`) en el tag.
///
/// Se usa la API de bajo nivel porque varios de los frames del contrato (`TCOP`,
/// `TCOM`, `TIT3`, `TOPE`, `TYER`, `TCMP`, `TSOA`, `TSOP`, `TSOT`) se escriben de
/// forma uniforme y explícita.
fn insert_text_frame(tag: &mut Id3v2Tag, id: &'static str, value: &str) {
    let frame = Frame::Text(TextInformationFrame::new(
        frame_id(id),
        TextEncoding::UTF8,
        value.to_owned(),
    ));
    let _ = tag.insert(frame);
}

/// Escribe el año en el frame `TYER` (obsoleto en ID3v2.4, conservado a propósito).
fn set_year_frame(tag: &mut Id3v2Tag, year: i32) {
    insert_text_frame(tag, "TYER", &year.to_string());
}

/// Lee el año almacenado en el frame `TYER`.
fn year_frame(tag: &Id3v2Tag) -> Option<i32> {
    tag.get_text(&frame_id("TYER"))
        .and_then(|value| value.parse::<i32>().ok())
}

/// Aplica los metadatos especificados al tag ID3
///
/// # Argumentos
///
/// * `tag` - Tag ID3v2 a modificar
/// * `title` - Título de la canción
/// * `artists` - Lista de artistas (se unirán con "; ")
/// * `album` - Nombre del álbum
/// * `year` - Año de publicación (frame `TYER`)
/// * `genre` - Género musical
/// * `track` - Número de pista
/// * `season` - Temporada (TPOS - útil para podcasts)
/// * `date` - Fecha de grabación (formato YYYY-MM-DD o YYYY; frame `TDRC`)
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
    tag: &mut Id3v2Tag,
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
        tag.set_title(title.to_string());
        changed = true;
    }

    if !artists.is_empty() {
        tag.set_artist(artists.join("; "));
        changed = true;
    }

    if let Some(album) = album {
        tag.set_album(album.to_string());
        changed = true;
    }

    if let Some(year) = year {
        set_year_frame(tag, year);
        changed = true;
    }

    if let Some(genre) = genre {
        tag.set_genre(genre.to_string());
        changed = true;
    }

    if let Some(track) = track {
        tag.set_track(track);
        changed = true;
    }

    if let Some(season_num) = season {
        tag.set_disk(season_num);
        changed = true;
    }

    if let Some(date) = date {
        // Acepta YYYY, YYYY-MM, YYYY-MM-DD
        if let Ok(timestamp) = date.parse::<Timestamp>() {
            tag.set_date(timestamp);
            changed = true;
        }
    }

    if let Some(copyright) = copyright {
        insert_text_frame(tag, "TCOP", copyright);
        changed = true;
    }

    if let Some(composer_text) = composer {
        insert_text_frame(tag, "TCOM", composer_text);
        changed = true;
    }

    if let Some(subtitle_text) = subtitle {
        insert_text_frame(tag, "TIT3", subtitle_text);
        changed = true;
    }

    if let Some(original_artist_text) = original_artist {
        insert_text_frame(tag, "TOPE", original_artist_text);
        changed = true;
    }

    if let Some(album_artist_text) = album_artist {
        insert_text_frame(tag, "TPE2", album_artist_text);
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
pub fn add_lyrics(tag: &mut Id3v2Tag, text: &str) -> bool {
    let lyrics_frame = Frame::UnsynchronizedText(UnsynchronizedTextFrame::new(
        TextEncoding::UTF8,
        *b"spa",
        String::new(),
        text.to_owned(),
    ));

    let _ = tag.insert(lyrics_frame);
    true
}

/// Añade URL al tag (WOAR - Official artist/performer webpage)
///
/// # Retorna
///
/// `true` si se añadió el frame correctamente
pub fn add_url(tag: &mut Id3v2Tag, url: &str) -> bool {
    let url_frame = Frame::Url(UrlLinkFrame::new(frame_id("WOAR"), url.to_owned()));
    let _ = tag.insert(url_frame);
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
    tag: &mut Id3v2Tag,
    compilation: bool,
    album_sort: Option<&str>,
    artist_sort: Option<&str>,
    title_sort: Option<&str>,
) -> bool {
    let mut changed = false;

    // TCMP - Compilation flag (1 = part of compilation)
    if compilation {
        insert_text_frame(tag, "TCMP", "1");
        changed = true;
    }

    // TSOA - Album sort order
    if let Some(sort) = album_sort {
        insert_text_frame(tag, "TSOA", sort);
        changed = true;
    }

    // TSOP - Performer/Artist sort order
    if let Some(sort) = artist_sort {
        insert_text_frame(tag, "TSOP", sort);
        changed = true;
    }

    // TSOT - Title sort order
    if let Some(sort) = title_sort {
        insert_text_frame(tag, "TSOT", sort);
        changed = true;
    }

    changed
}

/// Crea un Picture frame desde datos de imagen
///
/// La imagen se configura como carátula frontal (CoverFront) con
/// la descripción "Cover".
pub fn create_picture_frame(data: Vec<u8>, mime_type: &str) -> Picture {
    Picture::unchecked(data)
        .pic_type(PictureType::CoverFront)
        .mime_type(MimeType::from_str(mime_type))
        .description("Cover")
        .build()
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
    tag: &mut Id3v2Tag,
    cover_path: &Path,
    cover_data: Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mime_type =
        detect_mime_type(cover_path).map_err(|e| format!("{} (soportados: jpg, png, webp)", e))?;
    let picture = create_picture_frame(cover_data, mime_type);
    let _ = tag.insert_picture(picture);
    Ok(())
}

/// Elimina tags específicos del archivo
///
/// # Tags soportados (con nombres alternativos en español)
///
/// * title/título - Título de la canción
/// * artist/artista - Artista
/// * album/álbum - Álbum
/// * year/año - Año (frame `TYER`)
/// * genre/género - Género
/// * track/pista - Número de pista
/// * season/temporada - Temporada
/// * date/fecha - Fecha de grabación (frame `TDRC`)
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
pub fn remove_tags(tag: &mut Id3v2Tag, tags_to_remove: &[String]) -> bool {
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
                let _ = tag.remove(&frame_id("TYER"));
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
                tag.remove_disk();
                true
            }
            "date" | "fecha" => {
                tag.remove_date();
                true
            }
            "copyright" => {
                let _ = tag.remove(&frame_id("TCOP"));
                true
            }
            "composer" | "compositor" => {
                let _ = tag.remove(&frame_id("TCOM"));
                true
            }
            "subtitle" | "subtítulo" | "subtitulo" | "description" | "descripción"
            | "descripcion" => {
                let _ = tag.remove(&frame_id("TIT3"));
                true
            }
            "original_artist" | "original-artist" | "artista_original" | "artista-original" => {
                let _ = tag.remove(&frame_id("TOPE"));
                true
            }
            "album_artist" | "album-artist" | "artista_album" | "artista-album" => {
                let _ = tag.remove(&frame_id("TPE2"));
                true
            }
            "cover" | "carátula" | "caratula" => {
                tag.retain(|frame| !matches!(frame, Frame::Picture(_)));
                true
            }
            "lyrics" | "letra" => {
                let _ = tag.remove(&frame_id("USLT"));
                true
            }
            "url" => {
                let _ = tag.remove(&frame_id("WOAR"));
                true
            }
            "compilation" | "compilación" | "compilacion" => {
                let _ = tag.remove(&frame_id("TCMP"));
                true
            }
            "album_sort" | "album-sort" | "orden_album" | "orden-album" => {
                let _ = tag.remove(&frame_id("TSOA"));
                true
            }
            "artist_sort" | "artist-sort" | "orden_artista" | "orden-artista" => {
                let _ = tag.remove(&frame_id("TSOP"));
                true
            }
            "title_sort" | "title-sort" | "orden_titulo" | "orden-titulo" => {
                let _ = tag.remove(&frame_id("TSOT"));
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

/// Estado previo de los tags ID3 de un archivo tras el borrado físico.
///
/// Sustituye a `id3::v1v2::FormatVersion`, conservando los mismos cuatro estados
/// para que el binario pueda seguir distinguiéndolos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemovedId3 {
    /// No había ningún tag ID3
    None,
    /// Solo había un tag ID3v1
    Id3v1,
    /// Solo había un tag ID3v2
    Id3v2,
    /// Había tags ID3v1 e ID3v2
    Both,
}

/// Elimina físicamente todos los tags ID3 (v1 y v2) del archivo MP3
///
/// Borra el bloque ID3v2 completo y el tag ID3v1 del archivo, dejándolo
/// como si nunca hubiera tenido tags (el audio permanece intacto).
///
/// La presencia previa se determina sobre los propios bytes del archivo (cabecera
/// `ID3` al inicio para v2 y bloque `TAG` en los últimos 128 bytes para v1), que es
/// la misma definición física que usaba `id3::v1v2::remove_from_path`. El borrado se
/// delega en `lofty` a través de `TagType::remove_from`, tratando cada versión por
/// separado sobre un descriptor abierto en lectura/escritura.
///
/// # Argumentos
///
/// * `file` - Ruta del archivo MP3
///
/// # Retorna
///
/// Un [`RemovedId3`] que indica el estado previo de los tags.
///
/// # Errores
///
/// Retorna un error si no se puede acceder o modificar el archivo.
pub fn remove_all_id3(file: &Path) -> Result<RemovedId3, Box<dyn std::error::Error>> {
    let data = std::fs::read(file)?;

    let had_v2 = data.starts_with(b"ID3");
    let had_v1 = data.len() >= 128 && data[data.len() - 128..].starts_with(b"TAG");

    if had_v2 {
        let mut handle = OpenOptions::new().read(true).write(true).open(file)?;
        TagType::Id3v2.remove_from(&mut handle, WriteOptions::default())?;
    }
    if had_v1 {
        let mut handle = OpenOptions::new().read(true).write(true).open(file)?;
        TagType::Id3v1.remove_from(&mut handle, WriteOptions::default())?;
    }

    Ok(match (had_v1, had_v2) {
        (false, false) => RemovedId3::None,
        (true, false) => RemovedId3::Id3v1,
        (false, true) => RemovedId3::Id3v2,
        (true, true) => RemovedId3::Both,
    })
}

/// Muestra todos los tags del archivo MP3 en formato legible
///
/// Imprime a stdout todos los metadatos encontrados en el tag,
/// incluyendo título, artista, álbum, año, género, pista, fecha,
/// copyright, compositor, subtítulo, artista original, artista del álbum,
/// carátulas, letras, URL y metadatos de Apple.
pub fn display_tags(tag: &Id3v2Tag) {
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

    if let Some(year) = year_frame(tag) {
        println!("📅 Año:       {}", year);
    }

    if let Some(date) = tag.date() {
        println!("📆 Fecha:     {}", date);
    }

    if let Some(genre) = tag.genre() {
        println!("🎸 Género:    {}", genre);
    }

    if let Some(track) = tag.track() {
        println!("#️⃣  Pista:     {}", track);
    }

    if let Some(season) = tag.disk() {
        println!("📺 Temporada: {}", season);
    }

    if let Some(copyright) = tag.get_text(&frame_id("TCOP")) {
        println!("©️  Copyright: {}", copyright);
    }

    if let Some(composer) = tag.get_text(&frame_id("TCOM")) {
        println!("🎼 Compositor: {}", composer);
    }

    if let Some(subtitle) = tag.get_text(&frame_id("TIT3")) {
        println!("📄 Subtítulo: {}", subtitle);
    }

    if let Some(original_artist) = tag.get_text(&frame_id("TOPE")) {
        println!("🎙️  Artista original: {}", original_artist);
    }

    if let Some(album_artist) = tag.get_text(&frame_id("TPE2")) {
        println!("👥 Artista del álbum: {}", album_artist);
    }

    // Mostrar URL si existe
    if let Some(Frame::Url(url_frame)) = tag.get(&frame_id("WOAR")) {
        println!("🌐 URL: {}", url_frame.url());
    }

    let pictures: Vec<&Picture> = tag
        .iter()
        .filter_map(|frame| match frame {
            Frame::Picture(apic) => Some(&*apic.picture),
            _ => None,
        })
        .collect();
    if !pictures.is_empty() {
        println!("🖼️  Carátulas: {} imagen(es)", pictures.len());
        for (i, pic) in pictures.iter().enumerate() {
            println!(
                "   [{}] Tipo: {:?}, MIME: {}, Tamaño: {} bytes",
                i + 1,
                pic.pic_type(),
                pic.mime_type().map(|m| m.as_str()).unwrap_or(""),
                pic.data().len()
            );
        }
    }

    // Mostrar lyrics si existen
    if let Some(Frame::UnsynchronizedText(lyrics)) = tag
        .iter()
        .find(|frame| matches!(frame, Frame::UnsynchronizedText(_)))
    {
        println!("📝 Letra ({}):", String::from_utf8_lossy(&lyrics.language));
        // Mostrar solo las primeras 3 líneas como preview
        let lines: Vec<&str> = lyrics.content.lines().collect();
        for line in lines.iter().take(3) {
            println!("   {}", line);
        }
        if lines.len() > 3 {
            println!("   ... ({} líneas más)", lines.len() - 3);
        }
    }

    // Mostrar metadatos de Apple si existen
    if tag.get_text(&frame_id("TCMP")) == Some("1") {
        println!(" Compilación: Sí");
    }

    if let Some(album_sort) = tag.get_text(&frame_id("TSOA")) {
        println!("🔤 Orden álbum: {}", album_sort);
    }

    if let Some(artist_sort) = tag.get_text(&frame_id("TSOP")) {
        println!("🔤 Orden artista: {}", artist_sort);
    }

    if let Some(title_sort) = tag.get_text(&frame_id("TSOT")) {
        println!("🔤 Orden título: {}", title_sort);
    }

    // Mostrar otros frames si existen
    let frame_count = tag.iter().count();
    if frame_count > 0 {
        println!("\n📦 Total de frames: {}", frame_count);
    }

    println!("═══════════════════════════════════════\n");
}

#[cfg(test)]
mod tests;
