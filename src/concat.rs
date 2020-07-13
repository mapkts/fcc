use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write};
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
/// Generally speaking, when using `concat`, you'll first call [`new`],
/// then chain calls to methods to set each configuration, then call [`oepn`], passing the paths of
/// files you're trying to concatenate. And eventually, call [`write`] to actually concatenate
/// files and write the result.
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
    opts: ConcatOptions,
    view: bool,
}

// Represents `Concat`s configurations.
#[derive(Clone, Default, Debug)]
struct ConcatOptions {
    skip_start: usize,
    skip_end: usize,
    header: bool,
    newline: bool,
    crlf: bool,
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
            opts: Default::default(),
            view: Default::default(),
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
        let view = self.opts.newline
            || self.opts.header
            || self.opts.skip_start != 0
            || self.opts.skip_end != 0;
        Concat {
            paths: paths,
            opts: self.opts.clone(),
            view: view,
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
        self.opts.newline = newline;
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
    ///     let concat = concat.skip_start(1).open(vec!["foo.csv", "bar.csv"]);
    ///     concat.write(&mut std::io::stdout())?;
    ///     Ok(())
    /// }
    /// ```
    pub fn skip_start(&mut self, skip: usize) -> &mut Self {
        self.opts.skip_start = skip;
        self
    }

    /// Controls how many lines are skipped from the end of each file
    /// while concatenating.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fcc::{Concat, Result};
    ///
    /// fn main() -> Result<()> {
    ///     let mut concat = Concat::new();
    ///     let concat = concat.skip_end(1).open(vec!["foo.csv", "bar.csv"]);
    ///     concat.write(&mut std::io::stdout())?;
    ///     Ok(())
    /// }
    /// ```
    pub fn skip_end(&mut self, skip: usize) -> &mut Self {
        self.opts.skip_end = skip;
        self
    }

    /// Sets the option to extract the header of each file and put
    /// the first extracted header to the beginning of concatenation result.
    ///
    /// Note that this method will also set [`skip_start`] to 1 simultaneously
    /// due to its semantics. In other words, `header(true)` is equivalent to
    /// `header(true).skip_start(1)`. If you are intended to skip more than one line
    /// from the beginning of each file, remember to set `skip_start` option after
    /// setting `header(true)`.
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
    /// [`skip_start`]: struct.Concat.html#method.skip_start
    pub fn header(&mut self, header: bool) -> &mut Self {
        self.opts.header = header;
        self.opts.skip_start = 1;
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
        self.opts.padding = Some(padding.to_owned());
        self
    }

    /// Use `\r\n` for newline instead of `\n`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fcc::{Concat, Result};
    ///
    /// fn main() -> Result<()> {
    ///     let mut concat = Concat::new();
    ///     let concat = concat.newline(true).open(vec!["foo.csv", "bar.csv"]);
    ///     concat.write(&mut std::io::stdout())?;
    ///     Ok(())
    /// }
    /// ```
    pub fn use_crlf(&mut self, crlf: bool) -> &mut Self {
        self.opts.crlf = crlf;
        self
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
        // Dumps invalid paths.
        let mut paths = Vec::new();
        for path in self.paths.iter() {
            if fs::metadata(path)?.is_file() {
                paths.push(path);
            }
        }

        self.write_header(writer)?;

        // Concatenates the given files.
        for path in self.paths.iter() {
            self.write_contents(path, writer)?;

            if let Some(padding) = self.opts.padding.clone() {
                writer.write(&padding)?;
            }
        }

        // Writes a newline if the concatenation result doesn't end with newline.
        let mut last_file = File::open(&self.paths[self.paths.len() - 1])?;
        if !ends_with_newline(&mut last_file)? {
            if self.opts.crlf {
                writer.write(b"\r\n")?;
            } else {
                writer.write(b"\n")?;
            }
        }

        Ok(())
    }

    fn write_header<W: Write>(&self, writer: &mut W) -> Result<()> {
        if self.opts.header {
            let header = self.get_header()?;
            writer.write_all(&header)?;
        }
        Ok(())
    }

    fn write_contents<W: Write>(&self, path: &P, writer: &mut W) -> Result<()> {
        let mut file = File::open(path)?;

        let ends_nl = ends_with_newline(&mut file)?;

        if !self.view {
            // Just copy the file if viewing into the file is not required.
            io::copy(&mut file, writer)?;
        } else {
            if self.opts.skip_start > 0 || self.opts.skip_end > 0 {
                let mut seeker = ByteSeeker::new(&mut file);
                let start = seeker.seek_nth(b'\n', self.opts.skip_end)? as u64;
                seeker.reset();
                let end = seeker.seek_nth_back(b'\n', self.opts.skip_end)? as u64;
                seeker.reset();

                let mut reader = BufReader::new(file);
                let mut buf = [0; 1];
                reader.seek(SeekFrom::Start(end - 1))?;
                reader.read_exact(&mut buf)?;

                let handle = if buf[0] == b'\r' {
                    reader.take(end - 1)
                } else {
                    reader.take(end)
                };

                let mut f = handle.into_inner();
                f.seek(SeekFrom::Start(start - 1))?;
                loop {
                    let buffer = f.fill_buf()?;
                    let length = buffer.len();
                    if length == 0 {
                        break;
                    }
                    writer.write_all(buffer)?;
                    f.consume(length);
                }
            }

            if self.opts.newline && !ends_nl {
                let newline = if self.opts.crlf { "\r\n" } else { "\n" };
                writer.write(newline.as_bytes())?;
            }
        }

        Ok(())
    }
}

const DEFUALT_CHUNK_SIZE: usize = 1024 * 4;

/// A `Seeker` walks through anything that implements `Read` and `Seek`
/// to find the position of a certain `byte`.
#[derive(Debug)]
pub struct ByteSeeker<'a, RS: 'a + Read + Seek> {
    inner: &'a mut RS,
    buf: Vec<u8>,
    len: usize,
    lpos: usize,
    rpos: usize,
    done: bool,
    oneleft: bool,
}

impl<'a, RS: 'a + Read + Seek> ByteSeeker<'a, RS> {
    /// Creates a new `ByteSeeker` from something that implements `Read` and `Seek`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use fcc::{ByteSeeker, Result};
    ///
    /// fn main() -> Result<()> {
    ///     // `Cursor` implements `Read` and `Seek`.
    ///     let mut cursor = Cursor::new(vec![1, 2, b'\n', 3]);
    ///     let mut seeker = ByteSeeker::new(&mut cursor);
    ///
    ///     let pos = seeker.seek(b'\n')?;
    ///     assert_eq!(pos, 2);
    ///     Ok(())
    /// }
    /// ```
    pub fn new(inner: &'a mut RS) -> Self {
        // SAFETY: The unwraps here are safe beacause no negative offset has been sought.
        let len = inner.seek(SeekFrom::End(0)).unwrap() as usize;
        inner.seek(SeekFrom::Start(0)).unwrap();

        Self {
            inner: inner,
            buf: vecu8(DEFUALT_CHUNK_SIZE),
            len: len,
            lpos: 0,
            rpos: if len == 0 { 0 } else { len - 1 },
            done: false,
            oneleft: false,
        }
    }

