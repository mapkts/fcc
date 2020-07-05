use std::fs::{self, File};
use std::io::prelude::*;
use std::io::{BufReader, SeekFrom};
use std::path::Path;

use crate::error::{Error, ErrorKind, Result};

/// A structure for configuring how files are concatenated.
///
/// Generally speaking, when using `Concat`, you'll first call [`new`],
/// then chain calls to methods to set each configuration, then call [`open`],
/// passing the paths of files you're trying to concatenate. And eventually,
/// call [`write`] to actually concatenate files and write the result.
///
/// [`new`]: struct.Concat.html#method.new
/// [`open`]: struct.Concat.html#method.open
/// [`write`]: struct.Concat.html#method.write
///
/// # Examples
///
/// Concatenates the contents of two CSV files and prints the result:
///
/// ```no_run
/// use fcc::{Concat, Result};
///
/// fn main() -> Result<()> {
///     let paths = vec!["foo.csv", "bar.csv"];
///     let concat = Concat::new().newline(true).header(true).pad_with(b"---delimiter---\n").open(paths);
///     concat.write(&mut std::io::stdout());
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Concat<P: AsRef<Path>> {
    paths: Vec<P>,
    options: ConcatOptions,
}

/// Options to set configurations.
#[derive(Clone, Default, Debug)]
struct ConcatOptions {
    skip: usize,
    header: bool,
    newline: bool,
    padding: Option<Vec<u8>>,
}

impl<P: AsRef<Path>> Concat<P> {
    /// Constructs a new empty `Concat` instance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fcc::{Concat, Result};
    ///
    /// fn main() -> Result<()> {
    ///     let mut concat = Concat::new();
    ///     let concat = concat.newline(true).header(true).open(vec!["foo.csv", "bar.csv"]);
    ///     concat.write(&mut std::io::stdout())?;
    ///     Ok(())
    /// }
    /// ```
    pub fn new() -> Self {
        Concat {
            paths: Default::default(),
            options: Default::default(),
        }
    }

    /// Fills the `Concat` instance with the given paths.
    ///
    /// Note that this function does not check the validities of the given paths.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fcc::{Concat, Result};
    ///
    /// fn main() -> Result<()> {
    ///     let mut concat = Concat::new();
    ///     let concat = concat.newline(true).header(true).open(vec!["foo.csv", "bar.csv"]);
    ///     concat.write(&mut std::io::stdout())?;
    ///     Ok(())
    /// }
    /// ```
    pub fn open(&self, paths: Vec<P>) -> Self {
        Concat {
            paths: paths,
            options: self.options.clone(),
        }
    }

    /// Sets the option to append a newline ('\n') for each file if the file
    /// is not already ended with a newline.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fcc::{Concat, Result};
    ///
    /// fn main() -> Result<()> {
    ///     let mut concat = Concat::new();
    ///     let concat = concat.header(true).open(vec!["foo.csv", "bar.csv"]);
    ///     concat.write(&mut std::io::stdout())?;
    ///     Ok(())
    /// }
    /// ```
    pub fn newline(&mut self, newline: bool) -> &mut Self {
        self.options.newline = newline;
        self
    }

    /// Controls how many lines are skipped from the beginning of each file
    /// while concatenating.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fcc::{Concat, Result};
    ///
    /// fn main() -> Result<()> {
    ///     let mut concat = Concat::new();
    ///     let concat = concat.skip_line(1).open(vec!["foo.csv", "bar.csv"]);
    ///     concat.write(&mut std::io::stdout())?;
    ///     Ok(())
    /// }
    /// ```
    pub fn skip_line(&mut self, n: usize) -> &mut Self {
        self.options.skip = n;
        self
    }

