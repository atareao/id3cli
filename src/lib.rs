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
//!
//! La librería también expone la detección del formato de audio por *magic
//! bytes* ([`detect_format`]), que decide sobre el contenido del archivo y
//! nunca sobre la extensión del nombre.
//!
//! Para los contenedores MP4 (`.m4a`, `.mp4`, `.m4b`) la librería trabaja sobre
//! el tag genérico [`lofty::tag::Tag`] de tipo [`TagType::Mp4Ilst`], que cubre
//! todo el mapeo de átomos `ilst` que necesita el CLI. El mapeo flag→átomo vive
//! en una única tabla ([`MP4_ATOM_MAP`]) y la escritura se orquesta con
//! [`edit_mp4`], que crea el árbol `moov.udta.meta.ilst` cuando el archivo no lo
//! tiene. El subtítulo se escribe en el átomo *freeform*
//! `----:com.apple.iTunes:SUBTITLE`, que `lofty` mapea a
//! [`ItemKey::TrackSubtitle`].

#[macro_use]
pub mod output;

use lofty::TextEncoding;
use lofty::config::{ParseOptions, WriteOptions};
use lofty::file::TaggedFileExt;
use lofty::id3::v2::{
    Frame, FrameId, Id3v2Tag, TextInformationFrame, UnsynchronizedTextFrame, UrlLinkFrame,
};
use lofty::ogg::tag::VorbisComments;
use lofty::ogg::{OggPictureStorage, OpusFile, VorbisFile};
use lofty::picture::{MimeType, Picture, PictureInformation, PictureType};
use lofty::prelude::AudioFile;
use lofty::tag::items::Timestamp;
use lofty::tag::{Accessor, ItemKey, Tag, TagExt, TagType};
use std::fs::{File, OpenOptions};
use std::io::{self, Read};
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

/// Número de pista o de temporada, con su total opcional.
///
/// Lo usan tanto `--track` como `--season`, que aceptan un número suelto o un
/// par de número y total separados por una barra. El parseo ([`FromStr`]) valida
/// la forma dentro de `clap`, de modo que una forma inválida falla **antes de
/// abrir el archivo**. La sintaxis admitida es **uno o dos grupos de dígitos
/// decimales ASCII separados por una barra**:
///
/// ```text
/// 7       -> número 7, sin total
/// 7/12    -> número 7, total 12
/// ```
///
/// Cualquier otra forma (`7/`, `/12`, `7/12/3`, `7/x`, `+7`, la cadena vacía, un
/// número negativo o con espacios, `7\n` o cualquier dígito no ASCII) se rechaza
/// con un mensaje en español. Los ceros a la izquierda se aceptan y se
/// normalizan (`007` → `7`), y el cero se admite (`0` y `0/0`).
///
/// # Asimetría del cero entre contenedores
///
/// El tipo **no prohíbe** el cero, pero los contenedores no lo representan
/// igual: MP3 (frame de texto) y OGG (comentario Vorbis) escriben `0` tal cual y
/// lo muestran al releer, mientras que el átomo numérico de MP4 (`trkn`/`disk`)
/// lo descarta y el backend avisa de la degradación (ver [`Mp4EditOptions`]).
///
/// [`FromStr`]: std::str::FromStr
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumberWithTotal {
    /// Número (campo izquierdo del par).
    number: u32,
    /// Total (campo derecho del par), si se indicó.
    total: Option<u32>,
}

impl NumberWithTotal {
    /// Crea un [`NumberWithTotal`] a partir de su número y su total opcional.
    pub fn new(number: u32, total: Option<u32>) -> Self {
        Self { number, total }
    }

    /// Devuelve el número (campo izquierdo del par).
    pub fn number(&self) -> u32 {
        self.number
    }

    /// Devuelve el total (campo derecho del par), si se indicó.
    pub fn total(&self) -> Option<u32> {
        self.total
    }
}

impl std::str::FromStr for NumberWithTotal {
    type Err = String;

    /// Acepta `7`, `7/12` o un número con ceros a la izquierda (`007`);
    /// cualquier otra forma devuelve un error en español que nombra la sintaxis
    /// admitida.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let invalid = || {
            format!(
                "forma inválida «{value}»: se admite un número (7) o un número con total (7/12)"
            )
        };

        // Una sola barra como máximo: `7`, `7/12`. Con dos barras (`7/12/3`) o
        // con una barra de más (`7//12`) el tercer trozo existe y se rechaza.
        let mut parts = value.split('/');
        let number = parts.next().unwrap_or_default();
        let total = parts.next();
        if parts.next().is_some() {
            return Err(invalid());
        }

        // `parse::<u32>` **no** basta para exigir «enteros sin signo»: acepta el
        // signo `+` (`"+7".parse::<u32>()` es `Ok(7)`), los ceros a la izquierda
        // (que aquí sí se admiten y normalizan: `007` → `7`) y, por supuesto,
        // cualquier dígito ASCII. Para que `+7`, `-7`, ` 7`, `7\n` y los dígitos
        // no ASCII caigan, cada parte debe ser **no vacía y de solo dígitos
        // ASCII** antes de parsearse.
        let parse_part = |part: &str| -> Result<u32, String> {
            if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(invalid());
            }
            part.parse::<u32>().map_err(|_| invalid())
        };

        let number = parse_part(number)?;
        let total = match total {
            Some(total) => Some(parse_part(total)?),
            None => None,
        };

        Ok(Self { number, total })
    }
}

impl std::fmt::Display for NumberWithTotal {
    /// Escribe `7` cuando no hay total y `7/12` cuando lo hay.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.total {
            Some(total) => write!(f, "{}/{}", self.number, total),
            None => write!(f, "{}", self.number),
        }
    }
}

/// Compone el texto de un número con su total para los tres visualizadores.
///
/// Devuelve `7` cuando no hay total y `7/12` cuando lo hay, de modo que MP3,
/// MP4 y OGG muestren el par igual sin repetir la lógica. El número se acepta
/// como texto para que la visualización de OGG, que lee una clave de comentario,
/// no tenga que convertirla.
fn format_number_with_total(
    number: impl std::fmt::Display,
    total: Option<impl std::fmt::Display>,
) -> String {
    match total {
        Some(total) => format!("{number}/{total}"),
        None => number.to_string(),
    }
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
/// * `track` - Número de pista, con su total opcional (`TRCK`)
/// * `season` - Temporada, con su total opcional (`TPOS` - útil para podcasts)
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
    track: Option<NumberWithTotal>,
    season: Option<NumberWithTotal>,
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
        // `set_track` conserva el total del frame `TRCK`; el total solo se
        // sobrescribe si `--track` lo indica explícitamente.
        tag.set_track(track.number());
        if let Some(total) = track.total() {
            tag.set_track_total(total);
        }
        changed = true;
    }

    if let Some(season) = season {
        // Igual que la pista: `set_disk` conserva el total de `TPOS`.
        tag.set_disk(season.number());
        if let Some(total) = season.total() {
            tag.set_disk_total(total);
        }
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

/// Valida la extensión de una carátula y devuelve su tipo MIME.
///
/// Es la variante «de CLI» de [`detect_mime_type`]: añade al mensaje de error la
/// lista de formatos soportados, para que el texto sea idéntico en la carátula
/// de MP3 y en la de MP4 (que comparten esta validación y este mensaje).
///
/// # Errores
///
/// Retorna un error si la extensión no es soportada o no se puede determinar.
pub fn detect_cover_mime_type(path: &Path) -> Result<&'static str, String> {
    detect_mime_type(path).map_err(|e| format!("{} (soportados: jpg, png, webp)", e))
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
    let mime_type = detect_cover_mime_type(cover_path)?;
    let picture = create_picture_frame(cover_data, mime_type);
    let _ = tag.insert_picture(picture);
    Ok(())
}

/// Identificador canónico de un tag del CLI, independiente del nombre (inglés o
/// español) con el que se indique y del formato del archivo.
///
/// Es la salida de [`classify_tag_alias`], la **única** tabla de alias del
/// proyecto: tanto el borrado de MP3 ([`remove_tags`]) como el de MP4
/// ([`remove_tags_mp4`]) parten de este mismo identificador, de modo que la
/// lista de nombres aceptados no está duplicada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagKind {
    /// Título de la canción.
    Title,
    /// Artista.
    Artist,
    /// Álbum.
    Album,
    /// Año.
    Year,
    /// Género.
    Genre,
    /// Número de pista.
    Track,
    /// Temporada.
    Season,
    /// Fecha de grabación.
    Date,
    /// Copyright.
    Copyright,
    /// Compositor.
    Composer,
    /// Subtítulo / descripción.
    Subtitle,
    /// Artista original (sin equivalente en MP4).
    OriginalArtist,
    /// Artista del álbum.
    AlbumArtist,
    /// Carátula.
    Cover,
    /// Letra.
    Lyrics,
    /// URL asociada (sin equivalente en MP4).
    Url,
    /// Flag de compilación (Apple).
    Compilation,
    /// Orden de clasificación del álbum (Apple).
    AlbumSort,
    /// Orden de clasificación del artista (Apple).
    ArtistSort,
    /// Orden de clasificación del título (Apple).
    TitleSort,
}

/// Lista de nombres de tag válidos, tal y como se muestra al usuario cuando
/// indica uno desconocido. Es la representación legible de la tabla de alias.
pub const VALID_TAG_NAMES: &str = "title, artist, album, year, genre, track, season, date, \
                                    copyright, composer, subtitle, original_artist, album_artist, \
                                    cover, lyrics, url, compilation, album_sort, artist_sort, \
                                    title_sort";

/// Clasifica un nombre de tag (en inglés o en español) en su [`TagKind`].
///
/// Es la **única** tabla de alias del proyecto: la comparten el borrado de MP3
/// y el de MP4. La comparación no distingue mayúsculas. Devuelve `None` para un
/// nombre desconocido.
pub fn classify_tag_alias(name: &str) -> Option<TagKind> {
    Some(match name.to_lowercase().as_str() {
        "title" | "título" => TagKind::Title,
        "artist" | "artista" => TagKind::Artist,
        "album" | "álbum" => TagKind::Album,
        "year" | "año" => TagKind::Year,
        "genre" | "género" | "genero" => TagKind::Genre,
        "track" | "pista" => TagKind::Track,
        "season" | "temporada" => TagKind::Season,
        "date" | "fecha" => TagKind::Date,
        "copyright" => TagKind::Copyright,
        "composer" | "compositor" => TagKind::Composer,
        "subtitle" | "subtítulo" | "subtitulo" | "description" | "descripción" | "descripcion" => {
            TagKind::Subtitle
        }
        "original_artist" | "original-artist" | "artista_original" | "artista-original" => {
            TagKind::OriginalArtist
        }
        "album_artist" | "album-artist" | "artista_album" | "artista-album" => TagKind::AlbumArtist,
        "cover" | "carátula" | "caratula" => TagKind::Cover,
        "lyrics" | "letra" => TagKind::Lyrics,
        "url" => TagKind::Url,
        "compilation" | "compilación" | "compilacion" => TagKind::Compilation,
        "album_sort" | "album-sort" | "orden_album" | "orden-album" => TagKind::AlbumSort,
        "artist_sort" | "artist-sort" | "orden_artista" | "orden-artista" => TagKind::ArtistSort,
        "title_sort" | "title-sort" | "orden_titulo" | "orden-titulo" => TagKind::TitleSort,
        _ => return None,
    })
}

