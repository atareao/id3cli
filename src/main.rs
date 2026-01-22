use clap::Parser;
use id3::{Tag, TagLike};
use id3::frame::{Picture, PictureType};
use std::fs;
use std::path::PathBuf;

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

    /// Ruta del archivo JPG para la carátula
    #[arg(short, long)]
    cover: Option<PathBuf>,

    /// Mostrar todos los tags del archivo
    #[arg(short, long)]
    show: bool,
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

/// Crea un Picture frame desde datos de imagen
fn create_picture_frame(data: Vec<u8>, mime_type: &str) -> Picture {
    Picture {
        mime_type: mime_type.to_string(),
        picture_type: PictureType::CoverFront,
        description: "Cover".to_string(),
        data,
    }
}

/// Añade una carátula al tag desde un archivo
fn add_cover_art(tag: &mut Tag, cover_data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let picture = create_picture_frame(cover_data, "image/jpeg");
    tag.add_frame(picture);
    Ok(())
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
    
    let pictures: Vec<_> = tag.pictures().collect();
    if !pictures.is_empty() {
        println!("🖼️  Carátulas: {} imagen(es)", pictures.len());
        for (i, pic) in pictures.iter().enumerate() {
            println!("   [{}] Tipo: {:?}, MIME: {}, Tamaño: {} bytes", 
                i + 1, pic.picture_type, pic.mime_type, pic.data.len());
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

    // Añadir carátula
    let mut cover_added = false;
    if let Some(cover_path) = &args.cover {
        if !cover_path.exists() {
            eprintln!("Error: El archivo de carátula '{}' no existe", cover_path.display());
            std::process::exit(1);
        }

        let cover_data = fs::read(cover_path)?;
        add_cover_art(&mut tag, cover_data)?;
        println!("✓ Carátula añadida desde: {}", cover_path.display());
        cover_added = true;
    }

    // Guardar cambios
    if changed || cover_added {
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
        
        let result = add_cover_art(&mut tag, data.clone());
        assert!(result.is_ok());
        
        let pictures: Vec<_> = tag.pictures().collect();
        assert_eq!(pictures.len(), 1);
        assert_eq!(pictures[0].data, data);
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
}
