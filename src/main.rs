use clap::Parser;
use id3::{Tag, TagLike, Frame};
use id3::frame::{Content, Lyrics, Picture, PictureType};
use std::fs;
use std::path::{Path, PathBuf};

/// CLI para añadir tags ID3 y carátulas a archivos MP3
#[derive(Parser, Debug)]
#[command(name = "id3cli")]
#[command(about = "Añade tags ID3 y carátulas a archivos MP3", long_about = None)]
struct Args {
    /// Ruta del archivo MP3
    #[arg(short, long)]
    file: PathBuf,

    /// Título de la canción
    #[arg(short, long)]
    title: Option<String>,

    /// Artista (se puede especificar múltiples veces)
    #[arg(short, long)]
    artist: Vec<String>,

    /// Álbum
    #[arg(short = 'A', long)]
    album: Option<String>,

    /// Año
    #[arg(short, long)]
    year: Option<i32>,

    /// Género
    #[arg(short, long)]
    genre: Option<String>,

    /// Número de pista
    #[arg(short = 'T', long)]
    track: Option<u32>,

    /// Fecha de grabación (YYYY-MM-DD o YYYY)
    #[arg(short = 'd', long)]
    date: Option<String>,

    /// Copyright
    #[arg(short = 'C', long)]
    copyright: Option<String>,

    /// Ruta del archivo de imagen para la carátula (JPG, PNG, WEBP)
    #[arg(short, long)]
    cover: Option<PathBuf>,

    /// Letra de la canción (lyrics)
    #[arg(short = 'L', long)]
    lyrics: Option<String>,

    /// URL asociada (sitio web del artista, página oficial, etc.)
    #[arg(short = 'u', long)]
    url: Option<String>,

    /// Mostrar todos los tags del archivo
    #[arg(short, long)]
    show: bool,

    /// Eliminar tags específicos (title, artist, album, year, genre, track, date, copyright, cover, lyrics, url)
    #[arg(short, long)]
    remove: Vec<String>,
}

/// Aplica los metadatos especificados al tag ID3
fn apply_metadata(
    tag: &mut Tag,
    title: Option<&str>,
    artists: &[String],
    album: Option<&str>,
    year: Option<i32>,
    genre: Option<&str>,
    track: Option<u32>,
    date: Option<&str>,
    copyright: Option<&str>,
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

    changed
}

/// Añade letras (lyrics) al tag
fn add_lyrics(tag: &mut Tag, text: &str) -> bool {
    let lyrics_frame = Frame::with_content("USLT", Content::Lyrics(Lyrics {
        lang: "spa".to_string(),
        description: String::new(),
        text: text.to_string(),
    }));
    
    tag.add_frame(lyrics_frame);
    true
}

/// Añade URL al tag (WOAR - Official artist/performer webpage)
fn add_url(tag: &mut Tag, url: &str) -> bool {
    // WOAR es un frame de tipo Link, no Text
    let url_frame = Frame::with_content("WOAR", Content::Link(url.to_string()));
    tag.add_frame(url_frame);
    true
}

/// Crea un Picture frame desde datos de imagen
fn create_picture_frame(data: Vec<u8>, mime_type: &str) -> Picture {
    Picture {
        mime_type: mime_type.to_string(),
        picture_type: PictureType::CoverFront,
        description: "Cover".to_string(),
        data,
    }
}

/// Detecta el tipo MIME desde la extensión del archivo
fn detect_mime_type(path: &Path) -> Result<&'static str, String> {
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
fn add_cover_art(tag: &mut Tag, cover_path: &Path, cover_data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let mime_type = detect_mime_type(cover_path)
        .map_err(|e| format!("{} (soportados: jpg, png, webp)", e))?;
    let picture = create_picture_frame(cover_data, mime_type);
    tag.add_frame(picture);
    Ok(())
}

