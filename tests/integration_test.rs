use id3::{Tag, TagLike};
use lofty::file::TaggedFileExt;
use lofty::tag::{ItemKey, TagType};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
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

    // Caracterización del comportamiento previo (change de OGG + totales,
    // grupo 5): `--track N` escribe el frame `TRCK` con el número a secas,
    // SIN barra ni total. Se relee con `id3` (otra librería distinta de
    // `lofty`, que es quien escribe) para que la verificación sea entre
    // librerías. Cuando se introduzca la forma `N/T` esta expectativa cambiará.
    let track_text = tag
        .get("TRCK")
        .and_then(|frame| frame.content().text())
        .expect("`id3` debe encontrar el frame TRCK escrito por `lofty`");
    assert_eq!(track_text, "5", "`--track 5` debe guardar TRCK=\"5\"");
    assert!(
        !track_text.contains('/'),
        "TRCK no debe contener total (`/`): {track_text}"
    );

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

    // Verificar que se eliminó el último campo y, con él, el bloque ID3v2: el
    // archivo queda solo con audio, de modo que releerlo NO debe encontrar
    // ninguna etiqueta. `Tag::read_from_path` devuelve error cuando no hay tag.
    assert!(
        Tag::read_from_path(&mp3_path).is_err(),
        "tras eliminar el último campo el archivo no debe conservar ningún bloque ID3v2"
    );

    // El audio MUST quedar byte a byte intacto: tras añadir y luego eliminar el
    // único campo, el archivo debe volver a ser idéntico al fixture original
    // (audio sin ningún tag).
    let original = fs::read(fixture_mp3_path()).expect("Failed to read fixture");
    let resulting = fs::read(&mp3_path).expect("Failed to read resulting file");
    assert_eq!(
        resulting, original,
        "el audio debe quedar byte a byte intacto tras eliminar el último campo"
    );

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

    // Verificar que se eliminó el último campo y, con él, el bloque ID3v2: el
    // archivo queda solo con audio, de modo que releerlo NO debe encontrar
    // ninguna etiqueta. `Tag::read_from_path` devuelve error cuando no hay tag.
    assert!(
        Tag::read_from_path(&mp3_path).is_err(),
        "tras eliminar el último campo el archivo no debe conservar ningún bloque ID3v2"
    );

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

    // Verificar que se eliminó el último campo y, con él, el bloque ID3v2: el
    // archivo queda solo con audio, de modo que releerlo NO debe encontrar
    // ninguna etiqueta. `Tag::read_from_path` devuelve error cuando no hay tag.
    assert!(
        Tag::read_from_path(&mp3_path).is_err(),
        "tras eliminar el último campo el archivo no debe conservar ningún bloque ID3v2"
    );

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

    // Verificar que se eliminaron los últimos campos y, con ellos, el bloque
    // ID3v2: el archivo queda solo con audio, de modo que releerlo NO debe
    // encontrar ninguna etiqueta. `Tag::read_from_path` devuelve error cuando
    // no hay tag.
    assert!(
        Tag::read_from_path(&mp3_path).is_err(),
        "tras eliminar los últimos campos el archivo no debe conservar ningún bloque ID3v2"
    );

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

    // Verificar que se eliminaron los últimos campos y, con ellos, el bloque
    // ID3v2: el archivo queda solo con audio, de modo que releerlo NO debe
    // encontrar ninguna etiqueta. `Tag::read_from_path` devuelve error cuando
    // no hay tag.
    assert!(
        Tag::read_from_path(&mp3_path).is_err(),
        "tras eliminar los últimos campos el archivo no debe conservar ningún bloque ID3v2"
    );

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

    // Caracterización del comportamiento previo (change de OGG + totales,
    // grupo 5): `--season N` escribe el frame `TPOS` con el número a secas,
    // SIN barra ni total. Se relee con `id3` (diferente de `lofty`, que
    // escribe) para verificar entre librerías. Cuando se introduzca `N/T`
    // esta expectativa cambiará.
    let season_text = tag
        .get("TPOS")
        .and_then(|frame| frame.content().text())
        .expect("`id3` debe encontrar el frame TPOS escrito por `lofty`");
    assert_eq!(season_text, "3", "`--season 3` debe guardar TPOS=\"3\"");
    assert!(
        !season_text.contains('/'),
        "TPOS no debe contener total (`/`): {season_text}"
    );

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

    // Caracterización del comportamiento previo (change de OGG + totales,
    // grupo 5): ni `TRCK` ni `TPOS` llevan total. `--track 5` y `--season 2`
    // escriben el número a secas; se relee con `id3` (distinto de `lofty`,
    // que escribe) y se comprueba que no aparece la barra. Cambiará al
    // introducirse la forma `N/T`.
    let track_text = tag
        .get("TRCK")
        .and_then(|f| f.content().text())
        .expect("`id3` debe encontrar el frame TRCK");
    let season_text = tag
        .get("TPOS")
        .and_then(|f| f.content().text())
        .expect("`id3` debe encontrar el frame TPOS");
    assert_eq!(track_text, "5", "TRCK debe llevar el número a secas");
    assert_eq!(season_text, "2", "TPOS debe llevar el número a secas");
    assert!(
        !track_text.contains('/') && !season_text.contains('/'),
        "ni TRCK ni TPOS deben contener total (`/`): {track_text} / {season_text}"
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

    // Verificar que se eliminó el último campo y, con él, el bloque ID3v2: el
    // archivo queda solo con audio, de modo que releerlo NO debe encontrar
    // ninguna etiqueta. `Tag::read_from_path` devuelve error cuando no hay tag.
    assert!(
        Tag::read_from_path(&mp3_path).is_err(),
        "tras eliminar el último campo el archivo no debe conservar ningún bloque ID3v2"
    );

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

    // Se eliminaron los tres campos del tag (title, artist y lyrics), es decir
    // el último de ellos y, con él, el bloque ID3v2 completo: el archivo queda
    // solo con audio, de modo que releerlo NO debe encontrar ninguna etiqueta.
    // `Tag::read_from_path` devuelve error cuando no hay tag.
    assert!(
        Tag::read_from_path(&mp3_path).is_err(),
        "tras eliminar el último campo el archivo no debe conservar ningún bloque ID3v2"
    );

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

// ---------------------------------------------------------------------------
// Detección del formato por contenido (magic bytes)
// ---------------------------------------------------------------------------

/// Copia el fixture MP3 real a una ruta temporal con la extensión indicada.
///
/// Sirve para comprobar que el formato se decide por el contenido y no por la
/// extensión del nombre.
fn create_temp_fixture(extension: &str) -> PathBuf {
    let path = unique_temp_path("fixture", extension);
    fs::copy(fixture_mp3_path(), &path).expect("Failed to copy MP3 fixture");
    path
}

/// Escribe los bytes indicados (cabecera mágica incluida) en un temporal.
fn write_temp_bytes(label: &str, extension: &str, bytes: &[u8]) -> PathBuf {
    let path = unique_temp_path(label, extension);
    fs::write(&path, bytes).expect("Failed to write temp file");
    path
}

/// Contenedor MP4 **malformado** a propósito: declara `ftyp` en el offset 4
/// (por lo que se detecta como MP4/M4A) pero carece del átomo `moov`, de modo
/// que cualquier backend que lo intente interpretar debe fallar de forma limpia.
fn fake_mp4_bytes() -> Vec<u8> {
    let mut bytes = vec![0x00, 0x00, 0x00, 0x20];
    bytes.extend_from_slice(b"ftypM4A \x00\x00\x02\x00");
    bytes.extend_from_slice(b"mp42M4A ");
    bytes.extend_from_slice(&[0u8; 16]);
    bytes
}

#[test]
fn test_cli_mp3_renamed_to_m4a_is_edited_as_mp3() {
    // Un MP3 con extensión `.m4a` debe seguir editándose: la extensión no decide.
    let path = create_temp_fixture("m4a");

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "edit",
            path.to_str().unwrap(),
            "--title",
            "Renombrado",
        ])
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success());

    let tag = Tag::read_from_path(&path).expect("Failed to read tag");
    assert_eq!(tag.title(), Some("Renombrado"));

    cleanup_file(&path);
}

