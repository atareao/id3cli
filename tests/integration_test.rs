use id3::{Tag, TagLike};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Helper para crear un archivo MP3 temporal válido para testing
fn create_temp_mp3() -> PathBuf {
    let temp_dir = std::env::temp_dir();
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros();
    let mp3_path = temp_dir.join(format!("test_{}_{}.mp3", timestamp, id));

    // Crear un archivo MP3 mínimo (solo tag ID3v2.4)
    // El crate id3 puede trabajar con archivos que solo tienen el tag
    let minimal_id3 = vec![
        0x49, 0x44, 0x33, 0x04, 0x00, 0x00, // ID3v2.4 header
        0x00, 0x00, 0x00, 0x00, // Size (0)
    ];
    fs::write(&mp3_path, minimal_id3).expect("Failed to create temp MP3");

    mp3_path
}

/// Helper para limpiar archivos temporales
fn cleanup_file(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

#[test]
fn test_cli_adds_title() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "Test Song",
        ])
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success());

    // Verificar que el tag fue guardado
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.title(), Some("Test Song"));

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_adds_multiple_fields() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "Song",
            "--artist",
            "Artist",
            "--album",
            "Album",
            "--year",
            "2026",
            "--genre",
            "Rock",
        ])
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.title(), Some("Song"));
    assert_eq!(tag.artist(), Some("Artist"));
    assert_eq!(tag.album(), Some("Album"));
    assert_eq!(tag.year(), Some(2026));
    assert_eq!(tag.genre(), Some("Rock"));

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_fails_with_nonexistent_file() {
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            "/tmp/nonexistent_file_12345.mp3",
            "--title",
            "Test",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("no existe") || stderr.contains("Error"));
}