    /// Sets the option to extract the header of each file and put
    /// the first extracted header to the beginning of concatenation result.
    ///
    /// Note that this method will also set [`skip`] to 1 simultaneously
    /// due to its semantics. In other words, `header(true)` is equivalent to
    /// `header(true).skip(1)`. If you are intended to skip more than one line,
    /// set `skip` option after setting `header(true)`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fcc::{Concat, Result};
    ///
    /// fn main() -> Result<()> {
    ///     let mut concat = Concat::new();
    ///     let concat = concat.header(true).open(vec!["foo.csv", "bar.csv"]);
    ///     concat.write(&mut std::io::stdout())?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// [`skip`]: struct.Concat.html#method.skip
    pub fn header(&mut self, header: bool) -> &mut Self {
        self.options.header = header;
        self.options.skip = 1;
        self
    }

    /// Fills some padding between the contents of each file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fcc::{Concat, Result};
    ///
    /// fn main() -> Result<()> {
    ///     let mut concat = Concat::new();
    ///     let concat = concat.pad_with(b"some padding").open(vec!["foo.csv", "bar.csv"]);
    ///     concat.write(&mut std::io::stdout())?;
    ///     Ok(())
    /// }
    /// ```
    pub fn pad_with(&mut self, padding: &[u8]) -> &mut Self {
        self.options.padding = Some(padding.to_owned());
        self
    }

    /// Triggers the file concatenation process and writes the result.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fcc::{Concat, Result};
    ///
    /// fn main() -> Result<()> {
    ///     let mut concat = Concat::new();
    ///     let concat = concat.header(true).pad_with(b"some padding").open(vec!["foo.csv", "bar.csv"]);
    ///     concat.write(&mut std::io::stdout())?;
    ///     Ok(())
    /// }
    /// ```
    pub fn write<W: Write>(self, writer: &mut W) -> Result<()> {
        // Writes the header (if any).
        let header = match self.options.header {
            true => self.get_header()?,
            false => Vec::new(),
        };
        writer.write_all(&header)?;

        // Dumps invalid paths.
        let mut paths = Vec::new();
        for path in self.paths.iter() {
            if fs::metadata(path)?.is_file() {
                paths.push(path);
            }
        }

        // Concatenates the given files.
        let mut has_eof = false;
        for path in self.paths.iter() {
            let mut file = File::open(path)?;
            has_eof = ends_with_newline(&mut file)?;
            let mut reader = BufReader::new(file);

            if self.options.skip > 0 {
                let mut buf = Vec::new();
                let mut counter = 0;
                while counter < self.options.skip {
                    reader.read_until(b'\n', &mut buf)?;
                    counter += 1;
                }
            }

            loop {
                let buffer = reader.fill_buf()?;
                let length = buffer.len();
                if length == 0 {
                    break;
                }
                writer.write_all(buffer)?;
                reader.consume(length);
            }

            if self.options.newline && !has_eof {
                writer.write(&vec![b'\n'])?;
            }

            if let Some(padding) = self.options.padding.clone() {
                writer.write(&padding)?;
            }
        }

        // Writes a newline if the concatenation result doesn't end with newline.
        if !self.options.newline && !has_eof {
            writer.write(&[b'\n'])?;
        }

        Ok(())
    }

    /// Retrieves the header of the first passed-in file.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use fcc::{Concat, Result};
    /// fn main() -> Result<()> {
    ///     let concat = Concat::new().open(vec!["foo.csv", "bar.csv"]);
    ///     let header = concat.get_header()?;
    ///     println!("{:?}", header);
    ///     Ok(())
    /// }
    /// ```
    pub fn get_header(&self) -> Result<Vec<u8>> {
        if self.paths.len() == 0 {
            return Err(Error::new(ErrorKind::NothingPassed));
        }

        let mut header = Vec::new();
        let f = File::open(&self.paths[0])?;
        let mut reader = BufReader::new(f);
        reader.read_until(b'\n', &mut header)?;

        Ok(header)
    }
}

/// Returns the last byte of a file, an in-memory cursor, or anything that
/// implements `Read` and `Seek`.
///
/// Note that this function does not alter the internal cursor of the given
/// input.
///
/// # Errors
///
/// If the given reader is empty, an error variant of `ErrorKind::Seek` will
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
        return Err(Error::new(ErrorKind::Seek));
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
/// rather than return an error variant of `Errorkind::Seek`.
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
            ErrorKind::Seek => Ok(false),
            _ => Err(e),
        },
    }
}