#[test]
fn test_cli_rejects_malformed_mp4_and_leaves_file_untouched() {
    // El backend MP4 (`.m4a`, `.mp4`, `.m4b`) ya está soportado, así que este
    // test NO comprueba que MP4 esté sin soportar. `fake_mp4_bytes` solo declara
    // el átomo `ftyp`: le falta `moov`, así que no es un contenedor válido. Un
    // MP4 malformado debe fallar en las tres operaciones con un error legible
    // (que enumera los formatos soportados) y sin tocar un solo byte del archivo.
    for (op, extra) in [
        ("show", Vec::<&str>::new()),
        ("edit", vec!["--title", "X"]),
        ("remove", vec!["--all"]),
    ] {
        let path = write_temp_bytes("clip_m4a", "m4a", &fake_mp4_bytes());
        let before = fs::read(&path).expect("Failed to read temp file");

        let mut args = vec!["run", "--quiet", "--", op, path.to_str().unwrap()];
        args.extend_from_slice(&extra);
        let output = Command::new("cargo")
            .args(&args)
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success(),
            "'{op}' sobre un MP4 malformado debe terminar con código distinto de cero (stderr: {stderr})"
        );
        assert!(
            stderr.contains("MP3") && stderr.contains("MP4") && stderr.contains("OGG"),
            "'{op}' debe enumerar los formatos soportados (MP3, MP4 y OGG) en stderr, fue: {stderr}"
        );
        assert_eq!(
            fs::read(&path).expect("Failed to read temp file"),
            before,
            "'{op}' NO debe modificar un MP4 malformado"
        );

        cleanup_file(&path);
    }
}

#[test]
fn test_cli_mp4_renamed_to_mp3_is_detected_by_content() {
    // La extensión del nombre NO decide el formato: el contenido sí. Un M4A
    // **válido** renombrado a `.mp3` debe detectarse por sus magic bytes como
    // MP4 y procesarse con el backend MP4, no como MP3 ni rechazarse.
    let path = unique_temp_path("clip_as_mp3", "mp3");
    fs::copy(fixture_m4a_path(), &path).expect("Failed to copy M4A fixture");

    // `edit` debe tener éxito: el enrutado llega al backend MP4 por contenido.
    let edit = run_cli(&[
        "edit",
        path.to_str().unwrap(),
        "--title",
        "Renombrado a mp3",
    ]);
    assert!(
        edit.status.success(),
        "un M4A con nombre .mp3 debe editarse por su contenido, stderr: {}",
        String::from_utf8_lossy(&edit.stderr)
    );

    // El título se relee a través del propio CLI, que vuelve a decidir por
    // contenido pese a la extensión `.mp3`.
    let show = run_cli(&["show", path.to_str().unwrap()]);
    assert!(
        show.status.success(),
        "`show` sobre un M4A con nombre .mp3 no debe fallar por formato, stderr: {}",
        String::from_utf8_lossy(&show.stderr)
    );
    let stdout = String::from_utf8_lossy(&show.stdout);
    assert!(
        stdout.contains("Renombrado a mp3"),
        "el título escrito debe releerse, stdout: {stdout}"
    );

    // `remove` también opera por contenido sobre el archivo renombrado.
    let remove = run_cli(&["remove", path.to_str().unwrap(), "title"]);
    assert!(
        remove.status.success(),
        "`remove` sobre un M4A con nombre .mp3 debe funcionar, stderr: {}",
        String::from_utf8_lossy(&remove.stderr)
    );
    let show_after = run_cli(&["show", path.to_str().unwrap()]);
    assert!(show_after.status.success());
    assert!(
        !String::from_utf8_lossy(&show_after.stdout).contains("Renombrado a mp3"),
        "el título debe haberse eliminado, stdout: {}",
        String::from_utf8_lossy(&show_after.stdout)
    );

    // Y sigue siendo un MP4 válido: conserva el átomo `ftyp` en el offset 4,
    // luego nunca se ha reescrito como MP3.
    let bytes = fs::read(&path).expect("Failed to read temp file");
    assert_eq!(
        &bytes[4..8],
        b"ftyp",
        "el archivo debe seguir siendo MP4 por contenido"
    );

    cleanup_file(&path);
}

#[test]
fn test_cli_rejects_unsupported_formats_and_leaves_file_untouched() {
    // El caso `ogg_sin_paquete` NO es un «códec no soportado»: sus bytes forman
    // una página Ogg (`OggS`) cuyo byte 26 —el número de segmentos— vale `0`, es
    // decir sin ningún paquete. Como no hay identificador de códec que leer, el
    // contenido cae en «no se pudo determinar el formato» (`Unknown`), no en
    // «formato no soportado» (`Unsupported`). Sigue en esta lista porque los dos
    // mensajes enumeran los formatos soportados y el archivo queda intacto; los
    // casos de «códec no soportado» (Opus, Speex) van aparte, en
    // `test_cli_rejects_ogg_other_codecs_and_leaves_file_untouched`.
    let cases: [(&str, &str, &[u8]); 4] = [
        ("flac", "flac", b"fLaC\x00\x00\x00\x22"),
        ("ogg_sin_paquete", "ogg", b"OggS\x00\x02"),
        ("wav", "wav", b"RIFF\x00\x00\x00\x00WAVE"),
        ("aif", "aif", b"FORM\x00\x00\x00\x00AIFF"),
    ];

    for (label, extension, magic) in cases {
        let mut bytes = magic.to_vec();
        bytes.extend_from_slice(&[0u8; 32]);
        let path = write_temp_bytes(label, extension, &bytes);
        let before = fs::read(&path).expect("Failed to read temp file");

        let output = Command::new("cargo")
            .args([
                "run",
                "--quiet",
                "--",
                "edit",
                path.to_str().unwrap(),
                "--title",
                "X",
            ])
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success(),
            "'edit' sobre un .{extension} debe fallar (stderr: {stderr})"
        );
        assert!(
            stderr.contains("soportado"),
            "'edit' sobre un .{extension} debe indicar formato no soportado, fue: {stderr}"
        );
        assert!(
            stderr.contains("MP3"),
            "el mensaje debe enumerar los formatos soportados, fue: {stderr}"
        );
        assert!(
            output.stdout.is_empty(),
            "el error no debe ir a stdout, fue: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert_eq!(
            fs::read(&path).expect("Failed to read temp file"),
            before,
            "'edit' NO debe modificar el archivo .{extension}"
        );

        cleanup_file(&path);
    }
}

#[test]
fn test_cli_rejects_empty_file() {
    let path = write_temp_bytes("empty", "mp3", &[]);

    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "show", path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        stderr.contains("formato"),
        "'show' sobre un archivo vacío debe informar del formato, fue: {stderr}"
    );
    assert_eq!(
        fs::read(&path).expect("Failed to read temp file"),
        Vec::<u8>::new()
    );

    cleanup_file(&path);
}

#[test]
fn test_cli_rejects_unrecognized_content() {
    let path = write_temp_bytes("text", "mp3", b"Esto no es un archivo de audio valido");

    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "show", path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        stderr.contains("formato") && stderr.contains("MP3"),
        "'show' sobre contenido no reconocible debe informar del formato, fue: {stderr}"
    );

    cleanup_file(&path);
}

#[test]
fn test_cli_rejects_too_short_file() {
    // `0xFF` + bits altos a 1, pero sin bytes suficientes para validar la
    // cabecera MPEG: no se puede decidir el formato.
    let path = write_temp_bytes("short", "mp3", &[0xFF, 0xFB]);
    let before = fs::read(&path).expect("Failed to read temp file");

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "edit",
            path.to_str().unwrap(),
            "--title",
            "X",
        ])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        stderr.contains("formato"),
        "un archivo demasiado corto debe informar de que no se pudo decidir, fue: {stderr}"
    );
    assert_eq!(fs::read(&path).expect("Failed to read temp file"), before);

    cleanup_file(&path);
}

// ---------------------------------------------------------------------------
// OGG Vorbis: enrutado del CLI y operaciones completas (tareas 2.1 y 2.4)
// ---------------------------------------------------------------------------

/// Ruta absoluta al fixture Ogg Vorbis real y sin campos de usuario.
///
/// Es un Ogg Vorbis genuino: patrón `OggS` en el offset 0, versión `0` en el
/// byte 4, un segmento declarado en el byte 26 y el primer paquete `\x01vorbis`
/// en el offset 28. Solo lleva el comentario obligatorio (vendor `ffmpeg` y la
/// clave `ENCODER`), de modo que `edit` ejercita la creación de campos desde
/// cero.
fn fixture_ogg_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("sample.ogg")
}

/// Copia el fixture Ogg Vorbis real a una ruta temporal única.
///
/// NO se usa `create_temp_fixture(extension)`: ese helper copia **siempre** el
/// MP3 versionado y aquí hace falta un `.ogg` real que `lofty` parsee como
/// Vorbis. Nunca se opera sobre el fixture versionado directamente.
fn create_temp_ogg() -> PathBuf {
    let path = unique_temp_path("audio", "ogg");
    fs::copy(fixture_ogg_path(), &path).expect("Failed to copy OGG fixture");
    path
}