#[test]
fn test_cli_with_unicode() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "Canción con ñ",
            "--artist",
            "Artista español",
        ])
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.title(), Some("Canción con ñ"));
    assert_eq!(tag.artist(), Some("Artista español"));

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_preserves_existing_tags() {
    let mp3_path = create_temp_mp3();

    // Primer comando: agregar título
    let output1 = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "Original",
        ])
        .output()
        .expect("Failed to execute command");

    if !output1.status.success() {
        eprintln!("First command failed:");
        eprintln!("stdout: {}", String::from_utf8_lossy(&output1.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output1.stderr));
    }
    assert!(output1.status.success());

    // Segundo comando: agregar solo artista
    let output2 = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--artist",
            "Artist",
        ])
        .output()
        .expect("Failed to execute command");

    if !output2.status.success() {
        eprintln!("Second command failed:");
        eprintln!("stdout: {}", String::from_utf8_lossy(&output2.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output2.stderr));
    }
    assert!(output2.status.success());

    // Verificar que ambos tags existen
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.title(), Some("Original"));
    assert_eq!(tag.artist(), Some("Artist"));

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_multiple_artists() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--artist",
            "Artist 1",
            "--artist",
            "Artist 2",
            "--artist",
            "Artist 3",
        ])
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.artist(), Some("Artist 1; Artist 2; Artist 3"));

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_multiple_artists_with_title() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "Collaboration Song",
            "--artist",
            "DJ Snake",
            "--artist",
            "Justin Bieber",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.title(), Some("Collaboration Song"));
    assert_eq!(tag.artist(), Some("DJ Snake; Justin Bieber"));

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_show_tags() {
    let mp3_path = create_temp_mp3();

    // Primero añadir algunos tags
    Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "Show Test",
            "--artist",
            "Test Artist",
            "--album",
            "Test Album",
            "--year",
            "2026",
        ])
        .output()
        .expect("Failed to execute command");

    // Ahora mostrar los tags
    let output = Command::new("cargo")
        .args(&["run", "--quiet", "--", "show", mp3_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Show Test"));
    assert!(stdout.contains("Test Artist"));
    assert!(stdout.contains("Test Album"));
    assert!(stdout.contains("2026"));

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_show_empty_tags() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args(&["run", "--quiet", "--", "show", mp3_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_track_number() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "Track Test",
            "--track",
            "5",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.title(), Some("Track Test"));
    assert_eq!(tag.track(), Some(5));

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_all_metadata_with_track() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "Complete Song",
            "--artist",
            "Complete Artist",
            "--album",
            "Complete Album",
            "--year",
            "2026",
            "--genre",
            "Pop",
            "--track",
            "3",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.title(), Some("Complete Song"));
    assert_eq!(tag.artist(), Some("Complete Artist"));
    assert_eq!(tag.album(), Some("Complete Album"));
    assert_eq!(tag.year(), Some(2026));
    assert_eq!(tag.genre(), Some("Pop"));
    assert_eq!(tag.track(), Some(3));

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_date_and_copyright() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "Copyright Test",
            "--date",
            "2026-01-22",
            "--copyright",
            "© 2026 Test Records",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.title(), Some("Copyright Test"));
    assert_eq!(
        tag.date_recorded().map(|t| t.to_string()),
        Some("2026-01-22".to_string())
    );
    assert_eq!(
        tag.get("TCOP").and_then(|f| f.content().text()),
        Some("© 2026 Test Records")
    );

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_complete_metadata() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "Complete",
            "--artist",
            "Artist One",
            "--artist",
            "Artist Two",
            "--album",
            "Album",
            "--year",
            "2026",
            "--genre",
            "Jazz",
            "--track",
            "7",
            "--date",
            "2026-01",
            "--copyright",
            "© All Rights Reserved",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.title(), Some("Complete"));
    assert_eq!(tag.artist(), Some("Artist One; Artist Two"));
    assert_eq!(tag.album(), Some("Album"));
    assert_eq!(tag.year(), Some(2026));
    assert_eq!(tag.genre(), Some("Jazz"));
    assert_eq!(tag.track(), Some(7));
    assert_eq!(
        tag.date_recorded().map(|t| t.to_string()),
        Some("2026-01".to_string())
    );
    assert_eq!(
        tag.get("TCOP").and_then(|f| f.content().text()),
        Some("© All Rights Reserved")
    );

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_remove_title() {
    let mp3_path = create_temp_mp3();

    // Primero añadir tags
    Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "Remove Me",
            "--artist",
            "Keep Me",
        ])
        .output()
        .expect("Failed to execute command");

    // Luego eliminar solo el título
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--remove",
            "title",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.title(), None);
    assert_eq!(tag.artist(), Some("Keep Me")); // Debe preservarse

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_remove_multiple_tags() {
    let mp3_path = create_temp_mp3();

    // Añadir varios tags
    Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "Title",
            "--artist",
            "Artist",
            "--album",
            "Album",
            "--year",
            "2026",
        ])
        .output()
        .expect("Failed to execute command");

    // Eliminar varios tags
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--remove",
            "title",
            "--remove",
            "artist",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.title(), None);
    assert_eq!(tag.artist(), None);
    assert_eq!(tag.album(), Some("Album")); // Debe preservarse
    assert_eq!(tag.year(), Some(2026)); // Debe preservarse

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_remove_all_tags() {
    let mp3_path = create_temp_mp3();

    // Añadir tags
    Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "Title",
            "--artist",
            "Artist",
            "--album",
            "Album",
            "--year",
            "2026",
            "--genre",
            "Rock",
            "--track",
            "5",
        ])
        .output()
        .expect("Failed to execute command");

    // Eliminar todos
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--remove",
            "title",
            "--remove",
            "artist",
            "--remove",
            "album",
            "--remove",
            "year",
            "--remove",
            "genre",
            "--remove",
            "track",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.title(), None);
    assert_eq!(tag.artist(), None);
    assert_eq!(tag.album(), None);
    assert_eq!(tag.year(), None);
    assert_eq!(tag.genre(), None);
    assert_eq!(tag.track(), None);

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_cover_png() {
    let mp3_path = create_temp_mp3();
    let temp_dir = std::env::temp_dir();
    let cover_path = temp_dir.join("test_cover.png");

    // Crear un archivo PNG mínimo
    let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    fs::write(&cover_path, png_data).expect("Failed to create PNG");

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--cover",
            cover_path.to_str().unwrap(),
            "--title",
            "PNG Cover Test",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    let pictures: Vec<_> = tag.pictures().collect();
    assert_eq!(pictures.len(), 1);
    assert_eq!(pictures[0].mime_type, "image/png");

    cleanup_file(&mp3_path);
    cleanup_file(&cover_path);
}

