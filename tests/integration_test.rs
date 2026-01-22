use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};
use id3::{Tag, TagLike};

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
        0x00, 0x00, 0x00, 0x00,             // Size (0)
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
        .args(&["run", "--quiet", "--", "--file", mp3_path.to_str().unwrap(), "--title", "Test Song"])
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
            "run", "--quiet", "--",
            "--file", mp3_path.to_str().unwrap(),
            "--title", "Song",
            "--artist", "Artist",
            "--album", "Album",
            "--year", "2026",
            "--genre", "Rock"
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
        .args(&["run", "--quiet", "--", "--file", "/tmp/nonexistent_file_12345.mp3", "--title", "Test"])
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
            "run", "--quiet", "--",
            "--file", mp3_path.to_str().unwrap(),
            "--title", "Canción con ñ",
            "--artist", "Artista español"
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
        .args(&["run", "--quiet", "--", "--file", mp3_path.to_str().unwrap(), "--title", "Original"])
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
        .args(&["run", "--quiet", "--", "--file", mp3_path.to_str().unwrap(), "--artist", "Artist"])
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
            "run", "--quiet", "--",
            "--file", mp3_path.to_str().unwrap(),
            "--artist", "Artist 1",
            "--artist", "Artist 2",
            "--artist", "Artist 3"
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
            "run", "--quiet", "--",
            "--file", mp3_path.to_str().unwrap(),
            "--title", "Collaboration Song",
            "--artist", "DJ Snake",
            "--artist", "Justin Bieber"
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
            "run", "--quiet", "--",
            "--file", mp3_path.to_str().unwrap(),
            "--title", "Show Test",
            "--artist", "Test Artist",
            "--album", "Test Album",
            "--year", "2026"
        ])
        .output()
        .expect("Failed to execute command");
    
    // Ahora mostrar los tags
    let output = Command::new("cargo")
        .args(&[
            "run", "--quiet", "--",
            "--file", mp3_path.to_str().unwrap(),
            "--show"
        ])
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
        .args(&[
            "run", "--quiet", "--",
            "--file", mp3_path.to_str().unwrap(),
            "--show"
        ])
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
            "run", "--quiet", "--",
            "--file", mp3_path.to_str().unwrap(),
            "--title", "Track Test",
            "--track", "5"
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
            "run", "--quiet", "--",
            "--file", mp3_path.to_str().unwrap(),
            "--title", "Complete Song",
            "--artist", "Complete Artist",
            "--album", "Complete Album",
            "--year", "2026",
            "--genre", "Pop",
            "--track", "3"
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
            "run", "--quiet", "--",
            "--file", mp3_path.to_str().unwrap(),
            "--title", "Copyright Test",
            "--date", "2026-01-22",
            "--copyright", "© 2026 Test Records"
        ])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    
    let tag = Tag::read_from_path(&mp3_path).expect("Failed to read tag");
    assert_eq!(tag.title(), Some("Copyright Test"));
    assert_eq!(tag.date_recorded().map(|t| t.to_string()), Some("2026-01-22".to_string()));
    assert_eq!(tag.get("TCOP").and_then(|f| f.content().text()), Some("© 2026 Test Records"));
    
    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_complete_metadata() {
    let mp3_path = create_temp_mp3();
    
    let output = Command::new("cargo")
        .args(&[
            "run", "--quiet", "--",
            "--file", mp3_path.to_str().unwrap(),
            "--title", "Complete",
            "--artist", "Artist One",
            "--artist", "Artist Two",
            "--album", "Album",
            "--year", "2026",
            "--genre", "Jazz",
            "--track", "7",
            "--date", "2026-01",
            "--copyright", "© All Rights Reserved"
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
    assert_eq!(tag.date_recorded().map(|t| t.to_string()), Some("2026-01".to_string()));
    assert_eq!(tag.get("TCOP").and_then(|f| f.content().text()), Some("© All Rights Reserved"));
    
    cleanup_file(&mp3_path);
}

#[test]
fn test_cli_remove_title() {
    let mp3_path = create_temp_mp3();
    
    // Primero añadir tags
    Command::new("cargo")
        .args(&[
            "run", "--quiet", "--",
            "--file", mp3_path.to_str().unwrap(),
            "--title", "Remove Me",
            "--artist", "Keep Me"
        ])
        .output()
        .expect("Failed to execute command");
    
    // Luego eliminar solo el título
    let output = Command::new("cargo")
        .args(&[
            "run", "--quiet", "--",
            "--file", mp3_path.to_str().unwrap(),
            "--remove", "title"
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
            "run", "--quiet", "--",
            "--file", mp3_path.to_str().unwrap(),
            "--title", "Title",
            "--artist", "Artist",
            "--album", "Album",
            "--year", "2026"
        ])
        .output()
        .expect("Failed to execute command");
    
    // Eliminar varios tags
    let output = Command::new("cargo")
        .args(&[
            "run", "--quiet", "--",
            "--file", mp3_path.to_str().unwrap(),
            "--remove", "title",
            "--remove", "artist"
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
            "run", "--quiet", "--",
            "--file", mp3_path.to_str().unwrap(),
            "--title", "Title",
            "--artist", "Artist",
            "--album", "Album",
            "--year", "2026",
            "--genre", "Rock",
            "--track", "5"
        ])
        .output()
        .expect("Failed to execute command");
    
    // Eliminar todos
    let output = Command::new("cargo")
        .args(&[
            "run", "--quiet", "--",
            "--file", mp3_path.to_str().unwrap(),
            "--remove", "title",
            "--remove", "artist",
            "--remove", "album",
            "--remove", "year",
            "--remove", "genre",
            "--remove", "track"
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