/// Construye una página Ogg de un solo paquete con el identificador de códec
/// indicado como primer (y único) paquete.
///
/// Réplica del helper homónimo de `src/tests.rs`: el módulo `tests` de la
/// librería es privado y no es accesible desde este binario de integración, así
/// que la construcción se repite aquí con el mismo layout.
///
/// * bytes 0-3   : patrón de captura `OggS`
/// * byte 4      : versión del flujo (`0`)
/// * byte 5      : tipo de página (`0`)
/// * bytes 6-25  : granule, serial, secuencia y CRC (a cero)
/// * byte 26     : número de segmentos de página (`1`)
/// * byte 27     : tabla de segmentos (lacing), un segmento del tamaño del códec
/// * bytes 28-…  : primer paquete, el identificador de códec
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
fn test_cli_ogg_edit_then_show_roundtrip() {
    let path = create_temp_ogg();

    let edit = run_cli(&[
        "edit",
        path.to_str().unwrap(),
        "--title",
        "Canción OGG",
        "--artist",
        "Artista",
        "--album",
        "Álbum",
        "--year",
        "2026",
        "--genre",
        "Rock",
        "--track",
        "7",
        "--date",
        "2026-10-13",
    ]);
    assert!(
        edit.status.success(),
        "`edit` sobre un .ogg debe salir con 0, stderr: {}",
        String::from_utf8_lossy(&edit.stderr)
    );

    // Los valores se releen de forma independiente con `lofty` (vía la lectura
    // concreta de la librería) para no verificar la escritura consigo misma.
    let tag = id3cli::read_vorbis_tag(&path).expect("lofty debe releer el .ogg");
    assert_eq!(tag.get("TITLE"), Some("Canción OGG"));
    assert_eq!(tag.get("ARTIST"), Some("Artista"));
    assert_eq!(tag.get("ALBUM"), Some("Álbum"));
    assert_eq!(tag.get("YEAR"), Some("2026"));
    assert_eq!(tag.get("GENRE"), Some("Rock"));
    assert_eq!(tag.get("TRACKNUMBER"), Some("7"));
    assert_eq!(tag.get("DATE"), Some("2026-10-13"));

    // Y `show` los muestra en la salida estándar.
    let show = run_cli(&["show", path.to_str().unwrap()]);
    assert!(show.status.success());
    let stdout = String::from_utf8_lossy(&show.stdout);
    for value in ["Canción OGG", "Artista", "Álbum", "2026-10-13", "Rock", "7"] {
        assert!(
            stdout.contains(value),
            "`show` debe mostrar {value:?}, stdout: {stdout}"
        );
    }

    cleanup_file(&path);
}

#[test]
fn test_cli_ogg_renamed_to_mp3_is_edited_by_content() {
    // La extensión NO decide el formato: un `.ogg` **válido** renombrado a
    // `.mp3` sigue detectándose por sus bytes como Ogg Vorbis y editándose con
    // el backend OGG, igual que ya se prueba con el MP4.
    let path = unique_temp_path("ogg_as_mp3", "mp3");
    fs::copy(fixture_ogg_path(), &path).expect("Failed to copy OGG fixture");

    let edit = run_cli(&["edit", path.to_str().unwrap(), "--title", "Renombrado"]);
    assert!(
        edit.status.success(),
        "un .ogg con nombre .mp3 debe editarse por su contenido, stderr: {}",
        String::from_utf8_lossy(&edit.stderr)
    );

    let show = run_cli(&["show", path.to_str().unwrap()]);
    assert!(show.status.success());
    assert!(
        String::from_utf8_lossy(&show.stdout).contains("Renombrado"),
        "el título escrito debe releerse, stdout: {}",
        String::from_utf8_lossy(&show.stdout)
    );

    // Y sigue siendo Ogg por contenido: el patrón de captura `OggS` intacto.
    let bytes = fs::read(&path).expect("Failed to read temp file");
    assert_eq!(
        &bytes[..4],
        b"OggS",
        "el archivo debe seguir siendo Ogg por contenido"
    );

    cleanup_file(&path);
}

#[test]
fn test_cli_ogg_remove_field_keeps_rest() {
    let path = create_temp_ogg();
    let file = path.to_str().unwrap();

    let edit = run_cli(&[
        "edit", file, "--title", "T", "--artist", "A", "--album", "Alb",
    ]);
    assert!(edit.status.success());

    let remove = run_cli(&["remove", file, "title"]);
    assert!(
        remove.status.success(),
        "`remove title` debe salir con 0, stderr: {}",
        String::from_utf8_lossy(&remove.stderr)
    );

    let tag = id3cli::read_vorbis_tag(&path).expect("el .ogg debe seguir legible");
    assert_eq!(tag.get("TITLE"), None, "el título debe desaparecer");
    assert_eq!(
        tag.get("ARTIST"),
        Some("A"),
        "el artista no mencionado sobrevive"
    );
    assert_eq!(
        tag.get("ALBUM"),
        Some("Alb"),
        "el álbum no mencionado sobrevive"
    );

    // `show` deja de mostrarlo también.
    let show = run_cli(&["show", file]);
    assert!(show.status.success());
    let stdout = String::from_utf8_lossy(&show.stdout);
    assert!(
        !stdout.contains("🎵 Título:"),
        "show no debe mostrar el título: {stdout}"
    );

    cleanup_file(&path);
}

#[test]
fn test_cli_ogg_remove_all_empties_comment_and_preserves_vendor() {
    use lofty::ogg::OggPictureStorage;

    let path = create_temp_ogg();

    let edit = run_cli(&[
        "edit",
        path.to_str().unwrap(),
        "--title",
        "T",
        "--artist",
        "A",
    ]);
    assert!(edit.status.success());

    let remove = run_cli(&["remove", path.to_str().unwrap(), "--all"]);
    assert!(
        remove.status.success(),
        "`remove --all` debe salir con 0, stderr: {}",
        String::from_utf8_lossy(&remove.stderr)
    );
    assert!(
        String::from_utf8_lossy(&remove.stdout).contains("eliminados"),
        "debe confirmar el borrado, stdout: {}",
        String::from_utf8_lossy(&remove.stdout)
    );

    // No queda ningún campo de usuario ni carátula, pero el vendor se conserva.
    let tag = id3cli::read_vorbis_tag(&path).expect("el .ogg debe seguir legible");
    assert_eq!(
        tag.items().count(),
        0,
        "no debe quedar ningún campo de usuario"
    );
    assert!(tag.pictures().is_empty(), "no debe quedar ninguna carátula");
    assert_eq!(tag.vendor(), "ffmpeg", "el vendor debe conservarse");

    // Y el archivo sigue siendo un Ogg Vorbis válido por contenido.
    assert_eq!(
        id3cli::detect_format(&path).expect("detect_format debe poder leer el archivo"),
        id3cli::AudioFormat::OggVorbis,
        "el archivo debe seguir siendo Ogg Vorbis"
    );

    cleanup_file(&path);
}

#[test]
fn test_cli_rejects_ogg_other_codecs_and_leaves_file_untouched() {
    // Un contenedor Ogg es un sobre: solo Vorbis está soportado. Opus y Speex
    // se reconocen como contenido conocido pero fuera de alcance. El test NO
    // usa `OggS` a secas (eso es una página sin primer paquete, que cae en
    // «formato no determinado»): construye páginas Ogg **válidas** con el
    // identificador de códec en el primer paquete para que el rechazo sea por
    // códec y no por página incompleta.
    let cases: [(&str, &[u8]); 2] = [("opus", b"OpusHead"), ("speex", b"Speex   ")];

    for (label, codec) in cases {
        let path = write_temp_bytes(label, "ogg", &synthetic_ogg_page(codec));
        let before = fs::read(&path).expect("Failed to read temp file");

        for (op, extra) in [
            ("show", Vec::<&str>::new()),
            ("edit", vec!["--title", "X"]),
            ("remove", vec!["--all"]),
        ] {
            let mut args = vec![op, path.to_str().unwrap()];
            args.extend_from_slice(&extra);
            let output = Command::new("cargo")
                .args(["run", "--quiet", "--"])
                .args(&args)
                .output()
                .expect("Failed to execute command");

            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                !output.status.success(),
                "'{op}' sobre un Ogg con códec {label} debe fallar (stderr: {stderr})"
            );
            assert!(
                stderr.contains("soportado"),
                "'{op}' sobre un Ogg con códec {label} debe indicar formato no soportado, fue: {stderr}"
            );
            assert!(
                stderr.contains("MP3") && stderr.contains("MP4") && stderr.contains("OGG"),
                "el mensaje debe enumerar los formatos soportados (MP3, MP4 y OGG), fue: {stderr}"
            );
            assert!(
                output.stdout.is_empty(),
                "el error no debe ir a stdout, fue: {}",
                String::from_utf8_lossy(&output.stdout)
            );
            assert_eq!(
                fs::read(&path).expect("Failed to read temp file"),
                before,
                "'{op}' NO debe modificar el archivo con códec {label}"
            );
        }

        cleanup_file(&path);
    }
}

