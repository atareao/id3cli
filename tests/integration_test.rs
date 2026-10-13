use id3::{Tag, TagLike};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Ruta absoluta al fixture MP3 real y sin tags, versionado en el repositorio.
///
/// El fixture contiene audio MPEG reproducible (0,5 s de tono a 32 kbps) y
/// **ningún** tag ID3 (ni v1 ni v2). Se usa en lugar de fabricar bytes falsos
/// para que el fichero sea también parseable por el futuro backend `lofty`.
fn fixture_mp3_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("sample.mp3")
}

/// Helper para crear un archivo MP3 temporal válido para testing.
///
/// Copia el fixture MP3 real (audio sin tags) a una ruta temporal única en
/// lugar de escribir bytes falsos. El archivo devuelto NO tiene ningún tag
/// ID3, de modo que `edit` ejercita la rama de creación de tags nuevos.
fn create_temp_mp3() -> PathBuf {
    let mp3_path = unique_temp_path("audio", "mp3");
    fs::copy(fixture_mp3_path(), &mp3_path).expect("Failed to copy MP3 fixture");
    mp3_path
}

/// Helper para limpiar archivos temporales
fn cleanup_file(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

/// Helper para generar una ruta temporal única SIN crear el archivo.
///
/// Se deriva del contador atómico y del timestamp para evitar colisiones
/// entre ejecuciones concurrentes o residuos de ejecuciones abortadas.
fn unique_temp_path(label: &str, extension: &str) -> PathBuf {
    let temp_dir = std::env::temp_dir();
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros();
    temp_dir.join(format!("test_{}_{}_{}.{}", label, timestamp, id, extension))
}

#[test]
fn test_cli_adds_title() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args([
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
        .args([
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
        .args([
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
    assert!(
        stderr.contains("Error: El archivo") && stderr.contains("nonexistent_file_12345.mp3"),
        "El mensaje de error no es el esperado, fue: {}",
        stderr
    );
}

#[test]
fn test_cli_with_unicode() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args([
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
        .args([
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
        .args([
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
        .args([
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
        .args([
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
        .args([
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
        .args(["run", "--quiet", "--", "show", mp3_path.to_str().unwrap()])
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
        .args(["run", "--quiet", "--", "show", mp3_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_track_number() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args([
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
        .args([
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
        .args([
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
        .args([
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
        .args([
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
        .args([
            "run",
            "--quiet",
            "--",
            "remove",
            mp3_path.to_str().unwrap(),
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
        .args([
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
        .args([
            "run",
            "--quiet",
            "--",
            "remove",
            mp3_path.to_str().unwrap(),
            "title",
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
fn test_cli_remove_all() {
    // MP3 de audio real y sin tag ID3 (el fixture): el tag añadido vía `edit`
    // será el único bloque ID3 del archivo.
    let mp3_path = create_temp_mp3();

    // Añadir tags
    Command::new("cargo")
        .args([
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

    // Eliminar todos con --all
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "remove",
            mp3_path.to_str().unwrap(),
            "--all",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("eliminado"),
        "La salida debería confirmar el borrado, pero fue: {}",
        stdout
    );

    // El tag ID3v2 debe haber desaparecido físicamente
    assert!(Tag::read_from_path(&mp3_path).is_err());

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_remove_all_removes_v1() {
    // MP3 de audio real y sin tag ID3 (el fixture): el tag ID3v1 será el
    // único bloque ID3 del archivo.
    let mp3_path = create_temp_mp3();

    // Añadir un tag ID3v1 (128 bytes al final, termina en "TAG")
    let mut bytes = fs::read(&mp3_path).unwrap();
    let mut v1 = vec![0u8; 128];
    v1[0..3].copy_from_slice(b"TAG");
    v1[3..8].copy_from_slice(b"Title");
    v1[33..39].copy_from_slice(b"Artist");
    bytes.append(&mut v1);
    fs::write(&mp3_path, &bytes).unwrap();

    // Eliminar todos con --all
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "remove",
            mp3_path.to_str().unwrap(),
            "--all",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    // El CLI debe informar de que el tag ID3v1 fue eliminado
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("ID3v1"),
        "La salida debería mencionar el tag ID3v1, pero fue: {}",
        stdout
    );

    // El tag ID3v1 debe haber desaparecido físicamente
    assert!(Tag::read_from_path(&mp3_path).is_err());
    let end = &fs::read(&mp3_path).unwrap();
    assert_ne!(
        &end[end.len().saturating_sub(3)..],
        b"TAG",
        "El ID3v1 no fue eliminado físicamente"
    );

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_remove_all_no_tags_warns() {
    // MP3 sin ningún tag ID3: el fixture de audio real no lleva tags.
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "remove",
            mp3_path.to_str().unwrap(),
            "--all",
        ])
        .output()
        .expect("Failed to execute command");

    // Debe tener éxito (exit 0) e informar de que no había tags
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("no tenía tags"),
        "Debería informar de que no había tags, pero fue: {}",
        stdout
    );

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_cover_png() {
    let mp3_path = create_temp_mp3();
    let cover_path = unique_temp_path("cover", "png");

    // Crear un archivo PNG mínimo
    let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    fs::write(&cover_path, png_data).expect("Failed to create PNG");

    let output = Command::new("cargo")
        .args([
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
    let cover_path = unique_temp_path("cover", "webp");

    // Crear un archivo WEBP mínimo
    let webp_data = b"RIFF\x00\x00\x00\x00WEBP".to_vec();
    fs::write(&cover_path, webp_data).expect("Failed to create WEBP");

    let output = Command::new("cargo")
        .args([
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
    let cover_path = unique_temp_path("cover", "tiff");

    // Crear un archivo TIFF (no soportado)
    fs::write(&cover_path, b"II*\x00").expect("Failed to create TIFF");

    let output = Command::new("cargo")
        .args([
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

    // Debe fallar con el mensaje exacto de formato no soportado para .tiff
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Formato de imagen no soportado: .tiff"),
        "stderr debería contener el mensaje de formato no soportado, fue: {}",
        stderr
    );

    cleanup_file(&mp3_path);
    cleanup_file(&cover_path);
}

#[test]
fn test_cli_add_lyrics() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args([
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
        .args([
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
        .args(["run", "--quiet", "--", "show", mp3_path.to_str().unwrap()])
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
        .args([
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
        .args([
            "run",
            "--quiet",
            "--",
            "remove",
            mp3_path.to_str().unwrap(),
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
        .args([
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
        if frame.id() == "WOAR"
            && let id3::frame::Content::Link(url) = frame.content()
        {
            assert_eq!(url, "https://example.com/artist");
            found_url = true;
            break;
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
        .args([
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
        .args(["run", "--quiet", "--", "show", mp3_path.to_str().unwrap()])
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
        .args([
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
        .args([
            "run",
            "--quiet",
            "--",
            "remove",
            mp3_path.to_str().unwrap(),
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
        .args([
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
        .args([
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
        .args([
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
        .args(["run", "--quiet", "--", "show", mp3_path.to_str().unwrap()])
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
        .args([
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
        .args([
            "run",
            "--quiet",
            "--",
            "remove",
            mp3_path.to_str().unwrap(),
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
        .args([
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
        .args([
            "run",
            "--quiet",
            "--",
            "remove",
            mp3_path.to_str().unwrap(),
            "album_sort",
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
        .args([
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
        .args([
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
        .args([
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
        .args([
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
        .args([
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
        .args([
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
        .args(["run", "--quiet", "--", "show", mp3_path.to_str().unwrap()])
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
        .args([
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
        .args([
            "run",
            "--quiet",
            "--",
            "remove",
            mp3_path.to_str().unwrap(),
            "composer",
            "subtitle",
            "original_artist",
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
        .args([
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
        .args([
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
        .args([
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
        .args([
            "run",
            "--quiet",
            "--",
            "remove",
            mp3_path.to_str().unwrap(),
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
        .args([
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

// ---------------------------------------------------------------------------
// Tests de caracterización (fase RED de TDD) — fijan el comportamiento ACTUAL
// del CLI antes de migrar el backend de `id3` a `lofty`.
// ---------------------------------------------------------------------------

#[test]
fn test_cli_writes_id3v24_header() {
    // Se parte del fixture de audio real SIN header ID3 (create_temp_mp3 ya
    // no inyecta un header vacío), de modo que la aserción no es
    // semi-tautológica: el test falla si `edit` no escribe. Además ejercita
    // la rama "Creando nuevos tags" de main.rs.
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "X",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // El tag debe escribirse siempre como ID3v2.4: header "ID3" + 0x04 0x00
    let bytes = fs::read(&mp3_path).expect("Failed to read file");
    assert!(
        bytes.len() >= 5,
        "El archivo es demasiado corto para contener un header ID3"
    );
    assert_eq!(
        &bytes[0..5],
        &[0x49, 0x44, 0x33, 0x04, 0x00],
        "El header no es ID3v2.4, fue: {:02X?}",
        &bytes[0..5]
    );

    // Y el título se escribió de verdad (si `edit` no escribiera, esto fallaría)
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.title(), Some("X"));

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_edit_without_flags_does_not_touch_file() {
    let mp3_path = create_temp_mp3();

    let before_bytes = fs::read(&mp3_path).expect("Failed to read file before");
    let before_mtime = fs::metadata(&mp3_path)
        .expect("Failed to stat file before")
        .modified()
        .expect("Failed to read mtime before");

    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "edit", mp3_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let after_bytes = fs::read(&mp3_path).expect("Failed to read file after");
    let after_mtime = fs::metadata(&mp3_path)
        .expect("Failed to stat file after")
        .modified()
        .expect("Failed to read mtime after");

    assert_eq!(
        before_bytes, after_bytes,
        "El contenido cambió aunque no se pasó ningún flag"
    );
    assert_eq!(
        before_mtime, after_mtime,
        "La fecha de modificación cambió aunque no se pasó ningún flag"
    );

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_edit_empty_title_writes() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Un título vacío cuenta como cambio: el frame TIT2 existe con valor vacío
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert!(
        tag.get("TIT2").is_some(),
        "El frame TIT2 debería existir aunque el título esté vacío"
    );
    assert_eq!(tag.title(), Some(""));

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_edit_preserves_existing_id3v1() {
    let mp3_path = create_temp_mp3();

    // Añadir un tag ID3v1 (128 bytes al final, empieza por "TAG")
    let mut v1 = vec![0u8; 128];
    v1[0..3].copy_from_slice(b"TAG");
    v1[3..8].copy_from_slice(b"Title");
    v1[33..39].copy_from_slice(b"Artist");
    let original_bytes = fs::read(&mp3_path).expect("Failed to read file");
    let mut bytes = original_bytes.clone();
    bytes.extend_from_slice(&v1);
    fs::write(&mp3_path, &bytes).expect("Failed to write file with ID3v1");

    // Editar vía ID3v2
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "Nuevo",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // El ID3v1 debe seguir presente e intacto al final del archivo
    let result = fs::read(&mp3_path).expect("Failed to read result");
    assert!(
        result.len() >= 128,
        "El archivo resultante perdió el bloque ID3v1"
    );
    let tail = &result[result.len() - 128..];
    assert_eq!(tail[0..3], *b"TAG", "El ID3v1 no empieza por TAG");
    assert_eq!(tail, v1.as_slice(), "El ID3v1 fue modificado");

    // Y además se escribió el ID3v2
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.title(), Some("Nuevo"));

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_year_writes_tyer() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--year",
            "2026",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(
        tag.get("TYER").and_then(|f| f.content().text()),
        Some("2026"),
        "--year debe escribir el frame TYER"
    );
    assert!(
        tag.get("TDRC").is_none(),
        "--year no debe escribir el frame TDRC"
    );

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_date_writes_tdrc() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--date",
            "2026-10-13",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(
        tag.get("TDRC").and_then(|f| f.content().text()),
        Some("2026-10-13"),
        "--date debe escribir el frame TDRC"
    );
    assert!(
        tag.get("TYER").is_none(),
        "--date no debe escribir el frame TYER"
    );

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_year_and_date_coexist() {
    let mp3_path = create_temp_mp3();

    // 1ª invocación: solo --year → TYER presente, TDRC ausente
    let output_year = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--year",
            "2026",
        ])
        .output()
        .expect("Failed to execute command");
    assert!(output_year.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(
        tag.get("TYER").and_then(|f| f.content().text()),
        Some("2026"),
        "--year debe escribir TYER"
    );
    assert!(tag.get("TDRC").is_none(), "--year no debe escribir TDRC");

    // 2ª invocación sobre el mismo fichero: solo --date.
    // El frame TYER ya escrito debe SOBREVIVIR a la reescritura.
    let output_date = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--date",
            "2026-10-13",
        ])
        .output()
        .expect("Failed to execute command");
    assert!(output_date.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(
        tag.get("TYER").and_then(|f| f.content().text()),
        Some("2026"),
        "TYER (frame previo) debe sobrevivir a una reescritura con --date"
    );
    assert_eq!(
        tag.get("TDRC").and_then(|f| f.content().text()),
        Some("2026-10-13"),
        "--date debe escribir TDRC"
    );

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_remove_without_tags_fails() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "remove", mp3_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(
        !output.status.success(),
        "remove sin tags y sin --all debería salir con código distinto de cero"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("al menos un tag"),
        "stderr debería indicar que hay que especificar al menos un tag, fue: {}",
        stderr
    );

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_cover_missing_file_fails() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--cover",
            "/ruta/que/no/existe.jpg",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(
        !output.status.success(),
        "edit con una carátula inexistente debería fallar"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("El archivo de carátula") && stderr.contains("no existe"),
        "stderr debería indicar que la carátula no existe, fue: {}",
        stderr
    );

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_remove_unknown_tag_warns_and_does_not_modify() {
    let mp3_path = create_temp_mp3();

    // Añadir un tag válido para tener algo legible
    Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "Keep",
        ])
        .output()
        .expect("Failed to execute command");

    let before_bytes = fs::read(&mp3_path).expect("Failed to read file before");

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "remove",
            mp3_path.to_str().unwrap(),
            "inventado",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Tags válidos") && stderr.contains("title, artist, album"),
        "stderr debería listar los tags válidos, fue: {}",
        stderr
    );

    let after_bytes = fs::read(&mp3_path).expect("Failed to read file after");
    assert_eq!(
        before_bytes, after_bytes,
        "El archivo no debe modificarse cuando solo se pasan tags desconocidos"
    );

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_remove_all_preserves_audio() {
    let mp3_path = create_temp_mp3();

    // El audio real del fixture es lo que debe sobrevivir byte a byte.
    let audio = fs::read(&mp3_path).expect("Failed to read fixture copy");

    // Añadir un tag ID3v2 (se antepone) y un ID3v1 (se anexa) para que
    // `remove --all` tenga ambos bloques de tags que eliminar alrededor del
    // audio.
    let mut tag = Tag::new();
    tag.set_title("Title");
    tag.write_to_path(&mp3_path, id3::Version::Id3v24)
        .expect("Failed to write ID3v2 tag");
    let mut bytes = fs::read(&mp3_path).expect("Failed to read file with ID3v2");
    let mut v1 = vec![0u8; 128];
    v1[0..3].copy_from_slice(b"TAG");
    v1[3..8].copy_from_slice(b"Title");
    bytes.extend_from_slice(&v1);
    fs::write(&mp3_path, &bytes).expect("Failed to write composed file");

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "remove",
            mp3_path.to_str().unwrap(),
            "--all",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Solo deben desaparecer los bloques de tags: el audio permanece byte a byte
    let result = fs::read(&mp3_path).expect("Failed to read result");
    assert_eq!(
        result, audio,
        "El audio debería permanecer intacto tras remove --all"
    );

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_edit_non_writable_file_fails() {
    use std::os::unix::fs::PermissionsExt;

    let mp3_path = create_temp_mp3();

    // Poner el archivo en solo lectura
    let mut perms = fs::metadata(&mp3_path)
        .expect("Failed to stat file")
        .permissions();
    perms.set_mode(0o444);
    fs::set_permissions(&mp3_path, perms).expect("Failed to set read-only permission");

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "X",
        ])
        .output()
        .expect("Failed to execute command");

    // Restaurar permisos para poder limpiar el archivo con seguridad
    let mut perms = fs::metadata(&mp3_path)
        .expect("Failed to stat file")
        .permissions();
    perms.set_mode(0o644);
    fs::set_permissions(&mp3_path, perms).expect("Failed to restore permission");

    if output.status.success() {
        // Si el proceso corre como root, la escritura en solo lectura tiene éxito
        // y este test no puede caracterizar el fallo de permisos. Terminamos sin
        // ninguna aserción para no ser frágiles.
        eprintln!(
            "AVISO: se pudo escribir en un archivo de solo lectura (¿proceso ejecutándose como root?); \
             se omite la caracterización del error de permisos."
        );
        cleanup_file(&mp3_path);
        return;
    }

    assert!(
        !output.status.success(),
        "edit sobre un archivo de solo lectura debería fallar"
    );

    // CARACTERIZACIÓN: al baseline el error de E/S propagado por la librería
    // `id3` sale en INGLÉS (`Permission denied`), no en español. El spec se
    // alineará con este texto real observado.
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Permission denied") || stderr.contains("denied"),
        "stderr debería contener el error real de permisos, fue: {}",
        stderr
    );

    cleanup_file(&mp3_path);
}

// ---------------------------------------------------------------------------
// Tests de caracterización adicionales (auditoría: cierre de agujeros).
// ---------------------------------------------------------------------------

/// Escribe un PNG mínimo válido en la ruta indicada.
fn write_minimal_png(path: &std::path::Path) {
    fs::write(path, [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
        .expect("Failed to create PNG");
}

#[test]
fn test_cli_remove_spanish_names_all_four() {
    let mp3_path = create_temp_mp3();
    let cover_path = unique_temp_path("cover", "png");
    write_minimal_png(&cover_path);

    // Título, artista, álbum (superviviente), título de orden y carátula
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "T",
            "--artist",
            "A",
            "--album",
            "Alb",
            "--title-sort",
            "TS",
            "--cover",
            cover_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");
    assert!(output.status.success());

    // Eliminar usando SOLO nombres en español
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "remove",
            mp3_path.to_str().unwrap(),
            "título",
            "artista",
            "carátula",
            "orden-titulo",
        ])
        .output()
        .expect("Failed to execute command");
    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.title(), None, "título debería eliminarse");
    assert_eq!(tag.artist(), None, "artista debería eliminarse");
    assert_eq!(
        tag.get("TSOT").and_then(|f| f.content().text()),
        None,
        "orden-titulo debería eliminar TSOT"
    );
    assert_eq!(tag.pictures().count(), 0, "carátula debería eliminarse");
    assert_eq!(
        tag.album(),
        Some("Alb"),
        "el álbum no mencionado debe sobrevivir"
    );

    cleanup_file(&mp3_path);
    cleanup_file(&cover_path);
}

#[test]
fn test_cli_rewrite_preserves_id3v24() {
    let mp3_path = create_temp_mp3();

    // Primera escritura
    let output1 = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "First",
        ])
        .output()
        .expect("Failed to execute command");
    assert!(output1.status.success());

    // Segunda escritura (campo distinto) sobre el mismo archivo
    let output2 = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--artist",
            "Second",
        ])
        .output()
        .expect("Failed to execute command");
    assert!(output2.status.success());

    // Tras la SEGUNDA escritura el header debe seguir siendo ID3v2.4
    let bytes = fs::read(&mp3_path).expect("Failed to read file");
    assert!(bytes.len() >= 5, "El archivo es demasiado corto");
    assert_eq!(
        &bytes[0..5],
        &[0x49, 0x44, 0x33, 0x04, 0x00],
        "El header no es ID3v2.4 tras reescribir, fue: {:02X?}",
        &bytes[0..5]
    );

    // Sanity check: ambos campos presentes
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.title(), Some("First"));
    assert_eq!(tag.artist(), Some("Second"));

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_lyrics_empty() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--lyrics",
            "",
        ])
        .output()
        .expect("Failed to execute command");
    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    let mut found = false;
    for frame in tag.frames() {
        if let id3::frame::Content::Lyrics(lyrics) = frame.content() {
            assert_eq!(lyrics.lang, "spa");
            assert_eq!(lyrics.text, "");
            found = true;
            break;
        }
    }
    assert!(found, "Debería existir un frame USLT con texto vacío");

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_remove_mixed_language_names() {
    let mp3_path = create_temp_mp3();

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "T",
            "--artist",
            "A",
            "--lyrics",
            "Línea",
        ])
        .output()
        .expect("Failed to execute command");
    assert!(output.status.success());

    // Mezcla de nombres en inglés y español en la misma invocación
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "remove",
            mp3_path.to_str().unwrap(),
            "title",
            "artista",
            "letra",
        ])
        .output()
        .expect("Failed to execute command");
    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.title(), None, "title debería eliminarse");
    assert_eq!(tag.artist(), None, "artista debería eliminarse");
    let has_lyrics = tag
        .frames()
        .any(|f| matches!(f.content(), id3::frame::Content::Lyrics(_)));
    assert!(!has_lyrics, "letra debería eliminar el frame USLT");

    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_remove_accent_folding_limit() {
    let mp3_path = create_temp_mp3();
    let cover_path = unique_temp_path("cover", "png");
    write_minimal_png(&cover_path);

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "T",
            "--genre",
            "Rock",
            "--cover",
            cover_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");
    assert!(output.status.success());

    // "titulo" (sin tilde) NO está en la lista de alias → el título NO se elimina
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "remove",
            mp3_path.to_str().unwrap(),
            "titulo",
        ])
        .output()
        .expect("Failed to execute command");
    assert!(output.status.success());
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(
        tag.title(),
        Some("T"),
        "la variante sin tilde 'titulo' no debe eliminar el título"
    );

    // "genero" (sin tilde) SÍ está en la lista de alias → elimina el género
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "remove",
            mp3_path.to_str().unwrap(),
            "genero",
        ])
        .output()
        .expect("Failed to execute command");
    assert!(output.status.success());
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(
        tag.genre(),
        None,
        "la variante 'genero' sí debe eliminar el género"
    );

    // "caratula" (sin tilde) SÍ está en la lista de alias → elimina la carátula
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "remove",
            mp3_path.to_str().unwrap(),
            "caratula",
        ])
        .output()
        .expect("Failed to execute command");
    assert!(output.status.success());
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(
        tag.pictures().count(),
        0,
        "la variante 'caratula' sí debe eliminar la carátula"
    );

    cleanup_file(&mp3_path);
    cleanup_file(&cover_path);
}

#[test]
fn test_cli_cover_preserves_existing_metadata() {
    let mp3_path = create_temp_mp3();
    let cover_path = unique_temp_path("cover", "png");
    write_minimal_png(&cover_path);

    // Metadatos previos: título, artista y letra
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--title",
            "T",
            "--artist",
            "A",
            "--lyrics",
            "Línea 1\nLínea 2",
        ])
        .output()
        .expect("Failed to execute command");
    assert!(output.status.success());

    // Añadir SOLO la carátula
    let output = Command::new("cargo")
        .args([
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
    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.title(), Some("T"), "el título debe conservarse");
    assert_eq!(tag.artist(), Some("A"), "el artista debe conservarse");
    let has_lyrics = tag
        .frames()
        .any(|f| matches!(f.content(), id3::frame::Content::Lyrics(_)));
    assert!(has_lyrics, "la letra debe conservarse");
    assert_eq!(tag.pictures().count(), 1, "la carátula debe añadirse");

    cleanup_file(&mp3_path);
    cleanup_file(&cover_path);
}

#[test]
fn test_cli_album_artist_separator_not_applied() {
    let mp3_path = create_temp_mp3();

    // Varios --artist (se unen con "; ") junto con un --album-artist
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "edit",
            mp3_path.to_str().unwrap(),
            "--artist",
            "Luis Fonsi",
            "--artist",
            "Daddy Yankee",
            "--album-artist",
            "Various Artists",
        ])
        .output()
        .expect("Failed to execute command");
    assert!(output.status.success());

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");

    // El separador "; " se aplica SOLO al frame del artista (TPE1)...
    assert_eq!(tag.artist(), Some("Luis Fonsi; Daddy Yankee"));
    assert_eq!(
        tag.get("TPE1").and_then(|f| f.content().text()),
        Some("Luis Fonsi; Daddy Yankee")
    );

    // ...mientras que el artista del álbum (TPE2) conserva el valor literal
    assert_eq!(tag.album_artist(), Some("Various Artists"));
    assert_eq!(
        tag.get("TPE2").and_then(|f| f.content().text()),
        Some("Various Artists")
    );

    cleanup_file(&mp3_path);
}