#[test]
fn test_cli_cover_webp() {
    let mp3_path = create_temp_mp3();
    let temp_dir = std::env::temp_dir();
    let cover_path = temp_dir.join("test_cover.webp");

    // Crear un archivo WEBP mínimo
    let webp_data = b"RIFF\x00\x00\x00\x00WEBP".to_vec();
    fs::write(&cover_path, webp_data).expect("Failed to create WEBP");

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--cover",
            cover_path.to_str().unwrap(),
            "--title",
            "WEBP Cover Test",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    let pictures: Vec<_> = tag.pictures().collect();
    assert_eq!(pictures.len(), 1);
    assert_eq!(pictures[0].mime_type, "image/webp");

    cleanup_file(&mp3_path);
    cleanup_file(&cover_path);
}

#[test]
fn test_cli_cover_unsupported_format() {
    let mp3_path = create_temp_mp3();
    let temp_dir = std::env::temp_dir();
    let cover_path = temp_dir.join("test_cover.gif");

    // Crear un archivo GIF (no soportado)
    fs::write(&cover_path, b"GIF89a").expect("Failed to create GIF");

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--cover",
            cover_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    // Debe fallar con formato no soportado
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("no soportado") || stderr.contains("gif"));

    cleanup_file(&mp3_path);
    cleanup_file(&cover_path);
}