/// `show` sobre un Ogg recién creado (solo vendor y `encoder`) debe avisar de
/// que no hay metadatos, salir con 0 y no imprimir ni el vendor ni `ENCODER`.
///
/// El test unitario homónimo no puede capturar stdout (el harness de `cargo
/// test` no lo permite sin dependencias externas), así que la verificación real
/// del escenario «Visualización de un Ogg sin campos de usuario» tiene que
/// lanzar el binario y leer su salida.
#[test]
fn test_cli_ogg_show_without_user_fields_says_no_metadata() {
    let path = create_temp_ogg();
    let file = path.to_str().unwrap();

    // El fixture solo lleva el comentario obligatorio (vendor `ffmpeg` y la
    // clave `encoder=Lavc libvorbis`), sin ningún campo de usuario.
    let show = run_cli(&["show", file]);
    assert!(
        show.status.success(),
        "`show` sobre un .ogg sin campos de usuario debe salir con 0, stderr: {}",
        String::from_utf8_lossy(&show.stderr)
    );
    let stdout = String::from_utf8_lossy(&show.stdout);
    assert!(
        stdout.contains("⚠️  No se encontraron metadatos."),
        "debe avisar de que no hay metadatos, stdout: {stdout}"
    );
    for forbidden in ["ffmpeg", "Lavc", "encoder", "ENCODER"] {
        assert!(
            !stdout.contains(forbidden),
            "`show` NO debe imprimir {forbidden:?} (vendor/ENCODER), stdout: {stdout}"
        );
    }

    // Contraste: tras escribir un título, `show` sí lo muestra y sigue sin
    // exponer el vendor ni el `encoder`. Sin este contraste el test anterior
    // pasaría aunque `show` no imprimiera nunca nada.
    let title = "Título con contraste";
    let edit = run_cli(&["edit", file, "--title", title]);
    assert!(
        edit.status.success(),
        "no se pudo escribir el título: {}",
        String::from_utf8_lossy(&edit.stderr)
    );

    let show = run_cli(&["show", file]);
    assert!(
        show.status.success(),
        "`show` tras escribir el título debe salir con 0, stderr: {}",
        String::from_utf8_lossy(&show.stderr)
    );
    let stdout = String::from_utf8_lossy(&show.stdout);
    assert!(
        stdout.contains(title),
        "`show` debe mostrar el título escrito, stdout: {stdout}"
    );
    for forbidden in ["ffmpeg", "Lavc", "encoder", "ENCODER"] {
        assert!(
            !stdout.contains(forbidden),
            "aun con título, `show` NO debe imprimir {forbidden:?}, stdout: {stdout}"
        );
    }

    cleanup_file(&path);
}

// ---------------------------------------------------------------------------
// MP4 (M4A): enrutado del CLI y operaciones completas (tareas 4.9–4.11)
// ---------------------------------------------------------------------------

/// Ruta absoluta al fixture MP4 real y sin metadatos, versionado en el repo.
fn fixture_m4a_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("sample.m4a")
}

/// Copia el fixture MP4 real a una ruta temporal única sobre la que trabajar.
///
/// Nunca se opera sobre el fixture versionado directamente.
fn create_temp_m4a(label: &str) -> PathBuf {
    let path = unique_temp_path(label, "m4a");
    fs::copy(fixture_m4a_path(), &path).expect("Failed to copy M4A fixture");
    path
}

/// Comprueba si un bloque de bytes contiene una secuencia dada.
fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