/// Elimina tags específicos del tag ID3
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
        let removed = match classify_tag_alias(tag_name) {
            Some(TagKind::Title) => {
                tag.remove_title();
                true
            }
            Some(TagKind::Artist) => {
                tag.remove_artist();
                true
            }
            Some(TagKind::Album) => {
                tag.remove_album();
                true
            }
            Some(TagKind::Year) => {
                let _ = tag.remove(&frame_id("TYER"));
                true
            }
            Some(TagKind::Genre) => {
                tag.remove_genre();
                true
            }
            Some(TagKind::Track) => {
                tag.remove_track();
                true
            }
            Some(TagKind::Season) => {
                tag.remove_disk();
                true
            }
            Some(TagKind::Date) => {
                tag.remove_date();
                true
            }
            Some(TagKind::Copyright) => {
                let _ = tag.remove(&frame_id("TCOP"));
                true
            }
            Some(TagKind::Composer) => {
                let _ = tag.remove(&frame_id("TCOM"));
                true
            }
            Some(TagKind::Subtitle) => {
                let _ = tag.remove(&frame_id("TIT3"));
                true
            }
            Some(TagKind::OriginalArtist) => {
                let _ = tag.remove(&frame_id("TOPE"));
                true
            }
            Some(TagKind::AlbumArtist) => {
                let _ = tag.remove(&frame_id("TPE2"));
                true
            }
            Some(TagKind::Cover) => {
                tag.retain(|frame| !matches!(frame, Frame::Picture(_)));
                true
            }
            Some(TagKind::Lyrics) => {
                let _ = tag.remove(&frame_id("USLT"));
                true
            }
            Some(TagKind::Url) => {
                let _ = tag.remove(&frame_id("WOAR"));
                true
            }
            Some(TagKind::Compilation) => {
                let _ = tag.remove(&frame_id("TCMP"));
                true
            }
            Some(TagKind::AlbumSort) => {
                let _ = tag.remove(&frame_id("TSOA"));
                true
            }
            Some(TagKind::ArtistSort) => {
                let _ = tag.remove(&frame_id("TSOP"));
                true
            }
            Some(TagKind::TitleSort) => {
                let _ = tag.remove(&frame_id("TSOT"));
                true
            }
            None => {
                eprintln!(
                    "⚠️  Tag desconocido: '{}'. Tags válidos: {}",
                    tag_name, VALID_TAG_NAMES
                );
                false
            }
        };

        if removed {
            outln!("✓ Eliminado: {}", tag_name);
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
    outln!("\n📋 Tags ID3 encontrados:\n");
    outln!("═══════════════════════════════════════");

    if let Some(title) = tag.title() {
        outln!("🎵 Título:    {}", title);
    }

    if let Some(artist) = tag.artist() {
        outln!("🎤 Artista:   {}", artist);
    }

    if let Some(album) = tag.album() {
        outln!("💿 Álbum:     {}", album);
    }

    if let Some(year) = year_frame(tag) {
        outln!("📅 Año:       {}", year);
    }

    if let Some(date) = tag.date() {
        outln!("📆 Fecha:     {}", date);
    }

    if let Some(genre) = tag.genre() {
        outln!("🎸 Género:    {}", genre);
    }

    if let Some(track) = tag.track() {
        outln!(
            "#️⃣  Pista:     {}",
            format_number_with_total(track, tag.track_total())
        );
    }

    if let Some(season) = tag.disk() {
        outln!(
            "📺 Temporada: {}",
            format_number_with_total(season, tag.disk_total())
        );
    }

    if let Some(copyright) = tag.get_text(&frame_id("TCOP")) {
        outln!("©️  Copyright: {}", copyright);
    }

    if let Some(composer) = tag.get_text(&frame_id("TCOM")) {
        outln!("🎼 Compositor: {}", composer);
    }

    if let Some(subtitle) = tag.get_text(&frame_id("TIT3")) {
        outln!("📄 Subtítulo: {}", subtitle);
    }

    if let Some(original_artist) = tag.get_text(&frame_id("TOPE")) {
        outln!("🎙️  Artista original: {}", original_artist);
    }

    if let Some(album_artist) = tag.get_text(&frame_id("TPE2")) {
        outln!("👥 Artista del álbum: {}", album_artist);
    }

    // Mostrar URL si existe
    if let Some(Frame::Url(url_frame)) = tag.get(&frame_id("WOAR")) {
        outln!("🌐 URL: {}", url_frame.url());
    }

    let pictures: Vec<&Picture> = tag
        .iter()
        .filter_map(|frame| match frame {
            Frame::Picture(apic) => Some(&*apic.picture),
            _ => None,
        })
        .collect();
    if !pictures.is_empty() {
        outln!("🖼️  Carátulas: {} imagen(es)", pictures.len());
        for (i, pic) in pictures.iter().enumerate() {
            outln!(
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
        outln!("📝 Letra ({}):", String::from_utf8_lossy(&lyrics.language));
        // Mostrar solo las primeras 3 líneas como preview
        let lines: Vec<&str> = lyrics.content.lines().collect();
        for line in lines.iter().take(3) {
            outln!("   {}", line);
        }
        if lines.len() > 3 {
            outln!("   ... ({} líneas más)", lines.len() - 3);
        }
    }

    // Mostrar metadatos de Apple si existen
    if tag.get_text(&frame_id("TCMP")) == Some("1") {
        outln!(" Compilación: Sí");
    }

    if let Some(album_sort) = tag.get_text(&frame_id("TSOA")) {
        outln!("🔤 Orden álbum: {}", album_sort);
    }

    if let Some(artist_sort) = tag.get_text(&frame_id("TSOP")) {
        outln!("🔤 Orden artista: {}", artist_sort);
    }

    if let Some(title_sort) = tag.get_text(&frame_id("TSOT")) {
        outln!("🔤 Orden título: {}", title_sort);
    }

    // Mostrar otros frames si existen
    let frame_count = tag.iter().count();
    if frame_count > 0 {
        outln!("\n📦 Total de frames: {}", frame_count);
    }

    outln!("═══════════════════════════════════════\n");
}

// ---------------------------------------------------------------------------
// Detección del formato de audio (magic bytes)
// ---------------------------------------------------------------------------

/// Número máximo de bytes iniciales que se inspeccionan para decidir el formato.
///
/// Bastan para leer la cabecera ID3v2 (`ID3`), el átomo `ftyp` del offset 4 y una
/// cabecera de frame MPEG completa (4 bytes). Ogg exige llegar más lejos: la
/// cabecera de página ocupa 27 bytes, su tabla de segmentos hasta 255 más, y el
/// primer paquete puede empezar en el offset 282, así que hacen falta hasta 289
/// bytes para poder leer su identificador de códec (`\x01vorbis`). Se toma un
/// valor redondeado por encima de ese máximo.
const FORMAT_PROBE_LEN: usize = 512;

/// Magic bytes de formatos de audio reconocibles pero **fuera de alcance**.
///
/// Ogg **no** figura aquí: el contenedor Ogg se decide por el códec de su primer
/// paquete en [`detect_format_from_bytes`], no por la cabecera de página, porque
/// Vorbis y Opus están soportados mientras que Speex, FLAC y Theora no.
const UNSUPPORTED_MAGIC: &[&[u8]] = &[
    b"fLaC", // FLAC nativo (fuera de Ogg)
    b"RIFF", // Contenedores RIFF (WAV, AVI, …)
    b"FORM", // AIFF / AIFC
    b"MAC ", // Monkey's Audio (APE)
    b"wvpk", // WavPack
];

/// Formato de audio reconocido a partir del contenido del archivo.
///
/// La decisión se toma siempre sobre los bytes del archivo, nunca sobre la
/// extensión del nombre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    /// MP3: audio MPEG, con o sin cabecera ID3v2. Soportado.
    Mp3,
    /// Contenedor ISO-BMFF/MP4 (`.m4a`, `.mp4`, `.m4b`), reconocido por `ftyp`.
    Mp4,
    /// Contenedor Ogg con el códec Vorbis, reconocido por la cabecera de página
    /// (`OggS`, versión `0`) y por el identificador `\x01vorbis` del primer
    /// paquete. Soportado.
    OggVorbis,
    /// Contenedor Ogg con el códec Opus, reconocido por la cabecera de página
    /// (`OggS`, versión `0`) y por el identificador `OpusHead` del primer
    /// paquete. Soportado.
    OggOpus,
    /// Formato de audio reconocible pero fuera de alcance (FLAC, WAV, AIFF, APE…,
    /// o un contenedor Ogg con un códec distinto de Vorbis u Opus: Speex, FLAC,
    /// Theora…).
    Unsupported,
    /// Contenido no reconocible o archivo demasiado corto para decidirlo.
    Unknown,
}

/// Detecta el formato de audio de un archivo a partir de su contenido.
///
/// Reconoce MP3, MP4 y los contenedores Ogg con códec Vorbis u Opus (véase
/// [`detect_format_from_bytes`]). Recibe la ruta únicamente para **leer** sus
/// primeros bytes: la extensión del nombre no interviene en la decisión. El
/// archivo se abre solo en modo lectura, de modo que esta función nunca lo
/// modifica.
///
/// # Errores
///
/// Retorna un error de E/S si el archivo no se puede abrir o leer.
pub fn detect_format(path: &Path) -> io::Result<AudioFormat> {
    let mut file = File::open(path)?;
    let mut header = [0u8; FORMAT_PROBE_LEN];
    let mut read = 0;
    while read < header.len() {
        match file.read(&mut header[read..])? {
            0 => break,
            n => read += n,
        }
    }
    Ok(detect_format_from_bytes(&header[..read]))
}

/// Detecta el formato de audio a partir de un bloque de bytes iniciales.
///
/// * MP3: cabecera ID3v2 (`ID3`) o cabecera de frame MPEG válida.
/// * MP4: átomo `ftyp` en el offset 4, con independencia del brand declarado.
/// * Ogg (Vorbis u Opus): cabecera de página `OggS` completa, de versión `0`,
///   cuyo primer paquete empieza por el identificador de códec `\x01vorbis`
///   (Vorbis) u `OpusHead` (Opus). La decisión es por **códec**, no por
///   contenedor: un Ogg es un sobre, y Speex, FLAC o Theora siguen fuera de
///   alcance.
/// * [`AudioFormat::Unsupported`]: magic bytes de un formato conocido pero fuera
///   de alcance, o un contenedor Ogg con un códec no soportado.
/// * [`AudioFormat::Unknown`]: contenido no reconocible o demasiado corto para
///   decidirlo (incluida una página Ogg incompleta).
pub fn detect_format_from_bytes(header: &[u8]) -> AudioFormat {
    if header.starts_with(b"ID3") {
        return AudioFormat::Mp3;
    }

    if header.len() >= 8 && &header[4..8] == b"ftyp" {
        return AudioFormat::Mp4;
    }

    if is_valid_mpeg_frame_header(header) {
        return AudioFormat::Mp3;
    }

    // Ogg: se decide por el códec del primer paquete, no por la cabecera de
    // página. La cabecera de página son 27 bytes y el primer paquete empieza en
    // `27 + n`, donde `n` es el número de segmentos declarado en el byte 26 (la
    // tabla de segmentos ocupa tantos bytes como segmentos hay). Un Ogg es solo
    // un sobre: Vorbis y Opus están soportados, mientras que Speex, FLAC y
    // Theora siguen fuera de alcance.
    if header.starts_with(b"OggS") {
        // Sin la cabecera de página completa no se puede siquiera situar el
        // primer paquete: el formato queda sin determinar.
        if header.len() < 27 {
            return AudioFormat::Unknown;
        }
        // Versión del flujo (byte 4): solo la `0` es válida.
        if header[4] != 0 {
            return AudioFormat::Unknown;
        }

        let segments = header[26] as usize;
        // La tabla de segmentos va de `27` a `27 + segments`.
        if header.len() < 27 + segments {
            return AudioFormat::Unknown;
        }

        // Longitud declarada del primer paquete: se suman los valores de *lacing*
        // (`255` = el paquete continúa) hasta el primero que lo cierra (`< 255`).
        // Así un identificador corto (p. ej. `\x7fFLAC`, de 5 bytes) se reconoce
        // como paquete completo sin exigir 7 bytes fijos.
        let mut packet_len = 0usize;
        let mut closed = false;
        for &lace in &header[27..27 + segments] {
            packet_len += lace as usize;
            if lace < 255 {
                closed = true;
                break;
            }
        }
        // El primer paquete no termina en esta página: no se puede leer entero.
        if !closed {
            return AudioFormat::Unknown;
        }
        // Un primer paquete de longitud cero (el primer valor de *lacing* es `0`)
        // es «sin primer paquete»: la página no entrega ningún byte, así que no
        // hay identificador de códec que leer y el formato queda sin determinar.
        // No es un formato reconocible pero fuera de alcance (`Unsupported`).
        if packet_len == 0 {
            return AudioFormat::Unknown;
        }

        let payload = 27 + segments;
        // El primer paquete debe estar entero dentro del bloque inspeccionado.
        if header.len() < payload + packet_len {
            return AudioFormat::Unknown;
        }

        let identifier = &header[payload..payload + packet_len];
        return match identifier {
            id if id.starts_with(b"\x01vorbis") => AudioFormat::OggVorbis,
            id if id.starts_with(b"OpusHead") => AudioFormat::OggOpus,
            _ => AudioFormat::Unsupported,
        };
    }

    if UNSUPPORTED_MAGIC
        .iter()
        .any(|magic| header.starts_with(magic))
    {
        return AudioFormat::Unsupported;
    }

    AudioFormat::Unknown
}

/// Comprueba que los primeros bytes formen una cabecera de frame MPEG válida.
///
/// Además de la sincronía (`0xFF` y los tres bits altos del segundo byte a `1`)
/// se validan los campos de versión, layer, bitrate y frecuencia de muestreo:
/// una cabecera con el valor reservado o inválido en cualquiera de ellos no se
/// acepta como MP3. Es la guarda que evita confundir con MP3 un archivo cuyo
/// primer byte sea `0xFF` por casualidad.
fn is_valid_mpeg_frame_header(header: &[u8]) -> bool {
    if header.len() < 4 || header[0] != 0xFF || (header[1] & 0xE0) != 0xE0 {
        return false;
    }

    // Versión MPEG (bits 4-3 del segundo byte): `01` es el valor reservado.
    if ((header[1] >> 3) & 0b11) == 0b01 {
        return false;
    }
    // Layer (bits 2-1 del segundo byte): `00` es el valor reservado.
    if ((header[1] >> 1) & 0b11) == 0b00 {
        return false;
    }
    // Índice de bitrate (bits 7-4 del tercer byte): `1111` es el valor inválido.
    if ((header[2] >> 4) & 0b1111) == 0b1111 {
        return false;
    }
    // Índice de frecuencia de muestreo (bits 3-2 del tercer byte): `11` es el
    // valor reservado.
    if ((header[2] >> 2) & 0b11) == 0b11 {
        return false;
    }

    true
}

// ---------------------------------------------------------------------------
// Backend MP4 (metadatos estilo iTunes: átomos `ilst`)
// ---------------------------------------------------------------------------

/// Asignación de un flag del subcomando `edit` a su átomo `ilst` de MP4 y a la
/// clave genérica [`ItemKey`] con la que `lofty` lo representa.
///
/// Esta estructura y la tabla [`MP4_ATOM_MAP`] son la **única fuente** del
/// mapeo flag → átomo del backend MP4: tanto la escritura
/// ([`apply_metadata_mp4`]) como los tests la consultan, de modo que el mapeo
/// no queda repartido en ramas dispersas por función.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mp4AtomMapping {
    /// Nombre del flag del CLI, tal y como lo acepta `edit` (p. ej. `--title`).
    pub flag: &'static str,
    /// Identificador del átomo dentro de `ilst`. Para los átomos de copyright la
    /// forma es `©xxx`; el subtítulo es un átomo *freeform* con la forma
    /// `----:com.apple.iTunes:SUBTITLE`.
    pub atom: &'static str,
    /// Clave genérica de `lofty` asociada al átomo, o `None` cuando el valor no
    /// se guarda como item de texto (la carátula `covr`, que se gestiona como
    /// [`Picture`]).
    pub item_key: Option<ItemKey>,
}

