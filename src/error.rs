use std::error;
use std::fmt;
use std::io;
use std::result;

/// A type alias for `Result<T, fcc::Error>`.
///
/// This result type embeds the error type in this crate.
pub type Result<T> = result::Result<T, Error>;

/// An error that can occur when using `fcc`.
#[derive(Debug)]
pub struct Error(Box<ErrorKind>);

impl Error {
    /// A crate private constructor for `Error`.
    pub(crate) fn new(kind: ErrorKind) -> Error {
        Error(Box::new(kind))
    }

    /// Returns the specific type of this error.
    pub fn kind(&self) -> &ErrorKind {
        &self.0
    }

    /// Unwraps this error into its undelying type.
    pub fn into_kind(self) -> ErrorKind {
        *self.0
    }
}

/// The specific type of an error.
#[derive(Debug)]
pub enum ErrorKind {
    /// Represents an I/O error.
    ///
    /// Can occur when reading or writing to a file.
    Io(io::Error),
    /// Occurs if there is nothing to concat.
    NothingPassed,
    /// This error occurs when seeking to a negative offset.
    Seek,
    /// Hints that implies destructuring should not be exhaustive.
    ///
    /// This enum may grow additional variants, so this
    /// makes sure clients don't count on exhaustive matching.
    #[doc(hidden)]
    __Nonexhaustive,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self.0 {
            ErrorKind::Io(ref err) => err.fmt(f),
            ErrorKind::NothingPassed => {
                write!(f, "Cannot construct a `Concat` instance with no paths")
            }
            ErrorKind::Seek => write!(f, "Seek to a negative offset"),
            _ => unreachable!(),
        }
    }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::new(ErrorKind::Io(err))
    }
}

impl From<Error> for io::Error {
    fn from(err: Error) -> io::Error {
        io::Error::new(io::ErrorKind::Other, err)
    }
}
