//! This crate provides utilities to concatenate files.
//!
//! To use this crate, add `fcc` as a dependency to your project's `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! fcc = "0.2"
//! ```
//!
//!
//! # Example
//!
//! The core functionality of this crate is provided by [`Concat`] type builder.
//!
//! The following example concatenates a list of csv files with some tweaks and
//! prints the result:
//!
//! ```no_run
//! use fcc::{Concat, Result};
//!
//! fn main() -> Result<()> {
//!     let files = vec!("foo.csv", "bar.csv", "baz.csv");
//!
//!     let mut concat = Concat::new()
//!         .newline(true) // appends a '\n' to each file if the file does not ends with '\n'
//!         .header(true) // extracts the headers from each file
//!         .skip_end(1) // skips last line when concatenating
//!         .pad_with(b"---end of file---\n") // fills some paddings between files
//!         .open(files);
//!
//!     concat.write(&mut std::io::stdout())?;
//!
//!     Ok(())
//! }
//!
//! ```
//!
//! [`Concat`]: struct.Concat.html
#![warn(missing_docs)]

mod error;
pub use error::{Error, ErrorKind, Result};

mod concat;
pub use concat::{ByteSeeker, Concat};

mod util;
pub use util::{ends_with_newline, get_last_byte};