/// Mapeo único de los 18 flags del CLI a los átomos `ilst` de MP4.
///
/// Cubre las 18 asignaciones de la tabla del contrato (`specs/m4a-tagging`) con
/// 17 átomos distintos, porque `--date` y `--year` comparten el átomo `©day`.
pub const MP4_ATOM_MAP: &[Mp4AtomMapping] = &[
    Mp4AtomMapping {
        flag: "--title",
        atom: "©nam",
        item_key: Some(ItemKey::TrackTitle),
    },
    Mp4AtomMapping {
        flag: "--artist",
        atom: "©ART",
        item_key: Some(ItemKey::TrackArtist),
    },
    Mp4AtomMapping {
        flag: "--album",
        atom: "©alb",
        item_key: Some(ItemKey::AlbumTitle),
    },
    Mp4AtomMapping {
        flag: "--album-artist",
        atom: "aART",
        item_key: Some(ItemKey::AlbumArtist),
    },
    Mp4AtomMapping {
        flag: "--date",
        atom: "©day",
        item_key: Some(ItemKey::RecordingDate),
    },
    Mp4AtomMapping {
        flag: "--year",
        atom: "©day",
        item_key: Some(ItemKey::RecordingDate),
    },
    Mp4AtomMapping {
        flag: "--genre",
        atom: "©gen",
        item_key: Some(ItemKey::Genre),
    },
    Mp4AtomMapping {
        flag: "--track",
        atom: "trkn",
        item_key: Some(ItemKey::TrackNumber),
    },
    Mp4AtomMapping {
        flag: "--season",
        atom: "disk",
        item_key: Some(ItemKey::DiscNumber),
    },
    Mp4AtomMapping {
        flag: "--composer",
        atom: "©wrt",
        item_key: Some(ItemKey::Composer),
    },
    Mp4AtomMapping {
        flag: "--copyright",
        atom: "cprt",
        item_key: Some(ItemKey::CopyrightMessage),
    },
    Mp4AtomMapping {
        flag: "--lyrics",
        atom: "©lyr",
        item_key: Some(ItemKey::UnsyncLyrics),
    },
    Mp4AtomMapping {
        flag: "--cover",
        atom: "covr",
        item_key: None,
    },
    Mp4AtomMapping {
        flag: "--compilation",
        atom: "cpil",
        item_key: Some(ItemKey::FlagCompilation),
    },
    Mp4AtomMapping {
        flag: "--title-sort",
        atom: "sonm",
        item_key: Some(ItemKey::TrackTitleSortOrder),
    },
    Mp4AtomMapping {
        flag: "--artist-sort",
        atom: "soar",
        item_key: Some(ItemKey::TrackArtistSortOrder),
    },
    Mp4AtomMapping {
        flag: "--album-sort",
        atom: "soal",
        item_key: Some(ItemKey::AlbumTitleSortOrder),
    },
    Mp4AtomMapping {
        flag: "--subtitle",
        atom: "----:com.apple.iTunes:SUBTITLE",
        item_key: Some(ItemKey::TrackSubtitle),
    },
];

impl Mp4AtomMapping {
    /// Devuelve la asignación del flag indicado, si existe en [`MP4_ATOM_MAP`].
    pub fn for_flag(flag: &str) -> Option<&'static Mp4AtomMapping> {
        MP4_ATOM_MAP.iter().find(|mapping| mapping.flag == flag)
    }
}

/// Carátula a escribir en el átomo `covr` de un contenedor MP4.
///
/// Se entrega ya leída de disco para que la escritura del tag y la de la
/// carátula sean una sola operación atómica: de `path` se deriva el tipo MIME
/// (por la extensión) y `data` son los bytes de la imagen que se copian intactos
/// al átomo.
#[derive(Debug, Clone, Copy)]
pub struct Mp4Cover<'a> {
    /// Ruta al archivo de imagen; de su extensión se deriva el tipo MIME.
    pub path: &'a Path,
    /// Bytes de la imagen, ya leídos de disco.
    pub data: &'a [u8],
}

/// Opciones de edición de metadatos sobre un contenedor MP4.
///
/// Refleja uno a uno los flags del subcomando `edit`. Los campos que no se
/// indican quedan a `None` (o a `false`/vacío) y no se escriben.
#[derive(Debug, Clone, Default)]
pub struct Mp4EditOptions<'a> {
    /// Título de la canción (`©nam`).
    pub title: Option<&'a str>,
    /// Artistas; se unen con `"; "` en un único átomo `©ART`.
    pub artists: &'a [String],
    /// Álbum (`©alb`).
    pub album: Option<&'a str>,
    /// Artista del álbum (`aART`).
    pub album_artist: Option<&'a str>,
    /// Año (`©day`, compartido con `date`).
    pub year: Option<i32>,
    /// Género (`©gen`).
    pub genre: Option<&'a str>,
    /// Número de pista y su total (`trkn`).
    ///
    /// El átomo guarda el par en 16 bits: un número o un total fuera de
    /// `1..=65535` no es representable y el backend avisa (ver [`edit_mp4`]).
    pub track: Option<NumberWithTotal>,
    /// Temporada / número de disco y su total (`disk`).
    ///
    /// Misma limitación de 16 bits que [`Self::track`].
    pub season: Option<NumberWithTotal>,
    /// Fecha de grabación (`©day`, compartido con `year`).
    pub date: Option<&'a str>,
    /// Copyright (`cprt`).
    pub copyright: Option<&'a str>,
    /// Compositor (`©wrt`).
    pub composer: Option<&'a str>,
    /// Subtítulo (`----:com.apple.iTunes:SUBTITLE`).
    pub subtitle: Option<&'a str>,
    /// Letra (`©lyr`).
    pub lyrics: Option<&'a str>,
    /// Marca de compilación (`cpil`).
    pub compilation: bool,
    /// Orden de clasificación del título (`sonm`).
    pub title_sort: Option<&'a str>,
    /// Orden de clasificación del artista (`soar`).
    pub artist_sort: Option<&'a str>,
    /// Orden de clasificación del álbum (`soal`).
    pub album_sort: Option<&'a str>,
    /// Carátula (`covr`); `None` si no se indicó `--cover`.
    pub cover: Option<Mp4Cover<'a>>,
    /// URL asociada (`--url`). No tiene átomo equivalente en MP4: se omite con
    /// aviso, sin escribir nada.
    pub url: Option<&'a str>,
    /// Artista original (`--original-artist`). No tiene átomo equivalente en
    /// MP4: se omite con aviso, sin escribir nada.
    pub original_artist: Option<&'a str>,
}