/// Elimina tags específicos del archivo
fn remove_tags(tag: &mut Tag, tags_to_remove: &[String]) -> bool {
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
            "date" | "fecha" => {
                tag.remove_date_recorded();
                true
            }
            "copyright" => {
                tag.remove("TCOP");
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
            _ => {
                eprintln!("⚠️  Tag desconocido: '{}'. Tags válidos: title, artist, album, year, genre, track, date, copyright, cover, lyrics, url", tag_name);
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

/// Muestra todos los tags del archivo MP3
fn display_tags(tag: &Tag) {
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
    
    if let Some(copyright) = tag.get("TCOP").and_then(|f| f.content().text()) {
        println!("©️  Copyright: {}", copyright);
    }
    
    if let Some(album_artist) = tag.album_artist() {
        println!("👥 Artista del álbum: {}", album_artist);
    }
    
    // Mostrar URL si existe
    for frame in tag.frames() {
        if frame.id() == "WOAR" {
            if let Content::Link(url) = frame.content() {
                println!("🌐 URL: {}", url);
                break;
            }
        }
    }
    
    let pictures: Vec<_> = tag.pictures().collect();
    if !pictures.is_empty() {
        println!("🖼️  Carátulas: {} imagen(es)", pictures.len());
        for (i, pic) in pictures.iter().enumerate() {
            println!("   [{}] Tipo: {:?}, MIME: {}, Tamaño: {} bytes", 
                i + 1, pic.picture_type, pic.mime_type, pic.data.len());
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
    
    // Mostrar otros frames si existen
    let frame_count = tag.frames().count();
    if frame_count > 0 {
        println!("\n📦 Total de frames: {}", frame_count);
    }
    
    println!("═══════════════════════════════════════\n");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Verificar que el archivo MP3 existe
    if !args.file.exists() {
        eprintln!("Error: El archivo '{}' no existe", args.file.display());
        std::process::exit(1);
    }

    // Leer o crear tag
    let mut tag = match Tag::read_from_path(&args.file) {
        Ok(tag) => {
            if !args.show {
                println!("Tags existentes encontrados en '{}'", args.file.display());
            }
            tag
        }
        Err(_) => {
            if args.show {
                eprintln!("⚠️  No se encontraron tags ID3 en '{}'", args.file.display());
                std::process::exit(0);
            }
            println!("Creando nuevos tags para '{}'", args.file.display());
            Tag::new()
        }
    };

    // Si solo se quiere mostrar los tags, mostrarlos y salir
    if args.show {
        display_tags(&tag);
        return Ok(());
    }

    // Procesar eliminación de tags si se especificó
    let mut removed = false;
    if !args.remove.is_empty() {
        removed = remove_tags(&mut tag, &args.remove);
    }

    // Aplicar metadatos
    let changed = apply_metadata(
        &mut tag,
        args.title.as_deref(),
        &args.artist,
        args.album.as_deref(),
        args.year,
        args.genre.as_deref(),
        args.track,
        args.date.as_deref(),
        args.copyright.as_deref(),
    );

    // Imprimir cambios aplicados
    if let Some(title) = &args.title {
        println!("✓ Título: {}", title);
    }
    if !args.artist.is_empty() {
        println!("✓ Artista(s): {}", args.artist.join("; "));
    }
    if let Some(album) = &args.album {
        println!("✓ Álbum: {}", album);
    }
    if let Some(year) = args.year {
        println!("✓ Año: {}", year);
    }
    if let Some(genre) = &args.genre {
        println!("✓ Género: {}", genre);
    }
    if let Some(track) = args.track {
        println!("✓ Pista: {}", track);
    }
    if let Some(date) = &args.date {
        println!("✓ Fecha: {}", date);
    }
    if let Some(copyright) = &args.copyright {
        println!("✓ Copyright: {}", copyright);
    }

    // Añadir lyrics
    let mut lyrics_added = false;
    if let Some(lyrics_text) = &args.lyrics {
        add_lyrics(&mut tag, lyrics_text);
        let line_count = lyrics_text.lines().count();
        println!("✓ Letra: {} línea(s)", line_count);
        lyrics_added = true;
    }

    // Añadir URL
    let mut url_added = false;
    if let Some(url) = &args.url {
        add_url(&mut tag, url);
        println!("✓ URL: {}", url);
        url_added = true;
    }

    // Añadir carátula
    let mut cover_added = false;
    if let Some(cover_path) = &args.cover {
        if !cover_path.exists() {
            eprintln!("Error: El archivo de carátula '{}' no existe", cover_path.display());
            std::process::exit(1);
        }

        let cover_data = fs::read(cover_path)?;
        match add_cover_art(&mut tag, cover_path, cover_data) {
            Ok(_) => {
                println!("✓ Carátula añadida desde: {}", cover_path.display());
                cover_added = true;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Guardar cambios
    if changed || cover_added || removed || lyrics_added || url_added {
        tag.write_to_path(&args.file, id3::Version::Id3v24)?;
        println!("\n✅ Tags guardados correctamente en '{}'", args.file.display());
    } else {
        println!("\n⚠️  No se especificaron cambios. Usa --help para ver las opciones.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_metadata_title_only() {
        let mut tag = Tag::new();
        let changed = apply_metadata(&mut tag, Some("Test Title"), &[], None, None, None, None, None, None);
        
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
        let changed = apply_metadata(&mut tag, None, &[], None, None, None, None, None, None);
        
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
        apply_metadata(&mut tag, None, &[], Some("New Album"), None, None, None, None, None);
        
        // El título y artista originales deben preservarse
        assert_eq!(tag.title(), Some("Original Title"));
        assert_eq!(tag.artist(), Some("Original Artist"));
        assert_eq!(tag.album(), Some("New Album"));
    }

    #[test]
    fn test_apply_metadata_overwrites_existing() {
        let mut tag = Tag::new();
        tag.set_title("Old Title");
        
        apply_metadata(&mut tag, Some("New Title"), &[], None, None, None, None, None, None);
        
        assert_eq!(tag.title(), Some("New Title"));
    }

    #[test]
    fn test_year_negative() {
        let mut tag = Tag::new();
        apply_metadata(&mut tag, None, &[], None, Some(-1), None, None, None, None);
        
        assert_eq!(tag.year(), Some(-1));
    }

    #[test]
    fn test_year_future() {
        let mut tag = Tag::new();
        apply_metadata(&mut tag, None, &[], None, Some(3000), None, None, None, None);
        
        assert_eq!(tag.year(), Some(3000));
    }

    #[test]
    fn test_empty_strings() {
        let mut tag = Tag::new();
        let artists = vec!["".to_string()];
        let changed = apply_metadata(&mut tag, Some(""), &artists, Some(""), None, Some(""), None, None, None);
        
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
        );
        
        assert_eq!(tag.title(), Some("Título con ñ y acentos"));
        assert_eq!(tag.artist(), Some("Artista 日本語"));
        assert_eq!(tag.album(), Some("Álbum 🎵"));
    }

    #[test]
    fn test_multiple_artists() {
        let mut tag = Tag::new();
        let artists = vec!["Artist 1".to_string(), "Artist 2".to_string(), "Artist 3".to_string()];
        let changed = apply_metadata(&mut tag, None, &artists, None, None, None, None, None, None);
        
        assert!(changed);
        assert_eq!(tag.artist(), Some("Artist 1; Artist 2; Artist 3"));
    }

    #[test]
    fn test_multiple_artists_unicode() {
        let mut tag = Tag::new();
        let artists = vec!["Artista Español".to_string(), "アーティスト".to_string()];
        apply_metadata(&mut tag, None, &artists, None, None, None, None, None, None);
        
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
        let changed = apply_metadata(&mut tag, None, &[], None, None, None, Some(7), None, None);
        
        assert!(changed);
        assert_eq!(tag.track(), Some(7));
    }

    #[test]
    fn test_track_number_zero() {
        let mut tag = Tag::new();
        apply_metadata(&mut tag, None, &[], None, None, None, Some(0), None, None);
        
        assert_eq!(tag.track(), Some(0));
    }

    #[test]
    fn test_track_with_other_metadata() {
        let mut tag = Tag::new();
        let artists = vec!["Artist".to_string()];
        apply_metadata(&mut tag, Some("Title"), &artists, None, None, None, Some(3), None, None);
        
        assert_eq!(tag.title(), Some("Title"));
        assert_eq!(tag.artist(), Some("Artist"));
        assert_eq!(tag.track(), Some(3));
    }

    #[test]
    fn test_date_recorded() {
        let mut tag = Tag::new();
        let changed = apply_metadata(&mut tag, None, &[], None, None, None, None, Some("2026-01-22"), None);
        
        assert!(changed);
        assert_eq!(tag.date_recorded().map(|t| t.to_string()), Some("2026-01-22".to_string()));
    }

    #[test]
    fn test_copyright() {
        let mut tag = Tag::new();
        let changed = apply_metadata(&mut tag, None, &[], None, None, None, None, None, Some("© 2026 Records"));
        
        assert!(changed);
        assert_eq!(tag.get("TCOP").and_then(|f| f.content().text()), Some("© 2026 Records"));
    }

    #[test]
    fn test_date_and_copyright() {
        let mut tag = Tag::new();
        apply_metadata(&mut tag, None, &[], None, None, None, None, Some("2026"), Some("© Test"));
        
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
}
