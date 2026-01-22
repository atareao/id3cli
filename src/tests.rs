// Tests para la librería id3cli
use super::*;
use id3::{Tag, TagLike};
use id3::frame::{Content, PictureType};
use std::path::Path;

#[test]
fn test_apply_metadata_title_only() {
    let mut tag = Tag::new();
    let changed = apply_metadata(&mut tag, Some("Test Title"), &[], None, None, None, None, None, None, None, None, None, None);
    
    assert!(changed);
    assert_eq!(tag.title(), Some("Test Title"));
    assert_eq!(tag.artist(), None);
}

#[test]
fn test_apply_metadata_all_fields() {
    let mut tag = Tag::new();
    let artists = vec!["Artist".to_string()];
    let changed = apply_metadata(
        &mut tag,
        Some("Title"),
        &artists,
        Some("Album"),
        Some(2026),
        Some("Rock"),
        Some(5),
        Some("2026-01-22"),
        Some("© 2026 Test Records"),
        None,
        None,
        None,
        None,
    );
    
    assert!(changed);
    assert_eq!(tag.title(), Some("Title"));
    assert_eq!(tag.artist(), Some("Artist"));
    assert_eq!(tag.album(), Some("Album"));
    assert_eq!(tag.year(), Some(2026));
    assert_eq!(tag.genre(), Some("Rock"));
    assert_eq!(tag.track(), Some(5));
    assert_eq!(tag.date_recorded().map(|t| t.to_string()), Some("2026-01-22".to_string()));
    assert_eq!(tag.get("TCOP").and_then(|f| f.content().text()), Some("© 2026 Test Records"));
}

#[test]
fn test_apply_metadata_no_changes() {
    let mut tag = Tag::new();
    let changed = apply_metadata(&mut tag, None, &[], None, None, None, None, None, None, None, None, None, None);
    
    assert!(!changed);
}

#[test]
fn test_apply_metadata_partial() {
    let mut tag = Tag::new();
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
    );
    
    assert!(changed);
    assert_eq!(tag.title(), Some("Title"));
    assert_eq!(tag.artist(), None);
    assert_eq!(tag.album(), Some("Album"));
    assert_eq!(tag.year(), None);
}

#[test]
fn test_create_picture_frame() {
    let data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG magic bytes
    let picture = create_picture_frame(data.clone(), "image/jpeg");
    
    assert_eq!(picture.mime_type, "image/jpeg");
    assert_eq!(picture.picture_type, PictureType::CoverFront);
    assert_eq!(picture.description, "Cover");
    assert_eq!(picture.data, data);
}

#[test]
fn test_add_cover_art() {
    let mut tag = Tag::new();
    let data = vec![0xFF, 0xD8, 0xFF, 0xE0];
    let path = Path::new("test.jpg");
    
    let result = add_cover_art(&mut tag, path, data.clone());
    assert!(result.is_ok());
    
    let pictures: Vec<_> = tag.pictures().collect();
    assert_eq!(pictures.len(), 1);
    assert_eq!(pictures[0].data, data);
    assert_eq!(pictures[0].mime_type, "image/jpeg");
}

#[test]
fn test_apply_metadata_preserves_existing() {
    let mut tag = Tag::new();
    tag.set_title("Original Title");
    tag.set_artist("Original Artist");
    
    // Solo cambiar el álbum
    apply_metadata(&mut tag, None, &[], Some("New Album"), None, None, None, None, None, None, None, None, None);
    
    // El título y artista originales deben preservarse
    assert_eq!(tag.title(), Some("Original Title"));
    assert_eq!(tag.artist(), Some("Original Artist"));
    assert_eq!(tag.album(), Some("New Album"));
}

#[test]
fn test_apply_metadata_overwrites_existing() {
    let mut tag = Tag::new();
    tag.set_title("Old Title");
    
    apply_metadata(&mut tag, Some("New Title"), &[], None, None, None, None, None, None, None, None, None, None);
    
    assert_eq!(tag.title(), Some("New Title"));
}

#[test]
fn test_year_negative() {
    let mut tag = Tag::new();
    apply_metadata(&mut tag, None, &[], None, Some(-1), None, None, None, None, None, None, None, None);
    
    assert_eq!(tag.year(), Some(-1));
}

#[test]
fn test_year_future() {
    let mut tag = Tag::new();
    apply_metadata(&mut tag, None, &[], None, Some(3000), None, None, None, None, None, None, None, None);
    
    assert_eq!(tag.year(), Some(3000));
}