/// Resultado de aplicar metadatos a un tag MP4.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Mp4EditReport {
    /// `true` si se aplicó al menos un cambio.
    pub changed: bool,
    /// Warnings en español que el CLI debe emitir por stderr (p. ej. la colisión
    /// de fecha y año). [`edit_mp4`] ya los emite, y además los devuelve para
    /// que sean verificables por los tests.
    pub warnings: Vec<String>,
}

/// Resuelve el valor del átomo `©day`, compartido por `--date` y `--year`.
///
/// Devuelve el valor a escribir y si el año quedó absorbido por la fecha. La
/// regla es que `--date` gana por ser el valor más específico y que el año es su
/// prefijo; si solo se indica uno de los dos, se escribe sin colisión.
pub fn resolve_mp4_date(date: Option<&str>, year: Option<i32>) -> (Option<String>, bool) {
    match (date, year) {
        (Some(date), Some(_)) => (Some(date.to_owned()), true),
        (Some(date), None) => (Some(date.to_owned()), false),
        (None, Some(year)) => (Some(year.to_string()), false),
        (None, None) => (None, false),
    }
}

/// Construye el warning (en español) de la colisión de `--year` y `--date` en
/// el átomo `©day`, cuando gana la fecha.
pub fn mp4_year_absorbed_warning(date: &str, year: i32) -> String {
    format!(
        "⚠️  Advertencia: --year {year} queda absorbido por --date \"{date}\" en el átomo ©day \
         (el año es el prefijo de la fecha); se escribe la fecha."
    )
}

/// Construye el warning (en español) de un campo de `edit` que no tiene átomo
/// equivalente en MP4 (`--url` y `--original-artist`).
///
/// Se emite por stderr, el campo se omite y la operación continúa con el resto
/// de flags. No se crea ningún átomo *freeform* que supla la omisión.
pub fn mp4_field_omitted_warning(flag: &str) -> String {
    format!("⚠️  El campo '{flag}' no tiene átomo equivalente en MP4; se omite.")
}

/// Máximo que cabe en el átomo numérico de MP4 (`trkn`/`disk`), que guarda el
/// número y el total en 16 bits sin signo. `lofty` 0.25.4 descarta en silencio
/// todo valor fuera de `1..=65535` (y el cero).
const MP4_NUMBER_MAX: u32 = u16::MAX as u32;

/// Construye el warning (en español) de un **número** de `--track`/`--season`
/// que no cabe en el átomo numérico de MP4.
///
/// El átomo `trkn` (pista) y `disk` (temporada) guardan el par en 16 bits y, al
/// no ser representable el número, `lofty` lo descartaría en silencio. Para no
/// reportar un éxito falso se omite el **par completo** (número y total), de
/// modo que no quede un total huérfano, y se emite este aviso.
pub fn mp4_number_unrepresentable_warning(field: &str, value: u32) -> String {
    format!(
        "⚠️  Aviso: el número de {field} ({value}) no cabe en el átomo numérico de MP4 \
         (admite 1..=65535); se omiten el número y su total."
    )
}

/// Construye el warning (en español) de un **total** de `--track`/`--season` que
/// no cabe en el átomo numérico de MP4.
///
/// A diferencia del número, aquí sí se escribe el número: solo se omite el
/// total, que `lofty` descartaría en silencio.
pub fn mp4_total_unrepresentable_warning(field: &str, value: u32) -> String {
    format!(
        "⚠️  Aviso: el total de {field} ({value}) no cabe en el átomo numérico de MP4 \
         (admite 1..=65535); se escribe el número y se omite el total."
    )
}

/// Resuelve el par número/total de MP4 a lo que el átomo admite, avisando (una
/// sola vez por campo) de lo que no se puede representar.
///
/// Devuelve `None` cuando el número no es representable: se omite el par
/// completo para no dejar un total huérfano. Cuando solo falla el total,
/// devuelve el número con total `None`. El cero cae en ambos casos, porque el
/// átomo no lo distingue de «sin valor».
fn resolve_mp4_number_pair(
    field: &str,
    value: NumberWithTotal,
    warnings: &mut Vec<String>,
) -> Option<(u32, Option<u32>)> {
    let number = value.number();
    if !(1..=MP4_NUMBER_MAX).contains(&number) {
        warnings.push(mp4_number_unrepresentable_warning(field, number));
        return None;
    }

    match value.total() {
        Some(total) if !(1..=MP4_NUMBER_MAX).contains(&total) => {
            warnings.push(mp4_total_unrepresentable_warning(field, total));
            Some((number, None))
        }
        total => Some((number, total)),
    }
}

/// Construye el warning (en español) que acompaña a una carátula WebP escrita en
/// el átomo `covr` de un MP4.
///
/// El átomo no admite el tipo WebP: para no abortar, la carátula se escribe con
/// el tipo de dato reservado (0), que iTunes y macOS no muestran. El CLI debe
/// avisar de esa pérdida en vez de esconderla.
pub fn mp4_webp_cover_warning() -> String {
    "⚠️  Aviso: la carátula WebP se escribe en el átomo `covr` con el tipo de dato reservado (0), \
     que iTunes y macOS NO muestran; la carátula no será visible en esos reproductores."
        .to_string()
}

/// Escribe una carátula en el átomo `covr` de un tag MP4 con el tipo de dato
/// correcto.
///
/// JPEG y PNG se escriben con su tipo MIME well-known, que `lofty` traduce a los
/// tipos de dato 13 y 14. WebP no tiene tipo well-known en `covr`: `lofty`
/// 0.25.4 **devuelve un error** si se le entrega un tipo MIME que no reconoce,
/// así que la carátula se escribe dejando el MIME deliberadamente **sin
/// establecer** (lo que la librería mapea al tipo de dato reservado, 0) y es
/// responsabilidad del llamador avisar de que no se mostrará.
///
/// # Retorna
///
/// `true` si la carátula se escribió con el tipo reservado (es decir, si era
/// WebP), `false` en caso contrario.
fn push_mp4_cover(tag: &mut Tag, data: &[u8], mime: &str) -> bool {
    // WebP es el único formato que el CLI acepta y que carece de tipo well-known
    // en `covr`: se detecta para dejar el MIME sin establecer.
    let reserved_type = mime == "image/webp";

    let picture = if reserved_type {
        // No se llama a `.mime_type(...)`: el MIME queda `None`, que es el caso
        // que `lofty` escribe como `DataType::Reserved` (0).
        Picture::unchecked(data.to_vec())
            .pic_type(PictureType::CoverFront)
            .description("Cover")
            .build()
    } else {
        // JPEG y PNG: se reutiliza la misma construcción que en MP3 (portada
        // frontal, descripción "Cover" y MIME well-known).
        create_picture_frame(data.to_vec(), mime)
    };

    tag.push_picture(picture);
    reserved_type
}

/// Devuelve la [`ItemKey`] de texto asociada a un flag del CLI según
/// [`MP4_ATOM_MAP`].
///
/// Solo se invoca con los flags constantes que tienen átomo de texto, por lo
/// que la ausencia de mapeo es inalcanzable.
fn mp4_item_key(flag: &str) -> ItemKey {
    Mp4AtomMapping::for_flag(flag)
        .and_then(|mapping| mapping.item_key)
        .expect("todo flag de texto tiene una ItemKey asociada en MP4_ATOM_MAP")
}

/// Escribe un valor de texto en el átomo MP4 que corresponde al flag, tomando
/// la [`ItemKey`] de la tabla [`MP4_ATOM_MAP`] como única fuente del mapeo.
fn set_mp4_text(tag: &mut Tag, flag: &str, value: &str) {
    tag.insert_text(mp4_item_key(flag), value.to_owned());
}

/// Aplica los metadatos indicados a un tag [`TagType::Mp4Ilst`].
///
/// El mapeo de cada flag a su átomo se resuelve siempre a través de
/// [`MP4_ATOM_MAP`]. La carátula (`--cover`) no se escribe aquí: se gestiona
/// como [`Picture`] y es responsabilidad del grupo de carátulas.
///
/// # Colisión de fecha
///
/// `--date` y `--year` comparten el átomo `©day`. Cuando se indican ambos, gana
/// `--date` y se añade un warning al informe (sin abortar la operación).
#[must_use]
pub fn apply_metadata_mp4(tag: &mut Tag, options: &Mp4EditOptions<'_>) -> Mp4EditReport {
    let mut report = Mp4EditReport::default();

    if let Some(title) = options.title {
        set_mp4_text(tag, "--title", title);
        report.changed = true;
    }

    if !options.artists.is_empty() {
        set_mp4_text(tag, "--artist", &options.artists.join("; "));
        report.changed = true;
    }

    if let Some(album) = options.album {
        set_mp4_text(tag, "--album", album);
        report.changed = true;
    }

    if let Some(album_artist) = options.album_artist {
        set_mp4_text(tag, "--album-artist", album_artist);
        report.changed = true;
    }

    if let Some(genre) = options.genre {
        set_mp4_text(tag, "--genre", genre);
        report.changed = true;
    }

    if let Some(track) = options.track
        && let Some((number, total)) =
            resolve_mp4_number_pair("--track", track, &mut report.warnings)
    {
        set_mp4_text(tag, "--track", &number.to_string());
        if let Some(total) = total {
            // El total vive en el mismo átomo `trkn` que el número, como
            // `TrackTotal`; se inserta por la misma vía de bajo nivel que usa
            // `set_mp4_text`. Si no se indica total, la clave que ya estuviera
            // en el archivo se conserva.
            tag.insert_text(ItemKey::TrackTotal, total.to_string());
        }
        report.changed = true;
    }

    if let Some(season) = options.season
        && let Some((number, total)) =
            resolve_mp4_number_pair("--season", season, &mut report.warnings)
    {
        set_mp4_text(tag, "--season", &number.to_string());
        if let Some(total) = total {
            // Igual que la pista, con el átomo `disk` y `DiscTotal`.
            tag.insert_text(ItemKey::DiscTotal, total.to_string());
        }
        report.changed = true;
    }

    if let Some(copyright) = options.copyright {
        set_mp4_text(tag, "--copyright", copyright);
        report.changed = true;
    }

    if let Some(composer) = options.composer {
        set_mp4_text(tag, "--composer", composer);
        report.changed = true;
    }

    if let Some(subtitle) = options.subtitle {
        set_mp4_text(tag, "--subtitle", subtitle);
        report.changed = true;
    }

    if let Some(lyrics) = options.lyrics {
        set_mp4_text(tag, "--lyrics", lyrics);
        report.changed = true;
    }

    if options.compilation {
        set_mp4_text(tag, "--compilation", "1");
        report.changed = true;
    }

    if let Some(title_sort) = options.title_sort {
        set_mp4_text(tag, "--title-sort", title_sort);
        report.changed = true;
    }

    if let Some(artist_sort) = options.artist_sort {
        set_mp4_text(tag, "--artist-sort", artist_sort);
        report.changed = true;
    }

    if let Some(album_sort) = options.album_sort {
        set_mp4_text(tag, "--album-sort", album_sort);
        report.changed = true;
    }

    // `--date` y `--year` comparten `©day`: gana `--date`, con aviso.
    let (day, year_absorbed) = resolve_mp4_date(options.date, options.year);
    if let Some(value) = day {
        set_mp4_text(tag, "--date", &value);
        report.changed = true;
    }
    if year_absorbed {
        // En esta rama ambos están presentes, así que los desempaquetados son
        // seguros; se usan valores por defecto solo como red de seguridad.
        report.warnings.push(mp4_year_absorbed_warning(
            options.date.unwrap_or_default(),
            options.year.unwrap_or_default(),
        ));
    }

    // `--url` y `--original-artist` no tienen átomo equivalente en `ilst`: se
    // omiten con aviso, sin escribir ningún átomo y sin marcar el reporte como
    // cambiado (para que, si son los únicos flags, el archivo no se reescriba).
    if options.url.is_some() {
        report.warnings.push(mp4_field_omitted_warning("--url"));
    }
    if options.original_artist.is_some() {
        report
            .warnings
            .push(mp4_field_omitted_warning("--original-artist"));
    }

    report
}