/// Devuelve el *payload* del átomo `mdat` (sin su cabecera), delimitado por su
/// tamaño declarado.
///
/// Sirve para comprobar que el audio sobrevive byte a byte a las operaciones
/// que solo tocan el árbol de metadatos `moov.udta.meta.ilst`. Se compara solo
/// el payload porque en este fixture `moov` va **después** de `mdat` y cambia
/// al escribir metadatos.
fn mdat_payload(bytes: &[u8]) -> Vec<u8> {
    let pos = bytes
        .windows(4)
        .position(|window| window == b"mdat")
        .expect("el archivo MP4 debe contener el átomo `mdat`");
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

/// Lee el tag `ilst` de un M4A que se acaba de escribir con el CLI.
fn read_m4a_tag(path: &Path) -> Option<lofty::tag::Tag> {
    let tagged = lofty::read_from_path(path).expect("lofty debe leer el M4A");
    tagged.tag(TagType::Mp4Ilst).cloned()
}

/// Ejecuta `cargo run -- <args>` y devuelve la salida, fallando con contexto.
fn run_cli(args: &[&str]) -> std::process::Output {
    let mut full = vec!["run", "--quiet", "--"];
    full.extend_from_slice(args);
    let output = Command::new("cargo")
        .args(&full)
        .output()
        .expect("Failed to execute command");
    if !output.status.success() {
        eprintln!("args: {args:?}");
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    output
}

#[test]
fn test_cli_m4a_edit_then_show_roundtrip() {
    let path = create_temp_m4a("roundtrip");

    let output = run_cli(&[
        "edit",
        path.to_str().unwrap(),
        "--title",
        "Canción M4A",
        "--artist",
        "Artista",
        "--album",
        "Álbum",
        "--genre",
        "Rock",
        "--date",
        "2026-10-13",
    ]);
    assert!(output.status.success());

    // Los valores vuelven a leerse con `lofty` desde el propio archivo.
    let tag = read_m4a_tag(&path).expect("debe existir tag tras `edit`");
    assert_eq!(tag.get_string(ItemKey::TrackTitle), Some("Canción M4A"));
    assert_eq!(tag.get_string(ItemKey::TrackArtist), Some("Artista"));
    assert_eq!(tag.get_string(ItemKey::AlbumTitle), Some("Álbum"));
    assert_eq!(tag.get_string(ItemKey::Genre), Some("Rock"));
    assert_eq!(tag.get_string(ItemKey::RecordingDate), Some("2026-10-13"));

    // Y `show` los muestra.
    let show = run_cli(&["show", path.to_str().unwrap()]);
    assert!(show.status.success());
    let stdout = String::from_utf8_lossy(&show.stdout);
    assert!(stdout.contains("Canción M4A"), "stdout: {stdout}");
    assert!(stdout.contains("Artista"), "stdout: {stdout}");
    assert!(stdout.contains("Álbum"), "stdout: {stdout}");
    assert!(stdout.contains("Rock"), "stdout: {stdout}");
    assert!(stdout.contains("2026-10-13"), "stdout: {stdout}");

    cleanup_file(&path);
}

#[test]
fn test_cli_m4a_three_operations_do_not_fail_for_format() {
    let path = create_temp_m4a("three_ops");
    let file = path.to_str().unwrap();

    // `show` sobre un M4A sin metadatos: aviso, sin error.
    let show_empty = run_cli(&["show", file]);
    assert!(show_empty.status.success());
    let stderr = String::from_utf8_lossy(&show_empty.stderr);
    assert!(
        !stderr.contains("no soportado") && !stderr.contains("no está soportado"),
        "`show` no debe fallar por formato: {stderr}"
    );

    // `edit`: escribe y no falla por formato.
    let edit = run_cli(&["edit", file, "--title", "X"]);
    assert!(edit.status.success());

    // `show` de nuevo: ya hay metadatos.
    let show = run_cli(&["show", file]);
    assert!(show.status.success());
    assert!(String::from_utf8_lossy(&show.stdout).contains('X'));

    // `remove` de un tag concreto: no falla por formato.
    let remove = run_cli(&["remove", file, "title"]);
    assert!(remove.status.success());

    cleanup_file(&path);
}

#[test]
fn test_cli_m4a_and_mp3_show_same_field_set() {
    let mp3_path = create_temp_mp3();
    let m4a_path = create_temp_m4a("coherence");

    let common: [&str; 12] = [
        "--title",
        "Canción",
        "--artist",
        "Artista",
        "--album",
        "Álbum",
        "--genre",
        "Rock",
        "--track",
        "4",
        "--date",
        "2026-10-13",
    ];

    for path in [&mp3_path, &m4a_path] {
        let mut args = vec!["edit", path.to_str().unwrap()];
        args.extend_from_slice(&common);
        let edit = run_cli(&args);
        assert!(edit.status.success());
    }

    let mp3_out = run_cli(&["show", mp3_path.to_str().unwrap()]);
    let m4a_out = run_cli(&["show", m4a_path.to_str().unwrap()]);
    assert!(mp3_out.status.success());
    assert!(m4a_out.status.success());
    let mp3_stdout = String::from_utf8_lossy(&mp3_out.stdout);
    let m4a_stdout = String::from_utf8_lossy(&m4a_out.stdout);

    // El mismo conjunto de campos debe mostrarse en ambos formatos.
    for label in [
        "🎵 Título:",
        "🎤 Artista:",
        "💿 Álbum:",
        "🎸 Género:",
        "#️⃣  Pista:",
        "📆 Fecha:",
    ] {
        assert!(mp3_stdout.contains(label), "MP3 debería mostrar {label}");
        assert!(m4a_stdout.contains(label), "M4A debería mostrar {label}");
    }

    cleanup_file(&mp3_path);
    cleanup_file(&m4a_path);
}

#[test]
fn test_cli_m4a_remove_spanish_tag_keeps_rest() {
    let path = create_temp_m4a("remove_es");
    let file = path.to_str().unwrap();

    let edit = run_cli(&[
        "edit", file, "--title", "T", "--artist", "A", "--album", "Alb",
    ]);
    assert!(edit.status.success());

    // Eliminar SOLO el título, con su nombre en español.
    let remove = run_cli(&["remove", file, "título"]);
    assert!(remove.status.success());

    let tag = read_m4a_tag(&path).expect("el resto de átomos debe seguir legible");
    assert_eq!(
        tag.get_string(ItemKey::TrackTitle),
        None,
        "título eliminado"
    );
    assert_eq!(
        tag.get_string(ItemKey::TrackArtist),
        Some("A"),
        "el artista no mencionado debe sobrevivir"
    );
    assert_eq!(
        tag.get_string(ItemKey::AlbumTitle),
        Some("Alb"),
        "el álbum no mencionado debe sobrevivir"
    );

    cleanup_file(&path);
}

#[test]
fn test_cli_m4a_remove_all_removes_ilst_and_preserves_audio() {
    let path = create_temp_m4a("remove_all");

    // El audio del fixture (a partir de `mdat`) es lo que debe sobrevivir.
    let audio_before = mdat_payload(&fs::read(&path).expect("leer fixture"));

    let edit = run_cli(&["edit", path.to_str().unwrap(), "--title", "T"]);
    assert!(edit.status.success());

    // Editar crea el bloque `ilst` sin tocar el audio.
    let with_meta = fs::read(&path).expect("leer tras edit");
    assert!(
        contains_bytes(&with_meta, b"ilst"),
        "edit debe crear el bloque ilst"
    );
    assert_eq!(
        mdat_payload(&with_meta),
        audio_before,
        "editar no debe modificar el audio"
    );

    // `remove --all` elimina el bloque completo.
    let remove = run_cli(&["remove", path.to_str().unwrap(), "--all"]);
    assert!(remove.status.success());
    assert!(
        String::from_utf8_lossy(&remove.stdout).contains("eliminados"),
        "debe confirmar el borrado"
    );

    let after = fs::read(&path).expect("leer tras remove --all");
    assert!(
        !contains_bytes(&after, b"ilst"),
        "el bloque de metadatos ilst debe desaparecer"
    );
    assert_eq!(
        mdat_payload(&after),
        audio_before,
        "el audio debe permanecer intacto tras remove --all"
    );
    // El archivo sigue siendo MP4 por contenido.
    assert_eq!(&after[4..8], b"ftyp", "ftyp debe seguir en el offset 4");

    cleanup_file(&path);
}

#[test]
fn test_cli_m4a_remove_all_without_metadata_warns() {
    // Copia del fixture recién generado: no tiene metadatos.
    let path = create_temp_m4a("remove_all_empty");

    let remove = run_cli(&["remove", path.to_str().unwrap(), "--all"]);
    assert!(remove.status.success(), "sin metadatos debe salir con 0");
    let stdout = String::from_utf8_lossy(&remove.stdout);
    assert!(
        stdout.contains("no tenía metadatos"),
        "debe avisar de que no había metadatos, fue: {stdout}"
    );

    cleanup_file(&path);
}

#[test]
fn test_cli_unsupported_format_message_lists_mp3_mp4_and_ogg() {
    let path = write_temp_bytes("flac_msg", "flac", b"fLaC\x00\x00\x00\x22");

    let output = run_cli(&["edit", path.to_str().unwrap(), "--title", "X"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("MP3") && stderr.contains("MP4") && stderr.contains("OGG"),
        "el mensaje de formato no soportado debe enumerar MP3, MP4 y OGG, fue: {stderr}"
    );

    cleanup_file(&path);
}

// ---------------------------------------------------------------------------
// MP4 (M4A): carátulas (`covr`) y letras (`©lyr`) — grupo 5
// ---------------------------------------------------------------------------

/// Bytes de una imagen JPEG reconocible (signatura + inicio de APP0).
const JPEG_BYTES: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46];
/// Bytes de una imagen PNG reconocible (signatura + primer trozo).
const PNG_BYTES: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
];
/// Bytes de una imagen WebP reconocible.
const WEBP_BYTES: &[u8] = b"RIFF\x24\x00\x00\x00WEBPVP8 ";

/// Escribe una imagen temporal con la extensión indicada y la devuelve.
fn write_temp_image(label: &str, extension: &str, bytes: &[u8]) -> PathBuf {
    let path = unique_temp_path(label, extension);
    fs::write(&path, bytes).expect("Failed to write temp image");
    path
}

/// Devuelve `(tipo de dato, payload)` del primer `data` box que sigue al átomo
/// indicado, leyendo directamente los bytes del contenedor ISO-BMFF.
///
/// Comprueba el **tipo de dato real** escrito (13 JPEG, 14 PNG, 0 reservado) en
/// lugar de asumirlo. Layout: `[size][data][version/flags][locale][payload]`,
/// con el tipo en los 24 bits bajos de `version/flags`.
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
fn test_cli_m4a_cover_jpeg_uses_wellknown_type_and_keeps_bytes() {
    let path = create_temp_m4a("cover_jpeg_cli");
    let image = write_temp_image("cover_jpeg_cli", "jpg", JPEG_BYTES);

    let out = run_cli(&[
        "edit",
        path.to_str().unwrap(),
        "--cover",
        image.to_str().unwrap(),
        "--title",
        "Con JPG",
    ]);
    assert!(out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).is_empty(),
        "una carátula JPEG no debe emitir avisos, fue: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // El tipo de dato real del `data` box del `covr` es JPEG (13) y los bytes
    // de la imagen quedan intactos.
    let bytes = fs::read(&path).unwrap();
    let (data_type, payload) = data_box_after(&bytes, b"covr").expect("debe existir `covr`");
    assert_eq!(data_type, 13, "JPEG debe usar el tipo well-known 13");
    assert_eq!(payload, JPEG_BYTES);

    // `show` lista la carátula.
    let show = run_cli(&["show", path.to_str().unwrap()]);
    assert!(show.status.success());
    assert!(
        String::from_utf8_lossy(&show.stdout).contains("Carátulas: 1"),
        "`show` debe listar la carátula"
    );

    cleanup_file(&path);
    cleanup_file(&image);
}

#[test]
fn test_cli_m4a_cover_png_uses_wellknown_type_and_keeps_bytes() {
    let path = create_temp_m4a("cover_png_cli");
    let image = write_temp_image("cover_png_cli", "png", PNG_BYTES);

    let out = run_cli(&[
        "edit",
        path.to_str().unwrap(),
        "--cover",
        image.to_str().unwrap(),
        "--title",
        "Con PNG",
    ]);
    assert!(out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).is_empty(),
        "una carátula PNG no debe emitir avisos"
    );

    let bytes = fs::read(&path).unwrap();
    let (data_type, payload) = data_box_after(&bytes, b"covr").expect("debe existir `covr`");
    assert_eq!(data_type, 14, "PNG debe usar el tipo well-known 14");
    assert_eq!(payload, PNG_BYTES);

    cleanup_file(&path);
    cleanup_file(&image);
}

#[test]
fn test_cli_m4a_cover_webp_reserved_type_warns_and_succeeds() {
    let path = create_temp_m4a("cover_webp_cli");
    let image = write_temp_image("cover_webp_cli", "webp", WEBP_BYTES);

    let out = run_cli(&[
        "edit",
        path.to_str().unwrap(),
        "--cover",
        image.to_str().unwrap(),
        "--title",
        "Con WebP",
    ]);

    // No falla: termina con código de salida cero.
    assert!(out.status.success(), "WebP no debe abortar la operación");

    // Avisa por stderr de que no será visible en iTunes/macOS.
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("WebP") && stderr.contains("iTunes") && stderr.contains("macOS"),
        "debe advertir de que la carátula no es visible, fue: {stderr}"
    );

    // El tipo de dato real es el reservado (0), con los bytes intactos.
    let bytes = fs::read(&path).unwrap();
    let (data_type, payload) = data_box_after(&bytes, b"covr").expect("debe existir `covr`");
    assert_eq!(
        data_type, 0,
        "WebP debe quedar con el tipo de dato reservado (0)"
    );
    assert_eq!(payload, WEBP_BYTES);

    // El resto de campos indicados se escriben con normalidad.
    let tag = read_m4a_tag(&path).expect("el tag debe ser legible");
    assert_eq!(tag.get_string(ItemKey::TrackTitle), Some("Con WebP"));

    cleanup_file(&path);
    cleanup_file(&image);
}