#[test]
fn test_cli_add_lyrics() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "Song with Lyrics",
            "--lyrics",
            "Primera línea\nSegunda línea\nTercera línea",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.title(), Some("Song with Lyrics"));

    // Verificar que se añadió lyrics
    let mut found_lyrics = false;
    for frame in tag.frames() {
        if let id3::frame::Content::Lyrics(lyrics) = frame.content() {
            assert_eq!(lyrics.lang, "spa");
            assert!(lyrics.text.contains("Primera línea"));
            assert!(lyrics.text.contains("Tercera línea"));
            found_lyrics = true;
            break;
        }
    }
    assert!(found_lyrics);

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_show_lyrics() {
    let mp3_path = create_temp_mp3();

    // Añadir lyrics
    Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--lyrics",
            "Line 1\nLine 2\nLine 3\nLine 4\nLine 5",
        ])
        .output()
        .expect("Failed to execute command");

    // Mostrar tags
    let output = Command::new("cargo")
        .args(&["run", "--quiet", "--", "show", mp3_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Letra"));
    assert!(stdout.contains("Line 1"));
    assert!(stdout.contains("líneas más")); // Preview de solo 3 líneas

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_remove_lyrics() {
    let mp3_path = create_temp_mp3();

    // Añadir lyrics
    Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--lyrics",
            "Test lyrics",
        ])
        .output()
        .expect("Failed to execute command");

    // Verificar que se añadió
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    let has_lyrics = tag
        .frames()
        .any(|f| matches!(f.content(), id3::frame::Content::Lyrics(_)));
    assert!(has_lyrics);

    // Eliminar lyrics
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "-r",
            "lyrics",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Verificar que se eliminó
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    let has_lyrics = tag
        .frames()
        .any(|f| matches!(f.content(), id3::frame::Content::Lyrics(_)));
    assert!(!has_lyrics);

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_add_url() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "Song with URL",
            "--url",
            "https://example.com/artist",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.title(), Some("Song with URL"));

    // Verificar que se añadió URL
    let mut found_url = false;
    for frame in tag.frames() {
        if frame.id() == "WOAR" {
            if let id3::frame::Content::Link(url) = frame.content() {
                assert_eq!(url, "https://example.com/artist");
                found_url = true;
                break;
            }
        }
    }
    assert!(found_url);

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_show_url() {
    let mp3_path = create_temp_mp3();

    // Añadir URL
    Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--url",
            "https://myband.com",
        ])
        .output()
        .expect("Failed to execute command");

    // Mostrar tags
    let output = Command::new("cargo")
        .args(&["run", "--quiet", "--", "show", mp3_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("URL"));
    assert!(stdout.contains("https://myband.com"));

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_remove_url() {
    let mp3_path = create_temp_mp3();

    // Añadir URL
    Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--url",
            "https://test.com",
        ])
        .output()
        .expect("Failed to execute command");

    // Verificar que se añadió
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    let has_url = tag.frames().any(|f| f.id() == "WOAR");
    assert!(has_url);

    // Eliminar URL
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "-r",
            "url",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Verificar que se eliminó
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    let has_url = tag.frames().any(|f| f.id() == "WOAR");
    assert!(!has_url);

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_add_compilation() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--compilation",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Compilación"));

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.get("TCMP").and_then(|f| f.content().text()), Some("1"));

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_add_sort_orders() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--album-sort",
            "Sort Album",
            "--artist-sort",
            "Sort Artist",
            "--title-sort",
            "Sort Title",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(
        tag.get("TSOA").and_then(|f| f.content().text()),
        Some("Sort Album")
    );
    assert_eq!(
        tag.get("TSOP").and_then(|f| f.content().text()),
        Some("Sort Artist")
    );
    assert_eq!(
        tag.get("TSOT").and_then(|f| f.content().text()),
        Some("Sort Title")
    );

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_show_apple_metadata() {
    let mp3_path = create_temp_mp3();

    // Añadir metadatos de Apple
    Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--compilation",
            "--album-sort",
            "The Album",
        ])
        .output()
        .expect("Failed to execute command");

    // Mostrar tags
    let output = Command::new("cargo")
        .args(&["run", "--quiet", "--", "show", mp3_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Compilación"));
    assert!(stdout.contains("Orden álbum"));
    assert!(stdout.contains("The Album"));

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_remove_compilation() {
    let mp3_path = create_temp_mp3();

    // Añadir compilation
    Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--compilation",
        ])
        .output()
        .expect("Failed to execute command");

    // Verificar que se añadió
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert!(tag.get("TCMP").is_some());

    // Eliminar compilation
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "-r",
            "compilation",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Verificar que se eliminó
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert!(tag.get("TCMP").is_none());

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_remove_apple_sort_orders() {
    let mp3_path = create_temp_mp3();

    // Añadir sort orders
    Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--album-sort",
            "A",
            "--artist-sort",
            "B",
        ])
        .output()
        .expect("Failed to execute command");

    // Verificar que se añadieron
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert!(tag.get("TSOA").is_some());
    assert!(tag.get("TSOP").is_some());

    // Eliminar
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "-r",
            "album_sort",
            "-r",
            "artist_sort",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Verificar que se eliminaron
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert!(tag.get("TSOA").is_none());
    assert!(tag.get("TSOP").is_none());

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_add_composer() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--composer",
            "John Lennon",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(
        tag.get("TCOM").and_then(|f| f.content().text()),
        Some("John Lennon")
    );

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_add_subtitle() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--subtitle",
            "Extended Version",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(
        tag.get("TIT3").and_then(|f| f.content().text()),
        Some("Extended Version")
    );

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_add_original_artist() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--original-artist",
            "The Beatles",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(
        tag.get("TOPE").and_then(|f| f.content().text()),
        Some("The Beatles")
    );

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_add_album_artist() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--album-artist",
            "Various Artists",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.album_artist(), Some("Various Artists"));

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_podcast_metadata() {
    let mp3_path = create_temp_mp3();

    // Simular metadatos de un podcast
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "Episodio 42",
            "--subtitle",
            "Hablando de Rust",
            "--artist",
            "Lorenzo",
            "--album",
            "atareao con Linux",
            "--album-artist",
            "Lorenzo",
            "--composer",
            "Lorenzo",
            "--original-artist",
            "Lorenzo",
            "--genre",
            "Podcast",
            "--track",
            "42",
            "--date",
            "2026-01-22",
            "--copyright",
            "© 2026 CC BY 4.0",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.title(), Some("Episodio 42"));
    assert_eq!(
        tag.get("TIT3").and_then(|f| f.content().text()),
        Some("Hablando de Rust")
    );
    assert_eq!(tag.artist(), Some("Lorenzo"));
    assert_eq!(tag.album(), Some("atareao con Linux"));
    assert_eq!(tag.album_artist(), Some("Lorenzo"));
    assert_eq!(
        tag.get("TCOM").and_then(|f| f.content().text()),
        Some("Lorenzo")
    );
    assert_eq!(
        tag.get("TOPE").and_then(|f| f.content().text()),
        Some("Lorenzo")
    );
    assert_eq!(tag.genre(), Some("Podcast"));
    assert_eq!(tag.track(), Some(42));

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_show_new_tags() {
    let mp3_path = create_temp_mp3();

    // Añadir todas las etiquetas nuevas
    Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--composer",
            "Compositor",
            "--subtitle",
            "Subtítulo",
            "--original-artist",
            "Artista Original",
            "--album-artist",
            "Artista del Álbum",
        ])
        .output()
        .expect("Failed to execute command");

    // Mostrar tags
    let output = Command::new("cargo")
        .args(&["run", "--quiet", "--", "show", mp3_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Compositor"));
    assert!(stdout.contains("Subtítulo"));
    assert!(stdout.contains("Artista Original"));
    assert!(stdout.contains("Artista del Álbum"));

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_remove_new_tags() {
    let mp3_path = create_temp_mp3();

    // Añadir tags
    Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--composer",
            "Test",
            "--subtitle",
            "Test",
            "--original-artist",
            "Test",
            "--album-artist",
            "Test",
        ])
        .output()
        .expect("Failed to execute command");

    // Eliminar tags
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "-r",
            "composer",
            "-r",
            "subtitle",
            "-r",
            "original_artist",
            "-r",
            "album_artist",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Verificar eliminación
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert!(tag.get("TCOM").is_none());
    assert!(tag.get("TIT3").is_none());
    assert!(tag.get("TOPE").is_none());
    assert!(tag.album_artist().is_none());

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_add_season() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--season",
            "3",
        ])
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success());

    // Verificar que season fue guardado
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.disc(), Some(3));

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_podcast_with_season() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "El camino del héroe",
            "--artist",
            "Juan Pérez",
            "--album",
            "Mitología Clásica",
            "--track",
            "5",
            "--season",
            "2",
            "--genre",
            "Podcast",
            "--date",
            "2026-01-22",
            "--composer",
            "Juan Pérez",
            "--subtitle",
            "Episodio sobre mitología griega",
            "--copyright",
            "© 2026 Podcast Network",
        ])
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success());

    // Verificar todos los metadatos del podcast
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.title(), Some("El camino del héroe"));
    assert_eq!(tag.artist(), Some("Juan Pérez"));
    assert_eq!(tag.album(), Some("Mitología Clásica"));
    assert_eq!(tag.track(), Some(5));
    assert_eq!(tag.disc(), Some(2)); // Season
    assert_eq!(tag.genre(), Some("Podcast"));
    assert_eq!(
        tag.date_recorded().map(|t| t.to_string()),
        Some("2026-01-22".to_string())
    );
    assert_eq!(
        tag.get("TCOM").and_then(|f| f.content().text()),
        Some("Juan Pérez")
    );
    assert_eq!(
        tag.get("TIT3").and_then(|f| f.content().text()),
        Some("Episodio sobre mitología griega")
    );
    assert_eq!(
        tag.get("TCOP").and_then(|f| f.content().text()),
        Some("© 2026 Podcast Network")
    );

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_remove_season() {
    let mp3_path = create_temp_mp3();

    // Primero añadir season
    Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--season",
            "4",
        ])
        .output()
        .expect("Failed to execute command");

    // Verificar que se añadió
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.disc(), Some(4));

    // Ahora eliminar
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "-r",
            "season",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Verificar eliminación
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert!(tag.disc().is_none());

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_add_album() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--album",
            "My Test Album",
        ])
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success());

    // Verificar que el tag fue guardado
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.album(), Some("My Test Album"));

    cleanup_file(&mp3_path);
}