/// Lee el tag `ilst` (tipo [`TagType::Mp4Ilst`]) de un contenedor MP4.
///
/// Devuelve `Ok(None)` cuando el archivo es un MP4 válido pero no tiene átomo
/// de metadatos.
///
/// El formato se decide por el **contenido**, no por la extensión del nombre.
/// Usa [`TaggedFile::read_from`] con `read_properties(false)` para poder leer
/// archivos MP4 que contienen solo metadatos (sin pistas de audio), como los
/// que `atomicparsley` sí procesa pero `lofty::read_from()` rechazaría.
///
/// # Errores
///
/// Retorna un error si el archivo no se puede leer o no es un contenedor MP4
/// válido.
pub fn read_mp4_tag(path: &Path) -> Result<Option<Tag>, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    // `read_properties(false)`: lee el tag `ilst` de contenedores MP4 sin exigir
    // pistas de audio. Un MP4 con solo metadatos (sin el átomo `trak`) se lee
    // igual que uno normal; los MP4 corruptos siguen fallando.
    let tagged =
        lofty::file::TaggedFile::read_from(&mut file, ParseOptions::new().read_properties(false))?;
    Ok(tagged.tag(TagType::Mp4Ilst).cloned())
}

/// Escribe el tag en un contenedor MP4.
///
/// `lofty` crea el árbol `moov.udta.meta.ilst` cuando no existe y actualiza los
/// tamaños de `moov`/`udta` preservando el resto de átomos (en especial las
/// pistas de audio).
///
/// # Errores
///
/// Retorna un error si el archivo no existe, no es escribible o no se puede
/// interpretar como contenedor MP4.
pub fn write_mp4_tag(path: &Path, tag: &Tag) -> Result<(), Box<dyn std::error::Error>> {
    // Mismas opciones de parseo que en la lectura: sin propiedades para que un
    // MP4 sin pistas de audio también se pueda reescribir.
    tag.save_to_path(
        path,
        WriteOptions::new().parse_options(ParseOptions::new().read_properties(false)),
    )?;
    Ok(())
}

/// Lee, aplica y guarda los metadatos MP4 indicados sobre un archivo.
///
/// Es la operación completa del subcomando `edit` sobre un MP4: lee el tag
/// existente (o crea uno nuevo `Mp4Ilst` si el archivo no tiene metadatos),
/// aplica los cambios y, solo si hay alguno, reescribe el archivo. Los warnings
/// de la operación se emiten por stderr y además se devuelven en el informe.
///
/// La carátula se valida **antes** de escribir el archivo: si la extensión de la
/// imagen no está soportada, la función devuelve error sin modificar un solo
/// byte.
///
/// # Limitación de los átomos numéricos
///
/// Los átomos `trkn` (pista) y `disk` (temporada) guardan el número y el total
/// en **16 bits**: `lofty` 0.25.4 descarta en silencio un valor fuera de
/// `1..=65535` (y el cero) al serializar, y al leer trata el `0` como ausente.
/// Para no reportar un éxito falso, el backend avisa por stderr **y** en el
/// informe cuando el número o el total no son representables: si falla el
/// número se omite el **par completo** (no se deja un total huérfano) y, si solo
/// falla el total, se escribe el número y se omite el total. MP3 y OGG guardan
/// el par como texto, sin ese límite, y por eso **no** avisan: es una asimetría
/// legítima entre contenedores.
///
/// # Errores
///
/// Retorna un error si el archivo no se puede leer, interpretar o escribir, o si
/// la extensión de la carátula no está soportada.
pub fn edit_mp4(
    path: &Path,
    options: &Mp4EditOptions<'_>,
) -> Result<Mp4EditReport, Box<dyn std::error::Error>> {
    let mut tag = read_mp4_tag(path)?.unwrap_or_else(|| Tag::new(TagType::Mp4Ilst));

    let mut report = apply_metadata_mp4(&mut tag, options);

    // Carátula. La validación del MIME ocurre antes de tocar el disco, de modo
    // que una imagen inválida devuelve error sin dejar el archivo a medias.
    if let Some(cover) = options.cover {
        let mime = detect_cover_mime_type(cover.path)?;
        if push_mp4_cover(&mut tag, cover.data, mime) {
            report.warnings.push(mp4_webp_cover_warning());
        }
        report.changed = true;
    }

    if report.changed {
        write_mp4_tag(path, &tag)?;
    }

    for warning in &report.warnings {
        eprintln!("{warning}");
    }

    Ok(report)
}

/// Elimina del tag `ilst` todos los átomos que corresponden a un [`TagKind`].
///
/// La carátula se elimina vaciando la lista de imágenes; el resto de campos se
/// borra con [`Tag::remove_key`]. Los dos tags sin equivalente en MP4
/// ([`TagKind::Url`] y [`TagKind::OriginalArtist`]) no tienen átomo asociado.
fn remove_mp4_kind(tag: &mut Tag, kind: TagKind) -> bool {
    let item_key = match kind {
        TagKind::Title => ItemKey::TrackTitle,
        TagKind::Artist => ItemKey::TrackArtist,
        TagKind::Album => ItemKey::AlbumTitle,
        TagKind::Year | TagKind::Date => ItemKey::RecordingDate,
        TagKind::Genre => ItemKey::Genre,
        TagKind::Track => {
            // El número de pista y su total se guardan juntos en el átomo
            // `trkn` (`TrackNumber` + `TrackTotal`): borrar solo el número
            // dejaría un total huérfano, que el contrato de `track-numbering`
            // prohíbe, así que se borran los dos como una unidad.
            tag.remove_key(ItemKey::TrackTotal);
            tag.remove_key(ItemKey::TrackNumber);
            return true;
        }
        TagKind::Season => {
            // Igual con `disk`: `DiscNumber` y `DiscTotal` van juntos.
            tag.remove_key(ItemKey::DiscTotal);
            tag.remove_key(ItemKey::DiscNumber);
            return true;
        }
        TagKind::Copyright => ItemKey::CopyrightMessage,
        TagKind::Composer => ItemKey::Composer,
        TagKind::Subtitle => ItemKey::TrackSubtitle,
        TagKind::AlbumArtist => ItemKey::AlbumArtist,
        TagKind::Lyrics => {
            // `©lyr` se lee como `Lyrics` o `UnsyncLyrics`; se borran ambos.
            tag.remove_key(ItemKey::UnsyncLyrics);
            tag.remove_key(ItemKey::Lyrics);
            return true;
        }
        TagKind::Compilation => ItemKey::FlagCompilation,
        TagKind::AlbumSort => ItemKey::AlbumTitleSortOrder,
        TagKind::ArtistSort => ItemKey::TrackArtistSortOrder,
        TagKind::TitleSort => ItemKey::TrackTitleSortOrder,
        TagKind::Cover => {
            for index in (0..tag.pictures().len()).rev() {
                tag.remove_picture(index);
            }
            return true;
        }
        // Sin átomo equivalente en MP4: no hay nada que eliminar.
        TagKind::Url | TagKind::OriginalArtist => return false,
    };

    tag.remove_key(item_key);
    true
}

/// Elimina tags específicos de un tag `ilst` de MP4.
///
/// Acepta **los mismos nombres** (en inglés y en español) que [`remove_tags`]
/// sobre MP3, porque ambos comparten la tabla de alias [`classify_tag_alias`].
/// Los tags sin átomo equivalente en MP4 ([`TagKind::Url`] y
/// [`TagKind::OriginalArtist`]) se omiten con un aviso, sin abortar.
///
/// # Retorna
///
/// `true` si se reconoció al menos un tag (aunque el átomo no existiera), del
/// mismo modo que [`remove_tags`].
pub fn remove_tags_mp4(tag: &mut Tag, tags_to_remove: &[String]) -> bool {
    let mut changed = false;

    for tag_name in tags_to_remove {
        let removed = match classify_tag_alias(tag_name) {
            Some(TagKind::Url | TagKind::OriginalArtist) => {
                eprintln!(
                    "⚠️  El tag '{}' no tiene átomo equivalente en MP4; se omite.",
                    tag_name
                );
                false
            }
            Some(kind) => remove_mp4_kind(tag, kind),
            None => {
                eprintln!(
                    "⚠️  Tag desconocido: '{}'. Tags válidos: {}",
                    tag_name, VALID_TAG_NAMES
                );
                false
            }
        };

        if removed {
            outln!("✓ Eliminado: {}", tag_name);
            changed = true;
        }
    }

    changed
}

/// Elimina físicamente el bloque de metadatos `ilst` de un contenedor MP4.
///
/// Borra el árbol `moov.udta.meta.ilst` dejando el audio (`mdat`) intacto, del
/// mismo modo que [`remove_all_id3`] hace con los bloques ID3 de un MP3. La
/// presencia previa se determina leyendo el propio archivo con `lofty`.
///
/// # Retorna
///
/// `true` si el archivo tenía metadatos (y se eliminaron), `false` si no tenía.
///
/// # Errores
///
/// Retorna un error si el archivo no se puede leer, interpretar como MP4 o
/// escribir.
pub fn remove_all_mp4(file: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    let had_metadata = read_mp4_tag(file)?.is_some();

    if had_metadata {
        let mut handle = OpenOptions::new().read(true).write(true).open(file)?;
        // Mismas opciones de parseo que en la lectura (`read_properties(false)`):
        // permiten eliminar el `ilst` de un MP4 sin pistas de audio.
        TagType::Mp4Ilst.remove_from(
            &mut handle,
            WriteOptions::new().parse_options(ParseOptions::new().read_properties(false)),
        )?;
    }

    Ok(had_metadata)
}