    /// Reset the initialized `ByteSeeker` to its original state.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use fcc::{ByteSeeker, Result};
    ///
    /// fn main() -> Result<()> {
    ///     // `Cursor` implements `Read` and `Seek`.
    ///     let mut cursor = Cursor::new(vec![1, b'\n', 3, b'\n']);
    ///     let mut seeker = ByteSeeker::new(&mut cursor);
    ///
    ///     let pos = seeker.seek(b'\n')?;
    ///     assert_eq!(pos, 1);
    ///
    ///     seeker.reset();
    ///     let pos = seeker.seek_back(b'\n')?;
    ///     assert_eq!(pos, 3);
    ///     Ok(())
    /// }
    /// ```
    pub fn reset(&mut self) {
        self.inner.seek(SeekFrom::Start(0)).unwrap() as usize;
        self.buf = vecu8(DEFUALT_CHUNK_SIZE);
        self.lpos = 0;
        self.rpos = if self.len == 0 { 0 } else { self.len - 1 };
        self.done = false;
        self.oneleft = false;
    }

    /// Seeks the nth occurence of a specific byte **forwards**, and
    /// returns the new position from the start of the byte stream.
    ///
    /// # Errors
    ///
    /// If the nth occurence of the given byte cannot be found, an error of
    /// `ErrorKind::ByteNotFound` will be returned. If any other IO errors was encountered, an error of
    /// `ErrorKind::Io` will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use std::iter;
    /// use fcc::ByteSeeker;
    ///
    /// let bytes: Vec<u8> = iter::repeat(0)
    ///     .take(100)
    ///     .chain(iter::repeat(b'\n').take(1))
    ///     .chain(iter::repeat(1).take(100))
    ///     .chain(iter::repeat(b'\n').take(1))
    ///     .chain(iter::repeat(2).take(100))
    ///     .collect();
    ///
    /// let mut cursor = Cursor::new(bytes);
    /// let mut seeker = ByteSeeker::new(&mut cursor);
    /// assert_eq!(seeker.seek_nth(b'\n', 2).unwrap(), 100 + 1 + 100);
    /// ```
    pub fn seek_nth(&mut self, byte: u8, nth: usize) -> Result<usize> {
        let mut counter = nth;
        loop {
            let pos = self.seek(byte)?;
            counter -= 1;
            if counter == 0 {
                return Ok(pos);
            }
        }
    }

