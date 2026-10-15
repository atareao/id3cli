// Tests para la librería id3cli
use super::*;
// La producción se apoya en `lofty`; las aserciones en memoria usan su API.
use lofty::file::TaggedFileExt;
use lofty::id3::v2::{Frame, Id3v2Tag};
use lofty::picture::{Picture, PictureType};
use lofty::tag::{Accessor, ItemKey, TagType};
use std::fs;
use std::path::{Path, PathBuf};

// Los tests que escriben y releen del disco siguen usando el backend ANTERIOR
// `id3` (ahora dev-dependency): verifican la paridad entre librerías —`lofty`
// escribe y `id3` lee— sin tocar ninguna de sus expectativas.
use id3::{Tag, TagLike};

/// Devuelve las carátulas (`APIC`) del tag en memoria.
///
/// `lofty` no expone un `pictures()` sobre `Id3v2Tag`, así que se recogen los
/// frames de imagen directamente para poder aseverar el MIME y los datos.
fn collect_pictures(tag: &Id3v2Tag) -> Vec<&Picture> {
    tag.iter()
        .filter_map(|frame| match frame {
            Frame::Picture(apic) => Some(&*apic.picture),
            _ => None,
        })
        .collect()
}

#[test]
fn test_apply_metadata_title_only() {
    let mut tag = Id3v2Tag::new();
    let changed = apply_metadata(
        &mut tag,
        Some("Test Title"),
        &[],
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    assert!(changed);
    assert_eq!(tag.title().as_deref(), Some("Test Title"));
    assert_eq!(tag.artist().as_deref(), None);
}

#[test]
fn test_apply_metadata_all_fields() {
    let mut tag = Id3v2Tag::new();
    let artists = vec!["Artist".to_string()];
    let changed = apply_metadata(
        &mut tag,
        Some("Title"),
        &artists,
        Some("Album"),
        Some(2026),
        Some("Rock"),
        Some(NumberWithTotal::new(5, None)),
        None,
        Some("2026-01-22"),
        Some("© 2026 Test Records"),
        None,
        None,
        None,
        None,
    );

    assert!(changed);
    assert_eq!(tag.title().as_deref(), Some("Title"));
    assert_eq!(tag.artist().as_deref(), Some("Artist"));
    assert_eq!(tag.album().as_deref(), Some("Album"));
    // `--year` se conserva en el frame `TYER` (no en `TDRC`).
    assert_eq!(
        tag.get_text(&frame_id("TYER"))
            .and_then(|value| value.parse::<i32>().ok()),
        Some(2026)
    );
    assert_eq!(tag.genre().as_deref(), Some("Rock"));
    assert_eq!(tag.track(), Some(5));
    // `--date` se escribe en el frame `TDRC`; se lee directamente el frame
    // porque `Id3v2Tag::date()` de `lofty` solo reconoce la variante `Timestamp`
    // y en memoria el frame se inserta como texto.
    assert_eq!(
        tag.get_text(&frame_id("TDRC")),
        Some("2026-01-22"),
        "--date debe escribir el frame TDRC"
    );
    assert_eq!(tag.get_text(&frame_id("TCOP")), Some("© 2026 Test Records"));
}

#[test]
fn test_apply_metadata_no_changes() {
    let mut tag = Id3v2Tag::new();
    let changed = apply_metadata(
        &mut tag,
        None,
        &[],
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    assert!(!changed);
}

#[test]
fn test_apply_metadata_partial() {
    let mut tag = Id3v2Tag::new();
    let changed = apply_metadata(
        &mut tag,
        Some("Title"),
        &[],
        Some("Album"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    assert!(changed);
    assert_eq!(tag.title().as_deref(), Some("Title"));
    assert_eq!(tag.artist().as_deref(), None);
    assert_eq!(tag.album().as_deref(), Some("Album"));
    assert_eq!(tag.get(&frame_id("TYER")), None);
}

#[test]
fn test_create_picture_frame() {
    let data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG magic bytes
    let picture = create_picture_frame(data.clone(), "image/jpeg");

    assert_eq!(picture.mime_type().map(|m| m.as_str()), Some("image/jpeg"));
    assert_eq!(picture.pic_type(), PictureType::CoverFront);
    assert_eq!(picture.description(), Some("Cover"));
    assert_eq!(picture.data(), data.as_slice());
}

#[test]
fn test_add_cover_art() {
    let mut tag = Id3v2Tag::new();
    let data = vec![0xFF, 0xD8, 0xFF, 0xE0];
    let path = Path::new("test.jpg");

    let result = add_cover_art(&mut tag, path, data.clone());
    assert!(result.is_ok());

    let pictures = collect_pictures(&tag);
    assert_eq!(pictures.len(), 1);
    assert_eq!(pictures[0].data(), data.as_slice());
    assert_eq!(
        pictures[0].mime_type().map(|m| m.as_str()),
        Some("image/jpeg")
    );
}

#[test]
fn test_apply_metadata_preserves_existing() {
    let mut tag = Id3v2Tag::new();
    tag.set_title("Original Title".to_string());
    tag.set_artist("Original Artist".to_string());

    // Solo cambiar el álbum
    apply_metadata(
        &mut tag,
        None,
        &[],
        Some("New Album"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    // El título y artista originales deben preservarse
    assert_eq!(tag.title().as_deref(), Some("Original Title"));
    assert_eq!(tag.artist().as_deref(), Some("Original Artist"));
    assert_eq!(tag.album().as_deref(), Some("New Album"));
}

#[test]
fn test_apply_metadata_overwrites_existing() {
    let mut tag = Id3v2Tag::new();
    tag.set_title("Old Title".to_string());

    apply_metadata(
        &mut tag,
        Some("New Title"),
        &[],
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    assert_eq!(tag.title().as_deref(), Some("New Title"));
}

#[test]
fn test_year_negative() {
    let mut tag = Id3v2Tag::new();
    apply_metadata(
        &mut tag,
        None,
        &[],
        None,
        Some(-1),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    // El año se escribe (y se lee) en `TYER`.
    assert_eq!(
        tag.get_text(&frame_id("TYER"))
            .and_then(|value| value.parse::<i32>().ok()),
        Some(-1)
    );
}

#[test]
fn test_year_future() {
    let mut tag = Id3v2Tag::new();
    apply_metadata(
        &mut tag,
        None,
        &[],
        None,
        Some(3000),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    assert_eq!(
        tag.get_text(&frame_id("TYER"))
            .and_then(|value| value.parse::<i32>().ok()),
        Some(3000)
    );
}

#[test]
fn test_empty_strings() {
    let mut tag = Id3v2Tag::new();
    let artists = vec!["".to_string()];
    let changed = apply_metadata(
        &mut tag,
        Some(""),
        &artists,
        Some(""),
        None,
        Some(""),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    assert!(changed);
    assert_eq!(tag.title().as_deref(), Some(""));
    assert_eq!(tag.artist().as_deref(), Some(""));
}

#[test]
fn test_unicode_characters() {
    let mut tag = Id3v2Tag::new();
    let artists = vec!["Artista 日本語".to_string()];
    apply_metadata(
        &mut tag,
        Some("Título con ñ y acentos"),
        &artists,
        Some("Álbum 🎵"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    assert_eq!(tag.title().as_deref(), Some("Título con ñ y acentos"));
    assert_eq!(tag.artist().as_deref(), Some("Artista 日本語"));
    assert_eq!(tag.album().as_deref(), Some("Álbum 🎵"));
}

#[test]
fn test_multiple_artists() {
    let mut tag = Id3v2Tag::new();
    let artists = vec![
        "Artist 1".to_string(),
        "Artist 2".to_string(),
        "Artist 3".to_string(),
    ];
    let changed = apply_metadata(
        &mut tag, None, &artists, None, None, None, None, None, None, None, None, None, None, None,
    );

    assert!(changed);
    assert_eq!(
        tag.artist().as_deref(),
        Some("Artist 1; Artist 2; Artist 3")
    );
}

#[test]
fn test_multiple_artists_unicode() {
    let mut tag = Id3v2Tag::new();
    let artists = vec!["Artista Español".to_string(), "アーティスト".to_string()];
    apply_metadata(
        &mut tag, None, &artists, None, None, None, None, None, None, None, None, None, None, None,
    );

    assert_eq!(
        tag.artist().as_deref(),
        Some("Artista Español; アーティスト")
    );
}

#[test]
fn test_display_tags_empty() {
    let tag = Id3v2Tag::new();
    // Este test solo verifica que display_tags no hace panic con un tag vacío
    display_tags(&tag);
}

#[test]
fn test_display_tags_with_data() {
    let mut tag = Id3v2Tag::new();
    tag.set_title("Test Song".to_string());
    tag.set_artist("Test Artist".to_string());
    tag.set_album("Test Album".to_string());
    set_year_frame(&mut tag, 2026);
    tag.set_genre("Rock".to_string());

    // Este test verifica que display_tags no hace panic con datos
    display_tags(&tag);
}

#[test]
fn test_track_number() {
    let mut tag = Id3v2Tag::new();
    let changed = apply_metadata(
        &mut tag,
        None,
        &[],
        None,
        None,
        None,
        Some(NumberWithTotal::new(7, None)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    assert!(changed);
    assert_eq!(tag.track(), Some(7));
}

#[test]
fn test_track_number_zero() {
    let mut tag = Id3v2Tag::new();
    apply_metadata(
        &mut tag,
        None,
        &[],
        None,
        None,
        None,
        Some(NumberWithTotal::new(0, None)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    assert_eq!(tag.track(), Some(0));
}

#[test]
fn test_track_with_other_metadata() {
    let mut tag = Id3v2Tag::new();
    let artists = vec!["Artist".to_string()];
    apply_metadata(
        &mut tag,
        Some("Title"),
        &artists,
        None,
        None,
        None,
        Some(NumberWithTotal::new(3, None)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    assert_eq!(tag.title().as_deref(), Some("Title"));
    assert_eq!(tag.artist().as_deref(), Some("Artist"));
    assert_eq!(tag.track(), Some(3));
}

#[test]
fn test_date_recorded() {
    let mut tag = Id3v2Tag::new();
    let changed = apply_metadata(
        &mut tag,
        None,
        &[],
        None,
        None,
        None,
        None,
        None,
        Some("2026-01-22"),
        None,
        None,
        None,
        None,
        None,
    );

    assert!(changed);
    assert_eq!(
        tag.get_text(&frame_id("TDRC")),
        Some("2026-01-22"),
        "--date debe escribir el frame TDRC"
    );
}

#[test]
fn test_copyright() {
    let mut tag = Id3v2Tag::new();
    let changed = apply_metadata(
        &mut tag,
        None,
        &[],
        None,
        None,
        None,
        None,
        None,
        None,
        Some("© 2026 Records"),
        None,
        None,
        None,
        None,
    );

    assert!(changed);
    assert_eq!(tag.get_text(&frame_id("TCOP")), Some("© 2026 Records"));
}

#[test]
fn test_date_and_copyright() {
    let mut tag = Id3v2Tag::new();
    apply_metadata(
        &mut tag,
        None,
        &[],
        None,
        None,
        None,
        None,
        None,
        Some("2026"),
        Some("© Test"),
        None,
        None,
        None,
        None,
    );

    assert_eq!(
        tag.get_text(&frame_id("TDRC")),
        Some("2026"),
        "--date debe escribir el frame TDRC"
    );
    assert_eq!(tag.get_text(&frame_id("TCOP")), Some("© Test"));
}

#[test]
fn test_remove_title() {
    let mut tag = Id3v2Tag::new();
    tag.set_title("Test Title".to_string());
    assert_eq!(tag.title().as_deref(), Some("Test Title"));

    let changed = remove_tags(&mut tag, &["title".to_string()]);
    assert!(changed);
    assert_eq!(tag.title().as_deref(), None);
}

#[test]
fn test_remove_multiple_tags() {
    let mut tag = Id3v2Tag::new();
    tag.set_title("Title".to_string());
    tag.set_artist("Artist".to_string());
    tag.set_album("Album".to_string());

    let changed = remove_tags(&mut tag, &["title".to_string(), "artist".to_string()]);
    assert!(changed);
    assert_eq!(tag.title().as_deref(), None);
    assert_eq!(tag.artist().as_deref(), None);
    assert_eq!(tag.album().as_deref(), Some("Album")); // No eliminado
}

#[test]
fn test_remove_unknown_tag() {
    let mut tag = Id3v2Tag::new();
    tag.set_title("Title".to_string());

    let changed = remove_tags(&mut tag, &["invalid_tag".to_string()]);
    assert!(!changed);
    assert_eq!(tag.title().as_deref(), Some("Title")); // No afectado
}

#[test]
fn test_remove_tags_spanish() {
    let mut tag = Id3v2Tag::new();
    tag.set_title("Título".to_string());
    tag.set_artist("Artista".to_string());

    let changed = remove_tags(&mut tag, &["título".to_string(), "artista".to_string()]);
    assert!(changed);
    assert_eq!(tag.title().as_deref(), None);
    assert_eq!(tag.artist().as_deref(), None);
}

#[test]
fn test_remove_cover() {
    let mut tag = Id3v2Tag::new();
    let data = vec![0xFF, 0xD8, 0xFF, 0xE0];
    let path = Path::new("test.jpg");
    add_cover_art(&mut tag, path, data).unwrap();
    assert_eq!(collect_pictures(&tag).len(), 1);

    let changed = remove_tags(&mut tag, &["cover".to_string()]);
    assert!(changed);
    assert_eq!(collect_pictures(&tag).len(), 0);
}

#[test]
fn test_detect_mime_type_jpeg() {
    assert_eq!(
        detect_mime_type(Path::new("test.jpg")).unwrap(),
        "image/jpeg"
    );
    assert_eq!(
        detect_mime_type(Path::new("test.jpeg")).unwrap(),
        "image/jpeg"
    );
    assert_eq!(
        detect_mime_type(Path::new("test.JPG")).unwrap(),
        "image/jpeg"
    );
}

#[test]
fn test_detect_mime_type_png() {
    assert_eq!(
        detect_mime_type(Path::new("test.png")).unwrap(),
        "image/png"
    );
    assert_eq!(
        detect_mime_type(Path::new("test.PNG")).unwrap(),
        "image/png"
    );
}

#[test]
fn test_detect_mime_type_webp() {
    assert_eq!(
        detect_mime_type(Path::new("test.webp")).unwrap(),
        "image/webp"
    );
    assert_eq!(
        detect_mime_type(Path::new("test.WEBP")).unwrap(),
        "image/webp"
    );
}

#[test]
fn test_detect_mime_type_unsupported() {
    let result = detect_mime_type(Path::new("test.gif"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("no soportado"));
}

#[test]
fn test_add_cover_art_png() {
    let mut tag = Id3v2Tag::new();
    let data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG header
    let path = Path::new("cover.png");

    let result = add_cover_art(&mut tag, path, data.clone());
    assert!(result.is_ok());

    let pictures = collect_pictures(&tag);
    assert_eq!(pictures.len(), 1);
    assert_eq!(
        pictures[0].mime_type().map(|m| m.as_str()),
        Some("image/png")
    );
}

#[test]
fn test_add_cover_art_webp() {
    let mut tag = Id3v2Tag::new();
    let data = vec![0x52, 0x49, 0x46, 0x46]; // WEBP header
    let path = Path::new("cover.webp");

    let result = add_cover_art(&mut tag, path, data.clone());
    assert!(result.is_ok());

    let pictures = collect_pictures(&tag);
    assert_eq!(pictures.len(), 1);
    assert_eq!(
        pictures[0].mime_type().map(|m| m.as_str()),
        Some("image/webp")
    );
}

#[test]
fn test_add_lyrics() {
    let mut tag = Id3v2Tag::new();
    let lyrics_text = "Primera línea\nSegunda línea\nTercera línea";

    let result = add_lyrics(&mut tag, lyrics_text);
    assert!(result);

    // Verificar que se añadió el frame de lyrics con idioma "spa"
    let lyrics = tag
        .unsync_text()
        .next()
        .expect("Debería existir un frame USLT");
    assert_eq!(lyrics.language, *b"spa");
    assert_eq!(&*lyrics.content, lyrics_text);
}

#[test]
fn test_remove_lyrics() {
    let mut tag = Id3v2Tag::new();
    let lyrics_text = "Test lyrics";
    add_lyrics(&mut tag, lyrics_text);

    // Verificar que se añadió
    assert!(tag.unsync_text().next().is_some());

    // Eliminar lyrics
    let changed = remove_tags(&mut tag, &["lyrics".to_string()]);
    assert!(changed);

    // Verificar que se eliminó
    assert!(tag.unsync_text().next().is_none());
}

#[test]
fn test_remove_lyrics_spanish() {
    let mut tag = Id3v2Tag::new();
    add_lyrics(&mut tag, "Test");

    let changed = remove_tags(&mut tag, &["letra".to_string()]);
    assert!(changed);

    assert!(tag.unsync_text().next().is_none());
}

#[test]
fn test_add_url() {
    let mut tag = Id3v2Tag::new();
    let url = "https://example.com/artist";

    let result = add_url(&mut tag, url);
    assert!(result);

    // Verificar que se añadió el frame de URL `WOAR`
    let frame = tag.get(&frame_id("WOAR"));
    assert!(matches!(frame, Some(Frame::Url(_))));
    if let Some(Frame::Url(link)) = frame {
        assert_eq!(link.url(), url);
    }
}

#[test]
fn test_remove_url() {
    let mut tag = Id3v2Tag::new();
    add_url(&mut tag, "https://test.com");

    // Verificar que se añadió
    assert!(tag.get(&frame_id("WOAR")).is_some());

    // Eliminar URL
    let changed = remove_tags(&mut tag, &["url".to_string()]);
    assert!(changed);

    // Verificar que se eliminó
    assert!(tag.get(&frame_id("WOAR")).is_none());
}

#[test]
fn test_add_apple_metadata_compilation() {
    let mut tag = Id3v2Tag::new();
    let changed = add_apple_metadata(&mut tag, true, None, None, None);

    assert!(changed);
    assert_eq!(tag.get_text(&frame_id("TCMP")), Some("1"));
}

#[test]
fn test_add_apple_metadata_sort_orders() {
    let mut tag = Id3v2Tag::new();
    let changed = add_apple_metadata(
        &mut tag,
        false,
        Some("Album Sort"),
        Some("Artist Sort"),
        Some("Title Sort"),
    );

    assert!(changed);
    assert_eq!(tag.get_text(&frame_id("TSOA")), Some("Album Sort"));
    assert_eq!(tag.get_text(&frame_id("TSOP")), Some("Artist Sort"));
    assert_eq!(tag.get_text(&frame_id("TSOT")), Some("Title Sort"));
}

#[test]
fn test_add_apple_metadata_all() {
    let mut tag = Id3v2Tag::new();
    let changed = add_apple_metadata(
        &mut tag,
        true,
        Some("Sort Album"),
        Some("Sort Artist"),
        Some("Sort Title"),
    );

    assert!(changed);
    assert_eq!(tag.get_text(&frame_id("TCMP")), Some("1"));
    assert_eq!(tag.get_text(&frame_id("TSOA")), Some("Sort Album"));
    assert_eq!(tag.get_text(&frame_id("TSOP")), Some("Sort Artist"));
    assert_eq!(tag.get_text(&frame_id("TSOT")), Some("Sort Title"));
}

#[test]
fn test_add_apple_metadata_no_changes() {
    let mut tag = Id3v2Tag::new();
    let changed = add_apple_metadata(&mut tag, false, None, None, None);

    assert!(!changed);
}

#[test]
fn test_remove_compilation() {
    let mut tag = Id3v2Tag::new();
    add_apple_metadata(&mut tag, true, None, None, None);

    let changed = remove_tags(&mut tag, &["compilation".to_string()]);
    assert!(changed);
    assert!(tag.get(&frame_id("TCMP")).is_none());
}

#[test]
fn test_remove_apple_sort_tags() {
    let mut tag = Id3v2Tag::new();
    add_apple_metadata(&mut tag, false, Some("A"), Some("B"), Some("C"));

    let changed = remove_tags(
        &mut tag,
        &[
            "album_sort".to_string(),
            "artist_sort".to_string(),
            "title_sort".to_string(),
        ],
    );

    assert!(changed);
    assert!(tag.get(&frame_id("TSOA")).is_none());
    assert!(tag.get(&frame_id("TSOP")).is_none());
    assert!(tag.get(&frame_id("TSOT")).is_none());
}

#[test]
fn test_remove_compilation_spanish() {
    let mut tag = Id3v2Tag::new();
    add_apple_metadata(&mut tag, true, None, None, None);

    let changed = remove_tags(&mut tag, &["compilación".to_string()]);
    assert!(changed);
    assert!(tag.get(&frame_id("TCMP")).is_none());
}

#[test]
fn test_remove_apple_sort_spanish() {
    let mut tag = Id3v2Tag::new();
    add_apple_metadata(&mut tag, false, Some("A"), None, None);

    let changed = remove_tags(&mut tag, &["orden-album".to_string()]);
    assert!(changed);
    assert!(tag.get(&frame_id("TSOA")).is_none());
}

#[test]
fn test_add_composer() {
    let mut tag = Id3v2Tag::new();
    let changed = apply_metadata(
        &mut tag,
        None,
        &[],
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some("John Lennon"),
        None,
        None,
        None,
    );

    assert!(changed);
    assert_eq!(tag.get_text(&frame_id("TCOM")), Some("John Lennon"));
}

#[test]
fn test_add_subtitle() {
    let mut tag = Id3v2Tag::new();
    let changed = apply_metadata(
        &mut tag,
        None,
        &[],
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some("Extended Version"),
        None,
        None,
    );

    assert!(changed);
    assert_eq!(tag.get_text(&frame_id("TIT3")), Some("Extended Version"));
}

#[test]
fn test_add_original_artist() {
    let mut tag = Id3v2Tag::new();
    let changed = apply_metadata(
        &mut tag,
        None,
        &[],
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some("The Beatles"),
        None,
    );

    assert!(changed);
    assert_eq!(tag.get_text(&frame_id("TOPE")), Some("The Beatles"));
}

#[test]
fn test_add_album_artist() {
    let mut tag = Id3v2Tag::new();
    let changed = apply_metadata(
        &mut tag,
        None,
        &[],
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some("Various Artists"),
    );

    assert!(changed);
    assert_eq!(tag.get_text(&frame_id("TPE2")), Some("Various Artists"));
}

#[test]
fn test_add_all_new_tags() {
    let mut tag = Id3v2Tag::new();
    let changed = apply_metadata(
        &mut tag,
        Some("Title"),
        &[],
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some("Compositor"),
        Some("Subtitle"),
        Some("Original"),
        Some("Album Artist"),
    );

    assert!(changed);
    assert_eq!(tag.title().as_deref(), Some("Title"));
    assert_eq!(tag.get_text(&frame_id("TCOM")), Some("Compositor"));
    assert_eq!(tag.get_text(&frame_id("TIT3")), Some("Subtitle"));
    assert_eq!(tag.get_text(&frame_id("TOPE")), Some("Original"));
    assert_eq!(tag.get_text(&frame_id("TPE2")), Some("Album Artist"));
}

#[test]
fn test_remove_composer() {
    let mut tag = Id3v2Tag::new();
    insert_text_frame(&mut tag, "TCOM", "Test Composer");

    let changed = remove_tags(&mut tag, &["composer".to_string()]);
    assert!(changed);
    assert!(tag.get(&frame_id("TCOM")).is_none());
}

#[test]
fn test_remove_subtitle() {
    let mut tag = Id3v2Tag::new();
    insert_text_frame(&mut tag, "TIT3", "Test Subtitle");

    let changed = remove_tags(&mut tag, &["subtitle".to_string()]);
    assert!(changed);
    assert!(tag.get(&frame_id("TIT3")).is_none());
}

#[test]
fn test_remove_original_artist() {
    let mut tag = Id3v2Tag::new();
    insert_text_frame(&mut tag, "TOPE", "Test Original");

    let changed = remove_tags(&mut tag, &["original_artist".to_string()]);
    assert!(changed);
    assert!(tag.get(&frame_id("TOPE")).is_none());
}

#[test]
fn test_remove_album_artist() {
    let mut tag = Id3v2Tag::new();
    insert_text_frame(&mut tag, "TPE2", "Test Album Artist");

    let changed = remove_tags(&mut tag, &["album_artist".to_string()]);
    assert!(changed);
    assert!(tag.get(&frame_id("TPE2")).is_none());
}

#[test]
fn test_add_season() {
    let mut tag = Id3v2Tag::new();
    let changed = apply_metadata(
        &mut tag,
        None,
        &[],
        None,
        None,
        None,
        None,
        Some(NumberWithTotal::new(2, None)),
        None,
        None,
        None,
        None,
        None,
        None,
    );

    assert!(changed);
    assert_eq!(tag.disk(), Some(2));
}

#[test]
fn test_add_season_with_track() {
    let mut tag = Id3v2Tag::new();
    let changed = apply_metadata(
        &mut tag,
        Some("Episode 5"),
        &[],
        None,
        None,
        None,
        Some(NumberWithTotal::new(5, None)),
        Some(NumberWithTotal::new(2, None)),
        None,
        None,
        None,
        None,
        None,
        None,
    );

    assert!(changed);
    assert_eq!(tag.title().as_deref(), Some("Episode 5"));
    assert_eq!(tag.track(), Some(5));
    assert_eq!(tag.disk(), Some(2));
}

#[test]
fn test_remove_season() {
    let mut tag = Id3v2Tag::new();
    tag.set_disk(3);
    assert_eq!(tag.disk(), Some(3));

    let changed = remove_tags(&mut tag, &["season".to_string()]);
    assert!(changed);
    assert_eq!(tag.disk(), None);
}

#[test]
fn test_remove_season_spanish() {
    let mut tag = Id3v2Tag::new();
    tag.set_disk(5);

    let changed = remove_tags(&mut tag, &["temporada".to_string()]);
    assert!(changed);
    assert_eq!(tag.disk(), None);
}

/// Ruta absoluta al fixture MP3 real y sin tags, versionado en el repositorio.
///
/// Contiene audio MPEG reproducible y ningún tag ID3, por lo que es válido
/// tanto para el backend `id3` actual como para el futuro `lofty`.
fn fixture_mp3_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("sample.mp3")
}

/// Helper para crear un MP3 temporal mínimo sin tags ID3.
///
/// Copia el fixture MP3 real (audio sin tags) a una ruta temporal única, en
/// lugar de fabricar bytes falsos: así el fichero es audio MPEG válido.
fn create_temp_mp3() -> PathBuf {
    let temp_dir = std::env::temp_dir();
    let unique = format!(
        "id3cli_unit_{}_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        std::process::id()
    );
    let path = temp_dir.join(format!("{}.mp3", unique));
    // Audio MPEG real y sin ningún tag ID3 (el fixture versionado).
    fs::copy(fixture_mp3_path(), &path).expect("No se pudo copiar el fixture MP3");
    path
}

/// Helper para limpiar archivos temporales
fn cleanup_file(path: &Path) {
    let _ = fs::remove_file(path);
}

#[test]
fn test_remove_all_id3_removes_v2() {
    let path = create_temp_mp3();

    // Escribir un tag ID3v2
    let mut tag = Tag::new();
    tag.set_title("Title");
    tag.set_artist("Artist");
    tag.write_to_path(&path, id3::Version::Id3v24).unwrap();

    // Verificar precondición: el tag existe
    assert!(Tag::read_from_path(&path).is_ok());

    let format = remove_all_id3(&path).unwrap();
    assert!(matches!(format, RemovedId3::Id3v2));

    // Verificar ausencia física del tag
    assert!(Tag::read_from_path(&path).is_err());

    cleanup_file(&path);
}

#[test]
fn test_remove_all_id3_removes_both_v1_and_v2() {
    let path = create_temp_mp3();

    // Escribir un tag ID3v1 (128 bytes al final del archivo)
    let mut bytes = fs::read(&path).unwrap();
    let mut v1 = vec![0u8; 128];
    v1[0..3].copy_from_slice(b"TAG");
    v1[3..8].copy_from_slice(b"Title");
    v1[33..39].copy_from_slice(b"Artist");
    bytes.append(&mut v1);
    fs::write(&path, &bytes).unwrap();

    // Añadir también un tag ID3v2 encima
    let mut tag = Tag::new();
    tag.set_title("Title V2");
    tag.write_to_path(&path, id3::Version::Id3v24).unwrap();

    let format = remove_all_id3(&path).unwrap();
    assert!(matches!(format, RemovedId3::Both));

    assert!(Tag::read_from_path(&path).is_err());
    // ID3v1 debe haber desaparecido (el archivo ya no termina en "TAG")
    let end = &fs::read(&path).unwrap();
    assert_ne!(&end[end.len().saturating_sub(3)..], b"TAG");

    cleanup_file(&path);
}

#[test]
fn test_remove_all_id3_removes_only_v1() {
    let path = create_temp_mp3();

    // Añadir solo un tag ID3v1 (128 bytes al final del archivo),
    // sin ningún header ID3v2 al inicio
    let mut bytes = fs::read(&path).unwrap();
    let mut v1 = vec![0u8; 128];
    v1[0..3].copy_from_slice(b"TAG");
    v1[3..8].copy_from_slice(b"Title");
    v1[33..39].copy_from_slice(b"Artist");
    bytes.append(&mut v1);
    fs::write(&path, &bytes).unwrap();

    // Verificar precondición: hay un tag ID3v1 y ningún ID3v2
    assert!(id3::v1::Tag::read_from_path(&path).is_ok());
    assert!(Tag::read_from_path(&path).is_err());

    let format = remove_all_id3(&path).unwrap();
    assert!(matches!(format, RemovedId3::Id3v1));

    // Verificar ausencia física del tag
    assert!(id3::v1::Tag::read_from_path(&path).is_err());
    assert!(Tag::read_from_path(&path).is_err());
    let end = &fs::read(&path).unwrap();
    assert_ne!(&end[end.len().saturating_sub(3)..], b"TAG");

    cleanup_file(&path);
}

#[test]
fn test_remove_all_id3_no_tags_returns_none() {
    let path = create_temp_mp3();

    // MP3 sin tags
    let format = remove_all_id3(&path).unwrap();
    assert!(matches!(format, RemovedId3::None));

    cleanup_file(&path);
}

// ---------------------------------------------------------------------------
// Detección del formato de audio por contenido (magic bytes)
// ---------------------------------------------------------------------------

/// Escribe bytes arbitrarios en una ruta temporal única con la extensión dada.
fn create_temp_file(label: &str, extension: &str, bytes: &[u8]) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "id3cli_unit_{}_{}_{}.{}",
        label,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        std::process::id(),
        extension
    ));
    fs::write(&path, bytes).expect("No se pudo escribir el archivo temporal");
    path
}

#[test]
fn test_detect_format_mp3_by_id3_header() {
    // Cabecera ID3v2.4 vacía: basta con los tres primeros bytes `ID3`.
    let header = b"ID3\x04\x00\x00\x00\x00\x00\x00";
    assert_eq!(detect_format_from_bytes(header), AudioFormat::Mp3);
}

#[test]
fn test_detect_format_mp3_by_mpeg_frame_sync() {
    // Cabecera MPEG 1, Layer III, 128 kbps y 44.1 kHz (sin cabecera ID3).
    let header = [0xFF, 0xFB, 0x90, 0x44, 0x00, 0x00, 0x00, 0x00];
    assert_eq!(detect_format_from_bytes(&header), AudioFormat::Mp3);
}

#[test]
fn test_detect_format_mp3_from_real_fixture_header() {
    // Los cuatro primeros bytes del fixture de audio MPEG versionado.
    let header = b"\xFF\xF3\x70\xC0";
    assert_eq!(detect_format_from_bytes(header), AudioFormat::Mp3);
}

#[test]
fn test_detect_format_mp4_ftyp_accepts_any_brand() {
    let brands: [&[u8]; 6] = [b"M4A ", b"M4B ", b"mp42", b"isom", b"avc1", b"qt  "];
    for brand in brands {
        let mut header = vec![0x00, 0x00, 0x00, 0x20];
        header.extend_from_slice(b"ftyp");
        header.extend_from_slice(brand);
        header.extend_from_slice(&[0x00, 0x00, 0x02, 0x00]);
        assert_eq!(
            detect_format_from_bytes(&header),
            AudioFormat::Mp4,
            "el brand {:?} debe reconocerse como MP4",
            String::from_utf8_lossy(brand)
        );
    }
}

#[test]
fn test_detect_format_rejects_implausible_mpeg_sync() {
    // Guarda anti-falso-positivo: `0xFF` + bits altos del segundo byte a 1, pero
    // con un campo reservado o inválido, NO debe reconocerse como MP3.

    // Versión reservada (bits 4-3 del segundo byte = 01).
    assert_eq!(
        detect_format_from_bytes(&[0xFF, 0xEA, 0x70, 0x00]),
        AudioFormat::Unknown
    );
    // Layer reservado (bits 2-1 del segundo byte = 00).
    assert_eq!(
        detect_format_from_bytes(&[0xFF, 0xF8, 0x70, 0x00]),
        AudioFormat::Unknown
    );
    // Índice de bitrate inválido (bits 7-4 del tercer byte = 1111).
    assert_eq!(
        detect_format_from_bytes(&[0xFF, 0xFB, 0xF0, 0x00]),
        AudioFormat::Unknown
    );
    // Índice de frecuencia de muestreo reservado (bits 3-2 del tercer byte = 11).
    assert_eq!(
        detect_format_from_bytes(&[0xFF, 0xFB, 0x7C, 0x00]),
        AudioFormat::Unknown
    );
}

#[test]
fn test_detect_format_empty() {
    assert_eq!(detect_format_from_bytes(&[]), AudioFormat::Unknown);
}

#[test]
fn test_detect_format_too_short() {
    // Menos bytes de los necesarios para una cabecera de frame MPEG completa.
    assert_eq!(detect_format_from_bytes(&[0xFF]), AudioFormat::Unknown);
    assert_eq!(
        detect_format_from_bytes(&[0xFF, 0xFB]),
        AudioFormat::Unknown
    );
    assert_eq!(
        detect_format_from_bytes(&[0xFF, 0xFB, 0x90]),
        AudioFormat::Unknown
    );
    // Menos de 8 bytes: no se puede leer `ftyp` en el offset 4.
    assert_eq!(
        detect_format_from_bytes(b"\x00\x00\x00\x20fty"),
        AudioFormat::Unknown
    );
}

#[test]
fn test_detect_format_unsupported_magic_bytes() {
    let magics: [&[u8]; 5] = [b"fLaC", b"RIFF", b"FORM", b"MAC ", b"wvpk"];
    for magic in magics {
        let mut header = magic.to_vec();
        header.extend_from_slice(&[0u8; 8]);
        assert_eq!(
            detect_format_from_bytes(&header),
            AudioFormat::Unsupported,
            "el magic {:?} debe reconocerse como formato no soportado",
            String::from_utf8_lossy(magic)
        );
    }
}

#[test]
fn test_detect_format_unrecognized_content() {
    assert_eq!(
        detect_format_from_bytes(b"Esto no es un archivo de audio valido"),
        AudioFormat::Unknown
    );
}

#[test]
fn test_detect_format_ignores_extension() {
    // Contenido MP3 en un archivo llamado `.m4a`.
    let mp3_content = fs::read(fixture_mp3_path()).expect("No se pudo leer el fixture MP3");
    let path = create_temp_file("detect_mp3_as_m4a", "m4a", &mp3_content);
    assert_eq!(detect_format(&path).unwrap(), AudioFormat::Mp3);
    cleanup_file(&path);

    // Contenido MP4 (ftyp en el offset 4) en un archivo llamado `.mp3`.
    let mut mp4_content = vec![0x00, 0x00, 0x00, 0x20];
    mp4_content.extend_from_slice(b"ftypM4A \x00\x00\x02\x00");
    let path = create_temp_file("detect_mp4_as_mp3", "mp3", &mp4_content);
    assert_eq!(detect_format(&path).unwrap(), AudioFormat::Mp4);
    cleanup_file(&path);
}

/// Ruta absoluta al fixture Ogg Vorbis real y sin metadatos, versionado en el repo.
///
/// Es un Ogg Vorbis genuino: el patrón de captura `OggS` en el offset 0, la
/// versión `0` en el byte 4, un único segmento declarado en el byte 26 y el
/// primer paquete `\x01vorbis` (cabecera de identificación) en el offset 28.
fn fixture_ogg_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("sample.ogg")
}

/// Construye una página Ogg de un solo paquete con el identificador de códec
/// indicado como primer (y único) paquete.
///
/// Layout de la página Ogg (cabecera de 27 bytes + tabla de segmentos + datos):
/// * bytes 0-3   : patrón de captura `OggS`
/// * byte 4      : versión del flujo (`0`)
/// * byte 5      : tipo de página (`0`)
/// * bytes 6-13  : posición de granule (8 bytes)
/// * bytes 14-17 : número de serie del flujo (4 bytes)
/// * bytes 18-21 : número de secuencia de la página (4 bytes)
/// * bytes 22-25 : CRC (4 bytes)
/// * byte 26     : número de segmentos de página (`1`)
/// * byte 27     : tabla de segmentos (lacing), un segmento del tamaño del códec
/// * bytes 28-…  : primer paquete, el identificador de códec
///
/// La tabla de segmentos declara tantos bytes como ocupe el identificador, de
/// modo que la página sea estructuralmente válida y el primer paquete empiece
/// justo en el offset 28.
fn synthetic_ogg_page(codec: &[u8]) -> Vec<u8> {
    assert!(
        !codec.is_empty() && codec.len() <= 255,
        "el identificador de códec debe caber en un único segmento Ogg"
    );
    let mut page = Vec::with_capacity(28 + codec.len());
    page.extend_from_slice(b"OggS"); // 0-3: patrón de captura
    page.push(0x00); // 4: versión del flujo
    page.push(0x00); // 5: tipo de página (sin BOS ni EOS)
    page.extend_from_slice(&[0x00; 20]); // 6-25: granule + serial + secuencia + CRC
    page.push(0x01); // 26: número de segmentos de página
    page.push(codec.len() as u8); // 27: tabla de segmentos (un segmento)
    page.extend_from_slice(codec); // 28-…: primer paquete
    page
}

#[test]
fn test_detect_format_ogg_vorbis() {
    // El fixture versionado es un Ogg Vorbis real: el primer paquete arranca en
    // el offset 28 con la cabecera de identificación `\x01vorbis`.
    assert_eq!(
        detect_format(&fixture_ogg_path()).unwrap(),
        AudioFormat::OggVorbis
    );

    // La extensión del nombre no interviene: el mismo contenido renombrado como
    // `.mp3` sigue reconociéndose por sus bytes como Ogg Vorbis.
    let content = fs::read(fixture_ogg_path()).expect("No se pudo leer el fixture Ogg");
    let path = create_temp_file("detect_ogg_as_mp3", "mp3", &content);
    assert_eq!(detect_format(&path).unwrap(), AudioFormat::OggVorbis);
    cleanup_file(&path);

    // Por bytes: el identificador de códec vive en el offset 28, así que se
    // pasan 64 bytes (no 16) para poder reconocerlo.
    assert_eq!(
        detect_format_from_bytes(&content[..64]),
        AudioFormat::OggVorbis
    );
}

#[test]
fn test_detect_format_ogg_other_codecs() {
    // Contenedor Ogg con un códec reconocible pero fuera de alcance: son
    // formatos no soportados (`Unsupported`), no contenido desconocido.
    let codecs: [&[u8]; 4] = [
        b"OpusHead",   // Opus
        b"Speex   ",   // Speex (con los tres espacios finales)
        b"\x7fFLAC",   // FLAC dentro de Ogg
        b"\x80theora", // vídeo Theora
    ];
    for codec in codecs {
        let page = synthetic_ogg_page(codec);
        assert_eq!(
            detect_format_from_bytes(&page),
            AudioFormat::Unsupported,
            "el códec {:?} debe reconocerse como Ogg no soportado",
            String::from_utf8_lossy(codec)
        );
    }
}

#[test]
fn test_detect_format_ogg_incomplete_page() {
    // El contrato distingue «no se puede determinar el formato» (`Unknown`) de
    // «formato reconocible pero no soportado» (`Unsupported`). Sin la cabecera
    // de página completa y su primer paquete no se puede siquiera determinar
    // que el contenedor sea Ogg, así que los tres casos son `Unknown`.

    // (1) Solo el patrón de captura, sin cabecera de página.
    assert_eq!(detect_format_from_bytes(b"OggS"), AudioFormat::Unknown);

    // (2) Cabecera de página completa (27 bytes: patrón, versión, tipo, granule,
    // serial, secuencia, CRC y número de segmentos), pero sin tabla de
    // segmentos ni primer paquete.
    let mut header_only = b"OggS".to_vec();
    header_only.extend_from_slice(&[0x00; 23]); // versión + 22 bytes más
    assert_eq!(header_only.len(), 27);
    assert_eq!(detect_format_from_bytes(&header_only), AudioFormat::Unknown);

    // (3) Cabecera de página con su tabla de segmentos declarando un paquete,
    // pero cortada antes de que llegue el primer paquete completo: solo 4 de
    // los 7 bytes de `\x01vorbis`.
    let full_page = synthetic_ogg_page(b"\x01vorbis");
    assert_eq!(full_page.len(), 35);
    assert_eq!(
        detect_format_from_bytes(&full_page[..32]),
        AudioFormat::Unknown
    );
}

#[test]
fn test_detect_format_ogg_empty_packet() {
    // Flujo Ogg degenerado: la página declara un único segmento de *lacing* `0`,
    // de modo que el «primer paquete» es el slice vacío. No hay identificador de
    // códec que leer, así que el formato **no se puede determinar**: el contrato
    // lo deja fuera de «formato no soportado», porque nada reconoce aquí un
    // códec concreto. Se construye la página a mano (no vale `synthetic_ogg_page`,
    // que exige un identificador no vacío).
    let mut page = Vec::with_capacity(28);
    page.extend_from_slice(b"OggS"); // 0-3: patrón de captura
    page.push(0x00); // 4: versión del flujo (la única válida)
    page.push(0x00); // 5: tipo de página
    page.extend_from_slice(&[0x00; 20]); // 6-25: granule + serial + secuencia + CRC
    page.push(0x01); // 26: número de segmentos de página
    page.push(0x00); // 27: único valor de *lacing* `0` → primer paquete vacío
    assert_eq!(page.len(), 28);

    assert_eq!(detect_format_from_bytes(&page), AudioFormat::Unknown);
}

// ---------------------------------------------------------------------------
// Backend MP4 (`ilst`): mapeo, creación del árbol y escritura de átomos
// ---------------------------------------------------------------------------

/// Ruta absoluta al fixture MP4 real y sin metadatos, versionado en el repo.
///
/// Contiene `ftyp` en el offset 4 y **ningún** bloque `ilst`, de modo que
/// ejercita la creación del árbol `moov.udta.meta.ilst` desde cero.
fn fixture_m4a_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("sample.m4a")
}

/// Copia el fixture MP4 a una ruta temporal única sobre la que trabajar.
fn create_temp_m4a(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "id3cli_unit_m4a_{}_{}_{}.m4a",
        label,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        std::process::id()
    ));
    fs::copy(fixture_m4a_path(), &path).expect("No se pudo copiar el fixture MP4");
    path
}

/// Comprueba si un bloque de bytes contiene una secuencia dada.
fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

/// Escribe una imagen temporal con la extensión indicada y la devuelve.
fn write_temp_image(label: &str, extension: &str, bytes: &[u8]) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "id3cli_unit_img_{}_{}_{}.{}",
        label,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        std::process::id(),
        extension
    ));
    fs::write(&path, bytes).expect("No se pudo escribir la imagen temporal");
    path
}

/// Devuelve `(tipo de dato, payload)` del primer `data` box que sigue al átomo
/// indicado, leyendo directamente los bytes del contenedor ISO-BMFF.
///
/// Permite comprobar el **tipo de dato real** que `lofty` escribe (13 JPEG, 14
/// PNG, 0 reservado) en lugar de asumirlo. El layout del `data` box es
/// `[size][data][version/flags][locale][payload]` y el tipo de dato ocupa los 24
/// bits bajos de `version/flags`.
fn data_box_after(bytes: &[u8], atom: &[u8]) -> Option<(u32, Vec<u8>)> {
    let pos = bytes
        .windows(atom.len())
        .position(|window| window == atom)?;
    let box_pos = pos + atom.len();
    if box_pos + 16 > bytes.len() || &bytes[box_pos + 4..box_pos + 8] != b"data" {
        return None;
    }
    let size = u32::from_be_bytes(bytes[box_pos..box_pos + 4].try_into().ok()?) as usize;
    if box_pos + size > bytes.len() {
        return None;
    }
    let version_flags = u32::from_be_bytes(bytes[box_pos + 8..box_pos + 12].try_into().ok()?);
    Some((
        version_flags & 0x00FF_FFFF,
        bytes[box_pos + 16..box_pos + size].to_vec(),
    ))
}

#[test]
fn test_m4a_fixture_is_valid_mp4_without_ilst() {
    let bytes = fs::read(fixture_m4a_path()).expect("No se pudo leer el fixture M4A");

    // `ftyp` en el offset 4 (cabecera ISO-BMFF).
    assert_eq!(
        &bytes[4..8],
        b"ftyp",
        "el fixture debe tener `ftyp` en el offset 4"
    );
    // Sin bloque `ilst`: se probará su creación desde cero.
    assert!(
        !contains_bytes(&bytes, b"ilst"),
        "el fixture no debe contener ningún bloque `ilst`"
    );
    // El detector por contenido lo reconoce como MP4.
    assert_eq!(
        detect_format(&fixture_m4a_path()).unwrap(),
        AudioFormat::Mp4
    );
}

#[test]
fn test_m4a_atom_map_covers_all_assignments() {
    // 18 asignaciones flag→átomo con 17 átomos distintos.
    assert_eq!(
        MP4_ATOM_MAP.len(),
        18,
        "la tabla debe cubrir las 18 asignaciones"
    );

    let distinct: std::collections::HashSet<&str> =
        MP4_ATOM_MAP.iter().map(|mapping| mapping.atom).collect();
    assert_eq!(distinct.len(), 17, "debe haber 17 átomos distintos");

    // `--date` y `--year` comparten el átomo `©day`.
    let date = Mp4AtomMapping::for_flag("--date").expect("--date en la tabla");
    let year = Mp4AtomMapping::for_flag("--year").expect("--year en la tabla");
    assert_eq!(date.atom, "©day");
    assert_eq!(year.atom, "©day");
    assert_eq!(date.item_key, year.item_key);
}

#[test]
fn test_m4a_atom_map_expected_pairs() {
    // Contrato verificado contra `ILST_MAP` de `lofty` 0.25.4.
    let expected: [(&str, &str, Option<ItemKey>); 18] = [
        ("--title", "©nam", Some(ItemKey::TrackTitle)),
        ("--artist", "©ART", Some(ItemKey::TrackArtist)),
        ("--album", "©alb", Some(ItemKey::AlbumTitle)),
        ("--album-artist", "aART", Some(ItemKey::AlbumArtist)),
        ("--date", "©day", Some(ItemKey::RecordingDate)),
        ("--year", "©day", Some(ItemKey::RecordingDate)),
        ("--genre", "©gen", Some(ItemKey::Genre)),
        ("--track", "trkn", Some(ItemKey::TrackNumber)),
        ("--season", "disk", Some(ItemKey::DiscNumber)),
        ("--composer", "©wrt", Some(ItemKey::Composer)),
        ("--copyright", "cprt", Some(ItemKey::CopyrightMessage)),
        ("--lyrics", "©lyr", Some(ItemKey::UnsyncLyrics)),
        ("--cover", "covr", None),
        ("--compilation", "cpil", Some(ItemKey::FlagCompilation)),
        ("--title-sort", "sonm", Some(ItemKey::TrackTitleSortOrder)),
        ("--artist-sort", "soar", Some(ItemKey::TrackArtistSortOrder)),
        ("--album-sort", "soal", Some(ItemKey::AlbumTitleSortOrder)),
        (
            "--subtitle",
            "----:com.apple.iTunes:SUBTITLE",
            Some(ItemKey::TrackSubtitle),
        ),
    ];

    for (flag, atom, item_key) in expected {
        let mapping = Mp4AtomMapping::for_flag(flag)
            .unwrap_or_else(|| panic!("el flag {flag} debe estar en MP4_ATOM_MAP"));
        assert_eq!(mapping.atom, atom, "átomo de {flag}");
        assert_eq!(mapping.item_key, item_key, "ItemKey de {flag}");
    }
}

#[test]
fn test_m4a_subtitle_maps_to_freeform_atom() {
    // `lofty` resuelve el subtítulo de MP4 con el átomo freeform, no con uno
    // estándar: no hay que escribir átomos a mano.
    assert_eq!(
        ItemKey::TrackSubtitle.map_key(TagType::Mp4Ilst),
        Some("----:com.apple.iTunes:SUBTITLE")
    );
}

#[test]
fn test_m4a_creates_ilst_tree_when_absent() {
    let path = create_temp_m4a("create_tree");

    // Precondición: el fixture no tiene tag ni bloque `ilst`.
    assert!(read_mp4_tag(&path).unwrap().is_none());
    assert!(!contains_bytes(&fs::read(&path).unwrap(), b"ilst"));

    let options = Mp4EditOptions {
        title: Some("Prueba"),
        ..Default::default()
    };
    let report = edit_mp4(&path, &options).unwrap();
    assert!(report.changed);

    // El árbol `moov.udta.meta.ilst` se crea y el valor se relee.
    let tag = read_mp4_tag(&path)
        .unwrap()
        .expect("debe existir tag tras escribir");
    assert_eq!(tag.get_string(ItemKey::TrackTitle), Some("Prueba"));

    let bytes = fs::read(&path).unwrap();
    assert!(contains_bytes(&bytes, b"udta"), "debe crearse `udta`");
    assert!(contains_bytes(&bytes, b"meta"), "debe crearse `meta`");
    assert!(contains_bytes(&bytes, b"ilst"), "debe crearse `ilst`");
    assert!(
        contains_bytes(&bytes, b"\xA9nam"),
        "debe crearse el átomo `©nam`"
    );

    cleanup_file(&path);
}

#[test]
fn test_m4a_remains_reproducible_after_write() {
    let path = create_temp_m4a("reproducible");

    let options = Mp4EditOptions {
        title: Some("Cancion"),
        artists: &["Artista".to_string()],
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    // `lofty` lo relee sin error y el tag está presente.
    let tagged = lofty::read_from_path(&path).expect("lofty debe releer el M4A escrito");
    assert!(tagged.tag(TagType::Mp4Ilst).is_some());

    // El archivo sigue siendo un MP4 válido por contenido y conserva `ftyp`.
    assert_eq!(detect_format(&path).unwrap(), AudioFormat::Mp4);
    let bytes = fs::read(&path).unwrap();
    assert_eq!(&bytes[4..8], b"ftyp", "`ftyp` debe seguir en el offset 4");
    // El audio (`mdat`) sobrevive.
    assert!(contains_bytes(&bytes, b"mdat"), "el audio debe conservarse");

    cleanup_file(&path);
}

#[test]
fn test_m4a_no_fields_does_not_touch_file() {
    let path = create_temp_m4a("noop");
    let before = fs::read(&path).unwrap();

    let report = edit_mp4(&path, &Mp4EditOptions::default()).unwrap();
    assert!(!report.changed, "sin campos no hay cambios");
    assert!(report.warnings.is_empty());

    let after = fs::read(&path).unwrap();
    assert_eq!(before, after, "sin campos no se debe reescribir el archivo");
    assert!(read_mp4_tag(&path).unwrap().is_none());

    cleanup_file(&path);
}

// --- Un test por átomo ------------------------------------------------------

#[test]
fn test_m4a_writes_title_atom() {
    let path = create_temp_m4a("title");
    let options = Mp4EditOptions {
        title: Some("Titulo"),
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(tag.get_string(ItemKey::TrackTitle), Some("Titulo"));
    cleanup_file(&path);
}

#[test]
fn test_m4a_writes_artist_atom() {
    let path = create_temp_m4a("artist");
    let artists = vec!["Artista".to_string()];
    let options = Mp4EditOptions {
        artists: &artists,
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(tag.get_string(ItemKey::TrackArtist), Some("Artista"));
    cleanup_file(&path);
}

#[test]
fn test_m4a_writes_album_atom() {
    let path = create_temp_m4a("album");
    let options = Mp4EditOptions {
        album: Some("Album"),
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(tag.get_string(ItemKey::AlbumTitle), Some("Album"));
    cleanup_file(&path);
}

#[test]
fn test_m4a_writes_album_artist_atom() {
    let path = create_temp_m4a("album_artist");
    let options = Mp4EditOptions {
        album_artist: Some("Varios"),
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(tag.get_string(ItemKey::AlbumArtist), Some("Varios"));
    cleanup_file(&path);
}

#[test]
fn test_m4a_writes_genre_atom() {
    let path = create_temp_m4a("genre");
    let options = Mp4EditOptions {
        genre: Some("Rock"),
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(tag.get_string(ItemKey::Genre), Some("Rock"));
    cleanup_file(&path);
}

#[test]
fn test_m4a_writes_track_atom() {
    let path = create_temp_m4a("track");
    let options = Mp4EditOptions {
        track: Some(NumberWithTotal::new(7, None)),
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(
        tag.track(),
        Some(7),
        "el átomo `trkn` debe guardar la pista 7"
    );
    // El átomo `trkn` existe en disco.
    let bytes = fs::read(&path).unwrap();
    assert!(contains_bytes(&bytes, b"trkn"));

    // Caracterización del comportamiento previo (change de OGG + totales,
    // grupo 5): `--track N` guarda el total de pista a CERO. El `data` de
    // `trkn` son 8 bytes `[00 00][pista u16][total u16][00 00]`; aquí el
    // campo de total (bytes 4..6) debe ser 0, no un total real. Se parsean los
    // bytes del átomo directamente (sin `lofty`) y esta expectativa cambiará
    // cuando se introduzca la forma `N/T`.
    let (_, payload) = data_box_after(&bytes, b"trkn").expect("debe existir el átomo `trkn`");
    assert_eq!(payload.len(), 8, "el `data` de `trkn` debe tener 8 bytes");
    assert_eq!(
        u16::from_be_bytes([payload[2], payload[3]]),
        7,
        "el campo de pista del átomo `trkn` debe ser 7"
    );
    assert_eq!(
        u16::from_be_bytes([payload[4], payload[5]]),
        0,
        "el campo de total del átomo `trkn` debe ser 0 (sin `N/T`)"
    );
    cleanup_file(&path);
}

#[test]
fn test_m4a_writes_disk_atom_for_season() {
    let path = create_temp_m4a("season");
    let options = Mp4EditOptions {
        season: Some(NumberWithTotal::new(2, None)),
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(
        tag.disk(),
        Some(2),
        "el átomo `disk` debe guardar la temporada 2"
    );
    let bytes = fs::read(&path).unwrap();
    assert!(contains_bytes(&bytes, b"disk"));

    // Caracterización del comportamiento previo (change de OGG + totales,
    // grupo 5): `--season N` guarda el total de disco a CERO. El `data` de
    // `disk` son 8 bytes `[00 00][temporada u16][total u16][00 00]`; aquí el
    // campo de total (bytes 4..6) debe ser 0. Se parsean los bytes del átomo
    // directamente (sin `lofty`) y esta expectativa cambiará cuando se
    // introduzca la forma `N/T`.
    let (_, payload) = data_box_after(&bytes, b"disk").expect("debe existir el átomo `disk`");
    assert_eq!(payload.len(), 8, "el `data` de `disk` debe tener 8 bytes");
    assert_eq!(
        u16::from_be_bytes([payload[2], payload[3]]),
        2,
        "el campo de temporada del átomo `disk` debe ser 2"
    );
    assert_eq!(
        u16::from_be_bytes([payload[4], payload[5]]),
        0,
        "el campo de total del átomo `disk` debe ser 0 (sin `N/T`)"
    );
    cleanup_file(&path);
}

#[test]
fn test_m4a_writes_composer_atom() {
    let path = create_temp_m4a("composer");
    let options = Mp4EditOptions {
        composer: Some("Compositor"),
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(tag.get_string(ItemKey::Composer), Some("Compositor"));
    cleanup_file(&path);
}

#[test]
fn test_m4a_writes_copyright_atom() {
    let path = create_temp_m4a("copyright");
    let options = Mp4EditOptions {
        copyright: Some("© 2026 Registros"),
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(
        tag.get_string(ItemKey::CopyrightMessage),
        Some("© 2026 Registros")
    );
    assert!(contains_bytes(&fs::read(&path).unwrap(), b"cprt"));
    cleanup_file(&path);
}

#[test]
fn test_m4a_writes_compilation_flag() {
    let path = create_temp_m4a("compilation");
    let options = Mp4EditOptions {
        compilation: true,
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(tag.get_string(ItemKey::FlagCompilation), Some("1"));
    assert!(contains_bytes(&fs::read(&path).unwrap(), b"cpil"));
    cleanup_file(&path);
}

#[test]
fn test_m4a_writes_title_sort_atom() {
    let path = create_temp_m4a("title_sort");
    let options = Mp4EditOptions {
        title_sort: Some("Titulo, El"),
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(
        tag.get_string(ItemKey::TrackTitleSortOrder),
        Some("Titulo, El")
    );
    assert!(contains_bytes(&fs::read(&path).unwrap(), b"sonm"));
    cleanup_file(&path);
}

#[test]
fn test_m4a_writes_artist_sort_atom() {
    let path = create_temp_m4a("artist_sort");
    let options = Mp4EditOptions {
        artist_sort: Some("Artista, El"),
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(
        tag.get_string(ItemKey::TrackArtistSortOrder),
        Some("Artista, El")
    );
    assert!(contains_bytes(&fs::read(&path).unwrap(), b"soar"));
    cleanup_file(&path);
}

#[test]
fn test_m4a_writes_album_sort_atom() {
    let path = create_temp_m4a("album_sort");
    let options = Mp4EditOptions {
        album_sort: Some("Album, El"),
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(
        tag.get_string(ItemKey::AlbumTitleSortOrder),
        Some("Album, El")
    );
    assert!(contains_bytes(&fs::read(&path).unwrap(), b"soal"));
    cleanup_file(&path);
}

#[test]
fn test_m4a_writes_lyrics_atom() {
    let path = create_temp_m4a("lyrics");
    let options = Mp4EditOptions {
        lyrics: Some("Primera linea\nSegunda linea"),
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    // `©lyr` se relee como `Lyrics`/`UnsyncLyrics` (mismo átomo).
    let lyrics = tag
        .get_string(ItemKey::Lyrics)
        .or_else(|| tag.get_string(ItemKey::UnsyncLyrics));
    assert_eq!(lyrics, Some("Primera linea\nSegunda linea"));
    assert!(contains_bytes(&fs::read(&path).unwrap(), b"\xA9lyr"));
    cleanup_file(&path);
}

#[test]
fn test_m4a_writes_subtitle_as_freeform_atom() {
    let path = create_temp_m4a("subtitle");
    let options = Mp4EditOptions {
        subtitle: Some("Descripcion corta"),
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(
        tag.get_string(ItemKey::TrackSubtitle),
        Some("Descripcion corta")
    );

    // El átomo freeform está en disco con su `mean`/`name`, no como átomo
    // estándar.
    let bytes = fs::read(&path).unwrap();
    assert!(contains_bytes(&bytes, b"com.apple.iTunes"));
    assert!(contains_bytes(&bytes, b"SUBTITLE"));
    cleanup_file(&path);
}

#[test]
fn test_m4a_multiple_artists_joined_with_semicolon() {
    let path = create_temp_m4a("multi_artist");
    let artists = vec!["Luis Fonsi".to_string(), "Daddy Yankee".to_string()];
    let options = Mp4EditOptions {
        artists: &artists,
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(
        tag.get_string(ItemKey::TrackArtist),
        Some("Luis Fonsi; Daddy Yankee")
    );
    cleanup_file(&path);
}

#[test]
fn test_m4a_single_artist_has_no_separator() {
    let path = create_temp_m4a("single_artist");
    let artists = vec!["Queen".to_string()];
    let options = Mp4EditOptions {
        artists: &artists,
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(tag.get_string(ItemKey::TrackArtist), Some("Queen"));
    cleanup_file(&path);
}

#[test]
fn test_m4a_edit_preserves_unrelated_atoms() {
    let path = create_temp_m4a("preserve");

    // Primera edición: artista y álbum.
    let first = Mp4EditOptions {
        artists: &["Artista".to_string()],
        album: Some("Album"),
        ..Default::default()
    };
    edit_mp4(&path, &first).unwrap();

    // Se cuela un átomo ajeno a los flags del CLI (un comentario `©cmt`).
    let mut tag = read_mp4_tag(&path).unwrap().unwrap();
    tag.insert_text(ItemKey::Comment, "no tocar".to_string());
    write_mp4_tag(&path, &tag).unwrap();

    // Segunda edición: solo el título.
    let second = Mp4EditOptions {
        title: Some("Nuevo"),
        ..Default::default()
    };
    edit_mp4(&path, &second).unwrap();

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(tag.get_string(ItemKey::TrackTitle), Some("Nuevo"));
    assert_eq!(
        tag.get_string(ItemKey::TrackArtist),
        Some("Artista"),
        "los átomos preexistentes no indicados deben conservarse"
    );
    assert_eq!(tag.get_string(ItemKey::AlbumTitle), Some("Album"));
    assert_eq!(
        tag.get_string(ItemKey::Comment),
        Some("no tocar"),
        "un átomo ajeno a los flags debe sobrevivir"
    );

    cleanup_file(&path);
}

// --- Colisión de `--year` y `--date` en `©day` ------------------------------

#[test]
fn test_resolve_mp4_date_only_date() {
    let (value, absorbed) = resolve_mp4_date(Some("2026-10-13"), None);
    assert_eq!(value.as_deref(), Some("2026-10-13"));
    assert!(!absorbed);
}

#[test]
fn test_resolve_mp4_date_only_year() {
    let (value, absorbed) = resolve_mp4_date(None, Some(2026));
    assert_eq!(value.as_deref(), Some("2026"));
    assert!(!absorbed);
}

#[test]
fn test_resolve_mp4_date_both_flags() {
    let (value, absorbed) = resolve_mp4_date(Some("2026-10-13"), Some(2026));
    assert_eq!(value.as_deref(), Some("2026-10-13"), "gana la fecha");
    assert!(absorbed, "el año queda absorbido");
}

#[test]
fn test_m4a_date_only_writes_day_without_warning() {
    let path = create_temp_m4a("date_only");
    let options = Mp4EditOptions {
        date: Some("2026-10-13"),
        ..Default::default()
    };
    let report = edit_mp4(&path, &options).unwrap();
    assert!(report.warnings.is_empty(), "solo --date no debe avisar");

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(tag.get_string(ItemKey::RecordingDate), Some("2026-10-13"));
    cleanup_file(&path);
}

#[test]
fn test_m4a_year_only_writes_day_without_warning() {
    let path = create_temp_m4a("year_only");
    let options = Mp4EditOptions {
        year: Some(2026),
        ..Default::default()
    };
    let report = edit_mp4(&path, &options).unwrap();
    assert!(report.warnings.is_empty(), "solo --year no debe avisar");

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(tag.get_string(ItemKey::RecordingDate), Some("2026"));
    cleanup_file(&path);
}

#[test]
fn test_m4a_year_and_date_collide_date_wins_with_warning() {
    let path = create_temp_m4a("collision");
    let options = Mp4EditOptions {
        year: Some(2026),
        date: Some("2026-10-13"),
        ..Default::default()
    };
    let report = edit_mp4(&path, &options).unwrap();

    assert_eq!(report.warnings.len(), 1, "debe emitirse un warning");
    assert!(
        report.warnings[0].contains("--year"),
        "el aviso debe nombrar --year"
    );
    assert!(
        report.warnings[0].contains("--date"),
        "el aviso debe nombrar --date"
    );
    assert!(report.changed);

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(
        tag.get_string(ItemKey::RecordingDate),
        Some("2026-10-13"),
        "el átomo ©day debe contener la fecha"
    );
    cleanup_file(&path);
}

// --- Campos sin átomo equivalente en MP4 (`--url`, `--original-artist`) -----

#[test]
fn test_mp4_field_omitted_warning_names_flag_and_reason() {
    let warning = mp4_field_omitted_warning("--url");
    assert!(warning.contains("--url"), "el aviso debe nombrar el flag");
    assert!(
        warning.contains("no tiene átomo equivalente en MP4"),
        "el aviso debe explicar el motivo"
    );
    assert!(
        warning.contains("se omite"),
        "el aviso debe indicar la omisión"
    );
}

#[test]
fn test_m4a_url_and_original_artist_omitted_with_two_warnings_and_no_write() {
    let path = create_temp_m4a("omit_warn");
    let before = fs::read(&path).unwrap();

    let options = Mp4EditOptions {
        url: Some("https://artista.com"),
        original_artist: Some("Autor"),
        ..Default::default()
    };
    let report = edit_mp4(&path, &options).unwrap();

    // Los dos campos omitidos no constituyen un cambio real…
    assert!(
        !report.changed,
        "los dos campos omitidos no implican cambio"
    );
    // …pero sí dos avisos, uno por campo.
    assert_eq!(report.warnings.len(), 2, "debe haber dos avisos");
    assert!(
        report.warnings.iter().any(|w| w.contains("--url")),
        "debe avisarse de --url"
    );
    assert!(
        report
            .warnings
            .iter()
            .any(|w| w.contains("--original-artist")),
        "debe avisarse de --original-artist"
    );

    // El archivo no se reescribe (byte a byte idéntico) y no se creó ningún tag.
    assert_eq!(before, fs::read(&path).unwrap());
    assert!(read_mp4_tag(&path).unwrap().is_none());

    cleanup_file(&path);
}

#[test]
fn test_m4a_url_omitted_but_title_still_written() {
    let path = create_temp_m4a("url_with_title");
    let options = Mp4EditOptions {
        url: Some("https://artista.com"),
        title: Some("Cancion"),
        ..Default::default()
    };
    let report = edit_mp4(&path, &options).unwrap();
    assert!(report.changed);
    assert_eq!(report.warnings.len(), 1, "solo `--url` avisa");

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(tag.get_string(ItemKey::TrackTitle), Some("Cancion"));
    // No se crea ningún átomo freeform de reemplazo.
    assert!(
        !contains_bytes(&fs::read(&path).unwrap(), b"----:com.apple.iTunes"),
        "no debe crearse un átomo freeform para la URL"
    );

    cleanup_file(&path);
}

// --- Tabla de alias compartida por MP3 y MP4 --------------------------------

#[test]
fn test_classify_tag_alias_english_names() {
    assert_eq!(classify_tag_alias("title"), Some(TagKind::Title));
    assert_eq!(classify_tag_alias("artist"), Some(TagKind::Artist));
    assert_eq!(classify_tag_alias("album"), Some(TagKind::Album));
    assert_eq!(classify_tag_alias("year"), Some(TagKind::Year));
    assert_eq!(classify_tag_alias("date"), Some(TagKind::Date));
    assert_eq!(classify_tag_alias("cover"), Some(TagKind::Cover));
    assert_eq!(classify_tag_alias("lyrics"), Some(TagKind::Lyrics));
    assert_eq!(classify_tag_alias("url"), Some(TagKind::Url));
    assert_eq!(
        classify_tag_alias("original_artist"),
        Some(TagKind::OriginalArtist)
    );
    assert_eq!(classify_tag_alias("title-sort"), Some(TagKind::TitleSort));
}

#[test]
fn test_classify_tag_alias_spanish_names() {
    assert_eq!(classify_tag_alias("título"), Some(TagKind::Title));
    assert_eq!(classify_tag_alias("artista"), Some(TagKind::Artist));
    assert_eq!(classify_tag_alias("álbum"), Some(TagKind::Album));
    assert_eq!(classify_tag_alias("año"), Some(TagKind::Year));
    assert_eq!(classify_tag_alias("fecha"), Some(TagKind::Date));
    assert_eq!(classify_tag_alias("caratula"), Some(TagKind::Cover));
    assert_eq!(classify_tag_alias("letra"), Some(TagKind::Lyrics));
    assert_eq!(classify_tag_alias("orden-titulo"), Some(TagKind::TitleSort));
    assert_eq!(
        classify_tag_alias("artista-album"),
        Some(TagKind::AlbumArtist)
    );
}

#[test]
fn test_classify_tag_alias_is_case_insensitive_and_rejects_unknown() {
    assert_eq!(classify_tag_alias("TITLE"), Some(TagKind::Title));
    assert_eq!(classify_tag_alias("Título"), Some(TagKind::Title));
    assert_eq!(classify_tag_alias("inventado"), None);
    assert_eq!(classify_tag_alias(""), None);
}

// --- Borrado sobre MP4 ------------------------------------------------------

#[test]
fn test_m4a_remove_specific_tag_keeps_rest() {
    let path = create_temp_m4a("remove_specific");

    // Escribir tres átomos.
    let options = Mp4EditOptions {
        title: Some("T"),
        artists: &["A".to_string()],
        album: Some("Alb"),
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    // Eliminar solo el título con su nombre en español.
    let mut tag = read_mp4_tag(&path).unwrap().unwrap();
    let removed = remove_tags_mp4(&mut tag, &["título".to_string()]);
    assert!(removed);
    write_mp4_tag(&path, &tag).unwrap();

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(tag.get_string(ItemKey::TrackTitle), None);
    assert_eq!(tag.get_string(ItemKey::TrackArtist), Some("A"));
    assert_eq!(tag.get_string(ItemKey::AlbumTitle), Some("Alb"));

    cleanup_file(&path);
}

#[test]
fn test_m4a_remove_all_flags_of_a_kind_via_alias() {
    let path = create_temp_m4a("remove_aliases");

    let options = Mp4EditOptions {
        // Estos dos deben sobrevivir.
        title: Some("T"),
        artists: &["A".to_string()],
        // Estos cuatro se eliminan con alias.
        year: Some(2026),
        genre: Some("Rock"),
        compilation: true,
        title_sort: Some("T, El"),
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    let mut tag = read_mp4_tag(&path).unwrap().unwrap();
    // Se eliminan con alias: año (Year→©day), género, orden-titulo y compilación.
    assert!(remove_tags_mp4(
        &mut tag,
        &[
            "año".to_string(),
            "genero".to_string(),
            "orden-titulo".to_string(),
            "compilación".to_string(),
        ]
    ));
    write_mp4_tag(&path, &tag).unwrap();

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(tag.get_string(ItemKey::RecordingDate), None);
    assert_eq!(tag.get_string(ItemKey::Genre), None);
    assert_eq!(tag.get_string(ItemKey::TrackTitleSortOrder), None);
    assert_eq!(tag.get_string(ItemKey::FlagCompilation), None);
    // Los no mencionados sobreviven.
    assert_eq!(tag.get_string(ItemKey::TrackTitle), Some("T"));
    assert_eq!(tag.get_string(ItemKey::TrackArtist), Some("A"));

    cleanup_file(&path);
}

#[test]
fn test_m4a_remove_unknown_tag_is_ignored() {
    let path = create_temp_m4a("remove_unknown");

    let options = Mp4EditOptions {
        title: Some("T"),
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();
    let before = fs::read(&path).unwrap();

    let mut tag = read_mp4_tag(&path).unwrap().unwrap();
    // Un nombre desconocido no cambia nada.
    assert!(!remove_tags_mp4(&mut tag, &["inventado".to_string()]));
    // El título sigue ahí.
    assert_eq!(tag.get_string(ItemKey::TrackTitle), Some("T"));

    // No se reescribe: el archivo queda igual.
    assert_eq!(fs::read(&path).unwrap(), before);

    cleanup_file(&path);
}

#[test]
fn test_m4a_remove_url_and_original_artist_are_omitted() {
    let path = create_temp_m4a("remove_no_atom");

    let options = Mp4EditOptions {
        title: Some("T"),
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    let mut tag = read_mp4_tag(&path).unwrap().unwrap();
    // `url` y `original_artist` no tienen átomo en MP4: no cuentan como cambio.
    assert!(!remove_tags_mp4(
        &mut tag,
        &["url".to_string(), "original_artist".to_string()]
    ));
    // El título no se toca.
    assert_eq!(tag.get_string(ItemKey::TrackTitle), Some("T"));

    cleanup_file(&path);
}

#[test]
fn test_m4a_remove_all_reports_and_removes_ilst() {
    let path = create_temp_m4a("remove_all_mp4");

    // Sin metadatos: `false`, sin modificar el archivo.
    let before = fs::read(&path).unwrap();
    assert!(!remove_all_mp4(&path).unwrap());
    assert_eq!(fs::read(&path).unwrap(), before);

    // Con metadatos: `true`, el bloque `ilst` desaparece y `mdat` sobrevive.
    let options = Mp4EditOptions {
        title: Some("T"),
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();
    assert!(contains_bytes(&fs::read(&path).unwrap(), b"ilst"));

    let audio_before = mdat_payload(&fs::read(&path).unwrap());
    assert!(remove_all_mp4(&path).unwrap());

    let after = fs::read(&path).unwrap();
    assert!(!contains_bytes(&after, b"ilst"));
    assert_eq!(mdat_payload(&after), audio_before);

    // Y ya no hay metadatos.
    assert!(read_mp4_tag(&path).unwrap().is_none());

    cleanup_file(&path);
}

// --- Carátulas sobre MP4 (`covr`) y letras (`©lyr`) -------------------------

/// Bytes de una imagen JPEG reconocible (signatura + inicio de APP0).
const JPEG_BYTES: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46];
/// Bytes de una imagen PNG reconocible (signatura + primer trozo).
const PNG_BYTES: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
];
/// Bytes de una imagen WebP reconocible.
const WEBP_BYTES: &[u8] = b"RIFF\x24\x00\x00\x00WEBPVP8 ";

#[test]
fn test_m4a_writes_jpeg_cover_with_wellknown_type() {
    let path = create_temp_m4a("cover_jpeg");
    let image = write_temp_image("cover_jpeg", "jpg", JPEG_BYTES);

    let options = Mp4EditOptions {
        cover: Some(Mp4Cover {
            path: &image,
            data: JPEG_BYTES,
        }),
        ..Default::default()
    };
    let report = edit_mp4(&path, &options).unwrap();
    assert!(report.changed);
    assert!(report.warnings.is_empty(), "JPEG no debe emitir aviso");

    // El tipo de dato real del `data` box del `covr` es JPEG (13) y los bytes
    // de la imagen quedan intactos.
    let bytes = fs::read(&path).unwrap();
    let (data_type, payload) = data_box_after(&bytes, b"covr").expect("debe existir `covr`");
    assert_eq!(data_type, 13, "JPEG debe usar el tipo well-known 13");
    assert_eq!(
        payload, JPEG_BYTES,
        "los bytes de la imagen deben ser idénticos"
    );

    // Y `lofty` lo relee como `image/jpeg`.
    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(tag.pictures().len(), 1);
    assert_eq!(
        tag.pictures()[0].mime_type().map(|m| m.as_str()),
        Some("image/jpeg")
    );
    assert_eq!(tag.pictures()[0].data(), JPEG_BYTES);

    cleanup_file(&path);
    cleanup_file(&image);
}

#[test]
fn test_m4a_writes_png_cover_with_wellknown_type() {
    let path = create_temp_m4a("cover_png");
    let image = write_temp_image("cover_png", "png", PNG_BYTES);

    let options = Mp4EditOptions {
        cover: Some(Mp4Cover {
            path: &image,
            data: PNG_BYTES,
        }),
        ..Default::default()
    };
    let report = edit_mp4(&path, &options).unwrap();
    assert!(report.changed);
    assert!(report.warnings.is_empty(), "PNG no debe emitir aviso");

    let bytes = fs::read(&path).unwrap();
    let (data_type, payload) = data_box_after(&bytes, b"covr").expect("debe existir `covr`");
    assert_eq!(data_type, 14, "PNG debe usar el tipo well-known 14");
    assert_eq!(payload, PNG_BYTES);

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(
        tag.pictures()[0].mime_type().map(|m| m.as_str()),
        Some("image/png")
    );

    cleanup_file(&path);
    cleanup_file(&image);
}

#[test]
fn test_m4a_webp_cover_uses_reserved_type_warns_and_writes_other_fields() {
    let path = create_temp_m4a("cover_webp");
    let image = write_temp_image("cover_webp", "webp", WEBP_BYTES);

    let options = Mp4EditOptions {
        title: Some("Con portada WebP"),
        cover: Some(Mp4Cover {
            path: &image,
            data: WEBP_BYTES,
        }),
        ..Default::default()
    };
    let report = edit_mp4(&path, &options).unwrap();
    assert!(report.changed, "la carátula WebP debe contar como cambio");

    // Aviso de que no será visible en iTunes ni en macOS.
    assert_eq!(
        report.warnings.len(),
        1,
        "WebP debe emitir exactamente un aviso"
    );
    let warning = &report.warnings[0];
    assert!(
        warning.contains("WebP"),
        "el aviso debe nombrar WebP: {warning}"
    );
    assert!(
        warning.contains("iTunes") && warning.contains("macOS"),
        "el aviso debe nombrar iTunes y macOS: {warning}"
    );
    assert!(
        warning.contains("no será visible"),
        "el aviso debe advertir de la invisibilidad: {warning}"
    );

    // El tipo de dato real es el reservado (0), con los bytes intactos.
    let bytes = fs::read(&path).unwrap();
    let (data_type, payload) = data_box_after(&bytes, b"covr").expect("debe existir `covr`");
    assert_eq!(
        data_type, 0,
        "WebP debe escribirse con el tipo de dato reservado (0)"
    );
    assert_eq!(payload, WEBP_BYTES);

    // El resto de campos indicados se escriben con normalidad.
    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(
        tag.get_string(ItemKey::TrackTitle),
        Some("Con portada WebP")
    );
    // `lofty` relee la carátula sin MIME (tipo reservado).
    assert_eq!(tag.pictures().len(), 1);
    assert_eq!(tag.pictures()[0].mime_type(), None);
    assert_eq!(tag.pictures()[0].data(), WEBP_BYTES);

    cleanup_file(&path);
    cleanup_file(&image);
}

#[test]
fn test_m4a_cover_unsupported_extension_errors_without_writing() {
    let path = create_temp_m4a("cover_tiff");
    let before = fs::read(&path).unwrap();
    let image = write_temp_image("cover_tiff", "tiff", b"II*\x00");

    let options = Mp4EditOptions {
        // Aunque haya un campo de texto, tampoco se debe escribir.
        title: Some("No debe escribirse"),
        cover: Some(Mp4Cover {
            path: &image,
            data: b"II*\x00",
        }),
        ..Default::default()
    };
    let error = edit_mp4(&path, &options).expect_err("una extensión no soportada debe fallar");
    let message = error.to_string();
    assert!(
        message.contains("Formato de imagen no soportado") && message.contains(".tiff"),
        "el mensaje debe explicar el formato no soportado: {message}"
    );

    // No se escribió ni un byte.
    assert_eq!(
        fs::read(&path).unwrap(),
        before,
        "el archivo no debe modificarse si la carátula no es válida"
    );

    cleanup_file(&path);
    cleanup_file(&image);
}

#[test]
fn test_m4a_remove_cover_removes_covr_and_keeps_rest() {
    let path = create_temp_m4a("remove_cover");
    let image = write_temp_image("remove_cover", "png", PNG_BYTES);

    let options = Mp4EditOptions {
        title: Some("T"),
        cover: Some(Mp4Cover {
            path: &image,
            data: PNG_BYTES,
        }),
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();
    assert!(contains_bytes(&fs::read(&path).unwrap(), b"covr"));

    let mut tag = read_mp4_tag(&path).unwrap().unwrap();
    let removed = remove_tags_mp4(&mut tag, &["cover".to_string()]);
    assert!(removed);
    write_mp4_tag(&path, &tag).unwrap();

    assert!(
        !contains_bytes(&fs::read(&path).unwrap(), b"covr"),
        "el átomo `covr` debe desaparecer"
    );
    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert!(tag.pictures().is_empty());
    assert_eq!(
        tag.get_string(ItemKey::TrackTitle),
        Some("T"),
        "el resto de átomos debe sobrevivir"
    );

    cleanup_file(&path);
    cleanup_file(&image);
}

#[test]
fn test_m4a_lyrics_atom_has_no_language_prefix_and_keeps_text() {
    let path = create_temp_m4a("lyrics_lang");
    let lyrics = "Primera línea\nSegunda línea\nTercera línea";
    let options = Mp4EditOptions {
        lyrics: Some(lyrics),
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    // El `data` box de `©lyr` contiene EXACTAMENTE el texto: no hay campo ni
    // prefijo de idioma (a diferencia del `USLT` de MP3, que fija `spa`). El
    // tipo de dato 1 es UTF-8, no un idioma.
    let bytes = fs::read(&path).unwrap();
    let (data_type, payload) = data_box_after(&bytes, b"\xA9lyr").expect("debe existir `©lyr`");
    assert_eq!(data_type, 1, "el texto se guarda como UTF-8 (tipo 1)");
    assert_eq!(
        payload,
        lyrics.as_bytes(),
        "el `©lyr` no puede llevar prefijo de idioma"
    );

    cleanup_file(&path);
}

#[test]
fn test_is_year_only() {
    assert!(is_year_only("2026"));
    assert!(!is_year_only("2026-10-13"));
    assert!(!is_year_only("2026-10"));
    assert!(!is_year_only("26"));
    assert!(!is_year_only("abcd"));
    assert!(!is_year_only(""));
}

/// Devuelve el *payload* del átomo `mdat` (sin su cabecera), delimitado por su
/// tamaño declarado. Así la comparación no incluye los átomos que van después
/// de `mdat` (en este fixture, `moov`, que sí cambia al editar metadatos).
fn mdat_payload(bytes: &[u8]) -> Vec<u8> {
    let pos = bytes
        .windows(4)
        .position(|window| window == b"mdat")
        .expect("el archivo MP4 debe contener `mdat`");
    let size32 = u32::from_be_bytes(bytes[pos - 4..pos].try_into().unwrap());
    let payload_end = if size32 == 0 {
        bytes.len()
    } else if size32 == 1 {
        let size64 = u64::from_be_bytes(bytes[pos + 4..pos + 12].try_into().unwrap());
        (pos - 4) + size64 as usize
    } else {
        (pos - 4) + size32 as usize
    };
    let payload_start = if size32 == 1 { pos + 12 } else { pos + 4 };
    bytes[payload_start..payload_end].to_vec()
}

/// Extrae la lista entrecomillada de una clave que el cambio fija en UNA sola
/// línea dentro de `[package]` (p. ej. `keywords = ["id3", ...]`).
///
/// Sin dependencias de TOML: recorrido de líneas y recorte de comillas. Si la
/// clave no aparece o no se puede interpretar devuelve un `Err` con un mensaje
/// legible, para que el test falle por el contrato y no por un pánico opaco.
fn lista_manifiesto(manifiesto: &str, clave: &str) -> Result<Vec<String>, String> {
    let prefijo = format!("{clave} = [");
    let linea = manifiesto
        .lines()
        .map(str::trim)
        .find(|linea| linea.starts_with(&prefijo))
        .ok_or_else(|| {
            format!("`Cargo.toml` no declara `{clave}` en una línea `{clave} = [...]`")
        })?;

    let interior = linea
        .strip_prefix(&prefijo)
        .and_then(|resto| resto.strip_suffix(']'))
        .ok_or_else(|| {
            format!("la línea de `{clave}` debe ser `{clave} = [\"…\"]` en una sola línea: {linea}")
        })?;

    let mut elementos = Vec::new();
    for trozo in interior.split(',') {
        let trozo = trozo.trim();
        if trozo.is_empty() {
            continue; // permite la coma final
        }
        let valor = trozo
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .ok_or_else(|| {
                format!("el elemento `{trozo}` de `{clave}` debe ir entre comillas dobles")
            })?;
        elementos.push(valor.to_string());
    }
    Ok(elementos)
}

#[test]
fn test_manifiesto_keywords_validos() {
    let manifiesto = include_str!("../Cargo.toml");
    let keywords =
        lista_manifiesto(manifiesto, "keywords").unwrap_or_else(|error| panic!("{error}"));

    assert!(
        (1..=5).contains(&keywords.len()),
        "crates.io admite entre 1 y 5 keywords, pero `Cargo.toml` declara {}: {keywords:?}",
        keywords.len()
    );
    for keyword in &keywords {
        assert!(
            keyword.len() <= 20,
            "la keyword `{keyword}` supera los 20 caracteres que exige crates.io"
        );
        assert!(keyword.is_ascii(), "la keyword `{keyword}` debe ser ASCII");
        assert_eq!(
            keyword.to_ascii_lowercase(),
            *keyword,
            "la keyword `{keyword}` debe estar en minúsculas"
        );
        assert!(
            keyword
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-'),
            "la keyword `{keyword}` solo admite caracteres de `[a-z0-9_-]`"
        );
    }
    assert_eq!(
        keywords.as_slice(),
        ["id3", "mp3", "mp4", "metadata", "cli"],
        "el cambio fija estas keywords exactas en `Cargo.toml`"
    );
}

#[test]
fn test_manifiesto_categories_validas() {
    let manifiesto = include_str!("../Cargo.toml");
    let categories =
        lista_manifiesto(manifiesto, "categories").unwrap_or_else(|error| panic!("{error}"));

    assert!(
        !categories.is_empty(),
        "`categories` no puede estar vacío en `Cargo.toml`"
    );

    // Slugs verificados contra la taxonomía oficial de crates.io.
    const SLUGS_VALIDOS: [&str; 3] = ["command-line-utilities", "multimedia::audio", "filesystem"];
    for category in &categories {
        assert!(
            SLUGS_VALIDOS.contains(&category.as_str()),
            "la categoría `{category}` no pertenece a la taxonomía de crates.io: {SLUGS_VALIDOS:?}"
        );
    }
    assert_eq!(
        categories.as_slice(),
        ["command-line-utilities", "multimedia::audio", "filesystem"],
        "el cambio fija estas categorías exactas en `Cargo.toml`"
    );
}

#[test]
fn test_manifiesto_include_definido() {
    let manifiesto = include_str!("../Cargo.toml");

    // El bloque `[package]` abarca desde su cabecera hasta la siguiente sección
    // (la primera línea que empiece por `[`). Delimitar el fragmento es lo que
    // garantiza que una clave `include` declarada en otra sección no cuele.
    let mut bloque = String::new();
    let mut dentro = false;
    for linea in manifiesto.lines() {
        let recortada = linea.trim();
        if recortada == "[package]" {
            dentro = true;
            continue;
        }
        if dentro && recortada.starts_with('[') {
            break;
        }
        if dentro {
            bloque.push_str(linea);
            bloque.push('\n');
        }
    }
    assert!(
        !bloque.is_empty(),
        "`Cargo.toml` no declara el bloque `[package]`"
    );

    // Reconstruye el valor de `include`, admitiendo que la lista se reparta en
    // varias líneas, y extrae los patrones entrecomillados en orden.
    let mut valor = String::new();
    let mut encontrado = false;
    for linea in bloque.lines() {
        let recortada = linea.trim();
        if !encontrado {
            if let Some(resto) = recortada.strip_prefix("include") {
                let resto = resto.trim_start();
                if let Some(resto) = resto.strip_prefix('=') {
                    encontrado = true;
                    valor.push_str(resto);
                }
            }
        } else {
            valor.push(' ');
            valor.push_str(recortada);
        }
        if encontrado && valor.contains(']') {
            break;
        }
    }
    assert!(
        encontrado,
        "el bloque `[package]` de `Cargo.toml` no declara la clave `include` \
         con la lista blanca de ficheros publicables"
    );

    let interior = valor
        .split_once(']')
        .map(|(antes, _)| antes)
        .unwrap_or(valor.as_str());

    let mut patrones = Vec::new();
    for trozo in interior.split('"').skip(1).step_by(2) {
        patrones.push(trozo.to_string());
    }

    // Los patrones de primer nivel van anclados a la raíz con una barra inicial:
    // sin ella, la semántica de gitignore los hace casar a cualquier profundidad
    // y el paquete arrastraba un fichero ajeno del andamiaje del repositorio,
    // `openspec/changes/archive/2026-10-14-remove-empty-id3-header/README.md`.
    let esperados = [
        "/src/**",
        "/tests/**",
        "/Cargo.toml",
        "/Cargo.lock",
        "/README.md",
        "/LICENSE",
        "/CHANGELOG.md",
    ];

    for (indice, esperado) in esperados.iter().copied().enumerate() {
        match patrones.get(indice) {
            None => panic!(
                "`include` de `[package]` no declara {esperado:?} en la posición \
                 {indice}: {patrones:?}"
            ),
            Some(patron) if patron.as_str() == esperado => {}
            Some(patron) => panic!(
                "`include` de `[package]` tiene {patron:?} en la posición {indice}, \
                 pero se esperaba {esperado:?}: {patrones:?}"
            ),
        }
    }

    assert_eq!(
        patrones.len(),
        esperados.len(),
        "`include` declara {} patrones, pero se esperaban {}: {patrones:?}",
        patrones.len(),
        esperados.len()
    );
}

#[test]
fn test_mp4_metadata_only_file_reads_ok() {
    // Un archivo MP4 con `moov` pero sin `trak` (solo metadatos, sin pistas de
    // audio) debe leerse correctamente con `read_mp4_tag`, devolviendo `Ok(Some(tag))`
    // con los metadatos del `ilst`. Antes se rechazaba este caso, pero ahora
    // `read_mp4_tag` usa `ParseOptions::new().read_properties(false)` que no
    // exige pistas de audio.
    let path = std::env::temp_dir().join(format!(
        "id3cli_unit_mp4_no_trak_{}_{}.m4a",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        std::process::id()
    ));

    let bytes = build_mp4_without_trak();
    fs::write(&path, &bytes).expect("No se pudo escribir el MP4 de prueba");

    let result = read_mp4_tag(&path);
    let tag = result
        .expect("el MP4 sin pistas de audio debe leerse correctamente")
        .expect("debe contener el tag ilst con metadatos");
    assert_eq!(
        tag.get_string(ItemKey::TrackTitle),
        Some("Test"),
        "el título escrito en el ilst sintético debe leerse"
    );

    cleanup_file(&path);
}

#[test]
fn test_mp4_corrupt_file_rejected() {
    // Un archivo que empieza por `ftyp` pero es demasiado pequeño (solo el
    // encabezado `ftyp`) debe fallar con error porque `lofty` no puede
    // interpretarlo como MP4 válido.
    let path = std::env::temp_dir().join(format!(
        "id3cli_unit_mp4_corrupt_{}_{}.m4a",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        std::process::id()
    ));

    // Solo el encabezado ftyp (28 bytes), sin moov, sin nada más.
    let bytes = b"\x00\x00\x00\x1cftypM4A \x00\x00\x00\x00M4A mp42isom";
    fs::write(&path, bytes).expect("No se pudo escribir el MP4 corrupto de prueba");

    let result = read_mp4_tag(&path);
    assert!(
        result.is_err(),
        "se esperaba un error para MP4 corrupto (solo ftyp)"
    );

    cleanup_file(&path);
}

/// Construye un archivo MP4 artificial con `ftyp`, `moov` (sin `trak`) y `mdat`.
///
/// Estructura:
/// - `ftyp` (28 bytes): major brand "M4A ", compatible "M4A mp42isom"
/// - `moov` con `mvhd` + `udta` → `meta` → `ilst` → `©nam` ("Test")
/// - `mdat` (14 bytes): datos de relleno
///
/// El archivo no tiene ningún átomo `trak`, por lo que no es un archivo de
/// audio reproducible, aunque tenga metadatos.
fn build_mp4_without_trak() -> Vec<u8> {
    let mut buf = Vec::new();

    // ftyp atom: 8 header + 4 brand + 4 version + 12 compatible = 28 bytes
    buf.extend(b"\x00\x00\x00\x1cftypM4A \x00\x00\x00\x00M4A mp42isom");

    // --- moov body (sin trak) ---
    let mut body = Vec::new();

    // mvhd atom: 8 header + 100 padding = 108 bytes
    body.extend(b"\x00\x00\x00\x6c");
    body.extend(b"mvhd");
    body.extend(&[0u8; 100]);

    // data atom (dentro de ©nam): 8 header + 4 flags + 4 locale + 4 "Test" = 20 bytes
    let mut data_atom = Vec::new();
    data_atom.extend(b"\x00\x00\x00\x14"); // size = 20
    data_atom.extend(b"data");
    data_atom.extend(b"\x00\x00\x00\x01"); // type indicator = 1 (UTF-8 text)
    data_atom.extend(b"\x00\x00\x00\x00"); // locale
    data_atom.extend(b"Test"); // payload (4 bytes)

    // ©nam atom: 8 header + data_atom = 28 bytes
    let mut cnam = Vec::new();
    cnam.extend(b"\x00\x00\x00\x1c"); // size = 28
    cnam.extend(b"\xa9nam"); // type "©nam"
    cnam.extend(&data_atom);

    // ilst atom: 8 header + cnam = 36 bytes
    let mut ilst = Vec::new();
    ilst.extend(b"\x00\x00\x00\x24"); // size = 36
    ilst.extend(b"ilst");
    ilst.extend(&cnam);

    // meta atom (FullBox): 8 header + 4 version/flags + ilst = 48 bytes
    let mut meta = Vec::new();
    meta.extend(b"\x00\x00\x00\x30"); // size = 48
    meta.extend(b"meta");
    meta.extend(b"\x00\x00\x00\x00"); // version + flags
    meta.extend(&ilst);

    // udta atom: 8 header + meta = 56 bytes
    let mut udta = Vec::new();
    udta.extend(b"\x00\x00\x00\x38"); // size = 56
    udta.extend(b"udta");
    udta.extend(&meta);

    body.extend(&udta);

    // moov atom: 8 header + body
    let moov_size = 8u32 + body.len() as u32;
    buf.extend(moov_size.to_be_bytes());
    buf.extend(b"moov");
    buf.extend(&body);

    // mdat atom: 8 header + 6 bytes de relleno = 14 bytes
    buf.extend(b"\x00\x00\x00\x0e");
    buf.extend(b"mdat");
    buf.extend(b"\x00\x01\x02\x03\x04\x05");

    buf

// --- FIN tests MP4 ---

// --- INICIO: tests OGG (de la rama) ---

// ---------------------------------------------------------------------------
// Backend OGG (`VorbisComments`): mapeo, escritura, relectura, carátula y borrado
// ---------------------------------------------------------------------------
//
// Fase RED del grupo 4 del cambio `ogg-support-and-track-totals`: estos tests
// exigen el backend de OGG y todavía no compilan, porque las funciones y tipos
// de `src/lib.rs` que invocan aún no existen.

use lofty::file::AudioFile;
use lofty::ogg::OggPictureStorage;
use lofty::ogg::tag::VorbisComments;
use std::collections::HashSet;
use std::time::Duration;

/// Duración del audio de un `.ogg`, leída con `lofty`.
///
/// Sirve para comprobar que `remove --all` solo vacía el comentario y deja el
/// audio (y por tanto la duración) intacto, sin depender del layout de páginas.
fn ogg_duration(path: &Path) -> Duration {
    let mut file = fs::File::open(path).expect("No se pudo abrir el .ogg temporal");
    lofty::read_from(&mut file)
        .expect("lofty debe poder leer el .ogg")
        .properties()
        .duration()
}

#[test]
fn test_vorbis_comment_map_cubre_los_dieciocho_flags() {
    // La tabla es la única fuente del mapeo: 18 asignaciones flag → clave.
    assert_eq!(
        VORBIS_COMMENT_MAP.len(),
        18,
        "la tabla debe cubrir las 18 asignaciones"
    );

    // Ni flags ni claves repetidos.
    let flags: HashSet<&str> = VORBIS_COMMENT_MAP
        .iter()
        .map(|mapping| mapping.flag)
        .collect();
    let keys: HashSet<&str> = VORBIS_COMMENT_MAP
        .iter()
        .map(|mapping| mapping.key)
        .collect();
    assert_eq!(flags.len(), 18, "ningún flag de la tabla debe repetirse");
    assert_eq!(keys.len(), 18, "ninguna clave de la tabla debe repetirse");

    // Cada par exacto, en los dos sentidos.
    let expected: [(&str, &str); 18] = [
        ("--title", "TITLE"),
        ("--artist", "ARTIST"),
        ("--album", "ALBUM"),
        ("--album-artist", "ALBUMARTIST"),
        ("--year", "YEAR"),
        ("--date", "DATE"),
        ("--genre", "GENRE"),
        ("--track", "TRACKNUMBER"),
        ("--season", "DISCNUMBER"),
        ("--composer", "COMPOSER"),
        ("--copyright", "COPYRIGHT"),
        ("--lyrics", "LYRICS"),
        ("--cover", "METADATA_BLOCK_PICTURE"),
        ("--compilation", "COMPILATION"),
        ("--title-sort", "TITLESORT"),
        ("--artist-sort", "ARTISTSORT"),
        ("--album-sort", "ALBUMSORT"),
        ("--subtitle", "SUBTITLE"),
    ];
    for (flag, key) in expected {
        let mapping = VorbisCommentMapping::for_flag(flag)
            .unwrap_or_else(|| panic!("falta el flag {flag} en la tabla"));
        assert_eq!(mapping.key, key, "{flag} debe mapear a {key}");

        // El camino inverso clave → flag debe existir y coincidir.
        let reverse = VorbisCommentMapping::for_key(key)
            .unwrap_or_else(|| panic!("falta la clave {key} en la tabla"));
        assert_eq!(reverse.flag, flag, "{key} debe revertirse a {flag}");
    }

    // `--url` y `--original-artist` NO tienen equivalente en Vorbis.
    assert!(
        VorbisCommentMapping::for_flag("--url").is_none(),
        "--url no debe estar en la tabla"
    );
    assert!(
        VorbisCommentMapping::for_flag("--original-artist").is_none(),
        "--original-artist no debe estar en la tabla"
    );
    // Tampoco deben existir claves inventadas para ellos.
    assert!(VorbisCommentMapping::for_key("WEBSITE").is_none());
    assert!(VorbisCommentMapping::for_key("ORIGINALARTIST").is_none());

    // Todas las claves en mayúsculas y sin espacios.
    for mapping in VORBIS_COMMENT_MAP {
        assert_eq!(
            mapping.key,
            mapping.key.to_uppercase(),
            "la clave {} debe estar en mayúsculas",
            mapping.key
        );
        assert!(
            !mapping.key.contains(' '),
            "la clave {} no debe contener espacios",
            mapping.key
        );
    }
}

#[test]
fn test_apply_metadata_ogg_escribe_las_claves_del_mapeo() {
    let artists = vec!["Artista Uno".to_string(), "Artista Dos".to_string()];
    let options = OggEditOptions {
        title: Some("Título"),
        artists: &artists,
        album: Some("Álbum"),
        album_artist: Some("Artista del álbum"),
        year: Some(2026),
        genre: Some("Rock"),
        track: Some(NumberWithTotal::new(7, None)),
        season: Some(NumberWithTotal::new(2, None)),
        date: Some("2026-10-13"),
        copyright: Some("© 2026 Test Records"),
        composer: Some("Compositor"),
        subtitle: Some("Subtítulo"),
        compilation: true,
        title_sort: Some("Título, El"),
        artist_sort: Some("Artista, El"),
        album_sort: Some("Álbum, El"),
        ..Default::default()
    };

    let mut tag = VorbisComments::new();
    let report = apply_metadata_ogg(&mut tag, &options);

    assert!(report.changed);
    assert_eq!(tag.get("TITLE"), Some("Título"));
    // Los múltiples artistas se unen con `"; "`.
    assert_eq!(tag.get("ARTIST"), Some("Artista Uno; Artista Dos"));
    assert_eq!(tag.get("ALBUM"), Some("Álbum"));
    assert_eq!(tag.get("ALBUMARTIST"), Some("Artista del álbum"));
    assert_eq!(tag.get("GENRE"), Some("Rock"));
    assert_eq!(tag.get("TRACKNUMBER"), Some("7"));
    assert_eq!(tag.get("DISCNUMBER"), Some("2"));
    assert_eq!(tag.get("COMPOSER"), Some("Compositor"));
    assert_eq!(tag.get("COPYRIGHT"), Some("© 2026 Test Records"));
    assert_eq!(tag.get("SUBTITLE"), Some("Subtítulo"));
    // La compilación se escribe con el valor `"1"`.
    assert_eq!(tag.get("COMPILATION"), Some("1"));
    assert_eq!(tag.get("TITLESORT"), Some("Título, El"));
    assert_eq!(tag.get("ARTISTSORT"), Some("Artista, El"));
    assert_eq!(tag.get("ALBUMSORT"), Some("Álbum, El"));
    // `--year` y `--date` acaban en claves DISTINTAS, sin absorberse.
    assert_eq!(tag.get("YEAR"), Some("2026"));
    assert_eq!(tag.get("DATE"), Some("2026-10-13"));
}

#[test]
fn test_ogg_ida_y_vuelta_conserva_claves_ajenas() {
    let path = create_temp_file(
        "ogg_custom_field",
        "ogg",
        &fs::read(fixture_ogg_path()).expect("No se pudo leer el fixture Ogg"),
    );

    // Escribir una clave ajena al mapeo sobre la copia del fixture.
    let mut tag = read_vorbis_tag(&path).unwrap();
    tag.insert("MY_CUSTOM_FIELD".to_string(), "valor propio".to_string());
    write_vorbis_tag(&path, &tag).unwrap();

    // Editar el título con el backend de alto nivel.
    let options = OggEditOptions {
        title: Some("Título editado"),
        ..Default::default()
    };
    edit_ogg(&path, &options).unwrap();

    // Releer y comprobar la preservación exacta.
    let tag = read_vorbis_tag(&path).unwrap();
    assert_eq!(tag.get("TITLE"), Some("Título editado"));
    assert_eq!(
        tag.get("MY_CUSTOM_FIELD"),
        Some("valor propio"),
        "una clave ajena al mapeo debe sobrevivir a `edit`"
    );

    // `encoder` conserva su valor Y su caja original en minúsculas (lo que
    // justifica el tipo concreto `VorbisComments` frente al `Tag` genérico).
    assert_eq!(tag.get("encoder"), Some("Lavc libvorbis"));
    assert!(
        tag.items()
            .any(|(key, value)| key == "encoder" && value == "Lavc libvorbis"),
        "la clave `encoder` debe conservar su caja original en minúsculas"
    );

    // El vendor sigue siendo `ffmpeg`.
    assert_eq!(tag.vendor(), "ffmpeg");

    cleanup_file(&path);
}

#[test]
fn test_edit_ogg_sin_cambios_no_reescribe() {
    let path = create_temp_file(
        "ogg_noop",
        "ogg",
        &fs::read(fixture_ogg_path()).expect("No se pudo leer el fixture Ogg"),
    );
    let before = fs::read(&path).unwrap();

    let report = edit_ogg(&path, &OggEditOptions::default()).unwrap();
    assert!(!report.changed, "sin flags no debe haber ningún cambio");

    // El archivo debe quedar byte a byte idéntico.
    assert_eq!(
        fs::read(&path).unwrap(),
        before,
        "`edit` sin flags no debe reescribir el archivo"
    );

    cleanup_file(&path);
}

#[test]
fn test_ogg_omite_url_y_artista_original_con_aviso() {
    let path = create_temp_file(
        "ogg_omit",
        "ogg",
        &fs::read(fixture_ogg_path()).expect("No se pudo leer el fixture Ogg"),
    );

    // Claves presentes antes de editar, para exigir que no se añada ninguna.
    let before_keys: HashSet<String> = read_vorbis_tag(&path)
        .unwrap()
        .items()
        .map(|(key, _)| key.to_string())
        .collect();

    let options = OggEditOptions {
        title: Some("Con URL"),
        url: Some("https://ejemplo.test"),
        original_artist: Some("Artista Original"),
        ..Default::default()
    };
    let report = edit_ogg(&path, &options).unwrap();

    // El título sí se escribe, así que hubo cambio.
    assert!(report.changed, "el título indicado sí se escribe");
    // Un aviso por cada campo omitido, del helper correspondiente.
    assert_eq!(report.warnings.len(), 2, "un aviso por campo omitido");
    assert!(
        report
            .warnings
            .contains(&ogg_field_omitted_warning("--url")),
        "debe avisar de la omisión de --url: {:?}",
        report.warnings
    );
    assert!(
        report
            .warnings
            .contains(&ogg_field_omitted_warning("--original-artist")),
        "debe avisar de la omisión de --original-artist: {:?}",
        report.warnings
    );

    // Ninguno de los dos campos tiene clave en el mapeo.
    assert!(VorbisCommentMapping::for_key("WEBSITE").is_none());
    assert!(VorbisCommentMapping::for_key("ORIGINALARTIST").is_none());

    // Y el archivo no gana ninguna clave: solo `TITLE` además de las de antes.
    let after_keys: HashSet<String> = read_vorbis_tag(&path)
        .unwrap()
        .items()
        .map(|(key, _)| key.to_string())
        .collect();
    let mut expected = before_keys;
    expected.insert("TITLE".to_string());
    assert_eq!(
        after_keys, expected,
        "los campos omitidos no deben añadir ninguna clave al archivo"
    );

    cleanup_file(&path);
}

#[test]
fn test_ogg_caratula_en_metadata_block_picture() {
    for (extension, bytes, mime) in [
        ("jpg", JPEG_BYTES, "image/jpeg"),
        ("png", PNG_BYTES, "image/png"),
    ] {
        let path = create_temp_file(
            &format!("ogg_cover_{extension}"),
            "ogg",
            &fs::read(fixture_ogg_path()).expect("No se pudo leer el fixture Ogg"),
        );
        let image = write_temp_image(&format!("ogg_cover_{extension}"), extension, bytes);

        let options = OggEditOptions {
            cover: Some(OggCover {
                path: &image,
                data: bytes,
            }),
            ..Default::default()
        };
        let report = edit_ogg(&path, &options).unwrap();
        assert!(report.changed, "{extension} debe contar como cambio");

        // La clave `METADATA_BLOCK_PICTURE` existe físicamente en el archivo
        // (al releerla, `lofty` la interpreta como imagen, no como ítem).
        assert!(
            contains_bytes(&fs::read(&path).unwrap(), b"METADATA_BLOCK_PICTURE"),
            "{extension}: debe existir la clave de carátula"
        );

        let tag = read_vorbis_tag(&path).unwrap();
        assert_eq!(
            tag.pictures().len(),
            1,
            "{extension}: debe haber una imagen"
        );
        let picture = &tag.pictures()[0].0;
        assert_eq!(
            picture.mime_type().map(|m| m.as_str()),
            Some(mime),
            "{extension}: el MIME debe derivarse de la extensión"
        );
        assert_eq!(
            picture.pic_type(),
            PictureType::CoverFront,
            "{extension}: el tipo de carátula debe ser frontal"
        );

        cleanup_file(&path);
        cleanup_file(&image);
    }
}

#[test]
fn test_ogg_caratula_webp_sin_degradar() {
    let path = create_temp_file(
        "ogg_cover_webp",
        "ogg",
        &fs::read(fixture_ogg_path()).expect("No se pudo leer el fixture Ogg"),
    );
    let image = write_temp_image("ogg_cover_webp", "webp", WEBP_BYTES);

    let options = OggEditOptions {
        cover: Some(OggCover {
            path: &image,
            data: WEBP_BYTES,
        }),
        ..Default::default()
    };
    let report = edit_ogg(&path, &options).unwrap();
    assert!(report.changed, "la carátula WebP debe contar como cambio");
    // A diferencia de MP4, en OGG NO hay degradación de WebP ni aviso.
    assert!(
        report.warnings.is_empty(),
        "en OGG el WebP no se degrada y no debe emitir aviso: {:?}",
        report.warnings
    );

    let tag = read_vorbis_tag(&path).unwrap();
    assert_eq!(tag.pictures().len(), 1, "la carátula debe estar presente");
    assert_eq!(
        tag.pictures()[0].0.mime_type().map(|m| m.as_str()),
        Some("image/webp"),
        "el MIME debe ser el real de la extensión, sin degradar"
    );

    cleanup_file(&path);
    cleanup_file(&image);
}

#[test]
fn test_ogg_caratula_de_imagen_corta_no_aborta() {
    let path = create_temp_file(
        "ogg_cover_short",
        "ogg",
        &fs::read(fixture_ogg_path()).expect("No se pudo leer el fixture Ogg"),
    );
    // 3 bytes: por debajo del umbral de 8 con el que `lofty` falla al inferir
    // las dimensiones de la imagen.
    let short: &[u8] = &[0xFF, 0xD8, 0xFF];
    let image = write_temp_image("ogg_cover_short", "jpg", short);

    let options = OggEditOptions {
        cover: Some(OggCover {
            path: &image,
            data: short,
        }),
        ..Default::default()
    };
    let report = edit_ogg(&path, &options).expect("una imagen corta no debe abortar la edición");
    assert!(report.changed);

    let tag = read_vorbis_tag(&path).unwrap();
    assert_eq!(
        tag.pictures().len(),
        1,
        "la carátula corta debe quedar escrita"
    );
    assert_eq!(
        tag.pictures()[0].0.mime_type().map(|m| m.as_str()),
        Some("image/jpeg"),
        "el MIME se deriva de la extensión aunque las dimensiones no se puedan inferir"
    );

    cleanup_file(&path);
    cleanup_file(&image);
}

#[test]
fn test_ogg_letra_en_lyrics() {
    let path = create_temp_file(
        "ogg_lyrics",
        "ogg",
        &fs::read(fixture_ogg_path()).expect("No se pudo leer el fixture Ogg"),
    );
    let lyrics = "Primera línea\nSegunda línea\n\nCuarta con salto";

    let options = OggEditOptions {
        lyrics: Some(lyrics),
        ..Default::default()
    };
    edit_ogg(&path, &options).unwrap();

    let tag = read_vorbis_tag(&path).unwrap();
    assert_eq!(
        tag.get("LYRICS"),
        Some(lyrics),
        "la letra debe conservarse entera, incluidos los saltos de línea"
    );

    cleanup_file(&path);
}

#[test]
fn test_remove_tags_ogg_usa_los_alias_de_ingles_y_espanol() {
    // `remove title` y `remove título` eliminan `TITLE`, cada uno por su alias.
    for name in ["title", "título"] {
        let mut tag = VorbisComments::new();
        tag.insert("TITLE".to_string(), "T".to_string());
        assert!(
            remove_tags_ogg(&mut tag, &[name.to_string()]),
            "{name} debe reconocerse como alias de título"
        );
        assert!(tag.get("TITLE").is_none(), "{name} debe eliminar TITLE");
    }

    // `remove track` elimina `TRACKNUMBER` **y** su total `TRACKTOTAL`: borrar
    // el número de pista borra también el total para no dejar un total huérfano
    // (lo exige el contrato de `track-numbering`). El total se inserta a mano
    // porque su escritura por la vía de edición llega en el grupo 5 del change
    // `ogg-support-and-track-totals`; aquí solo se comprueba el borrado.
    let mut tag = VorbisComments::new();
    tag.insert("TRACKNUMBER".to_string(), "7".to_string());
    tag.insert("TRACKTOTAL".to_string(), "12".to_string());
    assert!(remove_tags_ogg(&mut tag, &["track".to_string()]));
    assert!(
        tag.get("TRACKNUMBER").is_none(),
        "`track` debe eliminar TRACKNUMBER"
    );
    assert!(
        tag.get("TRACKTOTAL").is_none(),
        "`track` debe eliminar también TRACKTOTAL: el total se va con su número"
    );

    // `remove season` (temporada) elimina `DISCNUMBER` **y** su total `DISCTOTAL`
    // por la misma razón.
    let mut tag = VorbisComments::new();
    tag.insert("DISCNUMBER".to_string(), "2".to_string());
    tag.insert("DISCTOTAL".to_string(), "3".to_string());
    assert!(remove_tags_ogg(&mut tag, &["season".to_string()]));
    assert!(
        tag.get("DISCNUMBER").is_none(),
        "`season` debe eliminar DISCNUMBER"
    );
    assert!(
        tag.get("DISCTOTAL").is_none(),
        "`season` debe eliminar también DISCTOTAL: el total se va con su número"
    );

    // `remove carátula` elimina la carátula escrita en `METADATA_BLOCK_PICTURE`.
    let path = create_temp_file(
        "ogg_remove_cover",
        "ogg",
        &fs::read(fixture_ogg_path()).expect("No se pudo leer el fixture Ogg"),
    );
    let image = write_temp_image("ogg_remove_cover", "jpg", JPEG_BYTES);
    let options = OggEditOptions {
        cover: Some(OggCover {
            path: &image,
            data: JPEG_BYTES,
        }),
        ..Default::default()
    };
    edit_ogg(&path, &options).unwrap();
    assert!(contains_bytes(
        &fs::read(&path).unwrap(),
        b"METADATA_BLOCK_PICTURE"
    ));

    let mut tag = read_vorbis_tag(&path).unwrap();
    assert!(remove_tags_ogg(&mut tag, &["carátula".to_string()]));
    write_vorbis_tag(&path, &tag).unwrap();

    assert!(
        !contains_bytes(&fs::read(&path).unwrap(), b"METADATA_BLOCK_PICTURE"),
        "`carátula` debe eliminar la clave METADATA_BLOCK_PICTURE"
    );
    assert!(read_vorbis_tag(&path).unwrap().pictures().is_empty());

    cleanup_file(&path);
    cleanup_file(&image);

    // Un nombre desconocido no cambia nada y no cuenta como eliminación.
    let mut tag = VorbisComments::new();
    tag.insert("TITLE".to_string(), "T".to_string());
    assert!(
        !remove_tags_ogg(&mut tag, &["inventado".to_string()]),
        "un nombre desconocido no debe reportar cambio"
    );
    assert_eq!(tag.get("TITLE"), Some("T"));
}

#[test]
fn test_remove_all_ogg_vacia_el_comentario_y_preserva_vendor_y_audio() {
    let path = create_temp_file(
        "ogg_remove_all",
        "ogg",
        &fs::read(fixture_ogg_path()).expect("No se pudo leer el fixture Ogg"),
    );
    let image = write_temp_image("ogg_remove_all", "png", PNG_BYTES);

    // Escribir campos de usuario y una carátula.
    let options = OggEditOptions {
        title: Some("T"),
        artists: &["A".to_string()],
        genre: Some("Rock"),
        cover: Some(OggCover {
            path: &image,
            data: PNG_BYTES,
        }),
        ..Default::default()
    };
    edit_ogg(&path, &options).unwrap();

    let duration_before = ogg_duration(&path);
    assert_eq!(read_vorbis_tag(&path).unwrap().get("TITLE"), Some("T"));
    assert_eq!(read_vorbis_tag(&path).unwrap().pictures().len(), 1);

    let removed = remove_all_ogg(&path).unwrap();
    assert!(removed, "había metadatos que eliminar");

    // El comentario sigue existiendo (es obligatorio), pero sin claves de usuario.
    let tag = read_vorbis_tag(&path).unwrap();
    assert!(tag.get("TITLE").is_none(), "TITLE debe desaparecer");
    assert!(tag.get("ARTIST").is_none(), "ARTIST debe desaparecer");
    assert!(tag.get("GENRE").is_none(), "GENRE debe desaparecer");
    assert!(tag.pictures().is_empty(), "no debe quedar ninguna carátula");
    assert!(
        !contains_bytes(&fs::read(&path).unwrap(), b"METADATA_BLOCK_PICTURE"),
        "la clave de carátula debe desaparecer del archivo"
    );

    // El vendor se conserva.
    assert_eq!(tag.vendor(), "ffmpeg", "el vendor debe conservarse");

    // El archivo sigue siendo un Ogg Vorbis válido y el audio no cambia.
    assert_eq!(detect_format(&path).unwrap(), AudioFormat::OggVorbis);
    assert_eq!(
        ogg_duration(&path),
        duration_before,
        "el audio debe quedar intacto: la duración no debe cambiar"
    );

    cleanup_file(&path);
    cleanup_file(&image);
}

#[test]
fn test_display_tags_ogg_no_muestra_vendor_ni_encoder() {
    // NOTA: `display_tags_ogg` escribe por stdout con `outln!`, que vuelca a la
    // salida estándar del proceso y no es capturable con el harness de
    // `cargo test` sin dependencias externas (los tests de `display_tags` y
    // `display_tags_mp4` tampoco la capturan y solo comprueban que no haya
    // pánico). Por eso aquí se verifica lo comprobable sin capturar stdout: que
    // el tag con solo vendor+encoder no expone ninguna clave del mapeo (nada que
    // mostrar al usuario) y que el tag con campos sí las expone.
    let mut only_vendor_encoder = VorbisComments::new();
    only_vendor_encoder.set_vendor("ffmpeg".to_string());
    only_vendor_encoder.insert("encoder".to_string(), "Lavc libvorbis".to_string());
    assert_eq!(only_vendor_encoder.vendor(), "ffmpeg");
    assert_eq!(only_vendor_encoder.get("encoder"), Some("Lavc libvorbis"));
    assert!(
        VORBIS_COMMENT_MAP
            .iter()
            .all(|mapping| only_vendor_encoder.get(mapping.key).is_none()),
        "un .ogg recién creado solo tiene vendor y encoder: ninguna clave del mapeo"
    );
    // No debe entrar en pánico (no hay nada que mostrar).
    display_tags_ogg(&only_vendor_encoder);

    let mut with_fields = VorbisComments::new();
    with_fields.set_vendor("ffmpeg".to_string());
    with_fields.insert("encoder".to_string(), "Lavc libvorbis".to_string());
    with_fields.insert("TITLE".to_string(), "T".to_string());
    with_fields.insert("ARTIST".to_string(), "A".to_string());
    with_fields.insert("ALBUM".to_string(), "Alb".to_string());
    // Los campos del mapeo sí están presentes y son los que `show` debe mostrar.
    assert_eq!(with_fields.get("TITLE"), Some("T"));
    assert_eq!(with_fields.get("ARTIST"), Some("A"));
    assert_eq!(with_fields.get("ALBUM"), Some("Alb"));
    display_tags_ogg(&with_fields);
}

/// Verifica la **serialización cruda** del comentario de Vorbis que escribe
/// `edit_ogg`, sin pasar por `lofty` ni por [`read_vorbis_tag`].
///
/// El resto de tests del backend releen con `read_vorbis_tag`, que es el lector
/// del propio CLI: si escritura y lectura compartieran un error de clave (por
/// ejemplo `TIT2` en lugar de `TITLE`), esos tests seguirían en verde. Aquí el
/// comentario se reparsea a mano sobre los bytes del fichero, de modo que la
/// verificación es entre dos caminos distintos y no una tautología: cambiar una
/// clave del mapeo rompe este test aunque la relectura del CLI no se enterase.
///
/// Formato **medido** (extracto real de `xxd` del fichero tras `edit_ogg`):
///
/// ```text
/// 00000070  6e 63 6f 64 65 72 3d 4c 61 76 63 20 6c 69 62 76  ncoder=Lavc libv
/// 00000080  6f 72 62 69 73 12 00 00 00 54 49 54 4c 45 3d 54  orbis....TITLE=T
/// 00000090  69 74 75 6c 6f 20 43 72 75 64 6f 0f 00 00 00 41  itulo Crudo....A
/// 000000a0  52 54 49 53 54 3d 55 6e 6f 3b 20 44 6f 73 11 00  RTIST=Uno; Dos..
/// ```
///
/// Tras la cabecera `\x03vorbis` y el vendor, cada campo es una longitud
/// little-endian de 4 bytes seguida del texto UTF-8 `CLAVE=valor`: el separador
/// es un **`=`** literal (`0x3D`), **no** un byte nulo. Por eso los patrones de
/// búsqueda en crudo son `b"TITLE=..."`, `b"ARTIST=..."`, etc., tal cual
/// aparecen en el volcado (no hay nulo entre la clave y el valor).
#[test]
fn test_ogg_claves_serializadas_en_el_comentario_crudo() {
    let path = create_temp_file(
        "ogg_crudo",
        "ogg",
        &fs::read(fixture_ogg_path()).expect("No se pudo leer el fixture Ogg"),
    );

    let artists = vec!["Uno".to_string(), "Dos".to_string()];
    let options = OggEditOptions {
        title: Some("Titulo Crudo"),
        artists: &artists,
        album: Some("Album Crudo"),
        track: Some(NumberWithTotal::new(7, None)),
        compilation: true,
        // Campos sin clave equivalente en Vorbis: se indican a propósito para
        // comprobar que su omisión con aviso no inventa ninguna clave en crudo.
        url: Some("https://ejemplo.test"),
        original_artist: Some("Original"),
        ..Default::default()
    };
    edit_ogg(&path, &options).unwrap();

    // Lectura cruda: el fichero entero como bytes, sin `lofty` ni
    // `read_vorbis_tag`.
    let bytes = fs::read(&path).unwrap();

    // Cada par se serializa como `CLAVE=valor` (con `=` literal, medido arriba).
    assert!(
        contains_bytes(&bytes, b"TITLE=Titulo Crudo"),
        "el comentario crudo debe contener `TITLE=Titulo Crudo`"
    );
    // Los múltiples artistas van unidos con el separador `"; "` exacto.
    assert!(
        contains_bytes(&bytes, b"ARTIST=Uno; Dos"),
        "el comentario crudo debe contener `ARTIST=Uno; Dos` (separador \"; \" exacto)"
    );
    assert!(
        contains_bytes(&bytes, b"ALBUM=Album Crudo"),
        "el comentario crudo debe contener `ALBUM=Album Crudo`"
    );
    assert!(
        contains_bytes(&bytes, b"TRACKNUMBER=7"),
        "el comentario crudo debe contener `TRACKNUMBER=7`"
    );
    // La compilación se serializa con el valor `"1"`.
    assert!(
        contains_bytes(&bytes, b"COMPILATION=1"),
        "el comentario crudo debe contener `COMPILATION=1`"
    );

    // La clave `encoder` escrita por `ffmpeg` sigue en el fichero crudo y con su
    // caja original en minúsculas: el tipo concreto `VorbisComments` no normaliza
    // la caja de las claves que no gestiona. Si el backend hubiera escrito
    // `ENCODER=...` o hubiera perdido la clave, esta comprobación fallaría.
    assert!(
        contains_bytes(&bytes, b"encoder=Lavc libvorbis"),
        "el comentario crudo debe conservar `encoder=Lavc libvorbis` en minúsculas"
    );

    // Los campos omitidos con aviso NO deben dejar clave alguna: ni una
    // inventada, ni una de reemplazo.
    assert!(
        !contains_bytes(&bytes, b"WEBSITE="),
        "`--url` no debe escribir ninguna clave en el comentario crudo"
    );
    assert!(
        !contains_bytes(&bytes, b"ORIGINALARTIST="),
        "`--original-artist` no debe escribir ninguna clave en el comentario crudo"
    );

    cleanup_file(&path);
}

// ---------------------------------------------------------------------------
// Totales de pista y temporada en los tres formatos (grupo 5)
//
// Fase RED del grupo 5 del cambio `ogg-support-and-track-totals`: estos tests
// exigen la forma `N/T` de `--track` y `--season` y todavía no compilan, porque
// el tipo `NumberWithTotal` y los campos `Option<NumberWithTotal>` de
// `apply_metadata`, `Mp4EditOptions` y `OggEditOptions` aún no existen. Nada de
// esto se implementa aquí: la implementación mínima llega en el GREEN del
// grupo 5.

use lofty::config::WriteOptions;

/// Devuelve el `payload` del `data` box del átomo MP4 indicado, exigiendo que exista.
///
/// Reutiliza [`data_box_after`] (que ya mide el tipo de dato real) y descarta el
/// tipo, porque aquí solo interesan los bytes del par número/total.
fn mp4_data_payload(bytes: &[u8], atom: &[u8]) -> Vec<u8> {
    let (_, payload) = data_box_after(bytes, atom)
        .unwrap_or_else(|| panic!("debe existir el átomo `{}`", String::from_utf8_lossy(atom)));
    payload
}

/// Extrae el par `(número, total)` del `payload` de 8 bytes de `trkn`/`disk`.
///
/// Disposición **medida** (volcado real del `data` de un `.m4a` escrito por
/// `lofty`): `[00 00][número u16 BE][total u16 BE][00 00]`.
fn mp4_number_pair(payload: &[u8]) -> (u16, u16) {
    assert_eq!(
        payload.len(),
        8,
        "el `data` de un par numérico MP4 debe tener 8 bytes"
    );
    assert_eq!(payload[0], 0, "byte 0 del par reservado a cero");
    assert_eq!(payload[1], 0, "byte 1 del par reservado a cero");
    assert_eq!(payload[6], 0, "byte 6 del par de relleno a cero");
    assert_eq!(payload[7], 0, "byte 7 del par de relleno a cero");
    (
        u16::from_be_bytes([payload[2], payload[3]]),
        u16::from_be_bytes([payload[4], payload[5]]),
    )
}

/// Construye un `Id3v2Tag` en memoria con el frame de texto indicado.
///
/// Sirve para fijar una precondición (un `TRCK`/`TPOS` con total) sin depender
/// de la escritura que se está probando.
fn id3v2_tag_with_text_frame(id: &'static str, value: &str) -> Id3v2Tag {
    let mut tag = Id3v2Tag::new();
    insert_text_frame(&mut tag, id, value);
    tag
}

#[test]
fn test_number_with_total_acepta_numero_solo() {
    let valor: NumberWithTotal = "7".parse().expect("`7` debe parsear como NumberWithTotal");
    assert_eq!(valor.number(), 7, "el número debe ser 7");
    assert_eq!(valor.total(), None, "sin barra no hay total");
    assert_eq!(
        valor.to_string(),
        "7",
        "la forma sin total se conserva sin barra"
    );

    // El constructor público y los accesores deben coincidir con el parseo.
    let construido = NumberWithTotal::new(7, None);
    assert_eq!(construido.number(), 7);
    assert_eq!(construido.total(), None);
    assert_eq!(construido.to_string(), "7");
}

#[test]
fn test_number_with_total_acepta_numero_y_total() {
    let valor: NumberWithTotal = "7/12".parse().expect("`7/12` debe parsear");
    assert_eq!(valor.number(), 7);
    assert_eq!(valor.total(), Some(12));
    assert_eq!(valor.to_string(), "7/12");

    // La temporada usa la misma forma `N/T`.
    let temporada: NumberWithTotal = "1/2".parse().expect("`1/2` debe parsear");
    assert_eq!(temporada.number(), 1);
    assert_eq!(temporada.total(), Some(2));
    assert_eq!(temporada.to_string(), "1/2");

    let construido = NumberWithTotal::new(7, Some(12));
    assert_eq!(construido.number(), 7);
    assert_eq!(construido.total(), Some(12));
    assert_eq!(construido.to_string(), "7/12");
}

#[test]
fn test_number_with_total_rechaza_formas_invalidas() {
    let invalidas = [
        "7/", "/12", "7/12/3", "7/x", "", "-1", " 7", "7 ", "7//12", "+7", "+7/+12",
    ];
    for entrada in invalidas {
        let resultado = entrada.parse::<NumberWithTotal>();
        let error = match resultado {
            Ok(valor) => panic!(
                "{entrada:?} debe rechazarse; se obtuvo número {} y total {:?}",
                valor.number(),
                valor.total()
            ),
            Err(error) => error,
        };
        // El mensaje, en español, nombra la forma admitida («7/12») y la
        // palabra «número» (cierre del grupo 5, tarea 5.2).
        assert!(
            error.contains("7/12"),
            "el mensaje de error para {entrada:?} debe nombrar la forma «7/12»: {error}"
        );
        assert!(
            error.contains("número"),
            "el mensaje de error para {entrada:?} debe contener la palabra `número`: {error}"
        );
    }

    // Nota de asimetría entre contenedores (constancia para el grupo 6, que la
    // documenta en el README): el cero es aceptable por contrato en los tres
    // formatos, pero solo se representa igual en MP3 y OGG. En MP3 (`TRCK`) y en
    // OGG (`TRACKNUMBER`) el `0` se escribe tal cual y se muestra al releer; en
    // MP4 el átomo `trkn`/`disk` lo descarta (y `show` no lo muestra) y el
    // backend avisa de la degradación. Es una asimetría legítima de contenedor,
    // no un fallo de este parser.
    // El contrato NO prohíbe el cero.
    let cero: NumberWithTotal = "0".parse().expect("`0` debe aceptarse");
    assert_eq!(cero.number(), 0);
    assert_eq!(cero.total(), None);

    let cero_con_total: NumberWithTotal = "0/0".parse().expect("`0/0` debe aceptarse");
    assert_eq!(cero_con_total.number(), 0);
    assert_eq!(cero_con_total.total(), Some(0));

    // Los ceros a la izquierda se aceptan y se normalizan (`007` → `7`): la
    // forma es «dígitos decimales», no una representación canónica.
    let con_ceros: NumberWithTotal = "007".parse().expect("`007` debe aceptarse");
    assert_eq!(
        con_ceros.number(),
        7,
        "los ceros a la izquierda se normalizan"
    );
    assert_eq!(con_ceros.total(), None);
    assert_eq!(
        con_ceros.to_string(),
        "7",
        "el Display muestra la forma normalizada"
    );
}

#[test]
fn test_mp3_escribe_total_de_pista() {
    let path = create_temp_mp3();

    let mut tag = Id3v2Tag::new();
    let changed = apply_metadata(
        &mut tag,
        None,
        &[],
        None,
        None,
        None,
        Some(NumberWithTotal::new(7, Some(12))),
        Some(NumberWithTotal::new(1, Some(2))),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    assert!(changed, "escribir pista y temporada debe marcar cambio");
    tag.save_to_path(&path, WriteOptions::default())
        .expect("lofty debe guardar el tag ID3v2 en el MP3");

    // Relectura con `id3` (librería distinta de `lofty`, que es quien escribe).
    let releido = Tag::read_from_path(&path).expect("`id3` debe releer el MP3");
    let trck = releido
        .get("TRCK")
        .and_then(|frame| frame.content().text())
        .expect("`id3` debe encontrar el frame TRCK");
    assert_eq!(trck, "7/12", "TRCK debe llevar el par número/total");
    assert_eq!(
        releido.total_tracks(),
        Some(12),
        "el total de pista debe releerse con `id3`"
    );

    let tpos = releido
        .get("TPOS")
        .and_then(|frame| frame.content().text())
        .expect("`id3` debe encontrar el frame TPOS");
    assert_eq!(tpos, "1/2", "TPOS debe llevar el par número/total");
    assert_eq!(
        releido.total_discs(),
        Some(2),
        "el total de temporada debe releerse con `id3`"
    );

    cleanup_file(&path);
}

#[test]
fn test_mp3_conserva_total_preexistente() {
    let mut tag = id3v2_tag_with_text_frame("TRCK", "3/12");

    apply_metadata(
        &mut tag,
        None,
        &[],
        None,
        None,
        None,
        Some(NumberWithTotal::new(5, None)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    assert_eq!(
        tag.get_text(&frame_id("TRCK")),
        Some("5/12"),
        "el total preexistente 12 debe sobrevivir al escribir la pista sin total"
    );
}

#[test]
fn test_mp3_sin_total_no_inventa_total() {
    let mut tag = Id3v2Tag::new();

    apply_metadata(
        &mut tag,
        None,
        &[],
        None,
        None,
        None,
        Some(NumberWithTotal::new(7, None)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    let trck = tag
        .get_text(&frame_id("TRCK"))
        .expect("debe existir el frame TRCK");
    assert_eq!(trck, "7", "TRCK debe ser exactamente `7`");
    assert!(
        !trck.contains('/'),
        "no debe aparecer barra ni total: {trck}"
    );
}

#[test]
fn test_mp4_escribe_total_de_pista() {
    let path = create_temp_m4a("track_total_write");
    let options = Mp4EditOptions {
        track: Some(NumberWithTotal::new(7, Some(12))),
        season: Some(NumberWithTotal::new(1, Some(2))),
        ..Default::default()
    };
    edit_mp4(&path, &options).expect("edit_mp4 debe escribir pista y temporada");

    let bytes = fs::read(&path).unwrap();

    // Disposición REAL medida del `data` de `trkn` (volcado del fichero):
    //   [00 00][número u16 BE][total u16 BE][00 00]  → 8 bytes.
    // Para `--track 7/12`: 00 00 00 07 00 0C 00 00.
    let trkn = mp4_data_payload(&bytes, b"trkn");
    assert_eq!(
        trkn,
        vec![0x00, 0x00, 0x00, 0x07, 0x00, 0x0C, 0x00, 0x00],
        "el `data` de `trkn` debe ser 00 00 00 07 00 0C 00 00 (número 7, total 12)"
    );
    assert_eq!(mp4_number_pair(&trkn), (7, 12));

    // Disposición REAL del `disk` para `--season 1/2`: 00 00 00 01 00 02 00 00.
    let disk = mp4_data_payload(&bytes, b"disk");
    assert_eq!(
        disk,
        vec![0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00],
        "el `data` de `disk` debe ser 00 00 00 01 00 02 00 00 (temporada 1, total 2)"
    );
    assert_eq!(mp4_number_pair(&disk), (1, 2));

    // Relectura con `lofty`.
    let tag = read_mp4_tag(&path).unwrap().expect("debe existir tag");
    assert_eq!(tag.track(), Some(7));
    assert_eq!(tag.track_total(), Some(12));
    assert_eq!(tag.disk(), Some(1));
    assert_eq!(tag.disk_total(), Some(2));

    cleanup_file(&path);
}

#[test]
fn test_mp4_conserva_total_preexistente() {
    let path = create_temp_m4a("track_total_keep");

    // Precondición: el átomo `trkn` ya está en 3/12.
    let primera = Mp4EditOptions {
        track: Some(NumberWithTotal::new(3, Some(12))),
        ..Default::default()
    };
    edit_mp4(&path, &primera).unwrap();

    // Editar solo el número, sin total.
    let segunda = Mp4EditOptions {
        track: Some(NumberWithTotal::new(5, None)),
        ..Default::default()
    };
    edit_mp4(&path, &segunda).unwrap();

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(tag.track(), Some(5), "el número debe actualizarse a 5");
    assert_eq!(
        tag.track_total(),
        Some(12),
        "el total preexistente 12 debe sobrevivir"
    );

    cleanup_file(&path);
}

#[test]
fn test_mp4_sin_total_no_inventa_total() {
    let path = create_temp_m4a("track_total_absent");
    let options = Mp4EditOptions {
        track: Some(NumberWithTotal::new(7, None)),
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    let bytes = fs::read(&path).unwrap();
    let trkn = mp4_data_payload(&bytes, b"trkn");
    let (number, total) = mp4_number_pair(&trkn);
    assert_eq!(number, 7, "el número debe ser 7");
    assert_eq!(total, 0, "no debe inventarse un total");
    assert_ne!(total, 12, "no debe colarse un 12 inventado");

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(tag.track(), Some(7));
    assert_eq!(tag.track_total(), None, "sin total no debe leerse ninguno");

    cleanup_file(&path);
}

#[test]
fn test_mp4_numero_no_representable_avisa_y_omite_el_par() {
    let path = create_temp_m4a("track_number_overflow");

    // `--track 70000/12`: el número 70000 no cabe en el átomo `trkn` (16 bits).
    // El backend debe avisar y NO escribir nada del par, para no dejar un total
    // huérfano.
    let options = Mp4EditOptions {
        track: Some(NumberWithTotal::new(70000, Some(12))),
        ..Default::default()
    };
    let report = edit_mp4(&path, &options).expect("edit_mp4 no debe fallar");

    assert!(
        !report.warnings.is_empty(),
        "el número no representable debe producir un aviso en el informe"
    );
    let aviso = report.warnings.join("\n");
    assert!(
        aviso.contains("70000"),
        "el aviso debe nombrar el valor que no cabe: {aviso}"
    );
    assert!(
        aviso.contains("MP4"),
        "el aviso debe explicar que MP4 no puede representarlo: {aviso}"
    );

    // Medido antes de escribir los asserts: no debe quedar átomo `trkn` (ni
    // número ni un total huérfano).
    let bytes = fs::read(&path).unwrap();
    assert!(
        data_box_after(&bytes, b"trkn").is_none(),
        "el número rechazado no debe dejar un átomo `trkn` (ni un total huérfano)"
    );

    // Relectura: no debe aparecer ni número ni total.
    if let Some(tag) = read_mp4_tag(&path).unwrap() {
        assert_eq!(tag.track(), None, "no debe releerse un número");
        assert_eq!(tag.track_total(), None, "no debe releerse un total");
    }

    cleanup_file(&path);
}

#[test]
fn test_mp4_total_no_representable_avisa_y_no_inventa_total() {
    let path = create_temp_m4a("track_total_overflow");

    // `--track 7/70000`: el número cabe, el total no. Debe escribirse el número
    // y omitirse el total, con aviso: nada de degradación silenciosa.
    let options = Mp4EditOptions {
        track: Some(NumberWithTotal::new(7, Some(70000))),
        ..Default::default()
    };
    let report = edit_mp4(&path, &options).expect("edit_mp4 no debe fallar");

    assert!(
        !report.warnings.is_empty(),
        "el total no representable debe producir un aviso en el informe"
    );
    let aviso = report.warnings.join("\n");
    assert!(
        aviso.contains("70000"),
        "el aviso debe nombrar el total que no cabe: {aviso}"
    );
    assert!(
        aviso.contains("MP4"),
        "el aviso debe explicar la limitación de MP4: {aviso}"
    );

    let bytes = fs::read(&path).unwrap();
    let trkn = mp4_data_payload(&bytes, b"trkn");
    assert_eq!(
        mp4_number_pair(&trkn),
        (7, 0),
        "el átomo debe llevar el número 7 y NO un total inventado"
    );

    let tag = read_mp4_tag(&path).unwrap().expect("debe existir tag");
    assert_eq!(tag.track(), Some(7));
    assert_eq!(
        tag.track_total(),
        None,
        "el total no representable no debe guardarse"
    );

    cleanup_file(&path);
}

#[test]
fn test_mp4_total_cero_avisa() {
    let path = create_temp_m4a("track_total_zero");

    // `--track 7/0`: el cero es aceptable por contrato, pero el átomo `trkn` no
    // lo distingue de «sin total»; se avisa y se omite en vez de degradarlo en
    // silencio.
    let options = Mp4EditOptions {
        track: Some(NumberWithTotal::new(7, Some(0))),
        ..Default::default()
    };
    let report = edit_mp4(&path, &options).expect("edit_mp4 no debe fallar");

    assert!(
        !report.warnings.is_empty(),
        "el total `0` en MP4 debe producir un aviso en el informe"
    );
    let aviso = report.warnings.join("\n");
    assert!(
        aviso.contains("(0)"),
        "el aviso debe nombrar el valor `0` que no cabe: {aviso}"
    );
    assert!(
        aviso.contains("MP4"),
        "el aviso debe explicar la limitación de MP4: {aviso}"
    );

    let bytes = fs::read(&path).unwrap();
    let trkn = mp4_data_payload(&bytes, b"trkn");
    assert_eq!(
        mp4_number_pair(&trkn),
        (7, 0),
        "el número 7 se escribe; el total `0` se omite"
    );

    let tag = read_mp4_tag(&path).unwrap().expect("debe existir tag");
    assert_eq!(tag.track(), Some(7));
    assert_eq!(tag.track_total(), None);

    cleanup_file(&path);
}

#[test]
fn test_ogg_escribe_total_de_pista() {
    let path = create_temp_file(
        "ogg_track_total",
        "ogg",
        &fs::read(fixture_ogg_path()).expect("No se pudo leer el fixture Ogg"),
    );

    let options = OggEditOptions {
        track: Some(NumberWithTotal::new(7, Some(12))),
        season: Some(NumberWithTotal::new(1, Some(2))),
        ..Default::default()
    };
    edit_ogg(&path, &options).unwrap();

    // Relectura con el lector del propio CLI.
    let tag = read_vorbis_tag(&path).unwrap();
    assert_eq!(tag.get("TRACKNUMBER"), Some("7"));
    assert_eq!(tag.get("TRACKTOTAL"), Some("12"));
    assert_eq!(tag.get("DISCNUMBER"), Some("1"));
    assert_eq!(tag.get("DISCTOTAL"), Some("2"));

    // Serialización cruda: cada par es `[longitud u32][CLAVE=valor]`.
    let bytes = fs::read(&path).unwrap();
    for par in [
        &b"TRACKNUMBER=7"[..],
        b"TRACKTOTAL=12",
        b"DISCNUMBER=1",
        b"DISCTOTAL=2",
    ] {
        assert!(
            contains_bytes(&bytes, par),
            "el comentario crudo debe contener {:?}",
            String::from_utf8_lossy(par)
        );
    }

    cleanup_file(&path);
}

#[test]
fn test_ogg_conserva_total_preexistente() {
    let path = create_temp_file(
        "ogg_track_total_keep",
        "ogg",
        &fs::read(fixture_ogg_path()).expect("No se pudo leer el fixture Ogg"),
    );

    // Precondición a mano: `TRACKTOTAL` ya a 12 (su escritura es lo que se
    // prueba en este grupo).
    let mut tag = read_vorbis_tag(&path).unwrap();
    tag.insert("TRACKNUMBER".to_string(), "3".to_string());
    tag.insert("TRACKTOTAL".to_string(), "12".to_string());
    write_vorbis_tag(&path, &tag).unwrap();

    let options = OggEditOptions {
        track: Some(NumberWithTotal::new(5, None)),
        ..Default::default()
    };
    edit_ogg(&path, &options).unwrap();

    let tag = read_vorbis_tag(&path).unwrap();
    assert_eq!(tag.get("TRACKNUMBER"), Some("5"), "el número debe ser 5");
    assert_eq!(
        tag.get("TRACKTOTAL"),
        Some("12"),
        "el total preexistente debe sobrevivir"
    );

    let bytes = fs::read(&path).unwrap();
    assert!(contains_bytes(&bytes, b"TRACKNUMBER=5"));
    assert!(contains_bytes(&bytes, b"TRACKTOTAL=12"));

    cleanup_file(&path);
}

#[test]
fn test_ogg_sin_total_no_inventa_total() {
    let path = create_temp_file(
        "ogg_track_total_absent",
        "ogg",
        &fs::read(fixture_ogg_path()).expect("No se pudo leer el fixture Ogg"),
    );

    let options = OggEditOptions {
        track: Some(NumberWithTotal::new(7, None)),
        ..Default::default()
    };
    edit_ogg(&path, &options).unwrap();

    let tag = read_vorbis_tag(&path).unwrap();
    assert_eq!(tag.get("TRACKNUMBER"), Some("7"));
    assert_eq!(
        tag.get("TRACKTOTAL"),
        None,
        "sin total no debe crearse TRACKTOTAL"
    );

    let bytes = fs::read(&path).unwrap();
    assert!(contains_bytes(&bytes, b"TRACKNUMBER=7"));
    assert!(
        !contains_bytes(&bytes, b"TRACKTOTAL"),
        "no debe aparecer TRACKTOTAL en el comentario crudo"
    );

    cleanup_file(&path);
}

#[test]
fn test_mp4_remove_track_elimina_el_atomo_completo() {
    let path = create_temp_m4a("remove_track_total");

    // Precondición: `trkn` 7/12 y `disk` 1/2, más un título que mantiene vivo el
    // bloque `ilst` al borrar los dos pares: sin ningún átomo, `lofty` elimina el
    // `ilst` entero (igual que el bloque ID3v2 vacío en MP3) y no habría tag que
    // releer.
    let options = Mp4EditOptions {
        title: Some("Pista"),
        track: Some(NumberWithTotal::new(7, Some(12))),
        season: Some(NumberWithTotal::new(1, Some(2))),
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    // Borrar la pista: número y total como una unidad.
    let mut tag = read_mp4_tag(&path).unwrap().unwrap();
    assert!(remove_tags_mp4(&mut tag, &["track".to_string()]));
    write_mp4_tag(&path, &tag).unwrap();

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(tag.track(), None, "el número de pista debe desaparecer");
    assert_eq!(
        tag.track_total(),
        None,
        "el total de pista no debe quedar huérfano"
    );
    let bytes = fs::read(&path).unwrap();
    assert!(
        !contains_bytes(&bytes, b"trkn"),
        "el átomo `trkn` completo debe desaparecer"
    );

    // Repetir con la temporada (`disk`).
    let mut tag = read_mp4_tag(&path).unwrap().unwrap();
    assert!(remove_tags_mp4(&mut tag, &["season".to_string()]));
    write_mp4_tag(&path, &tag).unwrap();

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(tag.disk(), None, "el número de temporada debe desaparecer");
    assert_eq!(
        tag.disk_total(),
        None,
        "el total de temporada no debe quedar"
    );
    let bytes = fs::read(&path).unwrap();
    assert!(
        !contains_bytes(&bytes, b"disk"),
        "el átomo `disk` completo debe desaparecer"
    );

    cleanup_file(&path);
}

#[test]
fn test_mp3_remove_track_elimina_el_total() {
    // `remove track` debe eliminar TRCK por completo (sin total huérfano).
    let mut tag = id3v2_tag_with_text_frame("TRCK", "7/12");
    assert!(remove_tags(&mut tag, &["track".to_string()]));
    assert!(
        tag.get(&frame_id("TRCK")).is_none(),
        "TRCK debe desaparecer por completo, sin total huérfano"
    );

    // `remove season` debe eliminar TPOS por completo.
    let mut tag = id3v2_tag_with_text_frame("TPOS", "1/2");
    assert!(remove_tags(&mut tag, &["season".to_string()]));
    assert!(
        tag.get(&frame_id("TPOS")).is_none(),
        "TPOS debe desaparecer por completo, sin total huérfano"
    );
}

// ---------------------------------------------------------------------------
// Coherencia entre la documentación y el código (grupo 6, tareas 6.2 y 6.3)
//
// Fase RED del grupo 6 del cambio `ogg-support-and-track-totals`: este test
// convierte en contrato verificable lo que el README y `Cargo.toml` deben decir
// una vez documentado el soporte de OGG y el total de pista y temporada. Hoy
// FALLA porque la documentación todavía describe solo MP3 y MP4.
// ---------------------------------------------------------------------------

/// Nivel de un encabezado markdown (`##` → 2), o `None` si la línea no lo es.
fn nivel_encabezado(linea: &str) -> Option<usize> {
    let recortada = linea.trim_start();
    if !recortada.starts_with('#') {
        return None;
    }
    let nivel = recortada.chars().take_while(|&c| c == '#').count();
    let resto = &recortada[nivel..];
    if resto.is_empty() || resto.starts_with(' ') {
        Some(nivel)
    } else {
        None
    }
}

/// Cuerpo de la primera sección cuyo encabezado satisface `predicado`.
///
/// La sección va desde la línea siguiente al encabezado hasta el próximo
/// encabezado de nivel **menor o igual** (o el final del documento), de modo
/// que una subsección `###` queda dentro de su `##` padre y las secciones
/// hermanas no se contaminan. `buscar` describe el encabezado esperado, para que
/// el fallo explique el contrato incumplido en vez de dar un pánico opaco.
fn seccion_markdown(
    markdown: &str,
    buscar: &str,
    predicado: impl Fn(&str) -> bool,
) -> Result<String, String> {
    let lineas: Vec<&str> = markdown.lines().collect();

    // Nivel de encabezado de cada línea, con `None` para lo que no lo es y para
    // las líneas dentro de un bloque de código ```: el README tiene comentarios
    // bash que empiezan por `#` (por ejemplo `# Eliminar todos los tags...`) y
    // no deben confundirse con encabezados.
    let mut niveles: Vec<Option<usize>> = Vec::with_capacity(lineas.len());
    let mut en_fence = false;
    for linea in &lineas {
        let recortada = linea.trim_start();
        if recortada.starts_with("```") {
            en_fence = !en_fence;
            niveles.push(None);
            continue;
        }
        niveles.push(if en_fence {
            None
        } else {
            nivel_encabezado(linea)
        });
    }

    let (indice, nivel) = niveles
        .iter()
        .enumerate()
        .find_map(|(i, n)| n.filter(|_| predicado(lineas[i])).map(|n| (i, n)))
        .ok_or_else(|| format!("no hay ninguna sección con encabezado que {buscar}"))?;

    let mut cuerpo = Vec::new();
    for i in indice + 1..lineas.len() {
        match niveles[i] {
            Some(nivel_hijo) if nivel_hijo <= nivel => break,
            _ => cuerpo.push(lineas[i]),
        }
    }
    Ok(cuerpo.join("\n"))
}

/// Cuenta las líneas cuyo `trim()` es exactamente `#[test]`.
///
/// Se cuenta el **fichero fuente** en vez de escribir la cifra a mano: así el
/// contrato de la documentación sigue vivo solo, sin que nadie tenga que
/// actualizar un número cada vez que crece la suite de tests.
fn contar_tests(fuente: &str) -> usize {
    fuente
        .lines()
        .filter(|linea| linea.trim() == "#[test]")
        .count()
}

/// ¿Aparece `palabra` como palabra completa en `linea` (sin distinguir caja)?
fn linea_contiene_palabra(linea: &str, palabra: &str) -> bool {
    linea
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .any(|token| token.eq_ignore_ascii_case(palabra))
}

/// Índice de la primera aparición de `etiqueta` en `linea`, sin distinguir caja.
fn posicion_ci(linea: &str, etiqueta: &str) -> Option<usize> {
    let aguja = etiqueta.to_lowercase();
    linea
        .char_indices()
        .find(|(i, _)| linea[*i..].to_lowercase().starts_with(&aguja))
        .map(|(i, _)| i)
}

/// Último entero que aparece en `texto`, o `None` si no hay ninguno.
fn ultimo_entero(texto: &str) -> Option<u64> {
    let bytes = texto.as_bytes();
    let mut i = texto.len();
    while i > 0 {
        i -= 1;
        if bytes[i].is_ascii_digit() {
            let fin = i + 1;
            let mut inicio = i;
            while inicio > 0 && bytes[inicio - 1].is_ascii_digit() {
                inicio -= 1;
            }
            return texto[inicio..fin].parse().ok();
        }
    }
    None
}

/// Último entero que aparece **antes** de la primera aparición de `etiqueta`.
fn cifra_antes_de(linea: &str, etiqueta: &str) -> Option<u64> {
    let pos = posicion_ci(linea, etiqueta)?;
    ultimo_entero(&linea[..pos])
}

/// Cifra del escudo de tests (`tests-<N>%20passing`) del README.
fn cifra_de_escudo(markdown: &str) -> Option<u64> {
    let marca = "tests-";
    let pos = markdown.find(marca)?;
    let digitos: String = markdown[pos + marca.len()..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    if digitos.is_empty() {
        None
    } else {
        digitos.parse().ok()
    }
}

#[test]
fn test_documentacion_describe_los_tres_formatos() {
    let readme = include_str!("../README.md");
    let manifiesto = include_str!("../Cargo.toml");

    // El recuento se MIDE del código fuente (ver `contar_tests`) en vez de
    // copiarse a mano; incluye el propio test que está corriendo.
    let tests_unitarios = contar_tests(include_str!("tests.rs"));
    let tests_integracion = contar_tests(include_str!("../tests/integration_test.rs"));
    let total_tests = tests_unitarios + tests_integracion;

    let mut fallos: Vec<String> = Vec::new();

    // (1) Los tres formatos soportados en `## Formatos admitidos`: la sección
    //     nombra MP3, MP4 y OGG; ninguna línea de «No soportado» menciona OGG; y
    //     esas líneas siguen listando Opus, Speex, FLAC, WAV, AIFF y APE.
    match seccion_markdown(readme, "contenga «Formatos admitidos»", |h| {
        h.contains("Formatos admitidos")
    }) {
        Err(error) => fallos.push(format!("[1] {error}")),
        Ok(seccion) => {
            for formato in ["MP3", "MP4", "OGG"] {
                if !seccion.contains(formato) {
                    fallos.push(format!(
                        "[1] la sección «Formatos admitidos» no nombra {formato}"
                    ));
                }
            }
            let no_soportados: Vec<&str> = seccion
                .lines()
                .filter(|linea| linea.contains("No soportado"))
                .collect();
            if no_soportados.is_empty() {
                fallos.push(
                    "[1] la sección «Formatos admitidos» no tiene ninguna línea de «No soportado»"
                        .to_string(),
                );
            }
            for linea in &no_soportados {
                if linea.contains("OGG") {
                    fallos.push(format!(
                        "[1] una línea de «No soportado» sigue tratando OGG como no soportado: {linea}"
                    ));
                }
            }
            let texto_no_soportados = no_soportados.join("\n");
            for pendiente in ["Opus", "Speex", "FLAC", "WAV", "AIFF", "APE"] {
                if !texto_no_soportados.contains(pendiente) {
                    fallos.push(format!(
                        "[1] las líneas de «No soportado» ya no mencionan {pendiente}"
                    ));
                }
            }
        }
    }

    // (2) Tres secciones de mapeo, una por formato, cada una con al menos una
    //     fila de tabla que nombra un flag del CLI.
    let secciones_mapeo = [
        (
            "MP3 (encabezado con `ID3v2`)",
            seccion_markdown(readme, "contenga «ID3v2»", |h| h.contains("ID3v2")),
        ),
        (
            "MP4 (encabezado con `Mapeo` y `MP4`)",
            seccion_markdown(readme, "contenga «Mapeo» y «MP4»", |h| {
                h.contains("Mapeo") && h.contains("MP4")
            }),
        ),
        (
            "OGG (encabezado con `Mapeo` y `OGG`)",
            seccion_markdown(readme, "contenga «Mapeo» y «OGG»", |h| {
                h.contains("Mapeo") && h.contains("OGG")
            }),
        ),
    ];
    for (etiqueta, resultado) in secciones_mapeo {
        match resultado {
            Err(error) => fallos.push(format!("[2] mapeo {etiqueta}: {error}")),
            Ok(seccion) => {
                let tiene_flag = seccion
                    .lines()
                    .any(|linea| linea.contains("--title") || linea.contains("--artist"));
                if !tiene_flag {
                    fallos.push(format!(
                        "[2] el mapeo {etiqueta} no tiene ninguna fila de tabla con `--title` o `--artist`"
                    ));
                }
            }
        }
    }

    // (3) Degradaciones por formato.
    match seccion_markdown(readme, "sea la sección de degradaciones de MP4", |h| {
        h.contains("MP4") && (h.contains("Limitacion") || h.contains("Degradac"))
    }) {
        Err(error) => fallos.push(format!("[3] {error}")),
        Ok(seccion) => {
            let minusculas = seccion.to_lowercase();
            if !minusculas.contains("webp") {
                fallos
                    .push("[3] la sección de degradaciones de MP4 no menciona `webp`".to_string());
            }
            if !(seccion.contains("©lyr") || minusculas.contains("letra")) {
                fallos.push(
                    "[3] la sección de degradaciones de MP4 no menciona `©lyr` ni `letra`"
                        .to_string(),
                );
            }
        }
    }
    // Criterio mínimo para OGG: alguna línea dice que en OGG el WebP NO se
    // degrada, es decir menciona `webp`, `OGG` y la palabra `no` a la vez. El
    // texto exacto es libre; si la redacción final lo repartiera en varias
    // líneas, se ajusta este criterio junto con el README (queda escrito aquí
    // a propósito).
    let ogg_no_degrada = readme.lines().any(|linea| {
        linea_contiene_palabra(linea, "webp")
            && linea_contiene_palabra(linea, "OGG")
            && linea_contiene_palabra(linea, "no")
    });
    if !ogg_no_degrada {
        fallos.push(
            "[3] el README no dice en ninguna línea que en OGG el WebP no se degrada \
             (`webp` + `OGG` + `no` en la misma línea)"
                .to_string(),
        );
    }

    // (4) Hoja de ruta: OGG como soportado y Opus, Speex y FLAC como pendientes.
    match seccion_markdown(readme, "contenga «Roadmap»", |h| h.contains("Roadmap")) {
        Err(error) => fallos.push(format!("[4] {error}")),
        Ok(seccion) => {
            let ogg_hecho = seccion.lines().any(|linea| {
                let recortada = linea.trim_start();
                recortada.starts_with("- [x]") && linea_contiene_palabra(recortada, "OGG")
            });
            if !ogg_hecho {
                fallos.push(
                    "[4] la hoja de ruta no marca OGG como soportado (falta una línea `- [x]` que lo mencione)"
                        .to_string(),
                );
            }
            for pendiente in ["Opus", "Speex", "FLAC"] {
                let sigue_pendiente = seccion.lines().any(|linea| {
                    let recortada = linea.trim_start();
                    recortada.starts_with("- [ ]") && linea_contiene_palabra(recortada, pendiente)
                });
                if !sigue_pendiente {
                    fallos.push(format!(
                        "[4] la hoja de ruta no mantiene {pendiente} como pendiente (falta una línea `- [ ]` que lo mencione)"
                    ));
                }
            }
        }
    }

    // (5) El recuento declarado coincide con el medido del código fuente.
    match cifra_de_escudo(readme) {
        Some(declarado) if declarado == total_tests as u64 => {}
        Some(declarado) => fallos.push(format!(
            "[5] el escudo del README declara {declarado} tests, pero la suite medida tiene \
             {total_tests} ({tests_unitarios} unitarios + {tests_integracion} de integración)"
        )),
        None => fallos.push("[5] el README no tiene un escudo `tests-<N>%20passing`".to_string()),
    }
    // La línea del desglose es la que nombra «unitario(s)» y «integración»; el
    // árbol de directorios del README también menciona «unitarios», pero sin
    // «integración», así que no se confunde con esta.
    let linea_desglose = readme.lines().find(|linea| {
        posicion_ci(linea, "unitario").is_some() && posicion_ci(linea, "integración").is_some()
    });
    match linea_desglose {
        None => fallos.push(
            "[5] el README no declara el desglose de tests (una línea con «unitario(s)» y «integración»)"
                .to_string(),
        ),
        Some(linea) => {
            let total_declarado = cifra_antes_de(linea, "tests");
            let unit_declarado = cifra_antes_de(linea, "unitario");
            let integ_declarado = cifra_antes_de(linea, "integración");
            if unit_declarado != Some(tests_unitarios as u64) {
                fallos.push(format!(
                    "[5] el README declara {unit_declarado:?} tests unitarios, pero la suite medida tiene {tests_unitarios}"
                ));
            }
            if integ_declarado != Some(tests_integracion as u64) {
                fallos.push(format!(
                    "[5] el README declara {integ_declarado:?} tests de integración, pero la suite medida tiene {tests_integracion}"
                ));
            }
            if total_declarado != Some(total_tests as u64) {
                fallos.push(format!(
                    "[5] el README declara {total_declarado:?} tests en total, pero la suite medida tiene {total_tests}"
                ));
            }
        }
    }

    // (6) Manifiesto: la `description` nombra los tres formatos y las `keywords`
    //     siguen siendo exactamente las cinco de hoy, en el mismo orden.
    match manifiesto
        .lines()
        .find(|linea| linea.trim_start().starts_with("description"))
    {
        None => fallos.push("[6] `Cargo.toml` no declara `description`".to_string()),
        Some(linea) => {
            for formato in ["MP3", "MP4", "OGG"] {
                if !linea.contains(formato) {
                    fallos.push(format!(
                        "[6] la `description` de `Cargo.toml` no menciona {formato}: {linea}"
                    ));
                }
            }
        }
    }
    match lista_manifiesto(manifiesto, "keywords") {
        Err(error) => fallos.push(format!("[6] {error}")),
        Ok(keywords) => {
            const ESPERADAS: [&str; 5] = ["id3", "mp3", "mp4", "metadata", "cli"];
            let declaradas: Vec<&str> = keywords.iter().map(String::as_str).collect();
            if declaradas != ESPERADAS {
                fallos.push(format!(
                    "[6] las `keywords` de `Cargo.toml` deben ser exactamente {ESPERADAS:?} en ese orden, pero declara {declaradas:?}"
                ));
            }
        }
    }

    assert!(
        fallos.is_empty(),
        "La documentación no cumple el contrato de los tres formatos (OGG) y el total de \
         pista/temporada. {} comprobación(es) fallida(s):\n- {}",
        fallos.len(),
        fallos.join("\n- ")
    );
}