/// Indica si un valor de fecha `©day` es un año suelto (`YYYY`).
///
/// El átomo `©day` lo comparten `--year` (que escribe `YYYY`) y `--date` (que
/// escribe `YYYY`, `YYYY-MM` o `YYYY-MM-DD`). Esta heurística decide bajo qué
/// etiqueta mostrarlo en [`display_tags_mp4`] conservando la coherencia con MP3.
fn is_year_only(value: &str) -> bool {
    value.len() == 4 && value.bytes().all(|byte| byte.is_ascii_digit())
}

/// Muestra los metadatos de un tag `ilst` de MP4 en el mismo formato legible
/// (mismos emojis y mismos nombres de campo) que [`display_tags`] usa en MP3.
///
/// El átomo `©day` se muestra como «Año» cuando contiene un año suelto y como
/// «Fecha» en caso contrario, de modo que el conjunto de campos coincide con el
/// que produce `show` sobre MP3 para los mismos flags.
pub fn display_tags_mp4(tag: &Tag) {
    outln!("\n📋 Metadatos encontrados:\n");
    outln!("═══════════════════════════════════════");

    if let Some(title) = tag.get_string(ItemKey::TrackTitle) {
        outln!("🎵 Título:    {}", title);
    }

    if let Some(artist) = tag.get_string(ItemKey::TrackArtist) {
        outln!("🎤 Artista:   {}", artist);
    }

    if let Some(album) = tag.get_string(ItemKey::AlbumTitle) {
        outln!("💿 Álbum:     {}", album);
    }

    if let Some(day) = tag.get_string(ItemKey::RecordingDate) {
        if is_year_only(day) {
            outln!("📅 Año:       {}", day);
        } else {
            outln!("📆 Fecha:     {}", day);
        }
    }

    if let Some(genre) = tag.get_string(ItemKey::Genre) {
        outln!("🎸 Género:    {}", genre);
    }

    if let Some(track) = tag.track() {
        outln!(
            "#️⃣  Pista:     {}",
            format_number_with_total(track, tag.track_total())
        );
    }

    if let Some(season) = tag.disk() {
        outln!(
            "📺 Temporada: {}",
            format_number_with_total(season, tag.disk_total())
        );
    }

    if let Some(copyright) = tag.get_string(ItemKey::CopyrightMessage) {
        outln!("©️  Copyright: {}", copyright);
    }

    if let Some(composer) = tag.get_string(ItemKey::Composer) {
        outln!("🎼 Compositor: {}", composer);
    }

    if let Some(subtitle) = tag.get_string(ItemKey::TrackSubtitle) {
        outln!("📄 Subtítulo: {}", subtitle);
    }

    if let Some(album_artist) = tag.get_string(ItemKey::AlbumArtist) {
        outln!("👥 Artista del álbum: {}", album_artist);
    }

    // Mostrar carátulas si existen
    let pictures = tag.pictures();
    if !pictures.is_empty() {
        outln!("🖼️  Carátulas: {} imagen(es)", pictures.len());
        for (i, pic) in pictures.iter().enumerate() {
            outln!(
                "   [{}] Tipo: {:?}, MIME: {}, Tamaño: {} bytes",
                i + 1,
                pic.pic_type(),
                pic.mime_type().map(|m| m.as_str()).unwrap_or(""),
                pic.data().len()
            );
        }
    }

    // Mostrar letra si existe (`©lyr` se lee como `Lyrics`/`UnsyncLyrics`)
    if let Some(lyrics) = tag
        .get_string(ItemKey::UnsyncLyrics)
        .or_else(|| tag.get_string(ItemKey::Lyrics))
    {
        outln!("📝 Letra:");
        // Mostrar solo las primeras 3 líneas como preview
        let lines: Vec<&str> = lyrics.lines().collect();
        for line in lines.iter().take(3) {
            outln!("   {}", line);
        }
        if lines.len() > 3 {
            outln!("   ... ({} líneas más)", lines.len() - 3);
        }
    }

    // Mostrar metadatos de Apple si existen
    if tag.get_string(ItemKey::FlagCompilation) == Some("1") {
        outln!(" Compilación: Sí");
    }

    if let Some(album_sort) = tag.get_string(ItemKey::AlbumTitleSortOrder) {
        outln!("🔤 Orden álbum: {}", album_sort);
    }

    if let Some(artist_sort) = tag.get_string(ItemKey::TrackArtistSortOrder) {
        outln!("🔤 Orden artista: {}", artist_sort);
    }

    if let Some(title_sort) = tag.get_string(ItemKey::TrackTitleSortOrder) {
        outln!("🔤 Orden título: {}", title_sort);
    }

    let item_count = tag.items().len();
    if item_count > 0 {
        outln!("\n📦 Total de metadatos: {}", item_count);
    }

    outln!("═══════════════════════════════════════\n");
}

// ---------------------------------------------------------------------------
// Backend OGG Vorbis (comentarios de Vorbis: pares `CLAVE=valor`)
// ---------------------------------------------------------------------------

/// Asignación de un flag del subcomando `edit` a la clave del comentario de
/// Vorbis (`VorbisComments`) que le corresponde en un archivo Ogg Vorbis.
///
/// Esta estructura y la tabla [`VORBIS_COMMENT_MAP`] son la **única fuente** del
/// mapeo flag → clave del backend OGG: tanto la escritura
/// ([`apply_metadata_ogg`]) como la visualización ([`display_tags_ogg`]) y los
/// tests la consultan, de modo que el mapeo no queda repartido en ramas dispersas
/// por función.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VorbisCommentMapping {
    /// Nombre del flag del CLI, tal y como lo acepta `edit` (p. ej. `--title`).
    pub flag: &'static str,
    /// Clave del comentario de Vorbis, siempre en mayúsculas (p. ej. `TITLE`).
    pub key: &'static str,
}

impl VorbisCommentMapping {
    /// Devuelve el mapeo del flag indicado, o `None` si no tiene clave
    /// equivalente en Vorbis.
    pub fn for_flag(flag: &str) -> Option<&'static VorbisCommentMapping> {
        VORBIS_COMMENT_MAP
            .iter()
            .find(|mapping| mapping.flag == flag)
    }

    /// Devuelve el mapeo de la clave indicada, o `None` si no está en la tabla.
    ///
    /// La comparación es exacta y distingue mayúsculas, porque las claves de la
    /// tabla se declaran siempre en mayúsculas.
    pub fn for_key(key: &str) -> Option<&'static VorbisCommentMapping> {
        VORBIS_COMMENT_MAP.iter().find(|mapping| mapping.key == key)
    }
}

/// Mapeo único de los 18 flags del CLI a las claves del comentario de Vorbis.
///
/// A diferencia de MP4, aquí `--year` y `--date` tienen claves **distintas**
/// (`YEAR` y `DATE`), por lo que cada flag se escribe en su propia clave sin
/// absorberse. `--url` y `--original-artist` no están en la tabla porque no
/// tienen clave equivalente en Vorbis: se omiten con aviso.
pub const VORBIS_COMMENT_MAP: &[VorbisCommentMapping] = &[
    VorbisCommentMapping {
        flag: "--title",
        key: "TITLE",
    },
    VorbisCommentMapping {
        flag: "--artist",
        key: "ARTIST",
    },
    VorbisCommentMapping {
        flag: "--album",
        key: "ALBUM",
    },
    VorbisCommentMapping {
        flag: "--album-artist",
        key: "ALBUMARTIST",
    },
    VorbisCommentMapping {
        flag: "--year",
        key: "YEAR",
    },
    VorbisCommentMapping {
        flag: "--date",
        key: "DATE",
    },
    VorbisCommentMapping {
        flag: "--genre",
        key: "GENRE",
    },
    VorbisCommentMapping {
        flag: "--track",
        key: "TRACKNUMBER",
    },
    VorbisCommentMapping {
        flag: "--season",
        key: "DISCNUMBER",
    },
    VorbisCommentMapping {
        flag: "--composer",
        key: "COMPOSER",
    },
    VorbisCommentMapping {
        flag: "--copyright",
        key: "COPYRIGHT",
    },
    VorbisCommentMapping {
        flag: "--lyrics",
        key: "LYRICS",
    },
    VorbisCommentMapping {
        flag: "--cover",
        key: "METADATA_BLOCK_PICTURE",
    },
    VorbisCommentMapping {
        flag: "--compilation",
        key: "COMPILATION",
    },
    VorbisCommentMapping {
        flag: "--title-sort",
        key: "TITLESORT",
    },
    VorbisCommentMapping {
        flag: "--artist-sort",
        key: "ARTISTSORT",
    },
    VorbisCommentMapping {
        flag: "--album-sort",
        key: "ALBUMSORT",
    },
    VorbisCommentMapping {
        flag: "--subtitle",
        key: "SUBTITLE",
    },
];

/// Carátula indicada a `edit --cover` sobre un archivo Ogg Vorbis.
///
/// Se entrega ya leída de disco para que la escritura del comentario y la de la
/// carátula sean una sola operación: de `path` se deriva el tipo MIME (por la
/// extensión) y `data` son los bytes de la imagen que se copian intactos al
/// bloque de carátula.
#[derive(Debug, Clone, Copy)]
pub struct OggCover<'a> {
    /// Ruta al archivo de imagen; de su extensión se deriva el tipo MIME.
    pub path: &'a Path,
    /// Bytes de la imagen, ya leídos de disco.
    pub data: &'a [u8],
}

/// Opciones de edición de metadatos sobre un comentario de Vorbis.
///
/// Refleja uno a uno los flags del subcomando `edit`. Los campos que no se
/// indican quedan a `None` (o a `false`/vacío) y no se escriben.
#[derive(Debug, Clone, Default)]
pub struct OggEditOptions<'a> {
    /// Título de la canción (`TITLE`).
    pub title: Option<&'a str>,
    /// Artistas; se unen con `"; "` en una única clave `ARTIST`.
    pub artists: &'a [String],
    /// Álbum (`ALBUM`).
    pub album: Option<&'a str>,
    /// Artista del álbum (`ALBUMARTIST`).
    pub album_artist: Option<&'a str>,
    /// Año (`YEAR`).
    pub year: Option<i32>,
    /// Género (`GENRE`).
    pub genre: Option<&'a str>,
    /// Número de pista y su total (`TRACKNUMBER` y `TRACKTOTAL`).
    pub track: Option<NumberWithTotal>,
    /// Temporada / número de disco y su total (`DISCNUMBER` y `DISCTOTAL`).
    pub season: Option<NumberWithTotal>,
    /// Fecha de grabación (`DATE`).
    pub date: Option<&'a str>,
    /// Copyright (`COPYRIGHT`).
    pub copyright: Option<&'a str>,
    /// Compositor (`COMPOSER`).
    pub composer: Option<&'a str>,
    /// Subtítulo (`SUBTITLE`).
    pub subtitle: Option<&'a str>,
    /// Letra (`LYRICS`).
    pub lyrics: Option<&'a str>,
    /// Marca de compilación (`COMPILATION`).
    pub compilation: bool,
    /// Orden de clasificación del título (`TITLESORT`).
    pub title_sort: Option<&'a str>,
    /// Orden de clasificación del artista (`ARTISTSORT`).
    pub artist_sort: Option<&'a str>,
    /// Orden de clasificación del álbum (`ALBUMSORT`).
    pub album_sort: Option<&'a str>,
    /// Carátula (`METADATA_BLOCK_PICTURE`); `None` si no se indicó `--cover`.
    pub cover: Option<OggCover<'a>>,
    /// URL asociada (`--url`). No tiene clave equivalente en Vorbis: se omite
    /// con aviso, sin escribir nada.
    pub url: Option<&'a str>,
    /// Artista original (`--original-artist`). No tiene clave equivalente en
    /// Vorbis: se omite con aviso, sin escribir nada.
    pub original_artist: Option<&'a str>,
}