    /// Seeks the nth occurence of a specific byte **backwards**, and
    /// returns the new position from the start of the byte stream.
    ///
    /// # Errors
    ///
    /// If the nth occurence of the given byte cannot be found, an error of
    /// `ErrorKind::ByteNotFound` will be returned. If any other IO errors was encountered, an error of
    /// `ErrorKind::Io` will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use std::iter;
    /// use fcc::ByteSeeker;
    ///
    /// let bytes: Vec<u8> = iter::repeat(0)
    ///     .take(100)
    ///     .chain(iter::repeat(b'\n').take(1))
    ///     .chain(iter::repeat(1).take(100))
    ///     .chain(iter::repeat(b'\n').take(1))
    ///     .chain(iter::repeat(2).take(100))
    ///     .collect();
    ///
    /// let mut cursor = Cursor::new(bytes);
    /// let mut seeker = ByteSeeker::new(&mut cursor);
    /// assert_eq!(seeker.seek_nth_back(b'\n', 2).unwrap(), 100);
    /// ```
    pub fn seek_nth_back(&mut self, byte: u8, nth: usize) -> Result<usize> {
        let mut counter = nth;
        loop {
            let pos = self.seek_back(byte)?;
            counter -= 1;
            if counter == 0 {
                return Ok(pos);
            }
        }
    }