#[test]
fn test_empty_strings() {
    let mut tag = Tag::new();
    let artists = vec!["".to_string()];
    let changed = apply_metadata(&mut tag, Some(""), &artists, Some(""), None, Some(""), None, None, None, None, None, None, None);
    
    assert!(changed);
    assert_eq!(tag.title(), Some(""));
    assert_eq!(tag.artist(), Some(""));
}

#[test]
fn test_unicode_characters() {
    let mut tag = Tag::new();
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
    );
    
    assert_eq!(tag.title(), Some("Título con ñ y acentos"));
    assert_eq!(tag.artist(), Some("Artista 日本語"));
    assert_eq!(tag.album(), Some("Álbum 🎵"));
}

#[test]
fn test_multiple_artists() {
    let mut tag = Tag::new();
    let artists = vec!["Artist 1".to_string(), "Artist 2".to_string(), "Artist 3".to_string()];
    let changed = apply_metadata(&mut tag, None, &artists, None, None, None, None, None, None, None, None, None, None);
    
    assert!(changed);
    assert_eq!(tag.artist(), Some("Artist 1; Artist 2; Artist 3"));
}

#[test]
fn test_multiple_artists_unicode() {
    let mut tag = Tag::new();
    let artists = vec!["Artista Español".to_string(), "アーティスト".to_string()];
    apply_metadata(&mut tag, None, &artists, None, None, None, None, None, None, None, None, None, None);
    
    assert_eq!(tag.artist(), Some("Artista Español; アーティスト"));
}

#[test]
fn test_display_tags_empty() {
    let tag = Tag::new();
    // Este test solo verifica que display_tags no hace panic con un tag vacío
    display_tags(&tag);
}

#[test]
fn test_display_tags_with_data() {
    let mut tag = Tag::new();
    tag.set_title("Test Song");
    tag.set_artist("Test Artist");
    tag.set_album("Test Album");
    tag.set_year(2026);
    tag.set_genre("Rock");
    
    // Este test verifica que display_tags no hace panic con datos
    display_tags(&tag);
}

#[test]
fn test_track_number() {
    let mut tag = Tag::new();
    let changed = apply_metadata(&mut tag, None, &[], None, None, None, Some(7), None, None, None, None, None, None);
    
    assert!(changed);
    assert_eq!(tag.track(), Some(7));
}

#[test]
fn test_track_number_zero() {
    let mut tag = Tag::new();
    apply_metadata(&mut tag, None, &[], None, None, None, Some(0), None, None, None, None, None, None);
    
    assert_eq!(tag.track(), Some(0));
}

#[test]
fn test_track_with_other_metadata() {
    let mut tag = Tag::new();
    let artists = vec!["Artist".to_string()];
    apply_metadata(&mut tag, Some("Title"), &artists, None, None, None, Some(3), None, None, None, None, None, None);
    
    assert_eq!(tag.title(), Some("Title"));
    assert_eq!(tag.artist(), Some("Artist"));
    assert_eq!(tag.track(), Some(3));
}

#[test]
fn test_date_recorded() {
    let mut tag = Tag::new();
    let changed = apply_metadata(&mut tag, None, &[], None, None, None, None, Some("2026-01-22"), None, None, None, None, None);
    
    assert!(changed);
    assert_eq!(tag.date_recorded().map(|t| t.to_string()), Some("2026-01-22".to_string()));
}

#[test]
fn test_copyright() {
    let mut tag = Tag::new();
    let changed = apply_metadata(&mut tag, None, &[], None, None, None, None, None, Some("© 2026 Records"), None, None, None, None);
    
    assert!(changed);
    assert_eq!(tag.get("TCOP").and_then(|f| f.content().text()), Some("© 2026 Records"));
}

#[test]
fn test_date_and_copyright() {
    let mut tag = Tag::new();
    apply_metadata(&mut tag, None, &[], None, None, None, None, Some("2026"), Some("© Test"), None, None, None, None);
    
    assert_eq!(tag.date_recorded().map(|t| t.to_string()), Some("2026".to_string()));
    assert_eq!(tag.get("TCOP").and_then(|f| f.content().text()), Some("© Test"));
}