/// Resultado de aplicar metadatos a un comentario de Vorbis.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OggEditReport {
    /// `true` si se aplicó al menos un cambio.
    pub changed: bool,
    /// Avisos en español que el CLI debe emitir por stderr (por ejemplo, la
    /// omisión de `--url` o `--original-artist`). [`edit_ogg`] ya los emite y,
    /// además, los devuelve para que sean verificables por los tests.
    pub warnings: Vec<String>,
}

/// Construye el aviso (en español) de un campo de `edit` que no tiene clave
/// equivalente en Vorbis (`--url` y `--original-artist`).
///
/// Se emite por stderr, el campo se omite y la operación continúa con el resto
/// de flags. No se crea ninguna clave inventada que supla la omisión.
pub fn ogg_field_omitted_warning(flag: &str) -> String {
    format!("⚠️  El campo '{flag}' no tiene clave equivalente en OGG; se omite.")
}

/// Devuelve la clave de Vorbis asociada a un flag del CLI según
/// [`VORBIS_COMMENT_MAP`].
///
/// Solo se invoca con los flags constantes que tienen clave, por lo que la
/// ausencia de mapeo es inalcanzable.
fn vorbis_key(flag: &str) -> &'static str {
    VorbisCommentMapping::for_flag(flag)
        .expect("todo flag de texto tiene una clave asociada en VORBIS_COMMENT_MAP")
        .key
}

/// Escribe un valor de texto en la clave de Vorbis que corresponde al flag,
/// tomando la clave de la tabla [`VORBIS_COMMENT_MAP`] como única fuente del
/// mapeo.
fn set_vorbis_text(tag: &mut VorbisComments, flag: &str, value: &str) {
    tag.insert(vorbis_key(flag).to_string(), value.to_string());
}

/// Aplica los metadatos indicados a un [`VorbisComments`].
///
/// El mapeo de cada flag a su clave se resuelve siempre a través de
/// [`VORBIS_COMMENT_MAP`]. La carátula (`--cover`) no se escribe aquí: se
/// gestiona como [`Picture`] y es responsabilidad de [`edit_ogg`].
///
/// `--year` y `--date` acaban en claves **distintas** (`YEAR` y `DATE`), así
/// que no colisionan ni se absorben entre sí. `--url` y `--original-artist` no
/// tienen clave equivalente: se omiten con aviso, sin abortar.
#[must_use]
pub fn apply_metadata_ogg(tag: &mut VorbisComments, options: &OggEditOptions<'_>) -> OggEditReport {
    let mut report = OggEditReport::default();

    if let Some(title) = options.title {
        set_vorbis_text(tag, "--title", title);
        report.changed = true;
    }

    if !options.artists.is_empty() {
        set_vorbis_text(tag, "--artist", &options.artists.join("; "));
        report.changed = true;
    }

    if let Some(album) = options.album {
        set_vorbis_text(tag, "--album", album);
        report.changed = true;
    }

    if let Some(album_artist) = options.album_artist {
        set_vorbis_text(tag, "--album-artist", album_artist);
        report.changed = true;
    }

    if let Some(year) = options.year {
        set_vorbis_text(tag, "--year", &year.to_string());
        report.changed = true;
    }

    if let Some(date) = options.date {
        set_vorbis_text(tag, "--date", date);
        report.changed = true;
    }

    if let Some(genre) = options.genre {
        set_vorbis_text(tag, "--genre", genre);
        report.changed = true;
    }

    if let Some(track) = options.track {
        set_vorbis_text(tag, "--track", &track.number().to_string());
        if let Some(total) = track.total() {
            // `TRACKTOTAL` no corresponde a ningún flag del CLI (no está en
            // `VORBIS_COMMENT_MAP`): es el total del número de pista, que Vorbis
            // guarda como clave independiente de `TRACKNUMBER`. Si no se indica
            // total, la clave que ya estuviera en el archivo se conserva.
            tag.insert("TRACKTOTAL".to_string(), total.to_string());
        }
        report.changed = true;
    }

    if let Some(season) = options.season {
        set_vorbis_text(tag, "--season", &season.number().to_string());
        if let Some(total) = season.total() {
            // Igual que `TRACKTOTAL`: `DISCTOTAL` es el total de la
            // temporada/disco, no un flag del CLI.
            tag.insert("DISCTOTAL".to_string(), total.to_string());
        }
        report.changed = true;
    }

    if let Some(copyright) = options.copyright {
        set_vorbis_text(tag, "--copyright", copyright);
        report.changed = true;
    }

    if let Some(composer) = options.composer {
        set_vorbis_text(tag, "--composer", composer);
        report.changed = true;
    }

    if let Some(subtitle) = options.subtitle {
        set_vorbis_text(tag, "--subtitle", subtitle);
        report.changed = true;
    }

    if let Some(lyrics) = options.lyrics {
        set_vorbis_text(tag, "--lyrics", lyrics);
        report.changed = true;
    }

    if options.compilation {
        set_vorbis_text(tag, "--compilation", "1");
        report.changed = true;
    }

    if let Some(title_sort) = options.title_sort {
        set_vorbis_text(tag, "--title-sort", title_sort);
        report.changed = true;
    }

    if let Some(artist_sort) = options.artist_sort {
        set_vorbis_text(tag, "--artist-sort", artist_sort);
        report.changed = true;
    }

    if let Some(album_sort) = options.album_sort {
        set_vorbis_text(tag, "--album-sort", album_sort);
        report.changed = true;
    }

    // `--url` y `--original-artist` no tienen clave equivalente en Vorbis: se
    // omiten con aviso, sin escribir ninguna clave y sin marcar el informe como
    // cambiado (para que, si son los únicos flags, el archivo no se reescriba).
    if options.url.is_some() {
        report.warnings.push(ogg_field_omitted_warning("--url"));
    }
    if options.original_artist.is_some() {
        report
            .warnings
            .push(ogg_field_omitted_warning("--original-artist"));
    }

    report
}

/// Escribe una carátula como imagen en un comentario de Vorbis.
///
/// El [`Picture`] se construye igual que en MP3 (portada frontal, descripción
/// "Cover" y el MIME que llega como argumento). La información de la imagen se
/// calcula con [`PictureInformation::from_picture`] y, si no se puede inferir
/// (por ejemplo, una imagen de menos de 8 bytes), se usa la información por
/// defecto en lugar de abortar: la carátula se escribe igualmente con sus
/// dimensiones a cero. El MIME **no** se degrada: una carátula WebP se guarda
/// con `image/webp`.
fn push_ogg_cover(tag: &mut VorbisComments, data: &[u8], mime: &str) {
    let picture = create_picture_frame(data.to_vec(), mime);
    let information = PictureInformation::from_picture(&picture).unwrap_or_default();
    let _ = tag.insert_picture(picture, Some(information));
}

/// Lee el comentario de Vorbis de un archivo Ogg.
///
/// Sirve a cualquier contenedor Ogg con comentarios de Vorbis: el códec del
/// primer paquete decide el lector concreto, [`VorbisFile`] para Vorbis y
/// [`OpusFile`] para Opus, de modo que un `.opus` se lee con la misma función
/// que un `.ogg`. El formato se decide por el **contenido**, no por la extensión
/// del nombre: el despacho reutiliza la misma sonda de la primera página que
/// [`detect_format`]. El comentario es obligatorio en el contenedor, por lo que
/// siempre existe: no se envuelve en `Option`.
///
/// Se devuelve el tipo concreto [`VorbisComments`] (no el `Tag` genérico)
/// porque conserva las claves ajenas al mapeo con su **caja original** y el
/// vendor del comentario, que es justo lo que el `Tag` genérico descarta.
///
/// # Errores
///
/// Retorna un error si el archivo no se puede leer o no se puede interpretar
/// como un contenedor Ogg con comentarios de Vorbis (Vorbis u Opus).
pub fn read_vorbis_tag(path: &Path) -> Result<VorbisComments, Box<dyn std::error::Error>> {
    match detect_format(path)? {
        AudioFormat::OggVorbis => {
            let mut file = File::open(path)?;
            let vorbis_file = VorbisFile::read_from(&mut file, ParseOptions::default())?;
            Ok(vorbis_file.vorbis_comments().clone())
        }
        AudioFormat::OggOpus => {
            let mut file = File::open(path)?;
            let opus_file = OpusFile::read_from(&mut file, ParseOptions::default())?;
            Ok(opus_file.vorbis_comments().clone())
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "el archivo no es un contenedor Ogg con comentarios de Vorbis",
        )
        .into()),
    }
}

/// Escribe el comentario de Vorbis en un archivo Ogg Vorbis.
///
/// `lofty` reemplaza solo el paquete de comentario y reescribe el resto del
/// contenedor, de modo que las páginas de audio quedan intactas. El vendor del
/// archivo se conserva: el escritor lo relee del propio archivo y sobrescribe el
/// que llevara el tag en memoria.
///
/// # Errores
///
/// Retorna un error si el archivo no existe, no es escribible o no se puede
/// interpretar como Ogg Vorbis.
pub fn write_vorbis_tag(
    path: &Path,
    tag: &VorbisComments,
) -> Result<(), Box<dyn std::error::Error>> {
    tag.save_to_path(path, WriteOptions::default())?;
    Ok(())
}

/// Lee, aplica y guarda los metadatos OGG indicados sobre un archivo.
///
/// Es la operación completa del subcomando `edit` sobre un Ogg Vorbis: lee el
/// comentario existente, aplica los cambios y, solo si hay alguno, reescribe el
/// archivo. Los avisos de la operación se emiten por stderr y además se devuelven
/// en el informe.
///
/// La carátula se valida **antes** de escribir el archivo: si la extensión de la
/// imagen no está soportada, la función devuelve error sin modificar un solo
/// byte. En OGG no hay degradación de WebP: se escribe el MIME real de la
/// extensión y no se emite ningún aviso de degradación.
///
/// # Errores
///
/// Retorna un error si el archivo no se puede leer, interpretar o escribir, o si
/// la extensión de la carátula no está soportada.
pub fn edit_ogg(
    path: &Path,
    options: &OggEditOptions<'_>,
) -> Result<OggEditReport, Box<dyn std::error::Error>> {
    let mut tag = read_vorbis_tag(path)?;

    let mut report = apply_metadata_ogg(&mut tag, options);

    // Carátula. La validación del MIME ocurre antes de tocar el disco, de modo
    // que una imagen inválida devuelve error sin dejar el archivo a medias.
    if let Some(cover) = options.cover {
        let mime = detect_cover_mime_type(cover.path)?;
        push_ogg_cover(&mut tag, cover.data, mime);
        report.changed = true;
    }

    if report.changed {
        write_vorbis_tag(path, &tag)?;
    }

    for warning in &report.warnings {
        eprintln!("{warning}");
    }

    Ok(report)
}

