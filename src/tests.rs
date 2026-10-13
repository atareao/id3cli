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
        Some(5),
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
        Some(7),
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
        Some(0),
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
        Some(3),
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
        Some(2),
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
        Some(5),
        Some(2),
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
    let magics: [&[u8]; 6] = [b"fLaC", b"OggS", b"RIFF", b"FORM", b"MAC ", b"wvpk"];
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
        track: Some(7),
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
    assert!(contains_bytes(&fs::read(&path).unwrap(), b"trkn"));
    cleanup_file(&path);
}

#[test]
fn test_m4a_writes_disk_atom_for_season() {
    let path = create_temp_m4a("season");
    let options = Mp4EditOptions {
        season: Some(2),
        ..Default::default()
    };
    edit_mp4(&path, &options).unwrap();

    let tag = read_mp4_tag(&path).unwrap().unwrap();
    assert_eq!(
        tag.disk(),
        Some(2),
        "el átomo `disk` debe guardar la temporada 2"
    );
    assert!(contains_bytes(&fs::read(&path).unwrap(), b"disk"));
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
