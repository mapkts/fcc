use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use crate::error::{Error, ErrorKind, Result};

/// Returns the last byte of a file, an in-memory cursor, or anything that
/// implements `Read` and `Seek`.
///
/// Note that this function does not alter the internal cursor of the given
/// input.
///
/// # Errors
///
/// If the given reader is empty, an error variant of `ErrorKind::SeekNegative` will
/// be returned. If this function encounters other errors, an error variant
/// of `ErrorKind::Io` will be returned.
///
/// # Examples
///
/// ```
/// use fcc::get_last_byte;
/// use std::io::Cursor;
///
/// let mut cursor = Cursor::new(vec![1, 2, 3, b'\n']);
/// let last_byte = get_last_byte(&mut cursor).unwrap();
///
/// assert_eq!(last_byte, b'\n');
/// ```
pub fn get_last_byte<R: Read + Seek>(f: &mut R) -> Result<u8> {
    let mut buf = [0; 1];
    if let Err(_) = f.seek(SeekFrom::End(-1)) {
        return Err(Error::new(ErrorKind::SeekNegative));
    }
    f.read_exact(&mut buf)?;
    f.seek(SeekFrom::Start(0))?; // reset the internal cursor

    Ok(buf[0])
}

/// Checks if a given file ends with newline.
///
/// This function returns `Ok(true)` if the given file ends with
/// a newline `\n`, or returns `Ok(false)` if the given file does
/// not end with a newline `\n'.
///
/// # Errors
///
/// This function has the same error semantics as [`get_last_byte`],
/// except that if the given file is empty, it will return `Ok(false)`
/// rather than return an error variant of `ErrorKind::SeekNegative`.
///
/// # Examples
///
/// ```no_run
/// use fcc::ends_with_newline;
/// use std::fs::File;
/// use std::io::prelude::*;
///
/// fn main() -> std::io::Result<()> {
///     let mut f = File::create("foo.txt")?;
///
///     f.write_all(b"Hello world!")?;
///     assert_eq!(ends_with_newline(&mut f).unwrap(), false);
///
///     f.write_all(b"Hello world!\n")?;
///     assert_eq!(ends_with_newline(&mut f).unwrap(), true);
///     Ok(())
/// }
/// ```
///
/// [`get_last_byte`]: ./fn.get_last_byte.html
pub fn ends_with_newline(f: &mut File) -> Result<bool> {
    let byte = get_last_byte(f);
    match byte {
        Ok(v) => match v {
            b'\n' => Ok(true),
            _ => Ok(false),
        },
        Err(e) => match e.kind() {
            ErrorKind::SeekNegative => Ok(false),
            _ => Err(e),
        },
    }
}
