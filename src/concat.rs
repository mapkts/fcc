use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::error::{Error, ErrorKind, Result};
use crate::util::ends_with_newline;

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
