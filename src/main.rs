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

    /// Compositor (TCOM)
    #[arg(long)]
    composer: Option<String>,

    /// Subtítulo o descripción (TIT3)
    #[arg(long)]
    subtitle: Option<String>,

    /// Artista original (TOPE)
    #[arg(long)]
    original_artist: Option<String>,

    /// Artista del álbum / Publisher (TPE2)
    #[arg(long)]
    album_artist: Option<String>,

    /// Ruta del archivo de imagen para la carátula (JPG, PNG, WEBP)
    #[arg(short, long)]
    cover: Option<PathBuf>,

    /// Letra de la canción (lyrics)
    #[arg(short = 'L', long)]
    lyrics: Option<String>,

    /// URL asociada (sitio web del artista, página oficial, etc.)
    #[arg(short = 'u', long)]
    url: Option<String>,

    /// Marcar como compilación (Apple TCMP)
    #[arg(long)]
    compilation: bool,

    /// Orden de clasificación del álbum (Apple TSOA)
    #[arg(long)]
    album_sort: Option<String>,

    /// Orden de clasificación del artista (Apple TSOP)
    #[arg(long)]
    artist_sort: Option<String>,

    /// Orden de clasificación del título (Apple TSOT)
    #[arg(long)]
    title_sort: Option<String>,

    /// Mostrar todos los tags del archivo
    #[arg(short, long)]
    show: bool,

    /// Eliminar tags específicos (title, artist, album, year, genre, track, date, copyright, cover, lyrics, url, compilation, album_sort, artist_sort, title_sort)
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
    composer: Option<&str>,
    subtitle: Option<&str>,
    original_artist: Option<&str>,
    album_artist: Option<&str>,
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

    if let Some(composer_text) = composer {
        tag.set_text("TCOM", composer_text);
        changed = true;
    }

    if let Some(subtitle_text) = subtitle {
        tag.set_text("TIT3", subtitle_text);
        changed = true;
    }

    if let Some(original_artist_text) = original_artist {
        tag.set_text("TOPE", original_artist_text);
        changed = true;
    }

    if let Some(album_artist_text) = album_artist {
        tag.set_album_artist(album_artist_text);
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

/// Añade metadatos de Apple al tag
fn add_apple_metadata(
    tag: &mut Tag, 
    compilation: bool,
    album_sort: Option<&str>,
    artist_sort: Option<&str>,
    title_sort: Option<&str>
) -> bool {
    let mut changed = false;
    
    // TCMP - Compilation flag (1 = part of compilation)
    if compilation {
        tag.set_text("TCMP", "1");
        changed = true;
    }
    
    // TSOA - Album sort order
    if let Some(sort) = album_sort {
        tag.set_text("TSOA", sort);
        changed = true;
    }
    
    // TSOP - Performer/Artist sort order
    if let Some(sort) = artist_sort {
        tag.set_text("TSOP", sort);
        changed = true;
    }
    
    // TSOT - Title sort order
    if let Some(sort) = title_sort {
        tag.set_text("TSOT", sort);
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
            "composer" | "compositor" => {
                tag.remove("TCOM");
                true
            }
            "subtitle" | "subtítulo" | "subtitulo" | "description" | "descripción" | "descripcion" => {
                tag.remove("TIT3");
                true
            }
            "original_artist" | "original-artist" | "artista_original" | "artista-original" => {
                tag.remove("TOPE");
                true
            }
            "album_artist" | "album-artist" | "artista_album" | "artista-album" => {
                tag.remove_album_artist();
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
            "compilation" | "compilación" | "compilacion" => {
                tag.remove("TCMP");
                true
            }
            "album_sort" | "album-sort" | "orden_album" | "orden-album" => {
                tag.remove("TSOA");
                true
            }
            "artist_sort" | "artist-sort" | "orden_artista" | "orden-artista" => {
                tag.remove("TSOP");
                true
            }
            "title_sort" | "title-sort" | "orden_titulo" | "orden-titulo" => {
                tag.remove("TSOT");
                true
            }
            _ => {
                eprintln!("⚠️  Tag desconocido: '{}'. Tags válidos: title, artist, album, year, genre, track, date, copyright, composer, subtitle, original_artist, album_artist, cover, lyrics, url, compilation, album_sort, artist_sort, title_sort", tag_name);
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
    
    if let Some(composer) = tag.get("TCOM").and_then(|f| f.content().text()) {
        println!("🎼 Compositor: {}", composer);
    }
    
    if let Some(subtitle) = tag.get("TIT3").and_then(|f| f.content().text()) {
        println!("📄 Subtítulo: {}", subtitle);
    }
    
    if let Some(original_artist) = tag.get("TOPE").and_then(|f| f.content().text()) {
        println!("🎙️  Artista original: {}", original_artist);
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
    
    // Mostrar metadatos de Apple si existen
    if let Some(compilation) = tag.get("TCMP").and_then(|f| f.content().text()) {
        if compilation == "1" {
            println!(" Compilación: Sí");
        }
    }
    
    if let Some(album_sort) = tag.get("TSOA").and_then(|f| f.content().text()) {
        println!("🔤 Orden álbum: {}", album_sort);
    }
    
    if let Some(artist_sort) = tag.get("TSOP").and_then(|f| f.content().text()) {
        println!("🔤 Orden artista: {}", artist_sort);
    }
    
    if let Some(title_sort) = tag.get("TSOT").and_then(|f| f.content().text()) {
        println!("🔤 Orden título: {}", title_sort);
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
        args.composer.as_deref(),
        args.subtitle.as_deref(),
        args.original_artist.as_deref(),
        args.album_artist.as_deref(),
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
    if let Some(composer) = &args.composer {
        println!("✓ Compositor: {}", composer);
    }
    if let Some(subtitle) = &args.subtitle {
        println!("✓ Subtítulo: {}", subtitle);
    }
    if let Some(original_artist) = &args.original_artist {
        println!("✓ Artista original: {}", original_artist);
    }
    if let Some(album_artist) = &args.album_artist {
        println!("✓ Artista del álbum: {}", album_artist);
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

    // Añadir metadatos de Apple
    let apple_added = add_apple_metadata(
        &mut tag,
        args.compilation,
        args.album_sort.as_deref(),
        args.artist_sort.as_deref(),
        args.title_sort.as_deref()
    );
    
    if args.compilation {
        println!("✓ Compilación: Sí");
    }
    if let Some(sort) = &args.album_sort {
        println!("✓ Orden álbum: {}", sort);
    }
    if let Some(sort) = &args.artist_sort {
        println!("✓ Orden artista: {}", sort);
    }
    if let Some(sort) = &args.title_sort {
        println!("✓ Orden título: {}", sort);
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
    if changed || cover_added || removed || lyrics_added || url_added || apple_added {
        tag.write_to_path(&args.file, id3::Version::Id3v24)?;
        println!("\n✅ Tags guardados correctamente en '{}'", args.file.display());
    } else {
        println!("\n⚠️  No se especificaron cambios. Usa --help para ver las opciones.");
    }

    Ok(())
}

#[cfg(test)]
mod tests;