    /// Searches for a specified byte **forwards** from the last `seek` position. If the
    /// initialized `ByteSeeker` haven't been called `seek` before, `seek` will start from
    /// the beginning.
    ///
    /// The `ByteSeeker` is stateful, which means you can call `seek` multiple times until
    /// reaching the end of underlying byte stream.
    ///
    /// # Errors
    /// If no given byte was found, an error variant of `ErrorKind::ByteNotFound` will be returned.
    /// If any other errors were encountered, an error variant of `ErrorKind::Io` will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use fcc::{ByteSeeker, Result};
    ///
    /// fn main() -> Result<()> {
    ///     let mut cursor = Cursor::new(vec![b'\n', 0, b'\n']);
    ///     let mut seeker = ByteSeeker::new(&mut cursor);
    ///
    ///     let pos = seeker.seek(b'\n')?;
    ///     assert_eq!(pos, 0);
    ///     let pos = seeker.seek(b'\n')?;
    ///     assert_eq!(pos, 2);
    ///     Ok(())
    /// }
    /// ```
    pub fn seek(&mut self, byte: u8) -> Result<usize> {
        if self.done || self.len == 0 {
            return Err(Error::new(ErrorKind::ByteNotFound));
        }

        if self.len == 1 || self.oneleft {
            let mut buf = [0; 1];
            self.inner.read_exact(&mut buf)?;
            self.done = true;
            if buf[0] == byte {
                return Ok(0);
            } else {
                return Err(Error::new(ErrorKind::ByteNotFound));
            }
        }

        loop {
            // Reads a chunk of contents.
            let remaining = self.len - self.lpos;
            // If the length of remaining bytes is greater than the length of internal buffer, just
            // read the exact number of bytes required to fill the internal buffer. Otherwise, we
            // truncate the length of internal buffer to the length of remaining bytes.
            let mut buflen = self.buf.len();
            let mut is_last_read = false;
            if remaining < buflen {
                unsafe {
                    self.buf.set_len(remaining);
                }
                buflen = remaining;
                is_last_read = true;
            }
            self.inner.read_exact(&mut self.buf)?;

            if let Some(pos) = self.buf.iter().position(|&x| x == byte) {
                let cpos = self.lpos + pos;
                self.lpos = self.inner.seek(SeekFrom::Start((cpos + 1) as u64))? as usize;
                if self.lpos > self.len - 1 {
                    self.oneleft = true;
                }
                return Ok(cpos);
            } else {
                if is_last_read {
                    self.done = true;
                    return Err(Error::new(ErrorKind::ByteNotFound));
                } else {
                    self.lpos = self
                        .inner
                        .seek(SeekFrom::Start((self.lpos + buflen) as u64))?
                        as usize;
                }
            }
        }
    }

    /// Searches for a specified byte **backwards** from the last `seek_back` position. If the
    /// initialized `ByteSeeker` haven't been called `seek_back` before, `seek_back` will start
    /// from the beginning.
    ///
    /// The `ByteSeeker` is stateful, which means you can call `seek_back` multiple times until
    /// reaching the end of underlying byte stream.
    ///
    /// # Errors
    /// If no given byte was found, an error variant of `ErrorKind::ByteNotFound` will be returned.
    /// If any other errors were encountered, an error variant of `ErrorKind::Io` will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use fcc::{ByteSeeker, Result};
    ///
    /// fn main() -> Result<()> {
    ///     let mut cursor = Cursor::new(vec![b'\n', 0, b'\n']);
    ///     let mut seeker = ByteSeeker::new(&mut cursor);
    ///
    ///     let pos = seeker.seek_back(b'\n')?;
    ///     assert_eq!(pos, 2);
    ///     let pos = seeker.seek_back(b'\n')?;
    ///     assert_eq!(pos, 0);
    ///     Ok(())
    /// }
    /// ```
    pub fn seek_back(&mut self, byte: u8) -> Result<usize> {
        if self.done || self.len == 0 {
            println!("loc 1");
            return Err(Error::new(ErrorKind::ByteNotFound));
        }

        if self.len == 1 || self.oneleft {
            println!("loc 2");
            let mut buf = [0; 1];
            self.inner.read_exact(&mut buf)?;
            self.done = true;
            if buf[0] == byte {
                return Ok(0);
            } else {
                return Err(Error::new(ErrorKind::ByteNotFound));
            }
        }

        loop {
            // Reads a chunk of contents.
            let remaining = self.rpos + 1;
            println!("remaining: {}, rpos: {}", remaining, self.rpos);
            // If the length of remaining bytes is greater than the length of internal buffer, just
            // read the exact number of bytes required to fill the internal buffer. Otherwise, we
            // truncate the length of internal buffer to the length of remaining bytes.
            let mut buflen = self.buf.len();
            let mut is_last_read = false;
            if remaining < buflen {
                unsafe {
                    self.buf.set_len(remaining);
                }
                buflen = remaining;
                is_last_read = true;
            }
            self.rpos =
                self.inner
                    .seek(SeekFrom::Start((remaining - buflen) as u64))? as usize;
            println!("before rpos: {}", self.rpos);
            self.inner.read_exact(&mut self.buf)?;

            if let Some(pos) = self.buf.iter().rev().position(|&x| x == byte) {
                let cpos = self.rpos + (buflen - pos - 1);
                if cpos == 0 {
                    self.done = true;
                    return Ok(cpos);
                }
                self.rpos = self.inner.seek(SeekFrom::Start((cpos - 1) as u64))? as usize;
                println!("after success rpos: {}", self.rpos);
                if self.rpos == 0 {
                    self.oneleft = true;
                }
                return Ok(cpos);
            } else {
                if is_last_read {
                    self.done = true;
                    self.rpos = self.inner.seek(SeekFrom::Start(0))? as usize;
                    println!("after last_read rpos: {}", self.rpos);
                    return Err(Error::new(ErrorKind::ByteNotFound));
                } else {
                    println!("after failed rpos: {}", self.rpos);
                }
            }
        }
    }
}

