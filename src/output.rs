//! Capa de salida de la salida estándar del CLI.
//!
//! Sustituye a `println!` para que un pipe cerrado por el consumidor no se
//! convierta en un pánico. `println!` hace `unwrap` del resultado de escribir
//! en la salida estándar, de modo que un [`ErrorKind::BrokenPipe`] (típico al
//! canalizar la salida a `head`) termina en un pánico con código 101. Aquí el
//! error se clasifica: el pipe cerrado es un final normal con código 0, y
//! cualquier otro fallo de escritura (por ejemplo `ENOSPC`) se informa en
//! español por stderr con un código distinto de cero.
//!
//! Las líneas ([[`print_line`]], vía `outln!`) y los bloques de texto
//! completos ([`print_block`], p. ej. el script de completions) comparten ese
//! contrato; `print_block` no añade un salto de línea extra al bloque.
//!
//! [`ErrorKind::BrokenPipe`]: std::io::ErrorKind::BrokenPipe

use std::io::{self, Write};

/// Escribe `args` en la salida estándar seguido de un salto de línea y
/// clasifica el error de escritura.
///
/// Es el destino de [`crate::outln!`]. Si la escritura falla con
/// [`ErrorKind::BrokenPipe`] el proceso termina con código 0 sin escribir nada;
/// con cualquier otro error se escribe un mensaje en español por stderr y se
/// termina con un código distinto de cero.
///
/// [`ErrorKind::BrokenPipe`]: std::io::ErrorKind::BrokenPipe
pub fn print_line(args: std::fmt::Arguments<'_>) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    if let Err(error) = writeln!(handle, "{args}") {
        if error.kind() == io::ErrorKind::BrokenPipe {
            std::process::exit(0);
        }
        eprintln!("Error: no se pudo escribir en la salida estándar: {error}");
        std::process::exit(1);
    }
}

/// Imprime una línea en la salida estándar, como [`println!`] pero sin pánico
/// si el consumidor cierra el pipe.
///
/// Acepta el mismo formato y los mismos argumentos que [`println!`], incluido
/// el caso sin argumentos (que imprime exactamente una línea en blanco).
#[macro_export]
macro_rules! outln {
    () => {
        $crate::output::print_line(format_args!(""))
    };
    ($($arg:tt)*) => {
        $crate::output::print_line(format_args!($($arg)*))
    };
}

/// Escribe un bloque de texto completo en la salida estándar, **sin** añadir un
/// salto de línea extra.
///
/// Es la vía de salida del script de completions: el script ya termina en `\n`
/// y un salto adicional alteraría su contenido. Clasifica el error de escritura
/// con el mismo contrato que [`print_line`]: un [`ErrorKind::BrokenPipe`]
/// (consumidor que cierra el pipe) es un final normal con código 0 y sin nada
/// por stderr, y cualquier otro fallo de escritura se informa en español por
/// stderr con un código distinto de cero.
///
/// [`ErrorKind::BrokenPipe`]: std::io::ErrorKind::BrokenPipe
pub fn print_block(texto: &str) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    if let Err(error) = handle.write_all(texto.as_bytes()) {
        if error.kind() == io::ErrorKind::BrokenPipe {
            std::process::exit(0);
        }
        eprintln!("Error: no se pudo escribir en la salida estándar: {error}");
        std::process::exit(1);
    }
}
