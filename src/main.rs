use admerge::{FileMerger, Newline, Pad, Skip};
use structopt::StructOpt;

use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::PathBuf;

macro_rules! stderr {
    ($($arg:tt)*) => {
        use std::io::Write;
        (writeln!(&mut std::io::stderr(), $($arg)*)).unwrap();
    }
}

#[derive(StructOpt, Debug)]
#[structopt(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    about = "reads files from <STDIN> and merges those files into <STDOUT>.",
    after_help = "NOTES:

    1. If read from <STDIN>, file paths must be space-separated or newline-separated."
)]
struct Opts {
    /// Sets the input files, reads from <STDIN> if not present
    #[structopt(
        long,
        short,
        display_order = 1,
        value_name = "PATH",
        parse(from_os_str)
    )]
    input: Option<Vec<PathBuf>>,
    /// Writes output to a specific <FILE> instead of <STDOUT>
    #[structopt(
        long,
        short,
        display_order = 2,
        value_name = "FILE",
        parse(from_os_str)
    )]
    output: Option<PathBuf>,
    /// Skips a number of lines from the head of each source
    #[structopt(
        long,
        short = "s",
        display_order = 3,
        value_name = "NUMBER",
        conflicts_with = "skip-head-once"
    )]
    skip_head: Option<usize>,
    /// Skips a number of lines from the tail of each source
    #[structopt(
        long,
        short = "e",
        display_order = 4,
        value_name = "NUMBER",
        conflicts_with = "skip-tail-once"
    )]
    skip_tail: Option<usize>,
    /// Leaves the first source untouched, and skips a number of lines from the head of the rest sources
    #[structopt(
        long,
        short = "S",
        display_order = 5,
        value_name = "NUMBER",
        conflicts_with = "headonce"
    )]
    skip_head_once: Option<usize>,
    /// Leaves the last source untouched, and skips a number of lines from the tail of the rest sources
    #[structopt(
        long,
        short = "E",
        display_order = 6,
        value_name = "NUMBER",
        conflicts_with = "tailonce"
    )]
    skip_tail_once: Option<usize>,
    /// Skips the head line from each source while leaves the headline of the first source untouched (equivalent to --skip-head-once=1)
    #[structopt(long, short = "H", display_order = 7, conflicts_with = "skip-head")]
    headonce: bool,
    /// Skips the tail line from each source while leaves the headline of the first source untouched (equivalent to --skip-tail-once=1)
    #[structopt(long, short = "T", display_order = 8, conflicts_with = "skip-tail")]
    tailonce: bool,
    /// Sets the skip mode
    #[structopt(
            long,
            short = "m",
            display_order = 9,
            value_name = "STRING",
            default_value = "lines",
            possible_values = &["bytes", "lines"],
        )]
    skip_mode: String,
    /// Skips a number of lines from the head of each source
    #[structopt(long, short = "p", display_order = 10, value_name = "STRING")]
    padding: Option<String>,
    /// Fills some padding between each source
    #[structopt(
            long,
            short = "P",
            display_order = 11,
            value_name = "STRING",
            default_value = "between",
            possible_values = &["beforestart", "afterend", "between", "all"],
        )]
    pad_mode: String,
    /// Appends a newline `\n` after each source if source is not already ended with newline
    #[structopt(long, short = "n", display_order = 12)]
    newline: bool,
    /// The style of newline, either unix-style `LF` or dos-style `CRLF`
    #[structopt(
            long,
            short = "N",
            display_order = 13,
            value_name = "STRING",
            default_value = "lf",
            possible_values = &["lf", "crlf"],
        )]
    newline_style: String,
}

fn main() {
    let opts = Opts::from_args();
    // println!("{:?}", opts);

    if let Err(e) = run(&opts) {
        stderr!("fcc: {}", e);
        std::process::exit(1);
    }
}

fn run(opts: &Opts) -> admerge::Result<()> {
    // Reads input from cli argument (primary) or `stdin` (fallback).
    let input = match &opts.input {
        Some(paths) => paths.clone(),
        None => {
            let mut buf = String::new();
            std::io::stdin().lock().read_to_string(&mut buf)?;

            let paths = buf
                .split(&[' ', '\n'][..])
                .filter(|v| v != &"")
                .map(|v| v.trim())
                .map(|v| PathBuf::from(v))
                .collect::<Vec<PathBuf>>();

            paths
        }
    };

    let mut merger = FileMerger::new();
    match opts.skip_mode.as_str() {
        "lines" => {
            if let Some(n) = opts.skip_head {
                merger.skip_head(Skip::Lines(n));
            }
            if let Some(n) = opts.skip_head_once {
                merger.skip_head(Skip::LinesOnce(n));
            }
            if opts.headonce {
                merger.skip_head(Skip::LinesOnce(1));
            }
            if let Some(n) = opts.skip_tail {
                merger.skip_tail(Skip::Lines(n));
            }
            if let Some(n) = opts.skip_tail_once {
                merger.skip_tail(Skip::LinesOnce(n));
            }
            if opts.tailonce {
                merger.skip_tail(Skip::LinesOnce(1));
            }
        }
        "bytes" => {
            if let Some(n) = opts.skip_head {
                merger.skip_head(Skip::Bytes(n));
            }
            if let Some(n) = opts.skip_head_once {
                merger.skip_head(Skip::BytesOnce(n));
            }
            if let Some(n) = opts.skip_tail {
                merger.skip_tail(Skip::Bytes(n));
            }
            if let Some(n) = opts.skip_tail_once {
                merger.skip_tail(Skip::BytesOnce(n));
            }
        }
        other => panic!("unexpected `{}` in skip-mode", other),
    }

    match (opts.newline, opts.newline_style.as_str()) {
        (true, "lf") => {
            merger.force_ending_newline(Newline::Lf);
        }
        (true, "crlf") => {
            merger.force_ending_newline(Newline::Crlf);
        }
        (false, "lf") => (),
        (false, "crlf") => (),
        (_, other) => panic!("unexpected `{}` in newline-style", other),
    }

    match (&opts.padding, opts.pad_mode.as_str()) {
        (Some(padding), "beforestart") => {
            merger.pad_with(Pad::Before(padding.as_bytes()));
        }
        (Some(padding), "afterend") => {
            merger.pad_with(Pad::After(padding.as_bytes()));
        }
        (Some(padding), "between") => {
            merger.pad_with(Pad::Between(padding.as_bytes()));
        }
        (Some(padding), "all") => {
            merger.pad_with(Pad::Custom(
                Some(padding.as_bytes()),
                Some(padding.as_bytes()),
                Some(padding.as_bytes()),
            ));
        }
        (None, "beforestart") => (),
        (None, "afterend") => (),
        (None, "between") => (),
        (None, "all") => (),
        (_, other) => panic!("unexpected `{}` in pad-mode", other),
    }

    // Writes result to file (primary) or `stdout` (fallback).
    match &opts.output {
        Some(path) => {
            let mut file = OpenOptions::new().create(true).write(true).open(path)?;
            merger.with_paths(input, &mut file)?;
        }
        None => {
            merger.with_paths(input, &mut std::io::stdout().lock())?;
        }
    };

    Ok(())
}