// Initializes a `Vec<u8>` whose capacity and length are exactly the same.
fn vecu8(len: usize) -> Vec<u8> {
    let mut vec = Vec::with_capacity(len);
    unsafe {
        vec.set_len(len);
    }
    vec
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::iter;

    #[test]
    fn test_vecu8() {
        let vec = vecu8(0);
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), 0);

        let vec = vecu8(42);
        assert_eq!(vec.len(), 42);
        assert_eq!(vec.capacity(), 42);
    }

    #[test]
    fn test_seek_vec0() {
        // TODO: Should update the implementation of `Error` and `ErrorKind` to make it possible to
        // compare if two errors are equivalent. By new we just check if there is an error or not.

        // test empty vec<u8>
        let bytes: Vec<u8> = vec![];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek(b'\n') {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_seek_vec1() {
        // test vec<u8> with only 1 byte.
        let bytes: Vec<u8> = vec![b'0'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek(b'\n') {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }

        let bytes: Vec<u8> = vec![b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek(b'\n').unwrap(), 0);
        match seeker.seek(b'\n') {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_seek_vec3() {
        // test vec<u8> with 3 bytes.
        let bytes: Vec<u8> = vec![b'\n', 0, b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek(b'\n').unwrap(), 0);
        assert_eq!(seeker.seek(b'\n').unwrap(), 2);
        match seeker.seek(b'\n') {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }

        let bytes: Vec<u8> = vec![0, 0, 0];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek(b'\n') {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_seek_vecn() {
        // test vec<u8> with more than `DEFUALT_CHUNK_SIZE` bytes.
        let bytes: Vec<u8> = iter::repeat(0)
            .take(DEFUALT_CHUNK_SIZE)
            .chain(iter::repeat(b'\n').take(1))
            .chain(iter::repeat(0).take(DEFUALT_CHUNK_SIZE))
            .chain(iter::repeat(b'\n').take(1))
            .collect();
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek(b'\n').unwrap(), DEFUALT_CHUNK_SIZE);
        assert_eq!(seeker.seek(b'\n').unwrap(), DEFUALT_CHUNK_SIZE * 2 + 1);
        match seeker.seek(b'\n') {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }

        let bytes: Vec<u8> = iter::repeat(0).take(DEFUALT_CHUNK_SIZE + 42).collect();
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek(b'\n') {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_seek_back_vec0() {
        // test empty vec<u8>
        let bytes: Vec<u8> = vec![];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek_back(b'\n') {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_seek_back_vec1() {
        // test vec<u8> with only 1 byte.
        let bytes: Vec<u8> = vec![b'0'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek_back(b'\n') {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }

        let bytes: Vec<u8> = vec![b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek_back(b'\n').unwrap(), 0);
        match seeker.seek_back(b'\n') {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_seek_back_vec3() {
        // test vec<u8> with 3 bytes.
        let bytes: Vec<u8> = vec![b'\n', 0, b'\n'];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek_back(b'\n').unwrap(), 2);
        assert_eq!(seeker.seek_back(b'\n').unwrap(), 0);
        match seeker.seek_back(b'\n') {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }

        let bytes: Vec<u8> = vec![0, 0, 0];
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek_back(b'\n') {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_seek_back_vecn() {
        // test vec<u8> with more than `DEFUALT_CHUNK_SIZE` bytes.
        let bytes: Vec<u8> = iter::repeat(0)
            .take(DEFUALT_CHUNK_SIZE)
            .chain(iter::repeat(b'\n').take(1))
            .chain(iter::repeat(0).take(DEFUALT_CHUNK_SIZE))
            .chain(iter::repeat(b'\n').take(1))
            .collect();
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek_back(b'\n').unwrap(), DEFUALT_CHUNK_SIZE * 2 + 1);
        assert_eq!(seeker.seek_back(b'\n').unwrap(), DEFUALT_CHUNK_SIZE);
        match seeker.seek_back(b'\n') {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }

        let bytes: Vec<u8> = iter::repeat(0).take(DEFUALT_CHUNK_SIZE + 42).collect();
        let mut cursor = Cursor::new(bytes);
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek_back(b'\n') {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_seek_nth() {
        let bytes: Vec<u8> = iter::repeat(0)
            .take(DEFUALT_CHUNK_SIZE)
            .chain(iter::repeat(b'\n').take(1))
            .chain(iter::repeat(0).take(DEFUALT_CHUNK_SIZE))
            .chain(iter::repeat(b'\n').take(1))
            .chain(iter::repeat(0).take(100))
            .chain(iter::repeat(b'\n').take(1))
            .collect();

        let mut cursor = Cursor::new(bytes.clone());
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek_nth(b'\n', 1).unwrap(), DEFUALT_CHUNK_SIZE);

        let mut cursor = Cursor::new(bytes.clone());
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(
            seeker.seek_nth(b'\n', 2).unwrap(),
            DEFUALT_CHUNK_SIZE * 2 + 1
        );

        let mut cursor = Cursor::new(bytes.clone());
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(
            seeker.seek_nth(b'\n', 3).unwrap(),
            DEFUALT_CHUNK_SIZE * 2 + 100 + 2
        );

        let mut cursor = Cursor::new(bytes.clone());
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek_nth(b'\n', 4) {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_seek_nth_back() {
        let bytes: Vec<u8> = iter::repeat(0)
            .take(DEFUALT_CHUNK_SIZE)
            .chain(iter::repeat(b'\n').take(1))
            .chain(iter::repeat(0).take(DEFUALT_CHUNK_SIZE))
            .chain(iter::repeat(b'\n').take(1))
            .chain(iter::repeat(0).take(100))
            .chain(iter::repeat(b'\n').take(1))
            .collect();

        let mut cursor = Cursor::new(bytes.clone());
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(
            seeker.seek_nth_back(b'\n', 1).unwrap(),
            DEFUALT_CHUNK_SIZE * 2 + 100 + 2
        );

        let mut cursor = Cursor::new(bytes.clone());
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(
            seeker.seek_nth_back(b'\n', 2).unwrap(),
            DEFUALT_CHUNK_SIZE * 2 + 1
        );

        let mut cursor = Cursor::new(bytes.clone());
        let mut seeker = ByteSeeker::new(&mut cursor);
        assert_eq!(seeker.seek_nth_back(b'\n', 3).unwrap(), DEFUALT_CHUNK_SIZE);

        let mut cursor = Cursor::new(bytes.clone());
        let mut seeker = ByteSeeker::new(&mut cursor);
        match seeker.seek_nth(b'\n', 4) {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }
    }
}