#[test]
fn test_cli_m4a_cover_missing_file_fails_without_writing() {
    let path = create_temp_m4a("cover_missing_cli");
    let before = fs::read(&path).unwrap();

    let out = run_cli(&[
        "edit",
        path.to_str().unwrap(),
        "--cover",
        "/ruta/que/no/existe.jpg",
        "--title",
        "X",
    ]);
    assert!(
        !out.status.success(),
        "una carátula inexistente debe fallar"
    );

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("no existe"),
        "debe informar en español de que el archivo no existe, fue: {stderr}"
    );
    assert_eq!(
        fs::read(&path).unwrap(),
        before,
        "no se debe escribir nada en el archivo"
    );

    cleanup_file(&path);
}

#[test]
fn test_cli_m4a_cover_unsupported_format_fails_without_writing() {
    let path = create_temp_m4a("cover_tiff_cli");
    let image = write_temp_image("cover_tiff_cli", "tiff", b"II*\x00");
    let before = fs::read(&path).unwrap();

    let out = run_cli(&[
        "edit",
        path.to_str().unwrap(),
        "--cover",
        image.to_str().unwrap(),
        "--title",
        "X",
    ]);
    assert!(
        !out.status.success(),
        "una extensión no soportada debe fallar"
    );

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Formato de imagen no soportado") && stderr.contains(".tiff"),
        "debe explicar en español que el formato no está soportado, fue: {stderr}"
    );
    assert_eq!(
        fs::read(&path).unwrap(),
        before,
        "no se debe escribir nada en el archivo"
    );

    cleanup_file(&path);
    cleanup_file(&image);
}

#[test]
fn test_cli_m4a_remove_cover_removes_covr_and_keeps_rest() {
    let path = create_temp_m4a("remove_cover_cli");
    let image = write_temp_image("remove_cover_cli", "png", PNG_BYTES);

    let edit = run_cli(&[
        "edit",
        path.to_str().unwrap(),
        "--title",
        "T",
        "--artist",
        "A",
        "--cover",
        image.to_str().unwrap(),
    ]);
    assert!(edit.status.success());
    assert!(contains_bytes(&fs::read(&path).unwrap(), b"covr"));

    let remove = run_cli(&["remove", path.to_str().unwrap(), "cover"]);
    assert!(remove.status.success());

    assert!(
        !contains_bytes(&fs::read(&path).unwrap(), b"covr"),
        "el átomo `covr` debe desaparecer"
    );
    let tag = read_m4a_tag(&path).expect("el resto debe seguir legible");
    assert!(tag.pictures().is_empty());
    assert_eq!(tag.get_string(ItemKey::TrackTitle), Some("T"));
    assert_eq!(tag.get_string(ItemKey::TrackArtist), Some("A"));

    cleanup_file(&path);
    cleanup_file(&image);
}

#[test]
fn test_cli_m4a_lyrics_written_shown_and_without_language() {
    let path = create_temp_m4a("lyrics_cli");
    let lyrics = "Primera línea\nSegunda línea\nTercera línea";

    let out = run_cli(&["edit", path.to_str().unwrap(), "--lyrics", lyrics]);
    assert!(out.status.success());

    // El texto se recupera íntegro por `show`.
    let show = run_cli(&["show", path.to_str().unwrap()]);
    assert!(show.status.success());
    let stdout = String::from_utf8_lossy(&show.stdout);
    assert!(stdout.contains("📝 Letra:"), "show debe mostrar la letra");
    assert!(stdout.contains("Primera línea"), "stdout: {stdout}");
    assert!(stdout.contains("Tercera línea"), "stdout: {stdout}");

    // El `©lyr` no lleva prefijo de idioma: su `data` box es exactamente el texto.
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
fn test_cli_lyrics_language_spa_in_mp3_still_preserved() {
    let mp3_path = create_temp_mp3();
    let lyrics = "Uno\nDos\nTres";

    let out = run_cli(&["edit", mp3_path.to_str().unwrap(), "--lyrics", lyrics]);
    assert!(out.status.success());

    // En MP3 el frame `USLT` sigue llevando el código de idioma `spa`.
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    let mut found = false;
    for frame in tag.frames() {
        if let id3::frame::Content::Lyrics(l) = frame.content() {
            assert_eq!(l.lang, "spa", "el USLT debe conservar el idioma `spa`");
            assert_eq!(l.text, lyrics, "la letra se conserva íntegra");
            found = true;
            break;
        }
    }
    assert!(found, "debe existir el frame USLT de la letra");

    cleanup_file(&mp3_path);
}

// ---------------------------------------------------------------------------
// MP4 (M4A): campos sin equivalente (`--url`, `--original-artist`) — grupo 6
// ---------------------------------------------------------------------------

/// Prefijo de todo átomo *freeform* `ilst`. Se usa para comprobar que el CLI no
/// suple la omisión de un campo con un átomo propietario.
const FREEFORM_PREFIX: &[u8] = b"----:com.apple.iTunes";

#[test]
fn test_cli_m4a_url_is_omitted_with_warning() {
    let path = create_temp_m4a("url_omitted");
    let out = run_cli(&[
        "edit",
        path.to_str().unwrap(),
        "--url",
        "https://artista.com",
        "--title",
        "Cancion",
    ]);

    // No aborta: termina con código de salida cero.
    assert!(out.status.success(), "`--url` en MP4 no debe abortar");

    // Avisa por stderr de que la URL se omite por no tener equivalente.
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--url") && stderr.contains("se omite"),
        "debe avisar de la omisión de --url por stderr, fue: {stderr}"
    );

    // El resto de campos indicados sí se escriben: `©nam` con "Cancion".
    let tag = read_m4a_tag(&path).expect("debe existir tag tras `edit`");
    assert_eq!(tag.get_string(ItemKey::TrackTitle), Some("Cancion"));

    // No se crea ningún átomo freeform de reemplazo para la URL.
    let bytes = fs::read(&path).unwrap();
    assert!(
        !contains_bytes(&bytes, FREEFORM_PREFIX),
        "el CLI no debe crear un átomo freeform para suplir la URL"
    );
    assert!(
        !contains_bytes(&bytes, b"URL"),
        "no debe existir ningún átomo que represente la URL"
    );

    cleanup_file(&path);
}

#[test]
fn test_cli_m4a_original_artist_is_omitted_with_warning() {
    let path = create_temp_m4a("original_artist_omitted");
    let out = run_cli(&[
        "edit",
        path.to_str().unwrap(),
        "--original-artist",
        "Autor",
        "--album",
        "Disco",
    ]);

    assert!(
        out.status.success(),
        "`--original-artist` en MP4 no debe abortar"
    );

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--original-artist") && stderr.contains("se omite"),
        "debe avisar de la omisión de --original-artist por stderr, fue: {stderr}"
    );

    // El resto de campos indicados sí se escriben: `©alb` con "Disco".
    let tag = read_m4a_tag(&path).expect("debe existir tag tras `edit`");
    assert_eq!(tag.get_string(ItemKey::AlbumTitle), Some("Disco"));

    // Ningún átomo freeform de reemplazo.
    assert!(
        !contains_bytes(&fs::read(&path).unwrap(), FREEFORM_PREFIX),
        "el CLI no debe crear un átomo freeform para el artista original"
    );

    cleanup_file(&path);
}

#[test]
fn test_cli_m4a_both_unequivalent_flags_warn_and_leave_file_untouched() {
    let path = create_temp_m4a("both_omitted");
    let before = fs::read(&path).unwrap();

    let out = run_cli(&[
        "edit",
        path.to_str().unwrap(),
        "--url",
        "https://artista.com",
        "--original-artist",
        "Autor",
    ]);

    assert!(out.status.success(), "no debe abortar");

    // Sendos avisos por stderr (uno por campo omitido).
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--url") && stderr.contains("--original-artist"),
        "deben emitirse los dos avisos, fue: {stderr}"
    );
    assert_eq!(
        stderr.matches("se omite").count(),
        2,
        "deben ser exactamente dos avisos, fue: {stderr}"
    );

    // Si esos dos son los únicos flags, no hay cambio real que escribir: el
    // archivo queda byte a byte idéntico.
    let after = fs::read(&path).unwrap();
    assert_eq!(
        before, after,
        "sin cambios reales, `edit` no debe reescribir el archivo"
    );

    cleanup_file(&path);
}