/// Elimina de un comentario de Vorbis todos los campos que corresponden a un
/// [`TagKind`].
///
/// La carátula se elimina vaciando la lista de imágenes; la letra borra tanto
/// `LYRICS` como `UNSYNCEDLYRICS`; el resto de campos se borra con
/// [`VorbisComments::remove`]. Los dos tags sin clave equivalente en Vorbis
/// ([`TagKind::Url`] y [`TagKind::OriginalArtist`]) no tienen nada que eliminar.
fn remove_ogg_kind(tag: &mut VorbisComments, kind: TagKind) -> bool {
    let key = match kind {
        TagKind::Title => "TITLE",
        TagKind::Artist => "ARTIST",
        TagKind::Album => "ALBUM",
        TagKind::Year => "YEAR",
        TagKind::Date => "DATE",
        TagKind::Genre => "GENRE",
        TagKind::Track => {
            // El número de pista y su total van juntos: un `TRACKTOTAL` sin
            // `TRACKNUMBER` no significa nada (un total huérfano), y el contrato
            // de `track-numbering` prohíbe ese estado, así que borrar el número
            // borra también el total.
            tag.remove("TRACKNUMBER").count();
            tag.remove("TRACKTOTAL").count();
            return true;
        }
        TagKind::Season => {
            // Igual que la pista: el total de temporada (`DISCTOTAL`) se va con
            // su número (`DISCNUMBER`) para no dejar un total huérfano.
            tag.remove("DISCNUMBER").count();
            tag.remove("DISCTOTAL").count();
            return true;
        }
        TagKind::Copyright => "COPYRIGHT",
        TagKind::Composer => "COMPOSER",
        TagKind::Subtitle => "SUBTITLE",
        TagKind::AlbumArtist => "ALBUMARTIST",
        TagKind::Lyrics => {
            // La letra puede estar en `LYRICS` o en `UNSYNCEDLYRICS` (escrita por
            // otra herramienta): se borran ambas.
            tag.remove("LYRICS").count();
            tag.remove("UNSYNCEDLYRICS").count();
            return true;
        }
        TagKind::Compilation => "COMPILATION",
        TagKind::AlbumSort => "ALBUMSORT",
        TagKind::ArtistSort => "ARTISTSORT",
        TagKind::TitleSort => "TITLESORT",
        TagKind::Cover => {
            tag.remove_pictures();
            return true;
        }
        // Sin clave equivalente en Vorbis: no hay nada que eliminar.
        TagKind::Url | TagKind::OriginalArtist => return false,
    };

    tag.remove(key).count();
    true
}

/// Elimina tags específicos de un comentario de Vorbis.
///
/// Acepta **los mismos nombres** (en inglés y en español) que [`remove_tags`]
/// sobre MP3, porque todos comparten la tabla de alias [`classify_tag_alias`].
/// Los tags sin clave equivalente en Vorbis ([`TagKind::Url`] y
/// [`TagKind::OriginalArtist`]) se omiten con un aviso, sin abortar.
///
/// # Retorna
///
/// `true` si se reconoció al menos un tag (aunque la clave no existiera), del
/// mismo modo que [`remove_tags`].
pub fn remove_tags_ogg(tag: &mut VorbisComments, tags_to_remove: &[String]) -> bool {
    let mut changed = false;

    for tag_name in tags_to_remove {
        let removed = match classify_tag_alias(tag_name) {
            Some(TagKind::Url | TagKind::OriginalArtist) => {
                eprintln!(
                    "⚠️  El tag '{}' no tiene clave equivalente en OGG; se omite.",
                    tag_name
                );
                false
            }
            Some(kind) => remove_ogg_kind(tag, kind),
            None => {
                eprintln!(
                    "⚠️  Tag desconocido: '{}'. Tags válidos: {}",
                    tag_name, VALID_TAG_NAMES
                );
                false
            }
        };

        if removed {
            outln!("✓ Eliminado: {}", tag_name);
            changed = true;
        }
    }

    changed
}

/// Vacía el comentario de Vorbis de un archivo Ogg Vorbis.
///
/// Elimina todas las claves de usuario y todas las carátulas, pero **preserva el
/// vendor**: el comentario es obligatorio en el contenedor y no se puede
/// eliminar, así que se deja el comentario sin ningún campo de usuario en lugar
/// de borrarlo. El audio queda intacto (solo se reemplaza el paquete de
/// comentario) y el archivo sigue siendo reproducible, del mismo modo que
/// [`remove_all_mp4`] deja el audio de un MP4 intacto.
///
/// # Retorna
///
/// `true` si el archivo tenía metadatos (y se eliminaron), `false` si no tenía.
///
/// # Errores
///
/// Retorna un error si el archivo no se puede leer, interpretar como Ogg Vorbis
/// o escribir.
pub fn remove_all_ogg(file: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    let tag = read_vorbis_tag(file)?;

    // Hay metadatos si queda alguna clave de usuario o alguna carátula; el
    // vendor no cuenta (es parte obligatoria del contenedor, no del contenido).
    let had_metadata = tag.items().next().is_some() || !tag.pictures().is_empty();

    if had_metadata {
        // Se vacía el comentario conservando el vendor: `lofty` lo relee del
        // archivo al escribir, así que basta con no llevar ningún campo.
        let mut emptied = VorbisComments::new();
        emptied.set_vendor(tag.vendor().to_string());
        write_vorbis_tag(file, &emptied)?;
    }

    Ok(had_metadata)
}

/// Muestra los metadatos de un comentario de Vorbis en el mismo formato legible
/// (mismos emojis y mismos nombres de campo) que [`display_tags`] usa en MP3 y
/// [`display_tags_mp4`] en MP4.
///
/// Solo se muestran los campos de la tabla de mapeo ([`VORBIS_COMMENT_MAP`]) más
/// las carátulas y la letra: el **vendor** y la clave `ENCODER` (que toda
/// herramienta escribe, pero no son metadatos del usuario) no se muestran. Las
/// claves se leen de la propia tabla para que el mapeo no quede duplicado. Cuando
/// no hay ningún campo de la tabla, ni carátulas ni letra, se informa de que no
/// se encontraron metadatos.
pub fn display_tags_ogg(tag: &VorbisComments) {
    outln!("\n📋 Metadatos encontrados:\n");
    outln!("═══════════════════════════════════════");

    let mut found = 0usize;

    if let Some(title) = tag.get(vorbis_key("--title")) {
        outln!("🎵 Título:    {}", title);
        found += 1;
    }

    if let Some(artist) = tag.get(vorbis_key("--artist")) {
        outln!("🎤 Artista:   {}", artist);
        found += 1;
    }

    if let Some(album) = tag.get(vorbis_key("--album")) {
        outln!("💿 Álbum:     {}", album);
        found += 1;
    }

    // `--year` y `--date` son claves distintas en OGG: cada una con su etiqueta.
    if let Some(year) = tag.get(vorbis_key("--year")) {
        outln!("📅 Año:       {}", year);
        found += 1;
    }

    if let Some(date) = tag.get(vorbis_key("--date")) {
        outln!("📆 Fecha:     {}", date);
        found += 1;
    }

    if let Some(genre) = tag.get(vorbis_key("--genre")) {
        outln!("🎸 Género:    {}", genre);
        found += 1;
    }

    // El total de pista (`TRACKTOTAL`) y el de temporada (`DISCTOTAL`) no son
    // flags del CLI, sino el total de sendos números: se pegan al número con una
    // barra cuando existen.
    if let Some(track) = tag.get(vorbis_key("--track")) {
        let total = tag.get("TRACKTOTAL");
        outln!("#️⃣  Pista:     {}", format_number_with_total(track, total));
        found += 1;
    }

    if let Some(season) = tag.get(vorbis_key("--season")) {
        let total = tag.get("DISCTOTAL");
        outln!("📺 Temporada: {}", format_number_with_total(season, total));
        found += 1;
    }

    if let Some(copyright) = tag.get(vorbis_key("--copyright")) {
        outln!("©️  Copyright: {}", copyright);
        found += 1;
    }

    if let Some(composer) = tag.get(vorbis_key("--composer")) {
        outln!("🎼 Compositor: {}", composer);
        found += 1;
    }

    if let Some(subtitle) = tag.get(vorbis_key("--subtitle")) {
        outln!("📄 Subtítulo: {}", subtitle);
        found += 1;
    }

    if let Some(album_artist) = tag.get(vorbis_key("--album-artist")) {
        outln!("👥 Artista del álbum: {}", album_artist);
        found += 1;
    }

    // Mostrar carátulas si existen
    let pictures = tag.pictures();
    if !pictures.is_empty() {
        outln!("🖼️  Carátulas: {} imagen(es)", pictures.len());
        for (i, (pic, _)) in pictures.iter().enumerate() {
            outln!(
                "   [{}] Tipo: {:?}, MIME: {}, Tamaño: {} bytes",
                i + 1,
                pic.pic_type(),
                pic.mime_type().map(|m| m.as_str()).unwrap_or(""),
                pic.data().len()
            );
        }
        found += pictures.len();
    }

    // Mostrar letra si existe (`LYRICS`, o `UNSYNCEDLYRICS` de otra herramienta)
    if let Some(lyrics) = tag
        .get(vorbis_key("--lyrics"))
        .or_else(|| tag.get("UNSYNCEDLYRICS"))
    {
        outln!("📝 Letra:");
        // Mostrar solo las primeras 3 líneas como preview
        let lines: Vec<&str> = lyrics.lines().collect();
        for line in lines.iter().take(3) {
            outln!("   {}", line);
        }
        if lines.len() > 3 {
            outln!("   ... ({} líneas más)", lines.len() - 3);
        }
        found += 1;
    }

    // Mostrar metadatos de Apple si existen
    if tag.get(vorbis_key("--compilation")) == Some("1") {
        outln!(" Compilación: Sí");
        found += 1;
    }

    if let Some(album_sort) = tag.get(vorbis_key("--album-sort")) {
        outln!("🔤 Orden álbum: {}", album_sort);
        found += 1;
    }

    if let Some(artist_sort) = tag.get(vorbis_key("--artist-sort")) {
        outln!("🔤 Orden artista: {}", artist_sort);
        found += 1;
    }

    if let Some(title_sort) = tag.get(vorbis_key("--title-sort")) {
        outln!("🔤 Orden título: {}", title_sort);
        found += 1;
    }

    if found > 0 {
        outln!("\n📦 Total de metadatos: {}", found);
    } else {
        outln!("\n⚠️  No se encontraron metadatos.");
    }

    outln!("═══════════════════════════════════════\n");
}

#[cfg(test)]
mod tests;