#[test]
fn test_remove_title() {
    let mut tag = Tag::new();
    tag.set_title("Test Title");
    assert_eq!(tag.title(), Some("Test Title"));
    
    let changed = remove_tags(&mut tag, &["title".to_string()]);
    assert!(changed);
    assert_eq!(tag.title(), None);
}

#[test]
fn test_remove_multiple_tags() {
    let mut tag = Tag::new();
    tag.set_title("Title");
    tag.set_artist("Artist");
    tag.set_album("Album");
    
    let changed = remove_tags(&mut tag, &["title".to_string(), "artist".to_string()]);
    assert!(changed);
    assert_eq!(tag.title(), None);
    assert_eq!(tag.artist(), None);
    assert_eq!(tag.album(), Some("Album")); // No eliminado
}

#[test]
fn test_remove_unknown_tag() {
    let mut tag = Tag::new();
    tag.set_title("Title");
    
    let changed = remove_tags(&mut tag, &["invalid_tag".to_string()]);
    assert!(!changed);
    assert_eq!(tag.title(), Some("Title")); // No afectado
}

#[test]
fn test_remove_tags_spanish() {
    let mut tag = Tag::new();
    tag.set_title("Título");
    tag.set_artist("Artista");
    
    let changed = remove_tags(&mut tag, &["título".to_string(), "artista".to_string()]);
    assert!(changed);
    assert_eq!(tag.title(), None);
    assert_eq!(tag.artist(), None);
}

#[test]
fn test_remove_cover() {
    let mut tag = Tag::new();
    let data = vec![0xFF, 0xD8, 0xFF, 0xE0];
    let path = Path::new("test.jpg");
    add_cover_art(&mut tag, path, data).unwrap();
    assert_eq!(tag.pictures().count(), 1);
    
    let changed = remove_tags(&mut tag, &["cover".to_string()]);
    assert!(changed);
    assert_eq!(tag.pictures().count(), 0);
}

#[test]
fn test_detect_mime_type_jpeg() {
    assert_eq!(detect_mime_type(Path::new("test.jpg")).unwrap(), "image/jpeg");
    assert_eq!(detect_mime_type(Path::new("test.jpeg")).unwrap(), "image/jpeg");
    assert_eq!(detect_mime_type(Path::new("test.JPG")).unwrap(), "image/jpeg");
}

#[test]
fn test_detect_mime_type_png() {
    assert_eq!(detect_mime_type(Path::new("test.png")).unwrap(), "image/png");
    assert_eq!(detect_mime_type(Path::new("test.PNG")).unwrap(), "image/png");
}

#[test]
fn test_detect_mime_type_webp() {
    assert_eq!(detect_mime_type(Path::new("test.webp")).unwrap(), "image/webp");
    assert_eq!(detect_mime_type(Path::new("test.WEBP")).unwrap(), "image/webp");
}