#[test]
fn test_cli_m4a_omission_warnings_go_only_to_stderr() {
    let path = create_temp_m4a("warn_streams");
    let out = run_cli(&[
        "edit",
        path.to_str().unwrap(),
        "--url",
        "https://artista.com",
        "--original-artist",
        "Autor",
        "--title",
        "Cancion",
    ]);
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    // Los avisos van a stderr…
    assert!(
        stderr.contains("se omite"),
        "el aviso de omisión va en stderr, fue: {stderr}"
    );
    // …y no contaminan stdout.
    assert!(
        !stdout.contains("se omite")
            && !stdout.contains("--url")
            && !stdout.contains("--original-artist"),
        "los avisos no deben aparecer en stdout, fue: {stdout}"
    );
    // La confirmación normal del campo que sí se escribe sigue en stdout.
    assert!(
        stdout.contains("Cancion"),
        "la confirmación normal va en stdout, fue: {stdout}"
    );

    // Y el campo que sí tiene equivalente se escribió.
    let tag = read_m4a_tag(&path).expect("debe existir tag tras `edit`");
    assert_eq!(tag.get_string(ItemKey::TrackTitle), Some("Cancion"));

    cleanup_file(&path);
}

#[test]
fn test_cli_mp3_url_and_original_artist_still_write_without_warning() {
    let mp3_path = create_temp_mp3();

    let out = run_cli(&[
        "edit",
        mp3_path.to_str().unwrap(),
        "--url",
        "https://artista.com",
        "--original-artist",
        "Autor",
    ]);
    assert!(out.status.success());
    assert!(
        out.stderr.is_empty(),
        "en MP3 no debe emitirse ningún aviso, fue: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // En MP3 `--url` escribe `WOAR` y `--original-artist`, `TOPE`.
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");

    let mut url = None;
    for frame in tag.frames() {
        if frame.id() == "WOAR"
            && let id3::frame::Content::Link(link) = frame.content()
        {
            url = Some(link.clone());
        }
    }
    assert_eq!(
        url.as_deref(),
        Some("https://artista.com"),
        "debe escribir WOAR"
    );
    assert_eq!(
        tag.get("TOPE").and_then(|f| f.content().text()),
        Some("Autor"),
        "debe escribir TOPE"
    );

    cleanup_file(&mp3_path);
}

// ---------------------------------------------------------------------------
// Auditoría de paridad: escenarios de la spec sin test dedicado (S9.2 y S5.1)
// ---------------------------------------------------------------------------

/// Devuelve los bytes de audio que siguen al bloque ID3v2 antepuesto.
///
/// `edit` escribe el tag ID3v2.4 **anteponiéndolo** al audio original, de modo
/// que el audio debe ser exactamente el sufijo que empieza tras la cabecera de
/// 10 bytes más el tamaño *synchsafe* declarado (y el posible pie de página de
/// otros 10). Comparar ese sufijo con el audio de partida prueba a la vez que
/// el archivo sigue siendo un MP3 válido y que el audio queda intacto.
fn audio_after_id3v2(bytes: &[u8]) -> &[u8] {
    assert!(bytes.len() >= 10, "el archivo debe tener cabecera ID3v2");
    assert_eq!(&bytes[0..3], b"ID3", "debe empezar con la cabecera ID3v2");
    let size = ((bytes[6] as usize & 0x7F) << 21)
        | ((bytes[7] as usize & 0x7F) << 14)
        | ((bytes[8] as usize & 0x7F) << 7)
        | (bytes[9] as usize & 0x7F);
    // El flag de pie de página (0x10) añade 10 bytes al final del bloque.
    let footer = if bytes[5] & 0x10 != 0 { 10 } else { 0 };
    &bytes[10 + size + footer..]
}

/// Escenario S9.2 (`edit` de un solo campo preserva el resto).
///
/// Con título, artista, álbum y género ya escritos, un `edit --year 2026` debe
/// tocar SOLO el año y dejar los demás frames exactamente igual: mismos valores
/// y sin duplicados ni pérdidas.
#[test]
fn test_cli_edit_year_preserves_other_fields() {
    let mp3_path = create_temp_mp3();

    // 1ª invocación: el MP3 ya tiene título, artista, álbum y género.
    let setup = run_cli(&[
        "edit",
        mp3_path.to_str().unwrap(),
        "--title",
        "Título Original",
        "--artist",
        "Artista Original",
        "--album",
        "Álbum Original",
        "--genre",
        "Rock",
    ]);
    assert!(setup.status.success());

    // 2ª invocación: SOLO --year (y nada más).
    let output = run_cli(&["edit", mp3_path.to_str().unwrap(), "--year", "2026"]);
    assert!(
        output.status.success(),
        "edit --year debe salir con código cero"
    );

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");

    // El año se escribió…
    assert_eq!(
        tag.get("TYER").and_then(|f| f.content().text()),
        Some("2026"),
        "--year debe escribir el frame TYER"
    );
    // …y el resto de campos siguen exactamente igual que antes.
    assert_eq!(tag.title(), Some("Título Original"));
    assert_eq!(tag.artist(), Some("Artista Original"));
    assert_eq!(tag.album(), Some("Álbum Original"));
    assert_eq!(tag.genre(), Some("Rock"));

    // Sin duplicados: cada frame aparece una única vez.
    for frame_id in ["TIT2", "TPE1", "TALB", "TCON", "TYER"] {
        let count = tag.frames().filter(|f| f.id() == frame_id).count();
        assert_eq!(
            count, 1,
            "el frame {frame_id} debe aparecer exactamente una vez"
        );
    }

    cleanup_file(&mp3_path);
}

/// Escenario S5.1 (`edit --cover portada.jpg`) para JPEG.
///
/// Debe terminar con código cero, escribir un frame `APIC` con MIME
/// `image/jpeg` y conservar **byte a byte** los bytes del archivo de imagen de
/// origen, sin dañar el audio.
#[test]
fn test_cli_cover_jpeg_keeps_image_bytes() {
    let mp3_path = create_temp_mp3();
    let audio = fs::read(&mp3_path).expect("Failed to read audio");
    let cover_path = write_temp_image("cover_jpeg_mp3", "jpg", JPEG_BYTES);

    let output = run_cli(&[
        "edit",
        mp3_path.to_str().unwrap(),
        "--cover",
        cover_path.to_str().unwrap(),
        "--title",
        "Con carátula JPEG",
    ]);
    assert!(
        output.status.success(),
        "edit --cover .jpg debe salir con cero"
    );

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    let pictures: Vec<_> = tag.pictures().collect();
    assert_eq!(pictures.len(), 1, "debe haber exactamente una carátula");
    assert_eq!(
        pictures[0].mime_type, "image/jpeg",
        "el MIME debe detectarse por la extensión"
    );

    // Los bytes de la imagen leídos del tag son idénticos a los del origen.
    let original = fs::read(&cover_path).expect("Failed to read cover file");
    assert_eq!(
        pictures[0].data.as_slice(),
        original.as_slice(),
        "los bytes de la carátula JPEG deben quedar intactos"
    );

    // El audio sigue intacto: es exactamente el sufijo tras el tag ID3v2.
    let result = fs::read(&mp3_path).expect("Failed to read result");
    assert_eq!(
        audio_after_id3v2(&result),
        audio.as_slice(),
        "el audio debe permanecer intacto y el archivo seguir siendo válido"
    );

    cleanup_file(&mp3_path);
    cleanup_file(&cover_path);
}

/// Escenario S5.1 (`edit --cover portada.png`) para PNG.
///
/// Igual que el caso JPEG pero comprobando también que los bytes del PNG
/// quedan intactos, algo que ningún test de integración cubría hasta ahora.
#[test]
fn test_cli_cover_png_keeps_image_bytes() {
    let mp3_path = create_temp_mp3();
    let audio = fs::read(&mp3_path).expect("Failed to read audio");
    let cover_path = write_temp_image("cover_png_mp3", "png", PNG_BYTES);

    let output = run_cli(&[
        "edit",
        mp3_path.to_str().unwrap(),
        "--cover",
        cover_path.to_str().unwrap(),
        "--title",
        "Con carátula PNG",
    ]);
    assert!(
        output.status.success(),
        "edit --cover .png debe salir con cero"
    );

    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    let pictures: Vec<_> = tag.pictures().collect();
    assert_eq!(pictures.len(), 1, "debe haber exactamente una carátula");
    assert_eq!(
        pictures[0].mime_type, "image/png",
        "el MIME debe detectarse por la extensión"
    );

    // Los bytes de la imagen leídos del tag son idénticos a los del origen.
    let original = fs::read(&cover_path).expect("Failed to read cover file");
    assert_eq!(
        pictures[0].data.as_slice(),
        original.as_slice(),
        "los bytes de la carátula PNG deben quedar intactos"
    );

    // El audio sigue intacto: es exactamente el sufijo tras el tag ID3v2.
    let result = fs::read(&mp3_path).expect("Failed to read result");
    assert_eq!(
        audio_after_id3v2(&result),
        audio.as_slice(),
        "el audio debe permanecer intacto y el archivo seguir siendo válido"
    );

    cleanup_file(&mp3_path);
    cleanup_file(&cover_path);
}

// ---------------------------------------------------------------------------
// Robustez de la salida estándar: pipe cerrado y error de escritura (fase RED)
// ---------------------------------------------------------------------------

/// `show` con el extremo de lectura del pipe ya cerrado no debe ser un error.
///
/// El archivo temporal se crea **con metadatos reales** (helper + `edit`) y no
/// vacío: `show` solo escribe en stdout cuando hay algo que mostrar, y sobre un
/// archivo sin etiquetas el CLI avisa por stderr. Como el contrato exige que
/// stderr quede vacío, el archivo debe tener metadatos para que el único motivo
/// posible de fallo sea el contrato nuevo y no un aviso ajeno al test.
#[test]
fn test_cli_pipe_cerrado_no_es_error() {
    let mp3_path = create_temp_mp3();

    let edit = run_cli(&[
        "edit",
        mp3_path.to_str().unwrap(),
        "--title",
        "Título para pipe",
        "--artist",
        "Artista para pipe",
        "--album",
        "Álbum para pipe",
        "--year",
        "2026",
        "--genre",
        "Rock",
    ]);
    assert!(
        edit.status.success(),
        "no se pudo preparar el MP3 con metadatos: {}",
        String::from_utf8_lossy(&edit.stderr)
    );

    // Se canaliza stdout y se **cierra el extremo de lectura antes de leer
    // nada**: se toma el `ChildStdout` y se suelta de inmediato. Si se leyera
    // primero, con pocos metadatos el proceso podría terminar antes de que se
    // cierre el pipe y la prueba no distinguiría nada; cerrándolo antes, la
    // primera escritura en stdout devuelve EPIPE y el fallo es determinista.
    let mut child = Command::new("cargo")
        .args(["run", "--quiet", "--", "show", mp3_path.to_str().unwrap()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    drop(child.stdout.take());

    let output = child
        .wait_with_output()
        .expect("Failed to wait for command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(
        output.status.code(),
        Some(0),
        "`show` con stdout cerrado debe salir con 0, no {:?}. stderr: {}",
        output.status.code(),
        stderr
    );
    assert!(
        stderr.is_empty(),
        "stderr debe quedar vacío cuando el pipe está cerrado, fue: {}",
        stderr
    );
    assert!(
        !stderr.contains("panicked")
            && !stderr.contains("failed printing to stdout")
            && !stderr.contains("RUST_BACKTRACE"),
        "no debe aparecer un pánico al escribir en stdout: {}",
        stderr
    );

    cleanup_file(&mp3_path);
}

/// Mismo contrato que `test_cli_pipe_cerrado_no_es_error` sobre un M4A.
///
/// El archivo se prepara con metadatos reales por la misma razón que en la
/// versión MP3: garantizar que stderr estaría vacío de no ser por el fallo que
/// describe el contrato.
#[test]
fn test_cli_pipe_cerrado_no_es_error_m4a() {
    let m4a_path = create_temp_m4a("pipe_cerrado");

    let edit = run_cli(&[
        "edit",
        m4a_path.to_str().unwrap(),
        "--title",
        "Título para pipe",
        "--artist",
        "Artista para pipe",
        "--album",
        "Álbum para pipe",
        "--date",
        "2026-10-14",
        "--genre",
        "Rock",
    ]);
    assert!(
        edit.status.success(),
        "no se pudo preparar el M4A con metadatos: {}",
        String::from_utf8_lossy(&edit.stderr)
    );

    // Igual que en el caso MP3: se cierra el extremo de lectura antes de leer
    // para que el EPIPE sea determinista y no dependa de una carrera.
    let mut child = Command::new("cargo")
        .args(["run", "--quiet", "--", "show", m4a_path.to_str().unwrap()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    drop(child.stdout.take());

    let output = child
        .wait_with_output()
        .expect("Failed to wait for command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(
        output.status.code(),
        Some(0),
        "`show` sobre M4A con stdout cerrado debe salir con 0, no {:?}. stderr: {}",
        output.status.code(),
        stderr
    );
    assert!(
        stderr.is_empty(),
        "stderr debe quedar vacío cuando el pipe está cerrado, fue: {}",
        stderr
    );
    assert!(
        !stderr.contains("panicked")
            && !stderr.contains("failed printing to stdout")
            && !stderr.contains("RUST_BACKTRACE"),
        "no debe aparecer un pánico al escribir en stdout: {}",
        stderr
    );

    cleanup_file(&m4a_path);
}

/// Mismo contrato que las versiones MP3 y M4A sobre un OGG.
///
/// Completa el escenario que exige que el cierre del pipe se comporte igual en
/// las **tres** rutas de visualización. El archivo se prepara con metadatos
/// reales por la misma razón que en sus hermanos: garantizar que stderr
/// estaría vacío de no ser por el fallo que describe el contrato.
#[test]
fn test_cli_pipe_cerrado_no_es_error_ogg() {
    let ogg_path = create_temp_ogg();

    let edit = run_cli(&[
        "edit",
        ogg_path.to_str().unwrap(),
        "--title",
        "Título para pipe",
        "--artist",
        "Artista para pipe",
        "--album",
        "Álbum para pipe",
        "--year",
        "2026",
        "--genre",
        "Rock",
    ]);
    assert!(
        edit.status.success(),
        "no se pudo preparar el OGG con metadatos: {}",
        String::from_utf8_lossy(&edit.stderr)
    );

    // Igual que en los casos MP3 y M4A: se cierra el extremo de lectura antes de
    // leer para que el EPIPE sea determinista y no dependa de una carrera.
    let mut child = Command::new("cargo")
        .args(["run", "--quiet", "--", "show", ogg_path.to_str().unwrap()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    drop(child.stdout.take());

    let output = child
        .wait_with_output()
        .expect("Failed to wait for command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(
        output.status.code(),
        Some(0),
        "`show` sobre OGG con stdout cerrado debe salir con 0, no {:?}. stderr: {}",
        output.status.code(),
        stderr
    );
    assert!(
        stderr.is_empty(),
        "stderr debe quedar vacío cuando el pipe está cerrado, fue: {}",
        stderr
    );
    assert!(
        !stderr.contains("panicked")
            && !stderr.contains("failed printing to stdout")
            && !stderr.contains("RUST_BACKTRACE"),
        "no debe aparecer un pánico al escribir en stdout: {}",
        stderr
    );

    cleanup_file(&ogg_path);
}

/// Un error de escritura real en stdout debe fallar como error, no con pánico.
///
/// Solo se ejecuta en Linux porque usa `/dev/full`: abrirlo funciona pero toda
/// escritura devuelve `ENOSPC`. A diferencia del pipe cerrado (que puede ser
/// ignorable), un dispositivo de salida lleno es un fallo duro: el proceso debe
/// terminar con un código distinto de cero y un mensaje en español por stderr,
/// nunca con un `panicked`.
#[cfg(target_os = "linux")]
#[test]
fn test_cli_error_de_escritura_falla() {
    let mp3_path = create_temp_mp3();

    let edit = run_cli(&[
        "edit",
        mp3_path.to_str().unwrap(),
        "--title",
        "Título para /dev/full",
        "--artist",
        "Artista para /dev/full",
        "--album",
        "Álbum para /dev/full",
        "--year",
        "2026",
        "--genre",
        "Rock",
    ]);
    assert!(
        edit.status.success(),
        "no se pudo preparar el MP3 con metadatos: {}",
        String::from_utf8_lossy(&edit.stderr)
    );

    // Toda escritura en `/dev/full` falla con ENOSPC. Se usa `spawn` +
    // `wait_with_output` en lugar de `output()` para no perder la redirección
    // de stdout que `output()` reemplazaría por un pipe.
    let full = fs::File::create("/dev/full").expect("se debe poder abrir /dev/full");
    let child = Command::new("cargo")
        .args(["run", "--quiet", "--", "show", mp3_path.to_str().unwrap()])
        .stdout(Stdio::from(full))
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    let output = child
        .wait_with_output()
        .expect("Failed to wait for command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "un error de escritura en stdout debe hacer fallar el proceso. stderr: {}",
        stderr
    );
    assert_ne!(
        output.status.code(),
        Some(0),
        "el código de salida debe ser distinto de cero. stderr: {}",
        stderr
    );
    assert!(
        !stderr.is_empty(),
        "debe aparecer un mensaje de error en español por stderr"
    );
    assert!(
        !stderr.contains("panicked"),
        "el fallo debe reportarse como error, no como pánico: {}",
        stderr
    );

    cleanup_file(&mp3_path);
}