#[test]
fn test_detect_mime_type_unsupported() {
    let result = detect_mime_type(Path::new("test.gif"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("no soportado"));
}

#[test]
fn test_add_cover_art_png() {
    let mut tag = Tag::new();
    let data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG header
    let path = Path::new("cover.png");
    
    let result = add_cover_art(&mut tag, path, data.clone());
    assert!(result.is_ok());
    
    let pictures: Vec<_> = tag.pictures().collect();
    assert_eq!(pictures.len(), 1);
    assert_eq!(pictures[0].mime_type, "image/png");
}

#[test]
fn test_add_cover_art_webp() {
    let mut tag = Tag::new();
    let data = vec![0x52, 0x49, 0x46, 0x46]; // WEBP header
    let path = Path::new("cover.webp");
    
    let result = add_cover_art(&mut tag, path, data.clone());
    assert!(result.is_ok());
    
    let pictures: Vec<_> = tag.pictures().collect();
    assert_eq!(pictures.len(), 1);
    assert_eq!(pictures[0].mime_type, "image/webp");
}

#[test]
fn test_add_lyrics() {
    let mut tag = Tag::new();
    let lyrics_text = "Primera línea\nSegunda línea\nTercera línea";
    
    let result = add_lyrics(&mut tag, lyrics_text);
    assert!(result);
    
    // Verificar que se añadió el frame de lyrics
    let mut found_lyrics = false;
    for frame in tag.frames() {
        if let Content::Lyrics(lyrics) = frame.content() {
            assert_eq!(lyrics.lang, "spa");
            assert_eq!(lyrics.text, lyrics_text);
            found_lyrics = true;
            break;
        }
    }
    assert!(found_lyrics);
}

#[test]
fn test_remove_lyrics() {
    let mut tag = Tag::new();
    let lyrics_text = "Test lyrics";
    add_lyrics(&mut tag, lyrics_text);
    
    // Verificar que se añadió
    let has_lyrics = tag.frames().any(|f| matches!(f.content(), Content::Lyrics(_)));
    assert!(has_lyrics);
    
    // Eliminar lyrics
    let changed = remove_tags(&mut tag, &["lyrics".to_string()]);
    assert!(changed);
    
    // Verificar que se eliminó
    let has_lyrics = tag.frames().any(|f| matches!(f.content(), Content::Lyrics(_)));
    assert!(!has_lyrics);
}

#[test]
fn test_remove_lyrics_spanish() {
    let mut tag = Tag::new();
    add_lyrics(&mut tag, "Test");
    
    let changed = remove_tags(&mut tag, &["letra".to_string()]);
    assert!(changed);
    
    let has_lyrics = tag.frames().any(|f| matches!(f.content(), Content::Lyrics(_)));
    assert!(!has_lyrics);
}

#[test]
fn test_add_url() {
    let mut tag = Tag::new();
    let url = "https://example.com/artist";
    
    let result = add_url(&mut tag, url);
    assert!(result);
    
    // Verificar que se añadió el frame de URL
    let mut found_url = false;
    for frame in tag.frames() {
        if frame.id() == "WOAR" {
            if let Content::Link(link) = frame.content() {
                assert_eq!(link, url);
                found_url = true;
                break;
            }
        }
    }
    assert!(found_url);
}

#[test]
fn test_remove_url() {
    let mut tag = Tag::new();
    add_url(&mut tag, "https://test.com");
    
    // Verificar que se añadió
    let has_url = tag.frames().any(|f| f.id() == "WOAR");
    assert!(has_url);
    
    // Eliminar URL
    let changed = remove_tags(&mut tag, &["url".to_string()]);
    assert!(changed);
    
    // Verificar que se eliminó
    let has_url = tag.frames().any(|f| f.id() == "WOAR");
    assert!(!has_url);
}

#[test]
fn test_add_apple_metadata_compilation() {
    let mut tag = Tag::new();
    let changed = add_apple_metadata(&mut tag, true, None, None, None);
    
    assert!(changed);
    assert_eq!(tag.get("TCMP").and_then(|f| f.content().text()), Some("1"));
}

#[test]
fn test_add_apple_metadata_sort_orders() {
    let mut tag = Tag::new();
    let changed = add_apple_metadata(
        &mut tag,
        false,
        Some("Album Sort"),
        Some("Artist Sort"),
        Some("Title Sort")
    );
    
    assert!(changed);
    assert_eq!(tag.get("TSOA").and_then(|f| f.content().text()), Some("Album Sort"));
    assert_eq!(tag.get("TSOP").and_then(|f| f.content().text()), Some("Artist Sort"));
    assert_eq!(tag.get("TSOT").and_then(|f| f.content().text()), Some("Title Sort"));
}

#[test]
fn test_add_apple_metadata_all() {
    let mut tag = Tag::new();
    let changed = add_apple_metadata(
        &mut tag,
        true,
        Some("Sort Album"),
        Some("Sort Artist"),
        Some("Sort Title")
    );
    
    assert!(changed);
    assert_eq!(tag.get("TCMP").and_then(|f| f.content().text()), Some("1"));
    assert_eq!(tag.get("TSOA").and_then(|f| f.content().text()), Some("Sort Album"));
    assert_eq!(tag.get("TSOP").and_then(|f| f.content().text()), Some("Sort Artist"));
    assert_eq!(tag.get("TSOT").and_then(|f| f.content().text()), Some("Sort Title"));
}

#[test]
fn test_add_apple_metadata_no_changes() {
    let mut tag = Tag::new();
    let changed = add_apple_metadata(&mut tag, false, None, None, None);
    
    assert!(!changed);
}

#[test]
fn test_remove_compilation() {
    let mut tag = Tag::new();
    add_apple_metadata(&mut tag, true, None, None, None);
    
    let changed = remove_tags(&mut tag, &["compilation".to_string()]);
    assert!(changed);
    assert_eq!(tag.get("TCMP"), None);
}

#[test]
fn test_remove_apple_sort_tags() {
    let mut tag = Tag::new();
    add_apple_metadata(&mut tag, false, Some("A"), Some("B"), Some("C"));
    
    let changed = remove_tags(&mut tag, &[
        "album_sort".to_string(),
        "artist_sort".to_string(),
        "title_sort".to_string()
    ]);
    
    assert!(changed);
    assert_eq!(tag.get("TSOA"), None);
    assert_eq!(tag.get("TSOP"), None);
    assert_eq!(tag.get("TSOT"), None);
}

#[test]
fn test_remove_compilation_spanish() {
    let mut tag = Tag::new();
    add_apple_metadata(&mut tag, true, None, None, None);
    
    let changed = remove_tags(&mut tag, &["compilación".to_string()]);
    assert!(changed);
    assert_eq!(tag.get("TCMP"), None);
}

#[test]
fn test_remove_apple_sort_spanish() {
    let mut tag = Tag::new();
    add_apple_metadata(&mut tag, false, Some("A"), None, None);
    
    let changed = remove_tags(&mut tag, &["orden-album".to_string()]);
    assert!(changed);
    assert_eq!(tag.get("TSOA"), None);
}

#[test]
fn test_add_composer() {
    let mut tag = Tag::new();
    let changed = apply_metadata(&mut tag, None, &[], None, None, None, None, None, None, Some("John Lennon"), None, None, None);
    
    assert!(changed);
    assert_eq!(tag.get("TCOM").and_then(|f| f.content().text()), Some("John Lennon"));
}

#[test]
fn test_add_subtitle() {
    let mut tag = Tag::new();
    let changed = apply_metadata(&mut tag, None, &[], None, None, None, None, None, None, None, Some("Extended Version"), None, None);
    
    assert!(changed);
    assert_eq!(tag.get("TIT3").and_then(|f| f.content().text()), Some("Extended Version"));
}

#[test]
fn test_add_original_artist() {
    let mut tag = Tag::new();
    let changed = apply_metadata(&mut tag, None, &[], None, None, None, None, None, None, None, None, Some("The Beatles"), None);
    
    assert!(changed);
    assert_eq!(tag.get("TOPE").and_then(|f| f.content().text()), Some("The Beatles"));
}

#[test]
fn test_add_album_artist() {
    let mut tag = Tag::new();
    let changed = apply_metadata(&mut tag, None, &[], None, None, None, None, None, None, None, None, None, Some("Various Artists"));
    
    assert!(changed);
    assert_eq!(tag.album_artist(), Some("Various Artists"));
}

#[test]
fn test_add_all_new_tags() {
    let mut tag = Tag::new();
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
        Some("Compositor"),
        Some("Subtitle"),
        Some("Original"),
        Some("Album Artist")
    );
    
    assert!(changed);
    assert_eq!(tag.title(), Some("Title"));
    assert_eq!(tag.get("TCOM").and_then(|f| f.content().text()), Some("Compositor"));
    assert_eq!(tag.get("TIT3").and_then(|f| f.content().text()), Some("Subtitle"));
    assert_eq!(tag.get("TOPE").and_then(|f| f.content().text()), Some("Original"));
    assert_eq!(tag.album_artist(), Some("Album Artist"));
}

#[test]
fn test_remove_composer() {
    let mut tag = Tag::new();
    tag.set_text("TCOM", "Test Composer");
    
    let changed = remove_tags(&mut tag, &["composer".to_string()]);
    assert!(changed);
    assert_eq!(tag.get("TCOM"), None);
}

#[test]
fn test_remove_subtitle() {
    let mut tag = Tag::new();
    tag.set_text("TIT3", "Test Subtitle");
    
    let changed = remove_tags(&mut tag, &["subtitle".to_string()]);
    assert!(changed);
    assert_eq!(tag.get("TIT3"), None);
}

#[test]
fn test_remove_original_artist() {
    let mut tag = Tag::new();
    tag.set_text("TOPE", "Test Original");
    
    let changed = remove_tags(&mut tag, &["original_artist".to_string()]);
    assert!(changed);
    assert_eq!(tag.get("TOPE"), None);
}

#[test]
fn test_remove_album_artist() {
    let mut tag = Tag::new();
    tag.set_album_artist("Test Album Artist");
    
    let changed = remove_tags(&mut tag, &["album_artist".to_string()]);
    assert!(changed);
    assert_eq!(tag.album_artist(), None);
}
